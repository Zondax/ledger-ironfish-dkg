import { defaultOptions, models, restoreKeysTestCases } from './common'
import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import { buildTx, runMethod, startTextFn } from './utils'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'
import { multisig, UnsignedTransaction, verifyTransactions } from '@ironfish/rust-nodejs'
import { Transaction } from '@ironfish/sdk'
import aggregateSignatureShares = multisig.aggregateSignatureShares

jest.setTimeout(450000)

// ONE_GLOBAL_APP: Use this flag if the whole DKG process will run in only one app (all participants, all rounds). This takes precedence over ONE_APP_PER_PARTICIPANT.
// ONE_APP_PER_PARTICIPANT: Use this flag if the whole DKG process will run in one app per participant
// Otherwise, if both are falsy, one app will be started per request (each round for each participant)
const ONE_GLOBAL_APP = 0
const ONE_APP_PER_PARTICIPANT = 1

describe.each(models)('restore keys', function (m) {
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

        let publicPackage = await runMethod(m, globalSims, 0, async (sim: Zemu, app: IronfishApp) => {
          let result = await app.dkgGetPublicPackage()

          return result.publicPackage.toString('hex')
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

        let signedTxRaw = aggregateSignatureShares(publicPackage, signingPackageHex, signatures)
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
