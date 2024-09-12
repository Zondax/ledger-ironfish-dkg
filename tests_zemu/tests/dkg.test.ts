/** ******************************************************************************
 *  (c) 2018 - 2024 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ******************************************************************************* */

import Zemu from '@zondax/zemu'
import {defaultOptions, models, PATH} from './common'
import IronfishApp, {IronfishKeys} from '@zondax/ledger-ironfish'
import {
    isValidPublicAddress,
    multisig,
    UnsignedTransaction
} from '@ironfish/rust-nodejs'
import {
    Transaction
} from '@ironfish/sdk'
import {buildTx} from "./utils";
import aggregateRawSignatureShares = multisig.aggregateRawSignatureShares;

jest.setTimeout(4500000)

const identities  = [
    {i:0, v: "72510338227d8ee51fa11e048b56ae479a655c5510b906b90d029112a11566bac776c69d4bcd6471ce832100f6dd9a4024bd9580b5cfea11b2c8cdb2be16a46a2117f1d22a47c4ab0804c21ce4d7b33b4527c861edf4fd588fff6d9e31ca08ebdd8abd4bf237e158c43df6f998b6f1421fd59b390522b2ecd3ae0d40c18e5fa304"},
    {i:1, v: "7232e78e0380a8104680ad7d2a9fc746464ee15ce5288ddef7d3fcd594fe400dfd4593b85e8307ad0b5a33ae3091985a74efda2e5b583f667f806232588ab7824cd7d2e031ca875b1fedf13e8dcd571ba5101e91173c36bbb7c67dba9c900d03e7a3728d4b182cce18f43cc5f36fdc3738cad1e641566d977e025dcef25e12900d"},
    {i:2, v: "72b1d21580d6905b99af410bb19197bcbbb1f64c663381534de0e4ec969bad4a38779b7f70f21ba296a4a8a47a98bb704666cb1ee5030a501ec42206a45ecaf062e0b6e85ca7b78577b92d89069cd01e97e1f7f1e2674b6adcd8b2bab618a221c8ee5ce37c9cca2ad9ff541f3dfd935d81bdf669cb4a4cac5fd7dba05aabcd7801"},
    {i:3, v: "72d24c7990826ada6846d662de4a0f74be95d337279522ffe7205e2f4bfd1c4b149b1f45f39dae6f46ebe378cf7073f190d79bde8c81f2f9e9ac8817de8804992cf9d26bcf0b656f34992a8f538cd13142691e35de19116109515aa0d85e17774870fad8c83abe9499d4530137ef0eae22285601775db9f79587155b7a19823c04"},
    {i:4, v: "720b2b6343ba169e623afe44d7158175a2bd6717cea522e548d54f4b2928602465b1d2cd6d1852ac533a4fd3a610f3ded1c289fb215c84232f5def44a5c5ad1400317ba787935d40e17214f8f563491c5ac7b8d70dd3ab9e9844eb46c734d78ee5071f7d05a18617e938a338a295d1afa509411a8f716d934a83a637f7b4b81d0d"},
    {i:5, v: "7257d63a116b75136faf89eea94baafdfe5fbfb1ab43bb196dfe209844c2259d5582fe64191677eb38b64a9e182ed0184b219d66cc4c34f43cac72f23608155a0bf183a70c18af4659730d894a139c4ce29e52d4cab85596e75829569e74d94e08470700a4510949ef91a12dde01c6985bb93e0b80641b47ea6dc2c80f5d550f05"}
]

// ONE_GLOBAL_APP: Use this flag if the whole DKG process will run in only one app (all participants, all rounds). This takes precedence over ONE_APP_PER_PARTICIPANT.
// ONE_APP_PER_PARTICIPANT: Use this flag if the whole DKG process will run in one app per participant
// Otherwise, if both are falsy, one app will be started per request (each round for each participant)
const ONE_GLOBAL_APP = 0;
const ONE_APP_PER_PARTICIPANT = 1;

// Reference taken from https://github.com/iron-fish/ironfish/pull/5324/files

describe.each(models)('DKG', function (m) {
    it.skip(`${m.name} - can start and stop container`, async function () {
        const sim = new Zemu(m.path)
        try {
            await sim.start({ ...defaultOptions, model: m.name  })
        } finally {
            await sim.close()
        }
    })

    describe.each([{p:4, min:2},{p:3, min:2},{p:2, min:2}])(`${m.name} - participants`, function ({p: participants, min: minSigners}){
        it("p: " + participants + " - min: " + minSigners, async function(){
            const checkSimRequired = (sims: Zemu[], i:number): {sim: Zemu, created:boolean} => {
                let created = false;
                let sim: Zemu | undefined;

                if(!sims.length){
                    sim = new Zemu(m.path)
                    created = true;
                } else if (sims.length === 1){
                    sim = sims[0];
                } else {
                    sim = sims[i];
                }

                if(!sim) throw new Error("sim should have a value here")
                return {sim, created}
            }

            const runMethod = async (rcvSims: Zemu[], i: number, fn: (app: IronfishApp)=> Promise<any>): Promise<any> => {
                const {sim, created} = checkSimRequired(rcvSims, i)

                try {
                    if(created) await sim.start({...defaultOptions, model: m.name})
                    const app = new IronfishApp(sim.getTransport())
                    return await fn(app)
                } finally {
                    if(created) await sim.close()
                }
            }

            const globalSims: Zemu[] = [];

            if(ONE_GLOBAL_APP) globalSims.push(new Zemu(m.path))
            else if (ONE_APP_PER_PARTICIPANT) for (let i = 0; i < participants; i++) globalSims.push(new Zemu(m.path))

            for (let i = 0; i < globalSims.length; i++)
                await globalSims[i].start({...defaultOptions, model: m.name})

            let identities: any[] = [];
            let round1s: any[] = [];
            let round2s: any[] = [];
            let commitments: any[] = [];
            let nonces: any[] = [];
            let publicPackages: any[] = [];
            let pks: any[] = [];
            let viewKeys: any[] = [];
            let proofKeys: any[] = [];
            let signatures: any[] = [];

            try {
                for(let i = 0; i < participants; i++){
                    const identity = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        const identity = await app.dkgGetIdentity(i)

                        expect(i + " " + identity.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(identity.errorMessage).toEqual('No errors')

                        return identity
                    });

                    if (!identity.identity) throw new Error("no identity found")

                    identities.push(identity.identity.toString('hex'))
                }

                for(let i = 0; i < participants; i++){
                    const round1 = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        const round1 = await app.dkgRound1(PATH, i, identities, minSigners);

                        expect(i + " " + round1.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(round1.errorMessage).toEqual('No errors')

                        return round1
                    });

                    if(!round1.publicPackage || !round1.secretPackage)
                        throw new Error("no round 1 found")

                    round1s.push({
                        publicPackage: round1.publicPackage.toString('hex'),
                        secretPackage: round1.secretPackage.toString('hex')
                    })
                }

                for(let i = 0; i < participants; i++){
                    const round2 = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        const round2 = await app.dkgRound2(PATH, i, round1s.map(r => r.publicPackage), round1s[i].secretPackage);

                        expect(i + " " + round2.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(round2.errorMessage).toEqual('No errors')

                        return round2
                    });

                    if(!round2.publicPackage || !round2.secretPackage)
                        throw new Error("no round 1 found")

                    round2s.push({
                        publicPackage: round2.publicPackage.toString('hex'),
                        secretPackage: round2.secretPackage.toString('hex')
                    })
                }

                for(let i = 0; i < participants; i++){
                    await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let round3 = await app.dkgRound3(
                            PATH,
                            i,
                            round1s.map(r => r.publicPackage),
                            round2s.filter((_, pos) => i != pos).map(r => r.publicPackage),
                            round2s[i].secretPackage
                        );

                        expect(i + " " + round3.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(round3.errorMessage).toEqual('No errors')

                        return round3
                    });
                }

                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgGetPublicPackage();

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect(result.publicPackage).toBeTruthy()

                        return result
                    });

                    if(!result.publicPackage)
                        throw new Error("no publicPackage found")

                    publicPackages.push(result.publicPackage.toString("hex"));
                }

                // Generate keys from the multisig DKG process just finalized
                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgRetrieveKeys(
                            IronfishKeys.PublicAddress
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect("publicAddress" in result).toBeTruthy()

                        return result
                    });

                    if(!result.publicAddress)
                        throw new Error("no publicAddress found")

                    expect(isValidPublicAddress(result.publicAddress.toString("hex")))
                    pks.push(result.publicAddress.toString("hex"));
                }

                // Check that the public address generated on each participant for the multisig account is the same
                const pksMap = pks.reduce((acc: {[key:string]: boolean}, pk) => {
                    if(!acc[pk]) acc[pk] = true
                    return acc
                }, {})
                console.log(JSON.stringify(pksMap))
                expect(Object.keys(pksMap).length).toBe(1);

                // Generate view keys from the multisig DKG process just finalized
                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgRetrieveKeys(
                            IronfishKeys.ViewKey
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect("viewKey" in result).toBeTruthy()
                        expect("ivk" in result).toBeTruthy()
                        expect("ovk" in result).toBeTruthy()

                        return result
                    });

                    if(!result.viewKey || !result.ivk || !result.ovk)
                        throw new Error("no view keys found")

                    viewKeys.push({
                        viewKey: result.viewKey.toString("hex"),
                        ivk: result.ivk.toString("hex"),
                        ovk: result.ovk.toString("hex"),
                    });
                }


                // Generate view keys from the multisig DKG process just finalized
                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgRetrieveKeys(
                            IronfishKeys.ProofGenerationKey
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect("ak" in result).toBeTruthy()
                        expect("nsk" in result).toBeTruthy()

                        return result
                    });

                    if(!result.ak || !result.nsk)
                        throw new Error("no proof keys found")

                    proofKeys.push({
                        ak: result.ak.toString("hex"),
                        nsk: result.nsk.toString("hex")
                    });
                }

                // Craft new tx, to get the tx hash and the public randomness
                // Pass those values to the following commands
                const unsignedTxRaw = buildTx(pks[0], viewKeys[0], proofKeys[0]);
                const unsignedTx = new UnsignedTransaction(unsignedTxRaw);

                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgGetCommitments(
                            PATH,
                            identities,
                            unsignedTx.hash().toString("hex")
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect(result.commitments).toBeTruthy()

                        return result
                    });

                    if(!result.commitments)
                        throw new Error("no commitment found")

                    commitments.push(result.commitments.toString("hex"));
                }


                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgGetNonces(
                            PATH,
                            identities,
                            unsignedTx.hash().toString("hex")
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect(result.nonces).toBeTruthy()

                        return result
                    });

                    if(!result.nonces)
                        throw new Error("no nonces found")

                    nonces.push(result.nonces.toString("hex"));
                }

                console.log(nonces.map(c => c.toString("hex")))

                const signingPackageHex = unsignedTx.signingPackageFromRaw(identities, commitments)
                const signingPackage = new multisig.SigningPackage(Buffer.from(signingPackageHex, "hex"))

                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgSign(
                            PATH,
                            unsignedTx.publicKeyRandomness(),
                            signingPackage.frostSigningPackage().toString("hex"),
                            nonces[i].toString("hex")
                        );

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect(result.signature).toBeTruthy()

                        return result
                    });

                    if(!result.signature)
                        throw new Error("no signature found")

                    signatures.push(result.signature.toString("hex"));
                }


                let signedTxRaw = aggregateRawSignatureShares(
                    identities,
                    publicPackages[0],
                    unsignedTxRaw.toString("hex"),
                    signingPackage.frostSigningPackage().toString("hex"),
                    signatures)
                const signedTx = new Transaction(signedTxRaw)

                expect(signedTx.spends.length).toBe(1);
                expect(signedTx.mints.length).toBe(1);
                expect(signedTx.burns.length).toBe(0);
            } finally {
                for (let i = 0; i < globalSims.length; i++)
                    await globalSims[i].close()
            }
        })
    })

    describe.skip.each(identities)('identities', function ({i, v}) {
        test(i + "", async function(){
            const sim = new Zemu(m.path)
            try {
                await sim.start({ ...defaultOptions, model: m.name })
                const app = new IronfishApp(sim.getTransport())
                const respIdentity = await app.dkgGetIdentity(i)

                expect(respIdentity.returnCode.toString(16)).toEqual("9000")
                expect(respIdentity.errorMessage).toEqual('No errors')
                expect(respIdentity.identity?.toString('hex')).toEqual(v)
            } finally {
                await sim.close()
            }
        })
    })
})
