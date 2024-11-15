import { defaultOptions, models, restoreKeysTestCases } from './common'
import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import { buildTx, IronfishKeySet, runMethod, startTextFn } from './utils'
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

const TEST_OUTPUT_KEY: IronfishKeySet = {
  publicAddress: '87318a66842817fcc22001782eced854259133792e8a7d492689384bc6933683',
  viewKey: {
    viewKey:
      '2b3171faacabc5c785b0eb25209220cc9177ef5a8261ad007fbec5bd5e92855f6eef1a5ad6aa2ceb3a684e9fe57810de29bb780a00f60771166ed72009ca575e',
    ivk: '9c907178ab742e018d3de3f01be9695b0cdcbbf195474ab8d707192353c2aa07',
    ovk: '6603a0b278f2be39cb9882ae46c8b1308907edbcb49ce8a4550b6c8d9b4cc043',
  },
  proofKey: {
    ak: '2b3171faacabc5c785b0eb25209220cc9177ef5a8261ad007fbec5bd5e92855f',
    nsk: '47fcc3d6746ad4fc83414cf8b2741c9a5ffca3e2c7a4a1f0cac14c67a4990902',
  },
}

describe.each(models)('review transaction', function (m) {
  describe.each(restoreKeysTestCases)(`${m.name}-review_tx_expert_mode`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []
      let commitments: any[] = []
      let signatures: any[] = []

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

        let senderKey: IronfishKeySet = {
          publicAddress: pubkey,
          viewKey: viewKey,
          proofKey: proofKey,
        }

        const unsignedTxRaw = buildTx(senderKey, { receiver: TEST_OUTPUT_KEY })
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction_expert_mode`)

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

  describe.each(restoreKeysTestCases)(`${m.name}-review_tx_normal_mode`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []
      let commitments: any[] = []
      let signatures: any[] = []

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
      }

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

        let senderKey: IronfishKeySet = {
          publicAddress: pubkey,
          viewKey: viewKey,
          proofKey: proofKey,
        }

        // Use only native tokens
        const unsignedTxRaw = buildTx(senderKey, { receiver: TEST_OUTPUT_KEY, nativeAssetOnly: true })
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction_normal_mode`)

            const result = await resultReq
            expect(result.hash.length).toBeTruthy()
            expect(result.hash.toString('hex')).toBe(unsignedTx.hash().toString('hex'))

            return result
          })
        }
      } finally {
        for (let i = 0; i < globalSims.length; i++) await globalSims[i].close()
      }
    })
  })

  describe.each(restoreKeysTestCases)(`${m.name}-review_tx_normal_mode_shows_change_address`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []
      let commitments: any[] = []
      let signatures: any[] = []

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
      }

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

        let senderKey: IronfishKeySet = {
          publicAddress: pubkey,
          viewKey: viewKey,
          proofKey: proofKey,
        }
        // Use only native tokens
        // and use sender address as the change, this must be displayed
        // because it is the only output in the transaction
        const unsignedTxRaw = buildTx(senderKey, { nativeAssetOnly: true })
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
            // Change the approve button type to hold, as we are signing a tx now.
            sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
            const resultReq = app.reviewTransaction(serialized.toString('hex'))

            await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
            await sim.compareSnapshotsAndApprove(
              '.',
              `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction_normal_mode_shows_change_address`,
            )

            const result = await resultReq
            expect(result.hash.length).toBeTruthy()
            expect(result.hash.toString('hex')).toBe(unsignedTx.hash().toString('hex'))

            return result
          })
        }
      } finally {
        for (let i = 0; i < globalSims.length; i++) await globalSims[i].close()
      }
    })
  })
})
