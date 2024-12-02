import {
  Asset,
  generateKey,
  generatePublicAddressFromIncomingViewKey,
  LATEST_TRANSACTION_VERSION,
  Note,
  Transaction,
} from '@ironfish/rust-nodejs'
import { deserializePublicPackage, deserializeRound2CombinedPublicPackage } from '@ironfish/rust-nodejs'
import { devUtils, Note as NoteSDK } from '@ironfish/sdk'
import { TModel } from '@zondax/zemu/dist/types'
import Zemu, { ButtonKind, DEFAULT_START_OPTIONS, IDeviceModel, isTouchDevice } from '@zondax/zemu'
//import IronfishApp from '@zondax/ledger-ironfish'
import { defaultOptions } from './common'

export const buildTx = (publicAddress: string, viewKeys: any, proofKey: any) => {
  let key = generateKey()
  console.log(key)

  //let pubaddr = generatePublicAddressFromIncomingViewKey(key.incomingViewKey)
  let pubaddr = '68fbb29d61e385420d26e3b9c6acc74dfec014978e0d2aeb4fb60b138e4f5d71'
  // create raw/proposed transaction
  let in_note = new Note(publicAddress, BigInt(42), Buffer.from(''), Asset.nativeId(), publicAddress)
  let out_note = new Note(pubaddr, BigInt(40), Buffer.from(''), Asset.nativeId(), publicAddress)
  let asset = new Asset(publicAddress, 'Testcoin', 'A really cool coin')

  let value = BigInt(5)
  //let mint_out_note = new Note(publicAddress, value, Buffer.from(''), asset.id(), publicAddress)

  let witness = devUtils.makeFakeWitness(new NoteSDK(in_note.serialize()))

  let transaction = new Transaction(LATEST_TRANSACTION_VERSION)
  transaction.spend(in_note, witness)
  transaction.output(out_note)
  //transaction.mint(asset, value)
  //transaction.output(mint_out_note)

  let intended_fee = BigInt(1)

  return transaction.build(proofKey.nsk, viewKeys.viewKey, viewKeys.ovk, intended_fee, publicAddress)
}

// export const minimizeRound3Inputs = (index: number, round1PublicPackages: string[], round2PublicPackages: string[]) => {
//   let round1Pkgs = round1PublicPackages.map(p => deserializePublicPackage(p))
//   let round2Pkgs = round2PublicPackages.map(p => deserializeRound2CombinedPublicPackage(p))

//   let identity: string = ''

//   const participants: string[] = []
//   const round1PublicPkgs: string[] = []
//   const round2PublicPkgs: string[] = []
//   const gskBytes: string[] = []

//   for (let i = 0; i < round1Pkgs.length; i++) {
//     gskBytes.push(round1Pkgs[i].groupSecretKeyShardEncrypted)

//     // TODO: is the index 0-indexed?
//     if (i == index) {
//       identity = round1Pkgs[i].identity
//     } else {
//       participants.push(round1Pkgs[i].identity)
//       round1PublicPkgs.push(round1Pkgs[i].frostPackage)
//     }
//   }

//   for (let i = 0; i < round2Pkgs.length; i++) {
//     for (let j = 0; j < round2Pkgs[i].packages.length; j++) {
//       if (round2Pkgs[i].packages[j].recipientIdentity == identity) {
//         round2PublicPkgs.push(round2Pkgs[i].packages[j].frostPackage)
//       }
//     }
//   }

//   return {
//     participants,
//     round1PublicPkgs,
//     round2PublicPkgs,
//     gskBytes,
//   }
// }

// // Not sure about the start text for flex and stax, so we go with what it always work, which is the app name.
// // That is always displayed on the main menu
// export const startTextFn = (model: TModel) => (isTouchDevice(model) ? 'Ironfish DKG' : DEFAULT_START_OPTIONS.startText)

// export const checkSimRequired = (m: IDeviceModel, sims: Zemu[], i: number): { sim: Zemu; created: boolean } => {
//   let created = false
//   let sim: Zemu | undefined

//   if (!sims.length) {
//     sim = new Zemu(m.path)
//     created = true
//   } else if (sims.length === 1) {
//     sim = sims[0]
//   } else {
//     sim = sims[i]
//   }

//   if (!sim) throw new Error('sim should have a value here')
//   return { sim, created }
// }

// export const runMethod = async (
//   m: IDeviceModel,
//   rcvSims: Zemu[],
//   i: number,
//   fn: (sim: Zemu, app: IronfishApp) => Promise<any>,
// ): Promise<any> => {
//   const { sim, created } = checkSimRequired(m, rcvSims, i)

//   try {
//     if (created)
//       await sim.start({
//         ...defaultOptions,
//         model: m.name,
//         startText: startTextFn(m.name),
//         approveKeyword: isTouchDevice(m.name) ? 'Approve' : '',
//         approveAction: ButtonKind.ApproveTapButton,
//       })
//     const app = new IronfishApp(sim.getTransport(), true)
//     const resp = await fn(sim, app)

//     // Clean events from previous commands as each sim lives for many commands (DKG generation + signing)
//     await sim.deleteEvents()

//     return resp
//   } finally {
//     if (created) await sim.close()
//   }
// }
