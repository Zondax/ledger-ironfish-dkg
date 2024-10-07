import TransportNodeHid from '@ledgerhq/hw-transport-node-hid'
import IronfishApp from '@zondax/ledger-ironfish'

async function main() {
  const transport = await TransportNodeHid.default.open()

  const app = new IronfishApp.default(transport, false)

  //const PATH = "m/44'/626'/0'/0/0"
  //const PATH_TESTNET = "m/44'/1'/0'/0'/0'"
  const get_resp = await app.getVersion()
  console.log(get_resp)
}

;(async () => {
  await main()
})()
