import { defaultOptions, identities, models, restoreKeysTestCases } from './common'
import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import { buildTx, IronfishKeySet, runMethod, startTextFn } from './utils'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'
import { UnsignedTransaction } from '@ironfish/rust-nodejs'

jest.setTimeout(450000)

// ONE_GLOBAL_APP: Use this flag if the whole DKG process will run in only one app (all participants, all rounds). This takes precedence over ONE_APP_PER_PARTICIPANT.
// ONE_APP_PER_PARTICIPANT: Use this flag if the whole DKG process will run in one app per participant
// Otherwise, if both are falsy, one app will be started per request (each round for each participant)
const ONE_GLOBAL_APP = 0
const ONE_APP_PER_PARTICIPANT = 1

describe.each(models)('wrong actions', function (m) {
  describe.each(restoreKeysTestCases)(`${m.name} - attempt to sign after sending wrong command`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []

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

        let senderKey: IronfishKeySet = {
          publicAddress: pubkey,
          viewKey: viewKey,
          proofKey: proofKey,
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

  describe.each(restoreKeysTestCases)(`${m.name} - attempt to sign an unknown token`, ({ index, encrypted }) => {
    test(index + '', async () => {
      const participants = encrypted.length
      const globalSims: Zemu[] = []

      let identities: any[] = []

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

        let senderKey: IronfishKeySet = {
          publicAddress: pubkey,
          viewKey: viewKey,
          proofKey: proofKey,
        }
        const unsignedTxRaw = buildTx(senderKey)
        const unsignedTx = new UnsignedTransaction(unsignedTxRaw)

        const serialized = unsignedTx.serialize()

        for (let i = 0; i < participants; i++) {
          await expect(
            runMethod(m, globalSims, i, async (sim: Zemu, app: IronfishApp) => {
              // Change the approve button type to hold, as we are signing a tx now.
              sim.startOptions.approveAction = ButtonKind.ApproveHoldButton
              try {
                await app.reviewTransaction(serialized.toString('hex'))
                // await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot())
                // await sim.compareSnapshotsAndApprove('.', `${m.prefix.toLowerCase()}-dkg-sign-${index}-review-transaction`)
              } catch (error) {
                // Convert unknown error to string for comparison
                const errorStr = String(error)
                console.log('ErrorStr ', errorStr)
                if (errorStr.includes('Expert mode is required')) {
                  throw new Error(errorStr)
                }
                throw error
              }
            }),
          ).rejects.toThrow(/Expert mode is required/)
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
})
