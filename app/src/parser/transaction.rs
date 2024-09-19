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

use super::{FromBytes, ObjectList, TransactionVersion};
pub use burns::Burn;
pub use mints::Mint;
pub use outputs::Output;
pub use spends::Spend;

// parser_error_t _read(parser_context_t *ctx, parser_tx_t *v) {
//     CHECK_ERROR(readTransactionVersion(ctx, &v->transactionVersion));
//     CHECK_ERROR(readUint64(ctx, &v->spends.elements));
//     CHECK_ERROR(readUint64(ctx, &v->outputs.elements));
//     CHECK_ERROR(readUint64(ctx, &v->mints.elements));
//     CHECK_ERROR(readUint64(ctx, &v->burns.elements));
//     CHECK_ERROR(readInt64(ctx, &v->fee));
//     CHECK_ERROR(readUint32(ctx, &v->expiration));
//
//     v->randomizedPublicKey.len = KEY_LENGTH;
//     CHECK_ERROR(readBytes(ctx, &v->randomizedPublicKey.ptr, v->randomizedPublicKey.len));
//
//     v->publicKeyRandomness.len = KEY_LENGTH;
//     CHECK_ERROR(readBytes(ctx, &v->publicKeyRandomness.ptr, v->publicKeyRandomness.len));
//
//     // Read Spends and Outputs
//     CHECK_ERROR(readSpends(ctx, &v->spends));
//     CHECK_ERROR(readOutputs(ctx, &v->outputs));
//
//     // Read Mints and Burns
//     CHECK_ERROR(readMints(ctx, &v->mints, v->transactionVersion));
//     CHECK_ERROR(readBurns(ctx, &v->burns));
//
//     v->bindingSignature.len = REDJUBJUB_SIGNATURE_LEN;
//     CHECK_ERROR(readBytes(ctx, &v->bindingSignature.ptr, v->bindingSignature.len));
//
//     if (ctx->bufferLen != ctx->offset) {
//         return parser_unexpected_buffer_end;
//     }
//
//     CHECK_ERROR(transaction_signature_hash(v, v->transactionHash));
//     return parser_ok;
// }

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
        let mut fields = Vec::new();

        // Add transaction version
        fields.push((
            "Tx Version".to_string(),
            self.tx_version.as_str().to_string(),
        ));

        // Now populate with outputDescrition::Note
        // for each note we render:
        // - Address Owner?
        // - Asset_id
        // - Amount
        for (i, output) in self.outputs.iter().enumerate() {
            let output_number = i + 1;

            // Safe to unwrap because MerkleNote was also parsed in outputs from_bytes impl
            let merkle_note = output.note().unwrap();
            // now get the encrypted Note
            let note = merkle_note.decrypt_note_for_spender(ovk)?;

            fields.push((
                format!("Owner {}", output_number),
                format!("{}", note.owner),
            ));
            fields.push((
                format!("Amount {}", output_number),
                format!("{}", note.value),
            ));

            fields.push((
                format!("AssetID {}", output_number),
                format!("{}", note.asset_id),
            ));
        }

        // Add fee
        fields.push(("Fee".to_string(), format!("{}", self.fee)));

        // Add expiration
        fields.push(("Expiration".to_string(), format!("{}", self.expiration)));

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
    use crate::*;

    // testing data for unit tests
    const TRANSACTION: &str = "020100000000000000030000000000000001000000000000000000000000000000010000000000000000000000f2a8a3d2067ef3a5fb4c1705e59e413ec48464510f7d537afa38e17cfe119dece5b3b1475844ebaa635ff75676655469e2bbb45f8cacfd01c917e1e48644180ee5b3b1475844ebaa635ff75676655469e2bbb45f8cacfd01c917e1e48644180e905739ec3e3a4c308f23e790670ccb3cab4d218907f8aba15fdbfe0bb9c9339a67970ecca424b9a203d2c03440d6553d84c63e944498d051ca462a80b455142758ac6ab06f9d7e0c298c82413b98bc2eda365f78a2c3cf5c409d5047ce48001600da574d78dfa28af9af57e34b74dc6ed6147c0b591590577e1583eff08e38e5add40b4ab6a9e66a4ad32d96cc996a1a949a00116587525a5e54b25e2243da84260166547e8460a954047ef7210c1234ea72aed7c09a956da160e4cf5a45a091a2bafd085ca35abc3e2f3da191721b2b0b70de51f7810879dd4bb40d40022395eea0dd547ce02b1a5135646cfadc0f7e4de4a5e2b04e399ecfe16271f535ad4078050000f2ee1341da2dd33b611980607072bed5d07eb84c9156c959738e0d8b71b1fc8b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008d5b139718ff6094c877e1bb26e512951c57f9c0d7987cf3bf126478af51180ab1e32511b7299c10b4bc2887febfb80dacf5953501634d4a86151b0cd2d6de3525f89803d6c04818730aea6a768de98a6c2c9d4e3b97c39747c268812128668616b8b7c535d9d98635baf0a97206932c7cdfd17c9c057d07e52989412a4db4de23362ffb00629c35c576f0fa37947143a36fe6d335f995b7932488ee2fc4bde90a01f53ab307ed82669d0488888716ffa04f86e0caacdff3538b7da3ca9fea3a181d0e8a366c5aacfeafecd2e9c3253ef363875af400028ff8be7784c2fbf4ba3f0719b2c985baf0878d84dd959e4bc85f99a9c005951f5eb7bcc7b4b6032a4609f11164a9bb2f5ae3cbdca23c2fff73734b67b7c9d377181b1c5c1f2ac8c41cd1fb49c0755ed42e8874dccdcb85a781a427b9e64a2fd0007d025b3142b2451158ce3b5c4b76d71fc1c8ef8821a11d8ecb26b86814265475ff407480a83709098e54a94c34175a9e82fef8e4aff9fbf9f1366aad094be4e2d915c1e3f67b37671aeea1aed4b2eadd279a3a03be8f15ad08ef2b92c1136b3c4d14cd5926edf1aa073fa9957df46b665ebab01995ea1b05953f34d4708395b5f6ad2fc60d4a396c07a7776fd76cc0745d64dfbed4c57885a1e0c2ae58dc377739bae7605f13f84079c3a3d47a5ba548803bd0475cb0fd09e61df5bada6da9c603e398b1fe5949c8c6a32c8c620b4a93b80684bb6d6e2c7b0c34adc5f5ec26fecccd17ad89dc20bfb1625af588980b05cf97c441aa75b76cca7f551b1a1b69f999fa68ae83cd1e312d28ace91318b88474bc7ec20bfd4298a1368bd239672b3688fb7a21cbf105e5fdaf62ae65ea87e90742a0135926116018839c73656f2a8237f4571fe504d292ccf1812274bdca2c6e40fec06df67a7515e604a4969ea9c580a45a0f69eb25aea035f16b83303c95577278a66daa81fd6295ff03df4911c397fd53d986ce4104ab1e18d111458af7ddb107170034e7d21e7ae588c4d25695bc4815a70a68c83798afabaf7a6c933fe2a60b5c73de8e2e5b70d0f56af3fadf15380e246c801830335ca8b7cbb1773f70d6e553f54992dd74c866a5ecfaa3f0b6f4a03539337f292a291c05a12966afb3f468546b6abdba338bcf694132ae8221372e509c4b21db40f9090b6d38573aef11e3e6c1ded0c53e5276ec644c318d5401dee601fea4310c22a45b0d9235a5e8ed221d2a4a557d63c34cf851bb7d986bfa8f51cad8198d879860e061a0fe091e8049bb1f4d93224dd950a22bb8079ecdaa441941741fd2e1c674bf063b8b1374c0f345ee87b840f91085d006a7d9ab0342b780e11e2e4555e156fff532beca54630938a93f06f7f9b28bfe2e25c7eec6d634b84da69eea487b3372ee31b2f5a82f4f407b2489cf76f533bae65c4576570e5c49fd01dee617cc84cd92bdd706ea6947a3d4049b9c8163f5e0bd280f8a0de806b9fcfe3fa6dd9561165de8c450e0c1caff52cf3ccdc349a2425d5c82797c3019e71cb1cce2ac8ae377d3a396a6dabdc588e2810f05218139d6977b8669f48a7631b6a7f9a87cb95a07e41c58bd07cf4cec5f6f5d4010b3801a8594624cb8af3c34fa6b0e0eb3bb31aa9ad533fb806a6cd9c373a7997835a52ee9359ffd9c5a0a778faf05f0a78c9f6ce9918a6fe166a18d120e846f09ddc9f721a180f9d1973d1cdeae0d5d512c768b4c1bdfdc7cebc8d1717d383c1139cd441a9c89bf73ee96303b731ca2fde2b29ce5476a5e6662f5033fbed10f9956a6f5c26e92ff383dd6cd6d624def6297938c98bf94cf57b3fd3c5d2d433c21d90679be5361fcbb161c075a6084a141aef843225ffe471956075b5cd956de35e30b48ac3d9f0445269e12becdc1e7c24a81c582217b65821cba3d35fe2de973e324e81c617dc2826402b4f76daa5bb06b0cb937690e1ad16c11e42292a3b8fb24d787e8907765a3a6cf2ee57cf4a7ea027c0690be44c9be7f86380e9182bae1b0b1a20ace661a6769687af98cc37bcaceecd21858c73cd6bc6ad8504ef9e4bb0d3140a67c9c95713c4f2358465a8e8d9b6e118a8761f0432ce7188eb9508cba4ce8c340dae435b5a6ccf409c5b9bb2f7ee2a4e163b246a8741ece6b6c10ca9423e7bdd80f67db1cebc55f4ccd76d2af23fc81c1e767387cfed993b12c5407e8082973dfc1bc49e5b3b1475844ebaa635ff75676655469e2bbb45f8cacfd01c917e1e48644180e841e3aa90ad740af652400c1698fd284116d086a2bc670ca87de9cc67299d4e63f9492b2175c32ba1f81dd8cae4bf6f888bf206959a4dadb6080ca1ff9e799728ed50b56f44aa0f8cb3d5b47d14ce772c115c92d47b1d89a5241038effdedc480446979c892d04f6161b2dfc11674f50ded792ec79ed66f8d3d022fa856fa7d20a6840ce11ed172af6343dee8fbaae07a9059bf3bbd946d8b2145d4d869cc011656d29ebe51bb056254f92e2e78c40c1a27c2199f84001c184a0e5c67b7fdb34c4397844dc126c9bd6793c28d3fe9ecb4b5aac7f22ca03398738053dc36457a954657374636f696e00000000000000000000000000000000000000000000000041207265616c6c7920636f6f6c20636f696e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000500000000000000c4397844dc126c9bd6793c28d3fe9ecb4b5aac7f22ca03398738053dc36457a90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c57171dbf307d8b01eed99fafa7b3b812c8fc4ce3246147d023f5baaf2408c1ed2218247824957148c71be7b59866083dab4b2b44ee0a58e745a4f0c8e10160e";

    #[test]
    fn parse_tx() {
        let tx = hex::decode(TRANSACTION).unwrap();
        let (_, tx) = Transaction::from_bytes(&tx).unwrap();
        assert_eq!(tx.num_spends(), 1);
        assert_eq!(tx.num_outputs(), 3);
        assert_eq!(tx.num_mints(), 1);
        assert_eq!(tx.num_burns(), 0);
    }
}
