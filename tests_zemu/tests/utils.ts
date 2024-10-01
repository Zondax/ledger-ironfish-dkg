import { Asset, LATEST_TRANSACTION_VERSION, Note, Transaction } from '@ironfish/rust-nodejs'
import { deserializePublicPackage, deserializeRound2CombinedPublicPackage } from '@ironfish/rust-nodejs'
import { devUtils, Note as NoteSDK } from '@ironfish/sdk'
import { TModel } from '@zondax/zemu/dist/types'
import { DEFAULT_START_OPTIONS, isTouchDevice } from '@zondax/zemu'

export const buildTx = (publicAddress: string, viewKeys: any, proofKey: any) => {
  // create raw/proposed transaction
  let in_note = new Note(publicAddress, BigInt(42), Buffer.from(''), Asset.nativeId(), publicAddress)
  let out_note = new Note(publicAddress, BigInt(40), Buffer.from(''), Asset.nativeId(), publicAddress)
  let asset = new Asset(publicAddress, 'Testcoin', 'A really cool coin')

  let value = BigInt(5)
  let mint_out_note = new Note(publicAddress, value, Buffer.from(''), asset.id(), publicAddress)

  let witness = devUtils.makeFakeWitness(new NoteSDK(in_note.serialize()))

  let transaction = new Transaction(LATEST_TRANSACTION_VERSION)
  transaction.spend(in_note, witness)
  transaction.output(out_note)
  transaction.mint(asset, value)
  transaction.output(mint_out_note)

  let intended_fee = BigInt(1)

  return transaction.build(proofKey.nsk, viewKeys.viewKey, viewKeys.ovk, intended_fee, publicAddress)
}

export const minimizeRound3Inputs = (index: number, round1PublicPackages: string[], round2PublicPackages: string[]) => {
  let round1Pkgs = round1PublicPackages.map(p => deserializePublicPackage(p))
  let round2Pkgs = round2PublicPackages.map(p => deserializeRound2CombinedPublicPackage(p))

  let identity: string = ''

  const participants: string[] = []
  const round1PublicPkgs: string[] = []
  const round2PublicPkgs: string[] = []
  const gskBytes: string[] = []

  for (let i = 0; i < round1Pkgs.length; i++) {
    gskBytes.push(round1Pkgs[i].groupSecretKeyShardEncrypted)

    // TODO: is the index 0-indexed?
    if (i == index) {
      identity = round1Pkgs[i].identity
    } else {
      participants.push(round1Pkgs[i].identity)
      round1PublicPkgs.push(round1Pkgs[i].frostPackage)
    }
  }

  for (let i = 0; i < round2Pkgs.length; i++) {
    for (let j = 0; j < round2Pkgs[i].packages.length; j++) {
      if (round2Pkgs[i].packages[j].recipientIdentity == identity) {
        round2PublicPkgs.push(round2Pkgs[i].packages[j].frostPackage)
      }
    }
  }

  return {
    participants,
    round1PublicPkgs,
    round2PublicPkgs,
    gskBytes,
  }
}

// That is always displayed on the main menu
export const startTextFn = (model: TModel) => (isTouchDevice(model) ? 'Ironfish DKG' : DEFAULT_START_OPTIONS.startText)
