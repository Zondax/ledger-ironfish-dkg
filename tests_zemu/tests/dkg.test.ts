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
import {defaultOptions, identities, models, PATH, restoreKeysTestCases} from './common'
import IronfishApp, {IronfishKeys} from '@zondax/ledger-ironfish'
import {isValidPublicAddress, multisig, UnsignedTransaction} from '@ironfish/rust-nodejs'
import {Transaction} from '@ironfish/sdk'
import {buildTx} from "./utils";
import aggregateRawSignatureShares = multisig.aggregateRawSignatureShares;

jest.setTimeout(4500000)


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

    describe.each([{p:2, min:2}])(`${m.name} - participants`, function ({p: participants, min: minSigners}){
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
            let encryptedKeys: any[] = [];
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
                        throw new Error("no round 2 found")

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

                console.log("publicPackages " + JSON.stringify(publicPackages, null, 2));

                for(let i = 0; i < participants; i++){
                    const result = await runMethod(globalSims, i, async (app: IronfishApp) => {
                        let result = await app.dkgBackupKeys();

                        expect(i + " " + result.returnCode.toString(16)).toEqual(i + " " + "9000")
                        expect(result.errorMessage).toEqual('No errors')
                        expect(result.encryptedKeys).toBeTruthy()

                        return result
                    });

                    if(!result.encryptedKeys)
                        throw new Error("no encryptedKeys found")

                    encryptedKeys.push(result.encryptedKeys.toString("hex"));
                }

                console.log("encryptedKeys " + JSON.stringify(encryptedKeys, null, 2));

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

                console.log("publicAddresses " + JSON.stringify(pks, null, 2));

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

                console.log("viewKeys " + JSON.stringify(viewKeys, null, 2));

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

                console.log("proofKeys " + JSON.stringify(proofKeys, null, 2));

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

    describe.each(restoreKeysTestCases)("restore keys", ({index, encrypted, publicAddress, proofKeys, viewKeys, publicPackage}) =>{
        test(index + "", async () => {
            for (let e of encrypted) {
                const sim = new Zemu(m.path)
                try {
                    await sim.start({...defaultOptions, model: m.name})
                    const app = new IronfishApp(sim.getTransport())
                    let resp: any= await app.dkgRestoreKeys(PATH, e)

                    expect(resp.returnCode.toString(16)).toEqual("9000")
                    expect(resp.errorMessage).toEqual('No errors')

                    resp = await app.dkgRetrieveKeys(IronfishKeys.ViewKey)

                    expect(resp.returnCode.toString(16)).toEqual("9000")
                    expect(resp.errorMessage).toEqual('No errors')
                    expect(resp.viewKey.toString("hex")).toEqual(viewKeys.viewKey)
                    expect(resp.ovk.toString("hex")).toEqual(viewKeys.ovk)
                    expect(resp.ivk.toString("hex")).toEqual(viewKeys.ivk)

                    resp = await app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)

                    expect(resp.returnCode.toString(16)).toEqual("9000")
                    expect(resp.errorMessage).toEqual('No errors')
                    expect(resp.ak.toString("hex")).toEqual(proofKeys.ak)
                    expect(resp.nsk.toString("hex")).toEqual(proofKeys.nsk)

                    resp = await app.dkgRetrieveKeys(IronfishKeys.PublicAddress)

                    expect(resp.returnCode.toString(16)).toEqual("9000")
                    expect(resp.errorMessage).toEqual('No errors')
                    expect(resp.publicAddress.toString("hex")).toEqual(publicAddress)


                    resp = await app.dkgGetPublicPackage()

                    expect(resp.returnCode.toString(16)).toEqual("9000")
                    expect(resp.errorMessage).toEqual('No errors')
                    expect(resp.publicPackage.toString("hex")).toEqual(publicPackage)
                } finally {
                    await sim.close()
                }
            }
        })
    })

    describe.each(identities)('identities', function ({i, v}) {
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
