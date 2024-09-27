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

import Zemu, { ButtonKind, DEFAULT_START_OPTIONS, IDeviceModel, isTouchDevice } from '@zondax/zemu'
import { defaultOptions, identities, models, restoreKeysTestCases } from './common'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'
import { isValidPublicAddress, multisig, UnsignedTransaction, verifyTransactions } from '@ironfish/rust-nodejs'
import { Transaction } from '@ironfish/sdk'
import { buildTx, minimizeRound3Inputs } from './utils'
import { TModel } from '@zondax/zemu/dist/types'
import aggregateSignatureShares = multisig.aggregateSignatureShares

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

const checkSimRequired = (m: IDeviceModel, sims: Zemu[], i: number): { sim: Zemu; created: boolean } => {
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

const runMethod = async (m: IDeviceModel, rcvSims: Zemu[], i: number, fn: (sim: Zemu, app: IronfishApp) => Promise<any>): Promise<any> => {
  const { sim, created } = checkSimRequired(m, rcvSims, i)

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

      try {
        // First: Generate identities
        for (let i = 0; i < participants; i++) {
          try {
            const identity = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              const result = await app.dkgGetIdentity(i, false)

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
            const round1 = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
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
            const round2 = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
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
            await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
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
          const result = await runMethod(m, globalSims, i, async (_sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgGetPublicPackage()

            expect(result.publicPackage.length).toBeTruthy()

            return result
          })

          publicPackages.push(result.publicPackage.toString('hex'))
        }

        console.log('publicPackages ' + JSON.stringify(publicPackages, null, 2))

        for (let i = 0; i < participants; i++) {
          try {
            const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              let resultReq = app.dkgBackupKeys()

              await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
              try {
                await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-tmp-dkg-p${participants}-m${minSigners}-${i}-backup`)
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
          const result = await runMethod(m, globalSims, i, async (_sim: Zemu, app: IronfishApp) => {
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
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
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
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
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

        // get identity from the multisig DKG process just finalized
        for (let i = 0; i < participants; i++) {
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgRetrieveKeys(IronfishKeys.DkgIdentity)

            expect('identity' in result).toBeTruthy()

            return result
          })

          if (!result.identity) throw new Error('no identity found')

          expect(result.identity.toString('hex')).toBe(identities[i])
        }

        // Craft new tx, to get the tx hash and the public randomness
        // Pass those values to the following commands
        const unsignedTxRaw = buildTx(pks[0], viewKeys[0], proofKeys[0])
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)
        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            try {
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-tmp-review-transaction`)
            } catch (e) {
              // TODO navigate and approve, but do not compare snapshots... needs to be added to zemu
              // Skip error, as a new public address is generated each time. Snapshots will be different in every run
            }

            const result = await resultReq
            expect(result.hash.length).toBeTruthy()
            expect(result.hash.toString('hex')).toBe(unsignedTx.hash().toString('hex'))

            return result
          })
        }

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgGetCommitments(unsignedTx.hash().toString('hex'))

            expect(result.commitments.length).toBeTruthy()

            return result
          })

          commitments.push(
            multisig.SigningCommitment.fromRaw(identities[i], result.commitments, unsignedTx.hash(), identities)
              .serialize()
              .toString('hex'),
          )
        }

        const signingPackageHex = unsignedTx.signingPackage(commitments)
        const signingPackage = new multisig.SigningPackage(Buffer.from(signingPackageHex, 'hex'))

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgSign(
              unsignedTx.publicKeyRandomness(),
              signingPackage.frostSigningPackage().toString('hex'),
              unsignedTx.hash().toString('hex'),
            )

            expect(result.signature.length).toBeTruthy()

            return result
          })

          signatures.push(
            multisig.SignatureShare.fromFrost(result.signature, Buffer.from(identities[i], 'hex')).frostSignatureShare().toString('hex'),
          )
        }

        let signedTxRaw = aggregateSignatureShares(publicPackages[0], signingPackage.frostSigningPackage().toString('hex'), signatures)
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
    ({ index, encrypted, publicAddress, identities, proofKeys, viewKeys, publicPackage }) => {
      test.concurrent(index + '', async () => {
        for (let i = 0; i < encrypted.length; i++) {
          const e = encrypted[i]

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
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-restore-keys`)
            let resp = await respReq
            await sim.deleteEvents()

            // Backup restored keys to compare snapshots for this process as it is deterministic (fixed keys)
            respReq = app.dkgBackupKeys()
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-backup-keys`)
            resp = await respReq
            await sim.deleteEvents()

            // Generate keys from the restored package to check we are generating the same keys when they were generated
            respReq = app.dkgRetrieveKeys(IronfishKeys.ViewKey, true)
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-view-keys`)
            resp = await respReq
            await sim.deleteEvents()

            expect(resp.viewKey.toString('hex')).toEqual(viewKeys.viewKey)
            expect(resp.ovk.toString('hex')).toEqual(viewKeys.ovk)
            expect(resp.ivk.toString('hex')).toEqual(viewKeys.ivk)

            respReq = app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey, true)
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-proof-keys`)
            resp = await respReq
            await sim.deleteEvents()

            expect(resp.ak.toString('hex')).toEqual(proofKeys.ak)
            expect(resp.nsk.toString('hex')).toEqual(proofKeys.nsk)

            respReq = app.dkgRetrieveKeys(IronfishKeys.PublicAddress, true)
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-public-addr`)
            resp = await respReq
            await sim.deleteEvents()

            expect(resp.publicAddress.toString('hex')).toEqual(publicAddress)

            respReq = app.dkgRetrieveKeys(IronfishKeys.DkgIdentity, true)
            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            try {
              await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-${index}-identity`)
            } catch (e) {
              // TODO navigate and approve, but do not compare snapshots... needs to be added to zemu
              // Skip error, as a new public address is generated each time. Snapshots will be different in every run
            }
            resp = await respReq
            await sim.deleteEvents()

            expect(resp.identity.toString('hex')).toEqual(identities[i])

            resp = await app.dkgGetPublicPackage()

            expect(resp.publicPackage.toString('hex')).toEqual(publicPackage)

            resp = await app.dkgGetIdentities()

            const identitiesStr = resp.identities.map((i: any) => i.toString('hex'))
            identities.forEach(i => expect(identitiesStr.includes(i)).toBeTruthy())
          } finally {
            await sim.close()
          }
        }
      })
    },
  )

  describe.each(restoreKeysTestCases)(`${m.name} - sign transaction`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []
      let commitments: any[] = []
      let signatures: any[] = []

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

      try {
        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = app.dkgRestoreKeys(encrypted[i])

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-restore-keys`)

            await result
          })
        }

        let viewKey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.ViewKey)

          return {
            viewKey: result.viewKey.toString('hex'),
            ivk: result.ivk.toString('hex'),
            ovk: result.ovk.toString('hex'),
          }
        })

        let proofKey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)

          return { ak: result.ak.toString('hex'), nsk: result.nsk.toString('hex') }
        })

        let pubkey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.PublicAddress)

          return result.publicAddress.toString('hex')
        })

        let publicPackages = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result = await app.dkgGetPublicPackage()

          return result.publicPackage
        })

        for (let i = 0; i < participants; i++) {
          const identity = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            return await app.dkgRetrieveKeys(IronfishKeys.DkgIdentity)
          })

          if (!identity.identity) throw new Error('no identity found')

          identities.push(identity.identity.toString('hex'))
        }

        const unsignedTxRaw = buildTx(pubkey, viewKey, proofKey)
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction`)

            const result = await resultReq
            expect(result.hash.length).toBeTruthy()
            expect(result.hash.toString('hex')).toBe(unsignedTx.hash().toString('hex'))

            return result
          })
        }

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgGetCommitments(unsignedTx.hash().toString('hex'))

            expect(result.commitments.length).toBeTruthy()

            return result
          })

          commitments.push(multisig.SigningCommitment.fromRaw(identities[i], result.commitments, unsignedTx.hash(), identities))
        }

        const signingPackageHex = unsignedTx.signingPackage(commitments)
        const signingPackage = new multisig.SigningPackage(Buffer.from(signingPackageHex, 'hex'))

        for (let i = 0; i < participants; i++) {
          const result = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = await app.dkgSign(
              unsignedTx.publicKeyRandomness(),
              signingPackage.frostSigningPackage().toString('hex'),
              unsignedTx.hash().toString('hex'),
            )

            expect(result.signature.length).toBeTruthy()

            return result
          })

          signatures.push(
            multisig.SignatureShare.fromFrost(result.signature, Buffer.from(identities[i], 'hex')).frostSignatureShare().toString('hex'),
          )
        }

        // Attempt to sign again. It should fail as the tx hash is cleaned
        for (let i = 0; i < participants; i++) {
          await expect(
            runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              await app.dkgSign(
                unsignedTx.publicKeyRandomness(),
                signingPackage.frostSigningPackage().toString('hex'),
                unsignedTx.hash().toString('hex'),
              )
            }),
          ).rejects.toThrow()
        }

        let signedTxRaw = aggregateSignatureShares(publicPackages[0], signingPackage.frostSigningPackage().toString('hex'), signatures)
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

  describe.each(restoreKeysTestCases)(`${m.name} - attempt to sign after sending wrong command`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []

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

      try {
        const reqs = []
        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            let result = app.dkgRestoreKeys(encrypted[i])

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-restore-keys`)

            await result
          })
        }

        let viewKey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.ViewKey)

          return {
            viewKey: result.viewKey.toString('hex'),
            ivk: result.ivk.toString('hex'),
            ovk: result.ovk.toString('hex'),
          }
        })

        let proofKey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.ProofGenerationKey)

          return { ak: result.ak.toString('hex'), nsk: result.nsk.toString('hex') }
        })

        let pubkey = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result: any = await app.dkgRetrieveKeys(IronfishKeys.PublicAddress)

          return result.publicAddress.toString('hex')
        })

        let publicPackages = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result = await app.dkgGetPublicPackage()

          return result.publicPackage
        })

        for (let i = 0; i < participants; i++) {
          const identity = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            return await app.dkgRetrieveKeys(IronfishKeys.DkgIdentity)
          })

          if (!identity.identity) throw new Error('no identity found')

          identities.push(identity.identity.toString('hex'))
        }

        const unsignedTxRaw = buildTx(pubkey, viewKey, proofKey)
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction`)

            const result = await resultReq
            expect(result.hash.length).toBeTruthy()
            expect(result.hash.toString('hex')).toBe(unsignedTx.hash().toString('hex'))

            return result
          })
        }

        // Send wrong command in the middle of signing process (review + commitments + sign)
        for (let i = 0; i < participants; i++) {
          const identity = await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            return await app.dkgRetrieveKeys(IronfishKeys.DkgIdentity)
          })

          if (!identity.identity) throw new Error('no identity found')
        }

        // Attempt to get commitments
        for (let i = 0; i < participants; i++) {
          await expect(
            runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              let result = await app.dkgGetCommitments(unsignedTx.hash().toString('hex'))

              expect(result.commitments.length).toBeTruthy()

              return result
            }),
          ).rejects.toThrow()
        }
      } finally {
        for (let i = 0; i < globalSims.length; i++) await globalSims[i].close()
      }
    })
  })

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
        const identityReq = app.dkgGetIdentity(i, true)

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
