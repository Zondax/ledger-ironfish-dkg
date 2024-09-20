use core::{mem::MaybeUninit, ptr::addr_of_mut};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use nom::{
    bytes::complete::take,
    number::complete::{le_i64, le_u32, le_u64, le_u8},
};

use crate::{
    bolos::{zlog, zlog_stack},
    ironfish::{errors::IronfishError, multisig::MultisigAccountKeys, view_keys::OutgoingViewKey},
    parser::{
        constants::{KEY_LENGTH, REDJUBJUB_SIGNATURE_LEN},
        SIGNATURE_HASH_PERSONALIZATION, TX_HASH_LEN,
    },
};

mod burns;
mod mints;
mod outputs;
mod spends;

use super::{FromBytes, ObjectList, ParserError, TransactionVersion};
pub use burns::Burn;
pub use mints::Mint;
pub use outputs::Output;
pub use spends::Spend;

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Transaction<'a> {
    tx_version: TransactionVersion,
    random_pubkey: &'a [u8; KEY_LENGTH],
    pubkey_randomness: &'a [u8; KEY_LENGTH],

    spends: ObjectList<'a, Spend<'a>>,
    outputs: ObjectList<'a, Output<'a>>,
    mints: ObjectList<'a, Mint<'a>>,
    burns: ObjectList<'a, Burn<'a>>,
    fee: i64,
    expiration: u32,
    binding_sig: &'a [u8; REDJUBJUB_SIGNATURE_LEN],
}

impl<'a> FromBytes<'a> for Transaction<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<super::ParserError>> {
        zlog_stack("Transaction::from_bytes_into\n");
        let out = out.as_mut_ptr();

        let (rem, raw_version) = le_u8(input)?;
        let version = TransactionVersion::try_from(raw_version)?;
        zlog_stack("Transaction::version ok\n");
        // now read the number of spends, outputs, mints and burns
        let (rem, num_spends) = le_u64(rem)?;
        let (rem, num_outputs) = le_u64(rem)?;
        let (rem, num_mints) = le_u64(rem)?;
        let (rem, num_burns) = le_u64(rem)?;
        // now read the fee and expiration
        let (rem, fee) = le_i64(rem)?;
        let (rem, expiration) = le_u32(rem)?;

        // This fields bellows are present in C parser, we need to figure out where to
        // place this information
        // rondomizedPublicKey
        let (rem, random_pubkey) = take(KEY_LENGTH)(rem)?;
        // publicKeyRandomness
        let (rem, randomness) = take(KEY_LENGTH)(rem)?;

        let random_pubkey = arrayref::array_ref![random_pubkey, 0, KEY_LENGTH];
        let pubkey_randomness = arrayref::array_ref![randomness, 0, KEY_LENGTH];

        let spends: &mut MaybeUninit<ObjectList<'a, Spend<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).spends).cast() };
        let rem = ObjectList::new_into_with_len(rem, spends, num_spends as usize)?;

        let outputs: &mut MaybeUninit<ObjectList<'a, Output<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).outputs).cast() };
        let rem = ObjectList::new_into_with_len(rem, outputs, num_outputs as usize)?;

        let mints: &mut MaybeUninit<ObjectList<'a, Mint<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).mints).cast() };
        let rem = ObjectList::new_into_with_len(rem, mints, num_mints as usize)?;

        let burns: &mut MaybeUninit<ObjectList<'a, Burn<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).burns).cast() };
        let rem = ObjectList::new_into_with_len(rem, burns, num_burns as usize)?;

        let (rem, sig) = take(REDJUBJUB_SIGNATURE_LEN)(rem)?;
        let binding_sig = arrayref::array_ref![sig, 0, REDJUBJUB_SIGNATURE_LEN];

        unsafe {
            addr_of_mut!((*out).tx_version).write(version);
            addr_of_mut!((*out).fee).write(fee);
            addr_of_mut!((*out).expiration).write(expiration);
            addr_of_mut!((*out).binding_sig).write(binding_sig);
            addr_of_mut!((*out).random_pubkey).write(random_pubkey);
            addr_of_mut!((*out).pubkey_randomness).write(pubkey_randomness);
        }
        zlog_stack("Transaction::from_bytes_into ok\n");

        Ok(input)
    }
}

impl<'a> Transaction<'a> {
    pub fn num_spends(&self) -> usize {
        self.spends.iter().count()
    }

    pub fn num_outputs(&self) -> usize {
        self.outputs.iter().count()
    }

    pub fn num_mints(&self) -> usize {
        self.mints.iter().count()
    }

    pub fn num_burns(&self) -> usize {
        self.burns.iter().count()
    }

    pub fn outputs_iter(&self) -> impl Iterator<Item = Output<'a>> {
        self.outputs.iter()
    }

    #[inline(never)]
    pub fn review_fields(
        &self,
        ovk: &OutgoingViewKey,
    ) -> Result<Vec<(String, String)>, IronfishError> {
        zlog_stack("Transaction::review_fields\n");

        let num_output = self.outputs_iter().count();

        // allocate spaces for num_outputs * 3(owner, amount, asset_it) + 1(fee) + 1(expiration) + 1(tx_version)
        let mut fields = Vec::with_capacity(num_output * 3 + 1 + 1 + 1);

        // Add transaction version
        fields.push((
            "Tx Version".to_string(),
            self.tx_version.as_str().to_string(),
        ));

        for (i, output) in self.outputs.iter().enumerate() {
            let output_number = i + 1;

            // Safe to unwrap because MerkleNote was also parsed in outputs from_bytes impl
            let Ok(merkle_note) = output.note() else {
                return Err(IronfishError::InvalidData);
            };

            // now get the encrypted Note
            let note = merkle_note.decrypt_note_for_spender(ovk)?;

            fields.push((
                format!("Owner {}", output_number),
                hex::encode(note.owner.public_address()),
            ));

            fields.push((
                format!("Amount {}", output_number),
                format!("{}", note.value),
            ));

            fields.push((
                format!("AssetID {}", output_number),
                hex::encode(note.asset_id.as_bytes()),
            ));
        }

        // Add fee
        fields.push(("Fee".to_string(), format!("{}", self.fee)));
        zlog_stack("Transaction::fee pushed\0");

        // Add expiration
        fields.push(("Expiration".to_string(), format!("{}", self.expiration)));
        zlog_stack("Transaction::expiration pushed\0");

        Ok(fields)
    }

    #[inline(never)]
    pub fn hash(&self) -> [u8; TX_HASH_LEN] {
        use blake2b_simd::Params as Blake2b;
        let mut hasher = Blake2b::new()
            .hash_length(TX_HASH_LEN)
            .personal(SIGNATURE_HASH_PERSONALIZATION)
            .to_state();

        hasher.update(&[self.tx_version as u8]);

        let expiration = self.expiration.to_le_bytes();
        let fee = (self.fee as i64).to_le_bytes();
        hasher.update(&expiration);
        hasher.update(&fee);

        hasher.update(self.random_pubkey);

        for spend in self.spends.iter() {
            spend.hash(&mut hasher);
        }

        for output in self.outputs.iter() {
            output.hash(&mut hasher);
        }

        for mint in self.mints.iter() {
            mint.hash(&mut hasher);
        }

        for burn in self.burns.iter() {
            burn.hash(&mut hasher);
        }

        let mut hash_result = [0; 32];
        hash_result[..].copy_from_slice(hasher.finalize().as_ref());

        hash_result
    }
}

#[cfg(test)]
mod transaction_test {
    use crate::{ironfish::view_keys::OutgoingViewKey, *};

    // testing data for unit tests
    const TRANSACTION: &str = "0101000000000000000300000000000000010000000000000000000000000000000100000000000000000000003475b26a991a739f6b77dd7bce822efa46d355a28cf0e4dc9c93b6ababf073c5ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb07ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb0795aef72203054fbfa24ffd1c375e6d69827111b74805477b38ef4932d143c6a44efc37619fbedc3a1b46d5613202ac6baf75caadb2ac6b6c9db0b02acf8698a1f51eb6a1a1dbebf7fa0b034803716d103c96e9893180cc971824bc2b978e1b1600ebd35de5d6b921b2ce8aa4c03ef7312c9d6efc7e259f4dd68d5c7b632a88bc259386faae6cea9e44e232a6cbf1079486a29e9d622e3c25e1985155226c4d48342be389600dc829f89aac5a81c444fd880f5b5ba1541b9434d620543a6e8be7eef40fe52914631d18a7d4dfb1c41beed6c2b7a51efd985f2e210059f36f6000ec8ee990b1228496c0d1140767f2aae76d79e09f5f777ff5af0f89ef6aefe7337805000045e9b744ed2afca6615aa6ba00dcf578979391b219ca05e90447abf21315a9be00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b9032b6225f92f518e019f07373891b6524df7bd3e25afc0b0fb7f5ca2da049c419860e0eb24aff16d830bc21061662393a76e80b2ef434d6f9bbfc3926644e933210d159074e80b368c16312710226160c38e0b05e69a8f353c5de3e0e6fd1e03892c58d7e8eeb5604aab425b445d86e9357341e2a175b8835b99653d75ba42c11500beeb61c90523a7dee736d73ab2a6e496b7aa8241ae8bd222475f3e0b510b0f5a57c5a97583d342d4c8740f66fb856d730caafa608296b2b02aa7bd4a99499b3cde309b75e0200df587d09952547f29046abc4fdd47714860d09f9cdcf3f3c24fd8d8fd0e0d2f83f1191ff59f37fa811d1a4c90faf70d4da89ef228ce6d8d0af19c715cae4d74fe07d3d66c473dff6799bc3d8979fafd5a9cef9b1d7650a1f488cd3478146e27075c1b1c73697ba64e94b37d13f29d22068ef58a7a721138213f218a11d1ab4b88d41420b40a010c9cbf2e10f00f7ae52dd54eeb9b716fac76db7d364b5cf43d278b61a0863234c03485364347505af9c86b8d582c453c1c98e52882d08e49d496791a899796ea3e5773435edc97caadffcc579341474cc31b87e6b19b0d526618585a6693184f903512ef641eda23f30e3a45782e95d1c3529230566b78aa1421834c58d0d44ab79467aff475858a38cbd67d4c4af4ed0cf677a063fea0b38b193ab6a18d81b3848f941d61b46bd86f348c613a9b0dc18610fae24d7328209168bb05d58d1085496a5a4769da339fa3ef1463956da25ccaff94a58beff6417d3f94d07eece27b8ca1db74ce29cce9a40556f8967d5ebb58da4efd01e4fb9b27578fc8e883320cff5f36a1b69706816cabdaff3ba9cb06756d8c37d760419d05bd9b8ffa84ed6ea98b952f0ea2e927ba0782e90a949b8821c305c20db46d66c00b48fc4150eeff0bc07a6fd890c80a802657573703a1b7aa9bd3b6eda9ad2adce2f162d1117aaf08093c779dd4db20c94a48e62357ade7daf5284d3b4d2f6885dc03a34b48aa2890ab4c60128769bf8bef2ee7a06f4d552eaed15c4df1da5ccf3762f37b818c9d62f7464416bc40f59c4f3c6299a929a67dd3d09e00d1bd68c16d38ee66e34b54272e0fdd2eb74c91e2eb66d4db7d061ddedb0b88f709fa9f027b00a5fbacb4f70f48612b11caf119866afea8a05de5b9e63b096e45634d7d68aef803cec2d21f91a55bea60be8abb233afe7dbbc415546945350432dc858064d42b82b0f0397fca660dddd45e9d95898d98d2fcbf2eb0c5de920aed36f3f56b511dd1ca5551a14b5d1b235be781c7c1a6f3297d4bccf83abb11bcaca33e12ea204c06849b39dae12e0cee817e0cbde1b1d1906b88cac1d96ed59c1377a0f7eac1a4efe9d2cafbcfd0c270f0d8311ac9839ad60cc834c1785e2796485dd844fa0694091206f8d3563830c2160a5510a41970da6cd5a0f810d36df27339db81152a7c2acc2f958f8d847ef2a9b18862eb2ebe6e2a635f905db4416d594d25268c61d163ee17d0b9052a6103d70139ec53b9f8328d5e3eed90541e494ad15da810b71fba9543198a4505e464608367a7fc83ae6ac1f7a63f3594d20452d928b86bbdccda5f9257f615eef5f05a7513cff8d074860980bca8873d387bb23894694650c9adad3ac1ba0ef617dbcec9946d874516131944e6e3a10eb017d696a20eaf9af5ffbc83b8f2db1e2cf82bfa1a2072523b844108d8fa9c980443c92121b2a2d47114b4dec3b0dbdab9ebd96295443d8a587682a044c2cc7ba72e9842ffb1156b7c774efdf0cdb5f96be7ad6cb1c361e3d7ee645c0c57dd23b4d247d940063cfa8a4afb72b671cc1d6f9037aac0cab8852d606d5616f41a3f8d7901176060a75e4c9e18d65cd2679fe5083c6e40e247c74f29b4ab9ea38827c7f4d1f1fac5a7409262374adcf69975be903a8561b83e37980567fd179e0f7eab72cb77f5fab2e44e49f8cb75f126b5d60cdbcacd00ab422d48d32d2db75442549ed7fe4b220ca969a9d8a4248cff3725f34cb64c6a59e73d8e841e5995075f32feec14d2954af1afec302d30d24214025e833f68676b1242f27dba75dd53426b46ea4fff0163ba0cce0e6e9ceecd5191199f5c9b867e62a4ab18781f7b440e6d9470ba05f09f0ed1062a745f4cb952ca862c91c3d3e88779f51c43bdc0973f03b5266f3296772fe19ff078ad2b7f237fcbbe47e4d2ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb07b1ed0a33d8f1fb025d2fdaa5af2801bdf2aff84d3b69bdbf6882b424803170ff6db19d9e05cff32d44dbb9c5d6c5e00683956df74a3db738668124f64928c5e10fe1b733def7f86b6532c9150286d71038490e719cd4dd271e883bcd278750140ba5b3dbd716a40d94a9eeed141ae4a6d007b8cfd43a175a6c862df720bd03f7c38706371ab73ab00489294f72c6b18eb470d8f395fac71672ae4b2dd9afc8cbb986aaab0a15954d29b323fa21c13772068078845159b7b52a5bf766cea25e2640fae059d8ee3361b7a08429867254523f937c7654ae6fe7b188c1f0da57e9cd54657374636f696e00000000000000000000000000000000000000000000000041207265616c6c7920636f6f6c20636f696e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000389e2f71e4e9bc3e37a8b3a5b49432f7a93574eced25fce9add210b41203ed42e414a3468188977dfa4f1bbb2e36dc1bce38e759b8a54c85bea6d75a2aeea008";
    const TX_HASH: &str = "722c8f5e8e02097b821c9c03be3165c3cecf2262f31cf2e31a10bada2fe1b033";
    const OVK: &str = "49bad8395ef448eb0048af132b5c942579024736d4c3cfd685b241b994f8f8e5";

    #[test]
    fn parse_tx() {
        let tx = hex::decode(TRANSACTION).unwrap();
        let (_, tx) = Transaction::from_bytes(&tx).unwrap();
        assert_eq!(tx.num_spends(), 1);
        assert_eq!(tx.num_outputs(), 3);
        assert_eq!(tx.num_mints(), 1);
        assert_eq!(tx.num_burns(), 0);
    }

    #[test]
    fn check_hash() {
        let tx = hex::decode(TRANSACTION).unwrap();
        let (_, tx) = Transaction::from_bytes(&tx).unwrap();
        let hash = hex::encode(tx.hash());
        std::println!("hash: {}", hash);
        assert_eq!(hash, TX_HASH);
    }

    #[test]
    fn tx_decrypt() {
        let tx = hex::decode(TRANSACTION).unwrap();
        let (_, tx) = Transaction::from_bytes(&tx).unwrap();

        let ovk = hex::decode(OVK).unwrap();
        let ovk = OutgoingViewKey::new(ovk.try_into().unwrap());
        tx.review_fields(&ovk).unwrap();
    }
}

#[cfg(test)]
mod review_transaction_test {
    use crate::ironfish::view_keys::OutgoingViewKey;
    use crate::parser::ParserError;
    use crate::test_ui::{with_leaked, MockDriver, Page, ReducedPage, Viewable};
    use crate::*;
    use serde::{Deserialize, Serialize};

    // testing data for unit tests
    const TRANSACTION: &str = "0101000000000000000300000000000000010000000000000000000000000000000100000000000000000000003475b26a991a739f6b77dd7bce822efa46d355a28cf0e4dc9c93b6ababf073c5ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb07ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb0795aef72203054fbfa24ffd1c375e6d69827111b74805477b38ef4932d143c6a44efc37619fbedc3a1b46d5613202ac6baf75caadb2ac6b6c9db0b02acf8698a1f51eb6a1a1dbebf7fa0b034803716d103c96e9893180cc971824bc2b978e1b1600ebd35de5d6b921b2ce8aa4c03ef7312c9d6efc7e259f4dd68d5c7b632a88bc259386faae6cea9e44e232a6cbf1079486a29e9d622e3c25e1985155226c4d48342be389600dc829f89aac5a81c444fd880f5b5ba1541b9434d620543a6e8be7eef40fe52914631d18a7d4dfb1c41beed6c2b7a51efd985f2e210059f36f6000ec8ee990b1228496c0d1140767f2aae76d79e09f5f777ff5af0f89ef6aefe7337805000045e9b744ed2afca6615aa6ba00dcf578979391b219ca05e90447abf21315a9be00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b9032b6225f92f518e019f07373891b6524df7bd3e25afc0b0fb7f5ca2da049c419860e0eb24aff16d830bc21061662393a76e80b2ef434d6f9bbfc3926644e933210d159074e80b368c16312710226160c38e0b05e69a8f353c5de3e0e6fd1e03892c58d7e8eeb5604aab425b445d86e9357341e2a175b8835b99653d75ba42c11500beeb61c90523a7dee736d73ab2a6e496b7aa8241ae8bd222475f3e0b510b0f5a57c5a97583d342d4c8740f66fb856d730caafa608296b2b02aa7bd4a99499b3cde309b75e0200df587d09952547f29046abc4fdd47714860d09f9cdcf3f3c24fd8d8fd0e0d2f83f1191ff59f37fa811d1a4c90faf70d4da89ef228ce6d8d0af19c715cae4d74fe07d3d66c473dff6799bc3d8979fafd5a9cef9b1d7650a1f488cd3478146e27075c1b1c73697ba64e94b37d13f29d22068ef58a7a721138213f218a11d1ab4b88d41420b40a010c9cbf2e10f00f7ae52dd54eeb9b716fac76db7d364b5cf43d278b61a0863234c03485364347505af9c86b8d582c453c1c98e52882d08e49d496791a899796ea3e5773435edc97caadffcc579341474cc31b87e6b19b0d526618585a6693184f903512ef641eda23f30e3a45782e95d1c3529230566b78aa1421834c58d0d44ab79467aff475858a38cbd67d4c4af4ed0cf677a063fea0b38b193ab6a18d81b3848f941d61b46bd86f348c613a9b0dc18610fae24d7328209168bb05d58d1085496a5a4769da339fa3ef1463956da25ccaff94a58beff6417d3f94d07eece27b8ca1db74ce29cce9a40556f8967d5ebb58da4efd01e4fb9b27578fc8e883320cff5f36a1b69706816cabdaff3ba9cb06756d8c37d760419d05bd9b8ffa84ed6ea98b952f0ea2e927ba0782e90a949b8821c305c20db46d66c00b48fc4150eeff0bc07a6fd890c80a802657573703a1b7aa9bd3b6eda9ad2adce2f162d1117aaf08093c779dd4db20c94a48e62357ade7daf5284d3b4d2f6885dc03a34b48aa2890ab4c60128769bf8bef2ee7a06f4d552eaed15c4df1da5ccf3762f37b818c9d62f7464416bc40f59c4f3c6299a929a67dd3d09e00d1bd68c16d38ee66e34b54272e0fdd2eb74c91e2eb66d4db7d061ddedb0b88f709fa9f027b00a5fbacb4f70f48612b11caf119866afea8a05de5b9e63b096e45634d7d68aef803cec2d21f91a55bea60be8abb233afe7dbbc415546945350432dc858064d42b82b0f0397fca660dddd45e9d95898d98d2fcbf2eb0c5de920aed36f3f56b511dd1ca5551a14b5d1b235be781c7c1a6f3297d4bccf83abb11bcaca33e12ea204c06849b39dae12e0cee817e0cbde1b1d1906b88cac1d96ed59c1377a0f7eac1a4efe9d2cafbcfd0c270f0d8311ac9839ad60cc834c1785e2796485dd844fa0694091206f8d3563830c2160a5510a41970da6cd5a0f810d36df27339db81152a7c2acc2f958f8d847ef2a9b18862eb2ebe6e2a635f905db4416d594d25268c61d163ee17d0b9052a6103d70139ec53b9f8328d5e3eed90541e494ad15da810b71fba9543198a4505e464608367a7fc83ae6ac1f7a63f3594d20452d928b86bbdccda5f9257f615eef5f05a7513cff8d074860980bca8873d387bb23894694650c9adad3ac1ba0ef617dbcec9946d874516131944e6e3a10eb017d696a20eaf9af5ffbc83b8f2db1e2cf82bfa1a2072523b844108d8fa9c980443c92121b2a2d47114b4dec3b0dbdab9ebd96295443d8a587682a044c2cc7ba72e9842ffb1156b7c774efdf0cdb5f96be7ad6cb1c361e3d7ee645c0c57dd23b4d247d940063cfa8a4afb72b671cc1d6f9037aac0cab8852d606d5616f41a3f8d7901176060a75e4c9e18d65cd2679fe5083c6e40e247c74f29b4ab9ea38827c7f4d1f1fac5a7409262374adcf69975be903a8561b83e37980567fd179e0f7eab72cb77f5fab2e44e49f8cb75f126b5d60cdbcacd00ab422d48d32d2db75442549ed7fe4b220ca969a9d8a4248cff3725f34cb64c6a59e73d8e841e5995075f32feec14d2954af1afec302d30d24214025e833f68676b1242f27dba75dd53426b46ea4fff0163ba0cce0e6e9ceecd5191199f5c9b867e62a4ab18781f7b440e6d9470ba05f09f0ed1062a745f4cb952ca862c91c3d3e88779f51c43bdc0973f03b5266f3296772fe19ff078ad2b7f237fcbbe47e4d2ad26e3f59270401ff48e7ba11d800eaea4d91dc89d4ccf0975afec009dcebb07b1ed0a33d8f1fb025d2fdaa5af2801bdf2aff84d3b69bdbf6882b424803170ff6db19d9e05cff32d44dbb9c5d6c5e00683956df74a3db738668124f64928c5e10fe1b733def7f86b6532c9150286d71038490e719cd4dd271e883bcd278750140ba5b3dbd716a40d94a9eeed141ae4a6d007b8cfd43a175a6c862df720bd03f7c38706371ab73ab00489294f72c6b18eb470d8f395fac71672ae4b2dd9afc8cbb986aaab0a15954d29b323fa21c13772068078845159b7b52a5bf766cea25e2640fae059d8ee3361b7a08429867254523f937c7654ae6fe7b188c1f0da57e9cd54657374636f696e00000000000000000000000000000000000000000000000041207265616c6c7920636f6f6c20636f696e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000389e2f71e4e9bc3e37a8b3a5b49432f7a93574eced25fce9add210b41203ed42e414a3468188977dfa4f1bbb2e36dc1bce38e759b8a54c85bea6d75a2aeea008";
    const TX_HASH: &str = "722c8f5e8e02097b821c9c03be3165c3cecf2262f31cf2e31a10bada2fe1b033";
    const OVK: &str = "49bad8395ef448eb0048af132b5c942579024736d4c3cfd685b241b994f8f8e5";

    struct TxFields(Vec<(String, String)>);

    #[derive(Serialize, Deserialize, Debug)]
    struct TransactionData {
        tx: String,
        ovk: String,
    }

    impl Viewable for TxFields {
        fn num_items(&self) -> Result<u8, ParserError> {
            Ok(self.0.len() as u8)
        }

        fn render_item(
            &self,
            item_idx: u8,
            title: &mut [u8],
            message: &mut [u8],
            page_idx: u8,
        ) -> Result<u8, ParserError> {
            use crate::test_ui::handle_ui_message;

            if item_idx as usize >= self.0.len() {
                return Err(ParserError::UnexpectedBufferEnd);
            }

            let (key, item) = &self.0[item_idx as usize];
            title[..key.len()].copy_from_slice(key.as_bytes());

            handle_ui_message(item.as_bytes(), message, page_idx)
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn tx_ui() {
        insta::glob!("testvectors/*.json", |path| {
            let file = std::fs::File::open(path)
                .unwrap_or_else(|e| panic!("Unable to open file {:?}: {:?}", path, e));
            let input: TransactionData = serde_json::from_reader(file)
                .unwrap_or_else(|e| panic!("Unable to read file {:?} as json: {:?}", path, e));

            let ovk = hex::decode(input.ovk).unwrap();
            let ovk = OutgoingViewKey::new(ovk.try_into().unwrap());

            let test = |data| {
                let ovk = ovk.clone();
                let (_, tx) = Transaction::from_bytes(data).expect("parse tx from data");
                let tx_fields = tx.review_fields(&ovk).expect("could not decrypt tx notes");

                let mut tx_fields = TxFields(tx_fields);

                let mut driver = MockDriver::<_, 18, 1024>::new(tx_fields);
                driver.drive();

                let ui = driver.out_ui();

                let reduced = ui
                    .iter()
                    .flat_map(|item| item.iter().map(ReducedPage::from))
                    .collect::<Vec<_>>();

                insta::assert_debug_snapshot!(reduced);
            };

            let data = hex::decode(input.tx).unwrap();

            unsafe { with_leaked(data, test) };
        });
    }
}
