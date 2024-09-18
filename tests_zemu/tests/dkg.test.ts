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

import Zemu, { ButtonKind, DEFAULT_START_OPTIONS, isTouchDevice } from '@zondax/zemu'
import { defaultOptions, identities, models, restoreKeysTestCases } from './common'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'
import { isValidPublicAddress, multisig, UnsignedTransaction, verifyTransactions } from '@ironfish/rust-nodejs'
import { Transaction } from '@ironfish/sdk'
import { buildTx, minimizeRound3Inputs } from './utils'
import { TModel } from '@zondax/zemu/dist/types'
import aggregateRawSignatureShares = multisig.aggregateRawSignatureShares

jest.setTimeout(4500000)

// Not sure about the start text for flex and stax, so we go with what it always work, which is the app name.
// That is always displayed on the main menu
const startTextFn = (model: TModel) => (isTouchDevice(model) ? 'Ironfish DKG' : DEFAULT_START_OPTIONS.startText)

// ONE_GLOBAL_APP: Use this flag if the whole DKG process will run in only one app (all participants, all rounds). This takes precedence over ONE_APP_PER_PARTICIPANT.
// ONE_APP_PER_PARTICIPANT: Use this flag if the whole DKG process will run in one app per participant
// Otherwise, if both are falsy, one app will be started per request (each round for each participant)
const ONE_GLOBAL_APP = 0
const ONE_APP_PER_PARTICIPANT = 1

const SKIP_ERRORS_IN_PHASE = true

// Reference taken from https://github.com/iron-fish/ironfish/pull/5324/files

describe.each(models)('DKG', function (m) {
  it.concurrent(`${m.name} - can start and stop container`, async function () {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
    } finally {
      await sim.close()
    }
  })

  describe.each([
    { p: 4, min: 2 },
    { p: 3, min: 2 },
    { p: 2, min: 2 },
  ])(`${m.name} - participants`, function ({ p: participants, min: minSigners }) {
    it.concurrent('p: ' + participants + ' - min: ' + minSigners, async function () {
      const checkSimRequired = (sims: Zemu[], i: number): { sim: Zemu; created: boolean } => {
        let created = false
        let sim: Zemu | undefined

        if (!sims.length) {
          sim = new Zemu(m.path)
          created = true
        } else if (sims.length === 1) {
          sim = sims[0]
        } else {
          sim = sims[i]
        }

        if (!sim) throw new Error('sim should have a value here')
        return { sim, created }
      }

      const runMethod = async (rcvSims: Zemu[], i: number, fn: (sim: Zemu, app: IronfishApp) => Promise<any>): Promise<any> => {
        const { sim, created } = checkSimRequired(rcvSims, i)

        try {
          if (created)
            await sim.start({
              ...defaultOptions,
              model: m.name,
              startText: startTextFn(m.name),
              approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
              approveAction: ButtonKind.ApproveTapButton,
            })
          const app = new IronfishApp(sim.getTransport(), true)
          const resp = await fn(sim, app)

          // Clean events from previous commands as each sim lives for many commands (DKG generation + signing)
          await sim.deleteEvents()

          return resp
        } finally {
          if (created) await sim.close()
        }
      }

      const globalSims: Zemu[] = []

      if (ONE_GLOBAL_APP) globalSims.push(new Zemu(m.path))
      else if (ONE_APP_PER_PARTICIPANT) for (let i = 0; i < participants; i++) globalSims.push(new Zemu(m.path))

      for (let i = 0; i < globalSims.length; i++)
        await globalSims[i].start({
          ...defaultOptions,
          model: m.name,
          startText: startTextFn(m.name),
          approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
          approveAction: ButtonKind.ApproveTapButton,
        })

      let identities: any[] = []
      let round1s: any[] = []
      let round2s: any[] = []
      let commitments: any[] = []
      let publicPackages: any[] = []
      let encryptedKeys: any[] = []
      let pks: any[] = []
      let viewKeys: any[] = []
      let proofKeys: any[] = []
      let signatures: any[] = []
      let error = false

      try {
        for (let i = 0; i < participants; i++) {
          try {
            const identity = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              const identityReq = app.dkgGetIdentity(i)

              // Wait until we are not in the main menu
              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-p${participants}-m${minSigners}-${i}-identity`)

              const result = await identityReq
              expect(result.identity.length).toBeTruthy()
              return result
            })

            if (!identity.identity) throw new Error('no identity found')

            identities.push(identity.identity.toString('hex'))
          } catch (e) {
            if (!SKIP_ERRORS_IN_PHASE || i + 1 === participants) throw e
          }
        }

        for (let i = 0; i < participants; i++) {
          try {
            const round1 = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              const round1Req = app.dkgRound1(i, identities, minSigners)

              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-p${participants}-m${minSigners}-${i}-round1`)

              const round1 = await round1Req
              expect(round1.publicPackage.length).toBeTruthy()
              expect(round1.secretPackage.length).toBeTruthy()
              return round1
            })

            round1s.push({
              publicPackage: round1.publicPackage.toString('hex'),
              secretPackage: round1.secretPackage.toString('hex'),
            })
          } catch (e) {
            if (!SKIP_ERRORS_IN_PHASE || i + 1 === participants) throw e
          }
        }

        for (let i = 0; i < participants; i++) {
          try {
            const round2 = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              const round2Req = app.dkgRound2(
                i,
                round1s.map(r => r.publicPackage),
                round1s[i].secretPackage,
              )

              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-p${participants}-m${minSigners}-${i}-round2`)

              const round2 = await round2Req
              expect(round2.publicPackage.length).toBeTruthy()
              expect(round2.secretPackage.length).toBeTruthy()
              return round2
            })

            round2s.push({
              publicPackage: round2.publicPackage.toString('hex'),
              secretPackage: round2.secretPackage.toString('hex'),
            })
          } catch (e) {
            if (!SKIP_ERRORS_IN_PHASE || i + 1 === participants) throw e
          }
        }

        for (let i = 0; i < participants; i++) {
          try {
            await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              const {
                participants: ids,
                round1PublicPkgs,
                round2PublicPkgs,
                gskBytes,
              } = minimizeRound3Inputs(
                i,
                round1s.map(r => r.publicPackage),
                round2s.filter((_, pos) => i != pos).map(r => r.publicPackage),
              )

              let round3Req = app.dkgRound3Min(i, ids, round1PublicPkgs, round2PublicPkgs, round2s[i].secretPackage, gskBytes)

              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-p${participants}-m${minSigners}-${i}-round3`)

              const round3 = await round3Req
              return round3
            })
          } catch (e) {
            if (!SKIP_ERRORS_IN_PHASE || i + 1 === participants) throw e
          }
        }

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (_sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgGetPublicPackage()

            expect(result.publicPackage.length).toBeTruthy()

            return result
          })

          publicPackages.push(result.publicPackage.toString('hex'))
        }

        console.log('publicPackages ' + JSON.stringify(publicPackages, null, 2))

        for (let i = 0; i < participants; i++) {
          try {
            const result = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              let resultReq = app.dkgBackupKeys()

              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              try {
                await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-p${participants}-m${minSigners}-${i}-backup`)
              } catch (e) {
                // TODO navigate and approve, but do not compare snapshots... needs to be added to zemu
                // Skip error, as a new public address is generated each time. Snapshots will be different in every run
              }

              const result = await resultReq

              expect(result.encryptedKeys.length).toBeTruthy()

              return result
            })

            if (!result.encryptedKeys) throw new Error('no encryptedKeys found')

            encryptedKeys.push(result.encryptedKeys.toString('hex'))
          } catch (e) {
            if (!SKIP_ERRORS_IN_PHASE || i + 1 === participants) throw e
          }
        }

        console.log('encryptedKeys ' + JSON.stringify(encryptedKeys, null, 2))

        // Generate keys from the multisig DKG process just finalized
        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (_sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgRetrieveKeys(IronfishKeys.PublicAddress)

            expect('publicAddress' in result).toBeTruthy()

            return result
          })

          if (!result.publicAddress) throw new Error('no publicAddress found')

          expect(isValidPublicAddress(result.publicAddress.toString('hex')))
          pks.push(result.publicAddress.toString('hex'))
        }

        console.log('publicAddresses ' + JSON.stringify(pks, null, 2))

        // Check that the public address generated on each participant for the multisig account is the same
        const pksMap = pks.reduce((acc: { [key: string]: boolean }, pk) => {
          if (!acc[pk]) acc[pk] = true
          return acc
        }, {})
        console.log(JSON.stringify(pksMap))
        expect(Object.keys(pksMap).length).toBe(1)

        // Generate view keys from the multisig DKG process just finalized
        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgRetrieveKeys(IronfishKeys.ViewKey)

            expect('viewKey' in result).toBeTruthy()
            expect('ivk' in result).toBeTruthy()
            expect('ovk' in result).toBeTruthy()

            return result
          })

          if (!result.viewKey || !result.ivk || !result.ovk) throw new Error('no view keys found')

          viewKeys.push({
            viewKey: result.viewKey.toString('hex'),
            ivk: result.ivk.toString('hex'),
            ovk: result.ovk.toString('hex'),
          })
        }

        console.log('viewKeys ' + JSON.stringify(viewKeys, null, 2))

        // Generate view keys from the multisig DKG process just finalized
        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)

            expect('ak' in result).toBeTruthy()
            expect('nsk' in result).toBeTruthy()

            return result
          })

          if (!result.ak || !result.nsk) throw new Error('no proof keys found')

          proofKeys.push({
            ak: result.ak.toString('hex'),
            nsk: result.nsk.toString('hex'),
          })
        }

        console.log('proofKeys ' + JSON.stringify(proofKeys, null, 2))

        // Craft new tx, to get the tx hash and the public randomness
        // Pass those values to the following commands
        const unsignedTxRaw = buildTx(pks[0], viewKeys[0], proofKeys[0])
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgGetCommitments(unsignedTx.hash().toString('hex'))

            expect(result.commitments.length).toBeTruthy()

            return result
          })

          commitments.push(result.commitments.toString('hex'))
        }

        const signingPackageHex = unsignedTx.signingPackageFromRaw(identities, commitments)
        const signingPackage = new multisig.SigningPackage(Buffer.from(signingPackageHex, 'hex'))

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgSign(
              unsignedTx.publicKeyRandomness(),
              signingPackage.frostSigningPackage().toString('hex'),
              unsignedTx.hash().toString('hex'),
            )

            expect(result.signature.length).toBeTruthy()

            return result
          })

          signatures.push(result.signature.toString('hex'))
        }

        let signedTxRaw = aggregateRawSignatureShares(
          identities,
          publicPackages[0],
          unsignedTxRaw.toString('hex'),
          signingPackage.frostSigningPackage().toString('hex'),
          signatures,
        )
        expect(verifyTransactions([signedTxRaw])).toBeTruthy()

        const signedTx = new Transaction(signedTxRaw)
        expect(signedTx.spends.length).toBe(1)
        expect(signedTx.mints.length).toBe(1)
        expect(signedTx.burns.length).toBe(0)
      } finally {
        for (let i = 0; i < globalSims.length; i++) await globalSims[i].close()
      }
    })
  })

  describe.each(restoreKeysTestCases)(
    `${m.name} - restore keys`,
    ({ index, encrypted, publicAddress, proofKeys, viewKeys, publicPackage }) => {
      test.concurrent(index + '', async () => {
        for (let e of encrypted) {
          const sim = new Zemu(m.path)
          try {
            await sim.start({
              ...defaultOptions,
              model: m.name,
              startText: startTextFn(m.name),
              approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
              approveAction: ButtonKind.ApproveTapButton,
            })
            const app = new IronfishApp(sim.getTransport(), true)

            // Restore keys
            let respReq: any = app.dkgRestoreKeys(e)
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-restore-keys`)
            let resp = await respReq
            await sim.deleteEvents()

            // Backup restored keys to compare snapshots for this process as it is deterministic (fixed keys)
            respReq = app.dkgBackupKeys()
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-backup-keys`)
            resp = await respReq
            await sim.deleteEvents()

            // Generate keys from the restored package to check we are generating the same keys when they were generated
            resp = await app.dkgRetrieveKeys(IronfishKeys.ViewKey)

            expect(resp.viewKey.toString('hex')).toEqual(viewKeys.viewKey)
            expect(resp.ovk.toString('hex')).toEqual(viewKeys.ovk)
            expect(resp.ivk.toString('hex')).toEqual(viewKeys.ivk)

            resp = await app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)

            expect(resp.ak.toString('hex')).toEqual(proofKeys.ak)
            expect(resp.nsk.toString('hex')).toEqual(proofKeys.nsk)

            resp = await app.dkgRetrieveKeys(IronfishKeys.PublicAddress)

            expect(resp.publicAddress.toString('hex')).toEqual(publicAddress)

            resp = await app.dkgGetPublicPackage()

            expect(resp.publicPackage.toString('hex')).toEqual(publicPackage)
          } finally {
            await sim.close()
          }
        }
      })
    },
  )

  test.concurrent(`${m.name} - attempt to retrieve viewKeys when no keys are present`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(app.dkgRetrieveKeys(IronfishKeys.ViewKey)).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  // TODO implement a way to send the command, and but no get the response
  /*
  test.concurrent(`${m.name} - attempt to retrieve result after another command`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      let respReq = app.dkgBackupKeys()

      await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
      await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-d`)

      const resp = await respReq
    } finally {
      await sim.close()
    }
  })
  */

  test.concurrent(`${m.name} - attempt to retrieve proof keys when no keys are present`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  test.concurrent(`${m.name} - attempt to retrieve public address when no keys are present`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(app.dkgRetrieveKeys(IronfishKeys.PublicAddress)).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  test.concurrent(`${m.name} - attempt to retrieve public package when no keys are present`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(app.dkgGetPublicPackage()).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  test.concurrent(`${m.name} - attempt to backup keys when no keys are present`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(app.dkgBackupKeys()).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  test.concurrent(`${m.name} - attempt to run round1 with 5 participants`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })
      const app = new IronfishApp(sim.getTransport(), true)

      await expect(
        app.dkgRound1(
          0,
          identities.map(({ v }) => v),
          3,
        ),
      ).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })

  // TODO complete me
  /*
  test.concurrent(`${m.name} - attempt to run round3 when no round1 was executed`, async () => {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
      const app = new IronfishApp(sim.getTransport(), true)
      let resp: any = await app.dkgRound3()

      await expect(app.dkgRound3()).rejects.toThrow()
    } finally {
      await sim.close()
    }
  })
  */

  describe.each(identities)(`${m.name} - generate identities`, function ({ i, v }) {
    test.concurrent(i + '', async function () {
      const sim = new Zemu(m.path)
      try {
        await sim.start({
          ...defaultOptions,
          model: m.name,
          startText: startTextFn(m.name),
          approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
          approveAction: ButtonKind.ApproveTapButton,
        })
        const app = new IronfishApp(sim.getTransport(), true)
        const identityReq = app.dkgGetIdentity(i)

        await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
        await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-identity-${i}`)

        let identity = await identityReq

        expect(identity.identity.toString('hex')).toEqual(v)
      } finally {
        await sim.close()
      }
    })
  })
})

describe.each(models)('ReviewTx', function (m) {
  it(`${m.name} - ReviewTransaction`, async function () {
    const sim = new Zemu(m.path)
    try {
      // await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
      await sim.start({
        ...defaultOptions,
        model: m.name,
        startText: startTextFn(m.name),
        approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
        approveAction: ButtonKind.ApproveTapButton,
      })

      const app = new IronfishApp(sim.getTransport())

      const unsignedTx =
        '02010000000000000004000000000000000100000000000000010000000000000001000000000000000000000077b55de659bda8a31c3627b9270139637c1fe55efc1cfc09dbb697d68d7583b6844339d824d73947aa6bdafac81c65910da301837980a57cbb1243c223192446494e0770fe825d36eb783ecc0ecc8c80b1ebb8915ae9517d4070ba5613bc23d4a975beeab6dc63e153830eda6a755b4986fbdd4449724b87218045de587f5b6c0991c3a8f88ae5a67a77a2f36fd63c58bc770a86760175b2af5213cd3a7bfebfa6b52508acc97a3f1b78e4be166aa6868f19995318ef578855546250957becf6ac79c1abb60962854e61ef3c4d35eccc0ac10c06bb31881bf9335cca1318509b7cd1f38ed7168e444569ecb97a5fcfa416f61f5aada0cb73d185723dfe1ad7508e5a74e170838290db9657ccbed94b2828d43b31b5abf89af0ed3a3c0caee460780500002f250c30c12c53a475af23d3d6419a6522e350f84d603e82f1b5aa661423c2d878cb1f72f28a40854402380066a711300db6ccc70182a1231e57fee03f32e131630dddc58592b032a4e48cea93798c894b8d35b16495b4298326aff034d41700841ca42e0e443e29bee9bd6b072b2181376acd607093fd18daaf33087c354aa3d02d56c8c4eb9a92e40db161d44d92e9b65efd9ff9fac0a42399e2d459e9d4e29abfb1c3fa0e9bab8d6aeb8177d2c73393f5501ff56171e7ec94f2ef7c8d1b4c18faf62e50930eb25d42bcea4e4a01c75d610e2770fc3e3a3eb1e7779bc7f4306fd485eae3547c12223cc1292ee6c6dd9610ef4fcebf08693a6a76363419e70326e1ac088466d4ed51e53c997abd606840492a7486730d72eb4ba6e4013006e91237100f692657ea6f3a60de2a061e40b1798fdd49e6edb2f03f9719804dab4ea54080718898b0ef77fed41cdf0417de59db823d43fe8fbcb3360da6c24267264f28f8e485f81839a0eef52322f67e56ea6a0d0f0f77e3a7f3de73ab92a67a82f7d00ff619aecb73c6e9960c8cf876ca3b242fcec567ab5066fea40e666eed3a9336ba9599b17b5483d680fe57f51dabfecda43fe4e7ed600e549191697e3b9fef0937bedb680fe7daa93ba4b8523e0cbf2d442aa0105c97d66591c2cfe61b71396c1410ae819fe7c5b6754ff9d855576c670b93242ab5f8188e01f31e567036eedc0aca8edb4e3f6fbc0606207a9f452199bfbb60dcf3c2e648ee47238c8ac602a23a316a20e51c4a04d0a1770cf0e11c4773e7c83b1b904ddd4c07a250826730e29ca9bc335b46c58910f767675b6988a5edd8c0abd0bbb744003f0fdc1ea703d50ff141380c7c838ccb12a2faf1041eac257261a306491ff098e3c29fbf8ea5ac909497cb00dafbcb1be53fe494a76926ee9428f8c5bd846e24c9e08aaa6aaa0de4c041c83adac515c0661a8d757226fbc03270e9633ec1aa312be7165ede814bcc3ef9a468ac112aa175841118c8f20b8d268b5b8cbdee72913cc043c81f2c5058dbae899669428de413c1b811e03f327e6caaff02728af22c72227950cf5653822c160cbc2adc5075a4eaa2d46d201e01b8ef8a69d9ae898a395e9800143f68fe0627abbf70c65cfc228c403408b3931461f088e925dbd5b4d5ccb0ec98bd8847aa1801628b76a24734ce7dc77b2f470969c95fc0d110cc473a640ea3146d57b4d4d3931d3bdbd3851c50cf0982538b5f84eb889e89b2def189767d1e4d3b416b470636b10402183c7c711a2219783dc7c63dbac78335e42383d3f2495ae5eed81cb6ddc345a9c24449cb3d6ab468f15ce2a24a829b568b5890172ec2ef58f3e895143926ccecb9d85141ee819207173dc2acebd97a89a23c0f9f63ae490f3dd52efb4c2b58cc4be254df48d8e5724e1f5e640b95c745508734be2f50ecb2100a09c1f7bb65accc6d54b1e2ef801bd39f677147566d61cd81041b76ff589dadb04a696ac3f1e2f904593231abe4b0b1d933e80b20bcf65a54fdc1b39a986a2470cd17d003f5f6f52773b5e99cd55f823783758b46ef86e48caa3b2177e69abc4217be50444efc52dc9407a6529aa514c577363b52eafed2c78e7fd81869f80b36577d513b319546756a80d9e829c4ccfe5f1abbcdb330d935a6e4ceb7eb98d8c8425ce7000743ba0b41793f8200349afe93c0799fc4bfe3e3d381df69a90c700a51cac12b2456f56593457e7ffb0ec63680ec1846a7bc43761a566864d2f0854d419727d415b99c2d1a7abc460f167541f30e822f977e022134ab8c24f9acb8a721da467e661fc745766553063ca6b880a19fcf929063ada26c6f38da00d0757a917c64759a90d92f3d4cb0fd78b55197780aec0415695d78b55669953fd258b5f05b1dcb6eecc47b410cf7ed06c4e57cb8f34f180d36e218c24acfc7d73c8c8e5ffb2bc2c938bfaba539f3972a0b9f515c8e7fbae327087364b4acaedde75c920cbf543efb17671204bd9f72437478cddad295ca5b70c44f8ed34d4d211ec18b65fb08a4ae5181e0a07deb45d0d27d2b864dd5557cf179d824671bc22d4e2017e4a9858b2fc62a525bc0eb2aa4a2f9d0196f13935502b977c4f56fc61f3932cde5f63b88206a59876923ec4a5169be61e75d4f7ecb1e1dd13e0c3798329aa890dddc6391e4a42948420b205c4752a0bd5feb016f1b78bf8dd5c5cae0696fc5697f4b47a4c0463e71e0d401612a2df5b5ca68618a0032de25847b6f305d0051cb092cde82f17cf806088de4c8ff95d8a00b6fec4e88436d1608718905c2fdef39e87d7bf789dc079570625516bca69bd7b6adb1dd1d9300c933fa4c74c33cd06178b095704cf4d67ceda6a397bf753918e0205f6041f714b4d2ed6a7063111cbe434ffa7018b71793352626346bf8e26804716f261ef6a4b11d73359e19eb566b2f97d1831a12b30e091b779442a219f8444137ddda1691487939f61fb7819e9f885193ee54bc499edbb42b9abe48944fb363e2aa01a26cf090b79ec2f113038d745656ed2f98af671640098dfae56769748ba96d6aed11c7e8d4e91b1f2bcbf8dfe6f3cb93caffd4c7bc24a3b8dddd6c755652b9a7c632876fb2eb9e28405295147c7fc39ad059f71b15a1d164594e272b36bede5ef080c4d3de38388fb8253067c1b5877290a66bde5cc994576d61fd31993337224531e2626dac76104c70ea3bd10855756a16362466fdf8cacc64d4d96e9f5c5b07f2dfd53567a56d45d97bc9abd507705f6e3bbad3a99aa3e921ce83314e60b9ae34221348223d115e45fa1bea7ead2694c4898792b1e6467efaad90df2e6806eb454658e8236cc773bcc9fde10127b67cd5917dfd17a9cce7eda1c1adfd0ebd43f93146f6d064dfa58a6e6b83737e8cffc4c717215edb22c5ca812adf1f52e9ed803bc2f0a303cbda0bf5175632b6c0fb384f873fcbb197d13440e4bdad9a41475e4514234bcc4e2bfc482c153771a5eb22950b50e6370dc838b28457cd3eb41fe31a340270c6018d486580b69bd82f164e62dfd0e1b81653006b89861e91221f9ca5af0e2a35df8ea65c90c5ce20e299beae7cec8ec88c6e3b1bd30f8eec812166997102fa4ecff0c3e6287c66b5e4a629eceef9f7d225b4175697bd7421acb8dc05e7891e3dc94dc13841feb3b0ad804a11d24896ce4cb07a46bf6888e4217c52e94c969cd58a0f64d134257326e81b57d9c1ffd0ade07f550a52ac96f2b3d51eeb47e5325cbcfbd27df221b4286ffe49fd7491bcea88a0f70d3e312f4ccf7183928232fa29061e9b881fbc34c15d9f1a39ad069efde9e7fa892345d0dafa21fca8408d720d6d79e0167e8ab13695f6d2c787545dad56f091cfb786e0f4459213497577e9cb9a54657374636f696e00000000000000000000000000000000000000000000000041207265616c6c7920636f6f6c20636f696e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004050000000000000079e0167e8ab13695f6d2c787545dad56f091cfb786e0f4459213497577e9cb9a00b00ad4cdfd5c66e1822c22acd26da08c21cb366632a03210c364d33d259deee66c50da30ba43bbcedcf1aff4370a01912069aca1355d360e5806653b8656820c6d9e3dd19d175b3eab6cb439d69f19b25a95b9f2c3270c23543578b3667c8ea20200000000000000280d529a652296987bce8140f0d72d548ccec277dd4add9c5f378d56178f86cfa230822819e6ae9f1bdb2ec9c00dbc954dc52e0730157860dd41a666213fea03'
      const hashResp = app.reviewTransaction(unsignedTx)

      await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
      await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-review_transaction`)

      let resp = await hashResp

      expect(resp.returnCode.toString(16)).toEqual('9000')
      expect(resp.errorMessage).toEqual('No errors')

      console.log('tx_hash :', resp.hash?.toString('hex'))

      expect(resp.hash?.toString('hex')).toEqual('')

      // Clean events from previous commands as each sim lives for many commands (DKG generation + signing)
      await sim.deleteEvents()
    } finally {
      await sim.close()
    }
  })
})
