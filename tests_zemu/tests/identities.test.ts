import { defaultOptions, identities, models } from './common'
import Zemu, { ButtonKind, isTouchDevice } from '@zondax/zemu'
import IronfishApp from '@zondax/ledger-ironfish'
import { startTextFn } from './utils'

jest.setTimeout(450000)

describe.each(models)('generate identities', function (m) {
  describe.each(identities)(`${m.name}`, function ({ i, v }) {
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
