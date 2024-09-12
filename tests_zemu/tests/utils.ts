import {Asset, LATEST_TRANSACTION_VERSION, Note, Transaction, makeTestWitness} from '@ironfish/rust-nodejs'

export const buildTx = (publicAddress: string, viewKeys: any, proofKey: any) => {
    console.log("here")
    // create raw/proposed transaction
    let in_note = new Note(publicAddress, BigInt(42), Buffer.from(""), Asset.nativeId(), publicAddress);
    let out_note = new Note(publicAddress, BigInt(40), Buffer.from(""), Asset.nativeId(), publicAddress);
    let asset = new Asset(publicAddress, "Testcoin", "A really cool coin")
    
    let value = BigInt(5);
    let mint_out_note = new Note(publicAddress, value, Buffer.from(""), asset.id(), publicAddress);

    let witness = makeTestWitness(in_note);

    let transaction = new Transaction(LATEST_TRANSACTION_VERSION);
    transaction.spendNative(in_note, witness)
    transaction.output(out_note)
    transaction.mint(asset, value)
    transaction.output(mint_out_note)

    let intended_fee = BigInt(1);

    return transaction.build(
        proofKey.nsk,
        viewKeys.viewKey,
        viewKeys.ovk,
        intended_fee,
        publicAddress
    );
}