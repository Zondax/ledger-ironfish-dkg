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

import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import { defaultOptions, models } from './common'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'
import { isValidPublicAddress, multisig, UnsignedTransaction, verifyTransactions } from '@ironfish/rust-nodejs'
import { Transaction } from '@ironfish/sdk'
import { buildTx, IronfishKeySet, minimizeRound3Inputs, runMethod, startTextFn } from './utils'
import aggregateSignatureShares = multisig.aggregateSignatureShares

jest.setTimeout(4500000)

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
      const globalSims: Zemu[] = []

      if (ONE_GLOBAL_APP) globalSims.push(new Zemu(m.path))
      else if (ONE_APP_PER_PARTICIPANT) for (let i = 0; i < participants; i++) globalSims.push(new Zemu(m.path))

      for (let i = 0; i < globalSims.length; i++) {
        let sim = globalSims[i]
        await sim.start({
          ...defaultOptions,
          model: m.name,
          startText: startTextFn(m.name),
          approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
          approveAction: ButtonKind.ApproveTapButton,
        })
        await sim.toggleExpertMode()
      }

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
        let senderKey: IronfishKeySet = {
          publicAddress: pks[0],
          viewKey: viewKeys[0],
          proofKey: proofKeys[0],
        }
        const unsignedTxRaw = buildTx(senderKey)
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
            multisig.SignatureShare.fromFrost(result.signature, Buffer.from(identities[i], 'hex')).serialize().toString('hex'),
          )
        }

        let signedTxRaw = aggregateSignatureShares(publicPackages[0], signingPackageHex, signatures)
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
})
