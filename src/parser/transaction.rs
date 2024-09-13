use core::{mem::MaybeUninit, ptr::addr_of_mut};

use nom::{
    bytes::complete::take,
    number::complete::{le_i64, le_u32, le_u64, le_u8},
};

use crate::parser::constants::{KEY_LENGTH, REDJUBJUB_SIGNATURE_LEN};

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
        let out = out.as_mut_ptr();

        let (rem, raw_version) = le_u8(input)?;
        let version = TransactionVersion::try_from(raw_version)?;
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
}

#[cfg(test)]
mod transaction_test {
    use crate::*;

    // testing data for unit tests
    const TEST_TX: &str = "02010000000000000004000000000000000100000000000000010000000000000001000000000000000000000077b55de659bda8a
31c3627b9270139637c1fe55efc1cfc09dbb697d68d7583b6844339d824d73947aa6bdafac81c65910da301837980a57cbb1243c223192446494e0770fe825d36
eb783ecc0ecc8c80b1ebb8915ae9517d4070ba5613bc23d4a975beeab6dc63e153830eda6a755b4986fbdd4449724b87218045de587f5b6c0991c3a8f88ae5a67
a77a2f36fd63c58bc770a86760175b2af5213cd3a7bfebfa6b52508acc97a3f1b78e4be166aa6868f19995318ef578855546250957becf6ac79c1abb60962854e
61ef3c4d35eccc0ac10c06bb31881bf9335cca1318509b7cd1f38ed7168e444569ecb97a5fcfa416f61f5aada0cb73d185723dfe1ad7508e5a74e170838290db9
657ccbed94b2828d43b31b5abf89af0ed3a3c0caee460780500002f250c30c12c53a475af23d3d6419a6522e350f84d603e82f1b5aa661423c2d878cb1f72f28a
40854402380066a711300db6ccc70182a1231e57fee03f32e131630dddc58592b032a4e48cea93798c894b8d35b16495b4298326aff034d41700841ca42e0e443
e29bee9bd6b072b2181376acd607093fd18daaf33087c354aa3d02d56c8c4eb9a92e40db161d44d92e9b65efd9ff9fac0a42399e2d459e9d4e29abfb1c3fa0e9b
ab8d6aeb8177d2c73393f5501ff56171e7ec94f2ef7c8d1b4c18faf62e50930eb25d42bcea4e4a01c75d610e2770fc3e3a3eb1e7779bc7f4306fd485eae3547c1
2223cc1292ee6c6dd9610ef4fcebf08693a6a76363419e70326e1ac088466d4ed51e53c997abd606840492a7486730d72eb4ba6e4013006e91237100f692657ea
6f3a60de2a061e40b1798fdd49e6edb2f03f9719804dab4ea54080718898b0ef77fed41cdf0417de59db823d43fe8fbcb3360da6c24267264f28f8e485f81839a
0eef52322f67e56ea6a0d0f0f77e3a7f3de73ab92a67a82f7d00ff619aecb73c6e9960c8cf876ca3b242fcec567ab5066fea40e666eed3a9336ba9599b17b5483
d680fe57f51dabfecda43fe4e7ed600e549191697e3b9fef0937bedb680fe7daa93ba4b8523e0cbf2d442aa0105c97d66591c2cfe61b71396c1410ae819fe7c5b
6754ff9d855576c670b93242ab5f8188e01f31e567036eedc0aca8edb4e3f6fbc0606207a9f452199bfbb60dcf3c2e648ee47238c8ac602a23a316a20e51c4a04
d0a1770cf0e11c4773e7c83b1b904ddd4c07a250826730e29ca9bc335b46c58910f767675b6988a5edd8c0abd0bbb744003f0fdc1ea703d50ff141380c7c838cc
b12a2faf1041eac257261a306491ff098e3c29fbf8ea5ac909497cb00dafbcb1be53fe494a76926ee9428f8c5bd846e24c9e08aaa6aaa0de4c041c83adac515c0
661a8d757226fbc03270e9633ec1aa312be7165ede814bcc3ef9a468ac112aa175841118c8f20b8d268b5b8cbdee72913cc043c81f2c5058dbae899669428de41
3c1b811e03f327e6caaff02728af22c72227950cf5653822c160cbc2adc5075a4eaa2d46d201e01b8ef8a69d9ae898a395e9800143f68fe0627abbf70c65cfc22
8c403408b3931461f088e925dbd5b4d5ccb0ec98bd8847aa1801628b76a24734ce7dc77b2f470969c95fc0d110cc473a640ea3146d57b4d4d3931d3bdbd3851c5
0cf0982538b5f84eb889e89b2def189767d1e4d3b416b470636b10402183c7c711a2219783dc7c63dbac78335e42383d3f2495ae5eed81cb6ddc345a9c24449cb
3d6ab468f15ce2a24a829b568b5890172ec2ef58f3e895143926ccecb9d85141ee819207173dc2acebd97a89a23c0f9f63ae490f3dd52efb4c2b58cc4be254df4
8d8e5724e1f5e640b95c745508734be2f50ecb2100a09c1f7bb65accc6d54b1e2ef801bd39f677147566d61cd81041b76ff589dadb04a696ac3f1e2f904593231
abe4b0b1d933e80b20bcf65a54fdc1b39a986a2470cd17d003f5f6f52773b5e99cd55f823783758b46ef86e48caa3b2177e69abc4217be50444efc52dc9407a65
29aa514c577363b52eafed2c78e7fd81869f80b36577d513b319546756a80d9e829c4ccfe5f1abbcdb330d935a6e4ceb7eb98d8c8425ce7000743ba0b41793f82
00349afe93c0799fc4bfe3e3d381df69a90c700a51cac12b2456f56593457e7ffb0ec63680ec1846a7bc43761a566864d2f0854d419727d415b99c2d1a7abc460
f167541f30e822f977e022134ab8c24f9acb8a721da467e661fc745766553063ca6b880a19fcf929063ada26c6f38da00d0757a917c64759a90d92f3d4cb0fd78
b55197780aec0415695d78b55669953fd258b5f05b1dcb6eecc47b410cf7ed06c4e57cb8f34f180d36e218c24acfc7d73c8c8e5ffb2bc2c938bfaba539f3972a0
b9f515c8e7fbae327087364b4acaedde75c920cbf543efb17671204bd9f72437478cddad295ca5b70c44f8ed34d4d211ec18b65fb08a4ae5181e0a07deb45d0d2
7d2b864dd5557cf179d824671bc22d4e2017e4a9858b2fc62a525bc0eb2aa4a2f9d0196f13935502b977c4f56fc61f3932cde5f63b88206a59876923ec4a5169b
e61e75d4f7ecb1e1dd13e0c3798329aa890dddc6391e4a42948420b205c4752a0bd5feb016f1b78bf8dd5c5cae0696fc5697f4b47a4c0463e71e0d401612a2df5
b5ca68618a0032de25847b6f305d0051cb092cde82f17cf806088de4c8ff95d8a00b6fec4e88436d1608718905c2fdef39e87d7bf789dc079570625516bca69bd
7b6adb1dd1d9300c933fa4c74c33cd06178b095704cf4d67ceda6a397bf753918e0205f6041f714b4d2ed6a7063111cbe434ffa7018b71793352626346bf8e268
04716f261ef6a4b11d73359e19eb566b2f97d1831a12b30e091b779442a219f8444137ddda1691487939f61fb7819e9f885193ee54bc499edbb42b9abe48944fb
363e2aa01a26cf090b79ec2f113038d745656ed2f98af671640098dfae56769748ba96d6aed11c7e8d4e91b1f2bcbf8dfe6f3cb93caffd4c7bc24a3b8dddd6c75
5652b9a7c632876fb2eb9e28405295147c7fc39ad059f71b15a1d164594e272b36bede5ef080c4d3de38388fb8253067c1b5877290a66bde5cc994576d61fd319
93337224531e2626dac76104c70ea3bd10855756a16362466fdf8cacc64d4d96e9f5c5b07f2dfd53567a56d45d97bc9abd507705f6e3bbad3a99aa3e921ce8331
4e60b9ae34221348223d115e45fa1bea7ead2694c4898792b1e6467efaad90df2e6806eb454658e8236cc773bcc9fde10127b67cd5917dfd17a9cce7eda1c1adf
d0ebd43f93146f6d064dfa58a6e6b83737e8cffc4c717215edb22c5ca812adf1f52e9ed803bc2f0a303cbda0bf5175632b6c0fb384f873fcbb197d13440e4bdad
9a41475e4514234bcc4e2bfc482c153771a5eb22950b50e6370dc838b28457cd3eb41fe31a340270c6018d486580b69bd82f164e62dfd0e1b81653006b89861e9
1221f9ca5af0e2a35df8ea65c90c5ce20e299beae7cec8ec88c6e3b1bd30f8eec812166997102fa4ecff0c3e6287c66b5e4a629eceef9f7d225b4175697bd7421
acb8dc05e7891e3dc94dc13841feb3b0ad804a11d24896ce4cb07a46bf6888e4217c52e94c969cd58a0f64d134257326e81b57d9c1ffd0ade07f550a52ac96f2b
3d51eeb47e5325cbcfbd27df221b4286ffe49fd7491bcea88a0f70d3e312f4ccf7183928232fa29061e9b881fbc34c15d9f1a39ad069efde9e7fa892345d0dafa
21fca8408d720d6d79e0167e8ab13695f6d2c787545dad56f091cfb786e0f4459213497577e9cb9a54657374636f696e000000000000000000000000000000000
00000000000000041207265616c6c7920636f6f6c20636f696e000000000000000000000000000000000000000000000000000000000000000000000000000000
00000000000000000000000000000000000000000000000000000000000000000000000000000004050000000000000079e0167e8ab13695f6d2c787545dad56f
091cfb786e0f4459213497577e9cb9a00b00ad4cdfd5c66e1822c22acd26da08c21cb366632a03210c364d33d259deee66c50da30ba43bbcedcf1aff4370a0191
2069aca1355d360e5806653b8656820c6d9e3dd19d175b3eab6cb439d69f19b25a95b9f2c3270c23543578b3667c8ea20200000000000000280d529a652296987
bce8140f0d72d548ccec277dd4add9c5f378d56178f86cfa230822819e6ae9f1bdb2ec9c00dbc954dc52e0730157860dd41a666213fea03";

    #[test]
    fn parse_tx() {
        let tx = hex::decode(TEST_TX).unwrap();
        let (_, tx) = Transaction::from_bytes(&tx).unwrap();
        assert_eq!(tx.num_spends(), 1usize);
        assert_eq!(tx.num_outputs(), 1usize);
        assert_eq!(tx.num_mints(), 1usize);
        assert_eq!(tx.num_burns(), 1usize);
    }
}
