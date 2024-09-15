import { IDeviceModel, DEFAULT_START_OPTIONS } from '@zondax/zemu'

import { resolve } from 'path'

export const APP_SEED = 'equip will roof matter pink blind book anxiety banner elbow sun young'

const APP_PATH_S = resolve('../target/nanos/release/ledger-ironfish-dkg')
const APP_PATH_X = resolve('../target/nanox/release/ledger-ironfish-dkg')
const APP_PATH_SP = resolve('../target/nanosplus/release/ledger-ironfish-dkg')
const APP_PATH_ST = resolve('../target/stax/release/ledger-ironfish-dkg')
const APP_PATH_FL = resolve('../target/flex/release/ledger-ironfish-dkg')

export const models: IDeviceModel[] = [
  // Nano S device is not supported
  // TODO enable nanox
  // { name: 'nanox', prefix: 'X', path: APP_PATH_X },
  { name: 'nanosp', prefix: 'SP', path: APP_PATH_SP },
  { name: 'stax', prefix: 'ST', path: APP_PATH_ST },
  { name: 'flex', prefix: 'FL', path: APP_PATH_FL },
]

export const defaultOptions = {
  ...DEFAULT_START_OPTIONS,
  logging: true,
  custom: `-s "${APP_SEED}"`,
  X11: false,
}

export const PATH = "m/44'/1338'/0"

type ExpectedValues = {
  publicAddress: string
  ak: string
  nsk: string
  viewKey: string
  ivk: string
  ovk: string
}

export const identities = [
  {
    i: 0,
    v: '72510338227d8ee51fa11e048b56ae479a655c5510b906b90d029112a11566bac776c69d4bcd6471ce832100f6dd9a4024bd9580b5cfea11b2c8cdb2be16a46a2117f1d22a47c4ab0804c21ce4d7b33b4527c861edf4fd588fff6d9e31ca08ebdd8abd4bf237e158c43df6f998b6f1421fd59b390522b2ecd3ae0d40c18e5fa304',
  },
  {
    i: 1,
    v: '7232e78e0380a8104680ad7d2a9fc746464ee15ce5288ddef7d3fcd594fe400dfd4593b85e8307ad0b5a33ae3091985a74efda2e5b583f667f806232588ab7824cd7d2e031ca875b1fedf13e8dcd571ba5101e91173c36bbb7c67dba9c900d03e7a3728d4b182cce18f43cc5f36fdc3738cad1e641566d977e025dcef25e12900d',
  },
  {
    i: 2,
    v: '72b1d21580d6905b99af410bb19197bcbbb1f64c663381534de0e4ec969bad4a38779b7f70f21ba296a4a8a47a98bb704666cb1ee5030a501ec42206a45ecaf062e0b6e85ca7b78577b92d89069cd01e97e1f7f1e2674b6adcd8b2bab618a221c8ee5ce37c9cca2ad9ff541f3dfd935d81bdf669cb4a4cac5fd7dba05aabcd7801',
  },
  {
    i: 3,
    v: '72d24c7990826ada6846d662de4a0f74be95d337279522ffe7205e2f4bfd1c4b149b1f45f39dae6f46ebe378cf7073f190d79bde8c81f2f9e9ac8817de8804992cf9d26bcf0b656f34992a8f538cd13142691e35de19116109515aa0d85e17774870fad8c83abe9499d4530137ef0eae22285601775db9f79587155b7a19823c04',
  },
  {
    i: 4,
    v: '720b2b6343ba169e623afe44d7158175a2bd6717cea522e548d54f4b2928602465b1d2cd6d1852ac533a4fd3a610f3ded1c289fb215c84232f5def44a5c5ad1400317ba787935d40e17214f8f563491c5ac7b8d70dd3ab9e9844eb46c734d78ee5071f7d05a18617e938a338a295d1afa509411a8f716d934a83a637f7b4b81d0d',
  },
  {
    i: 5,
    v: '7257d63a116b75136faf89eea94baafdfe5fbfb1ab43bb196dfe209844c2259d5582fe64191677eb38b64a9e182ed0184b219d66cc4c34f43cac72f23608155a0bf183a70c18af4659730d894a139c4ce29e52d4cab85596e75829569e74d94e08470700a4510949ef91a12dde01c6985bb93e0b80641b47ea6dc2c80f5d550f05',
  },
]

// This test case was generated from the outputs of the device.
// This just ensures we are consistent to what keys we generated after restoring the keys.
// They are not meant to verify generated keys are correctly done, as protocol says.
export const restoreKeysTestCases = [
  {
    index: 1,
    encrypted: [
      'fdcb77a22614ef31a9865c700115294121651758ea20bf8fc8b5b8a1121283025dccf9c392f9e8c3b735cfb05423c3a81f8d9c97593ddacad7c82172d93c3f46e8c6b46d3c92b1a47250e69cd3957b261eb1ea207987c9c43aa6d55a57e204ec506ec6e4e1b02b4044b071699424526217dbd361244b186519904e56b9dc090c819965345b30e9bf83101655f8def61d594af4d8ccf408eaf528fbb3bb87650ff9976904f0c0342143f59d889af332e72fdc0b6d87db589a3dfb2b9a39e281a6f067ca73b219464444a92fb704d513d8f6575fb23b55bf6a7c588cea0e4a9f133249df18cad4762d0fc5c650d074fc42a5efac48a7922c44576202d9b69b0d4e5faea726a1130e16c5ae2f33aa9b0b5de123e67fe5a9def5aefa901bba80381fad7a1af2597a433cbc63e784375cea6d54fbe6e789c9e0015a5c1fe4a5adbc8e2e5d13999eed5649a664050eaed3315e400e3fc4061dd6714428d8ff53654864646242719fbee9e229a3c9ee05cc45683d5ffd3f41d77f86f2fd1dfac15b8f030c759110218d719a654f68eab55f247019c4d10f87c1077eed86c39dd2d83df291f1fb1c39df824ed61d866adc99396b88680a1925b6001f914afea854130c3a582eaa842af1f16370c9324ad8824ca2cf4b2c80c325f68aa56f628dc241f8c2c0fdec2b912615f637b57732da51e807f7757bd2e21e87724860c3256a2be8be14a2802292bfb0b0d8c7da50fe7fb1ac8f302f0277cd4db9af1746e54c62363741951be15ee700c68c906c923813b9dd4a1da4c102aab440018233a20e849aab7777ad01b6534086fafcccf98be9080e34d18771c53ddf3e725ec811eaff734ca5801fa69ea47f990b4f9cab96faf6511bc0ae9cef682c42d04bcbc282c67d00237774e666f85a8527cd70a906682207110bc20dc97d2e715336d7d685a4bc2e1fef4816bbd4753431df38559a706d1ad1224aa21fe79aa05d0a16b0325abf463ab277bbf5b7b86976266b503af99c79ca62f196c9896f21d8e70036073b5871d9b4d5f6fe17e7c0188c6791b8486347c04649ddc8ee28c28f2ce0bb6dcfdaf12af4ce7d7721b0f8e8735acc3f76abd8b3d154cdda60f66a823731e476d7f51004b5937750d2528f3e7b70065eba595923d56149ed75f855a2245f8d775b8ff8f016447e6030eddd779ef0122355ba7f514620e51972e958c8bb01d2cb38830145e2c709e8582e71443833d87eed24605b5b6da9736c71754c5d69561087addec794189f21a266b1955ded462a86143991a3f1aa3e3e532b61024183aa9cd5b54bf34db3617a65bfba533f8b84bf54fb61a1e28a7475d3199d1d5c297d37bdb7b33a7b38721fc2c5e1976612d43909aec346fd34300c883b1a6b8f89229cae95d2e1bd7633b90c2f5fb978b211c788e4edf04c5fad5c1ef2c40e543e85711173665857671adaa2444b324228',
      '75737125db317f8ddff74be74043b7b5bccd7d6222d50c54d1e1c1b8966def6cb1b23012f9751572f8f4dc2933183e062d5944170e5329a6e046261da6fcfb46c46ddbe95d6c5602922ac6d6b8e1ec0f62766d76391bf815dfa978e00d33cf53f00aaa050148d15a8d7551491394bf2e4c9df2e0a5ebdd54eef2b67aac88d1db32e38ac5fc83a8139ef0cde91e1ea8894090aab7d37c266a91aabbcc53f5369f43f46587ea693e6172c762300f1f11074dd37b228037061e2dfbcf5cef7ef89be148e6b76315ebe484d959e72119600be000d1a47fd0cfb6f33e0f38347453a675dc22012e58584dc3b0768f3755d1d9ced5779e394bdfcf04fb1b7fd6d836466c7e5f09bc1c140b148acd92c0b1ab3ec85f7cb21d8c6e2a764f70d8cfe15ec0bd7ca019e558eba7b18459dd28f2b8890634ebc171fa95f9c793a0917ca1b02107065b81f71f3e10e5e706aeb8238c1db99fa0e480b1469f7e3761edc4d734bc98bb73eda85563f570f9875ab13bc1910d7d4970da8b2cb7e7cf9089e05b0d1173cb617725ae68fb59e2b67b1568cb4e6a97607788f943a9360c50c654f4b3fce84d061a35101f7a9d7f1d9fae7ce38cb295de78e166ab7465d191046b7d38579dc446bd0068367412804d874096d9237bf7e09614773a540ac4f504eb9a01757b08d15def64b98d3c6ccda73003542c267ae4c0adab7e009978dba7518d0f25b52eead6e660db76a76d8d9cb178a7007c0b19ac5d91fe320831d5515cbe1ceb3c8aaf012a3d2f8089eb7c2102ec67d043a01efb4d54136ec8c3dfc080cf72d40ed8bf33973bee70808562f9356abbc833eac3929fd43ef4db46ebfd3c758cb22f38ce0bbdaf8c55a4434b2328c78acd6959e2f3c9e3e518c7fc49ccd4280bf22f2587e7f7fe43431c72e07527689bcdd919098d0e4d4f391936620a1fdbdf28a6e01abd3aae4700b8a018ee24fccc2240a22293f14b457b446dfa6e3c1357828a0b9422f35363b82d78e1d5b178a6227a5c8c0c691f25b88ba19fa9180db5b6da23f98d88e9a68b7cf8d3c5b0580a12828b4741ac9b547889a32e5f158e77556999990d0b41845ce27a30f90f6c83a86db4dc9ff6829184878b5aee131d9a19c4cf4b60607b33de139775c050b581d321c48c0c0f535bdd21c80f4ef06bf443d219ddcaf1d8780684b7658bacaedbcac12a90cd1fb3c1fd91366f7338c1c5bc91130e8dd8cb5234aeb8305697c996bb363de9294547daa8dae93104704760f34ec79a83a4c2348d7a42a86df0ecaac5200daacfad8bf34abca2f904d9bfcaf29a62b5ae92b92d372a417d813c876603d0e4256087861f7f43d2d954c8bc83d976e2a027625a47ac4d23e00cd30e3b111ead1eabe798bc3eb5db21cbd3ee527c9be993dd19ae90512ca260d3ed28a9dcfff51a18e5cd59184abf11e6ed36f6e57b720c10868d4487d82afcb9',
      '177a700267cfe34ea3241a96ff08f181c62db08ef4d260f6ddbb39a9ddb5411dfa50cac3a5be70fa362623cefa2b0dbb4a9f043359002f7afe6fceac57c9bdc95886eb2b5e6badedc1c96179a36e35dc5871ded3916a748232a5733d3d8edc424a7679ec4da30227ecbb67a1f5748d4f0ec8b78b658784eea191cdca92a7b336e3185c1cf0739b3a036b98396b066519b2874518c9de142c6e41e436f2f3f4cd72155eaecbec55039aa211bd5a03eb3f019631d30dfd088dca0ed05835f2e77cb6238edf47971d328de740f3a07cbc9dbbdf92ec81b9c12a0956694a7833be2d27d3696881a09b58852cfd990dd5eba47540f51cbb56a7037e99f7335918454cb841fced81445849f18525726eebd7b823944514b8af527dd48c19d02fc9337cfbaddd475018cb54afe48e8edbc00f57aa27a036435b441842ad6268c19d3035edb3635c3dba5f6969c418bba07d59b8a9dcba00afe4ac0a8412e83db312af420548e9a7b022cb29f0f6d7b60960b74567c5a0259c1d0c9b054572f7a1559f4bd37244322b9c37631428c22dc045f6682bc9ef21e3434b622da6109b3416e0842ab2f04d757d4c1dc8a18a8d486cef15ce664d4a0860d461085b8181a2ff5ec871c66e5526734ea256e1a88f6cd4a7d8a17e87b879c374ebc6ebff9556e1d2d8cff7bf5e2a4c98512519219958ab42add10cceb734d3f875f14201988eb6786f8e923c63778792adf0406505c43e776fa97b9d4e3f98bf3b955139ca79935681e3fda83f4ac55f7f149cf8a6df43806101c69d03b3813908d3038e4d529e13704aca89b96ce7e698edfff6ec8411224f605b2d7c8095638f99225461795827f55b6129cd11240f240b5933615445636ddca100c6a88ecde483cefdbc40e0983b1084d51678133f5413f9d59e1bc3fcf5e85857725e6f4d44abf0274cfb8a9fad21193daf8758261c5fc273c2f67599b776a0d70ea18a31198fa1b4eae3e111fb3047315d6227e0f6b61445d7613ec57572c699425c31796a09d0308a69a2b5c478b0320f80758cb4e74adf7d6daf4cf3731b0d3ea4066ad7c9e6decbbd59125a8c321af5bde57d80888c4b921d4684067674acf191a7870876488c5facdc9a76a4b4e2ac7b2726a72ec4c81d322abe668e90b1c03594e1824e14e8caa69b61042cb1b17e8a0166d7e529148ca10ff55d5e6719a6469a90bb63d8e350f98f7d42ae0cbf3978970d60adf9be485f3631ab384cd3a17411e46414f074b07ec162b9f60133e140a44846d499b9c5c9ba376acd14fa521f892fd9bc2847a8291cbb397ab8450d549acedd435de7fb5895baf528b1bc373e2419ead194c51f804ca2c9d28f5858f9705e1ba992165930b32daf9a4c969dc51ff0be3d8e6266c0b3b50039a88877ab2f22d529afee9b4bb4ddd46d5ac003aabda2d0383155b9d4f3511c1274521edb45cc2547a42b59',
      '969333a729ae83ee43348114ef6c9addbe91dc0e40655925101757daa38139312e4ed5cb5e375d6d73fcd78f697ee9bb7139f95a7b88659823b328513a2a8979a30afd6040336f02a695d8847b8fb900c7d30e57547fe988beb12de8f38f9edbf463f06b4f1e5d275765b8532cddf6f31cf9237e0167a9292240c938ebba6d8d8c229c7a06b9e707510be9f9b084f56645bdbb42473a04d6c957880974d5223d6ed25ca25730ab6b5969c7d71c0a866041d4a77becdc672db900a70e4619d93147cc443b7e2ef8197388cbc1913da2e6bd85fecdddb23c5b97ad803eee20f97f9ecbcac42da19b163ad64d4cbd3bcf32d29bec80b36c7b3bd20db19b468fe734be2e07d042fc25282ade36ee190959319c36299f5529187a0d0c831ddb9c29002f74676784a249d56d6208b4a84c7e9ef07ec6e87fefff58d6e355da5be82ca84f636bd865d2ca8c8a6d200541eaf17a989ab4c9212dace07bd05fa7c56f68326b7b1e9d9d1af52bc13d8193d97f98a01512be9457923fe0f1ec765ebb2e9cf2eb59792bc183ed825dd6f86042a08b64e1a6b37754bc30d360bbf0adae572b155075404131443b646ba4a3f7e67354c8c06f778a6bf435327d2a15e50ea9a09606c6ba6d9f928537e6bebfcf1a41fb7c9025a83e021a06f275819c8ce1f9e20abc8af6b93f6eb326bc3fd738834b97cac3220df38a43ae4fbe2974d1c8ae117cc55c0cfb730a4181ac9bc4cf833ab5c9b990ba3be1d4b7ebf4b634a45e1264c8efa8a765a8d6d6105ef45584735a35c2ef8eefa7d4ea929cbfec70d3c660efd0fc9896580929186085ec3985638a54e4906e889d7769c207b2614f02fbdcc6ff32fd1dad51d962ca3d7bdf8865670fec7d3fdc2fb158aab1f84001f36acbab0066623e11fc19b2f1c13387d81fb462271fa0a340b650f24863a1ee4ec64f07e0977fee7ed6150e1785bbb2d709a74bf3789e09a32cb307fce511dfcdac2b336b5d203295d56fce8baee52592949643ed6ef1851c599d255710f1449189c097aa3028c73fcac0c20a84857606996e018e7b9ac754cac0c50d8229c57f6e18e2124f9280dffae31553c58d61a3134d1319fe81ede3e18d94aa9673c49d79d67db70275b5764f433b453b53c99c1b6decb9fead6480a8f3f391d2ec9ed82aaf6a2242b3f833f56efd4643a7e60079d62b7a199a799657ee4473296561e16ef639a0d8be186364244f3b5d3bdb5db46326062119ae26a174395d18a7ecc94f9f1600d8e93f74a25ff4466113bc09ebf6ce1abc3d39fd61e6ba734734206efb8dc5054ab2aeda8f9401fc7de84fc3769fbdd3cbd0329d693e05a63e847b5e622c5f79553dd313bdb694e7588e3a4dd1ec32099dfed12d7890ec4f09ecdc4e9b3fe302b1c8a44830049b411f05ab61b6ca49bf72e84c082deccc1d258ea646802174fa4dc413deb7f9f569bcfea014',
    ],
    viewKeys: {
      viewKey:
        'dbc837e839522217a348fedf16cde63dec0806c9396cdd5027c85736889d6817fb92659f6aaef3c86653bc773a5250fb63db56ca061b90b62273a7130c8c38ca',
      ivk: '88c9f814dd793038b4d571b39263877953590f72c5fcf28e75c44d563a76fc01',
      ovk: '49bad8395ef448eb0048af132b5c942579024736d4c3cfd685b241b994f8f8e5',
    },
    proofKeys: {
      ak: 'dbc837e839522217a348fedf16cde63dec0806c9396cdd5027c85736889d6817',
      nsk: '9c8cd4749b1ac2534731da75112579feb2ea90babf8b0a9574d86e0720d4380c',
    },
    publicAddress: '40fae059d8ee3361b7a08429867254523f937c7654ae6fe7b188c1f0da57e9cd',
    publicPackage:
      '2601000000c3d2051e0418502bad754d59ebeef12f9e9c5825c6f05095f1f17a1168a82c9bfed9f50505c58b788ebd0b8a92bb509c8981122deaaf20a5aa5a985d2a15fcfcd494aa6b31dfb34bf3d7dd613ec1fd883006684d9b6b75615215bb101827367e7965061f05974e1c5b95220cd3ac8afa3506c6c91c89c09e0412c7bcd17ba6195bdad9c0e7596542d703696faacac5139e4466e5d5c2344159a5ff2cdf4df8156a9ad3520a585fa5318a6a340c16f20ea125153a9ede32d31497d442fd033f2a4e0133362da5beb1a31fbb2d3aabed3f46768d15a0e1c0d87aad1f50873c93aaa145acf00c9375cbf349db4b8afa63d0350fadfcd1135ee8029a3831fec3493211a24b7787dbc837e839522217a348fedf16cde63dec0806c9396cdd5027c85736889d68170400000072510338227d8ee51fa11e048b56ae479a655c5510b906b90d029112a11566bac776c69d4bcd6471ce832100f6dd9a4024bd9580b5cfea11b2c8cdb2be16a46a2117f1d22a47c4ab0804c21ce4d7b33b4527c861edf4fd588fff6d9e31ca08ebdd8abd4bf237e158c43df6f998b6f1421fd59b390522b2ecd3ae0d40c18e5fa3047232e78e0380a8104680ad7d2a9fc746464ee15ce5288ddef7d3fcd594fe400dfd4593b85e8307ad0b5a33ae3091985a74efda2e5b583f667f806232588ab7824cd7d2e031ca875b1fedf13e8dcd571ba5101e91173c36bbb7c67dba9c900d03e7a3728d4b182cce18f43cc5f36fdc3738cad1e641566d977e025dcef25e12900d72b1d21580d6905b99af410bb19197bcbbb1f64c663381534de0e4ec969bad4a38779b7f70f21ba296a4a8a47a98bb704666cb1ee5030a501ec42206a45ecaf062e0b6e85ca7b78577b92d89069cd01e97e1f7f1e2674b6adcd8b2bab618a221c8ee5ce37c9cca2ad9ff541f3dfd935d81bdf669cb4a4cac5fd7dba05aabcd780172d24c7990826ada6846d662de4a0f74be95d337279522ffe7205e2f4bfd1c4b149b1f45f39dae6f46ebe378cf7073f190d79bde8c81f2f9e9ac8817de8804992cf9d26bcf0b656f34992a8f538cd13142691e35de19116109515aa0d85e17774870fad8c83abe9499d4530137ef0eae22285601775db9f79587155b7a19823c040200',
  },
]
