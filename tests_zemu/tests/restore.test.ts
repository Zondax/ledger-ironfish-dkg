import { defaultOptions, models, restoreKeysTestCases } from './common'
import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import { startTextFn } from './utils'
import IronfishApp, { IronfishKeys } from '@zondax/ledger-ironfish'

jest.setTimeout(450000)

describe.each(models)('restore keys', function (m) {
  describe.each(restoreKeysTestCases)(
    `${m.name}`,
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
})
