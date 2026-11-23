// Is this chain ready for Ledger (Polkadot Generic app) signing? Device-free.
//
//   npm ci && node ledger-ready.mjs wss://<rpc>
//
// Signs a system.remark with a THROWAWAY, UNFUNDED ed25519 key through the same payload
// path a Ledger uses (mode:1 + merkleized-metadata hash) and asks the chain to VALIDATE it.
// Nothing is ever submitted.
//   correct hash -> "Invalid: Payment"  (hash + signature accepted; the key just has no funds)
//   wrong hash   -> "Invalid: BadProof" (proves the runtime really checks the hash)
//   correct hash -> "Unknown: CannotLookup" => runtime built WITHOUT the metadata hash: NOT READY.
// Exit code 0 = READY, 1 = NOT READY / inconclusive.
import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";
import { u8aToHex, u8aConcat } from "@polkadot/util";
import { blake2AsU8a, cryptoWaitReady, mnemonicGenerate } from "@polkadot/util-crypto";
import { merkleizeMetadata } from "@polkadot-api/merkleize-metadata";

const rpc = process.argv[2];
if (!rpc) { console.error("usage: node ledger-ready.mjs wss://<rpc>"); process.exit(2); }

await cryptoWaitReady();
const api = await ApiPromise.create({ provider: new WsProvider(rpc), noInitWarn: true, throwOnConnect: true });
// The token a wallet (and the Ledger) sees: the chain's system_properties, as decoded by polkadot.js.
const symbol = api.registry.chainTokens[0];
const decimals = api.registry.chainDecimals[0];
const ss58 = api.registry.chainSS58 ?? 42;
const rv = api.runtimeVersion;
console.log(`chain ${(await api.rpc.system.chain()).toString()}  spec ${rv.specName}/${rv.specVersion}  token ${symbol}/${decimals}  ss58 ${ss58}`);

const opaque = await api.call.metadata.metadataAtVersion(15);
if (opaque.isNone) { console.log("NOT READY: runtime does not expose metadata V15"); process.exit(1); }
const merkleized = merkleizeMetadata(opaque.unwrap().toU8a(true), {
  decimals, tokenSymbol: symbol,
  specName: rv.specName.toString(), specVersion: rv.specVersion.toNumber(),
});
const metadataHash = u8aToHex(merkleized.digest());
console.log(`metadata hash (from system_properties token): ${metadataHash}`);

const pair = new Keyring({ type: "ed25519", ss58Format: ss58 }).addFromMnemonic(mnemonicGenerate());
const signer = {
  signPayload: async (payload) => {
    const bytes = api.registry.createType("ExtrinsicPayload", payload, { version: payload.version }).toU8a({ method: true });
    merkleized.getProofForExtrinsicPayload(u8aToHex(bytes)); // must build, as it would for the device
    const msg = bytes.length > 256 ? blake2AsU8a(bytes, 256) : bytes;
    return { id: 1, signature: u8aToHex(u8aConcat(new Uint8Array([0]), pair.sign(msg))) };
  },
};

// Raw state_call: polkadot.js's api.call wrapper mis-encodes `tx` for metadata-v16
// runtimes (the runtime panics on decode). Args: TransactionSource::External ++ tx ++ at.
async function validateWith(hash) {
  const tx = api.tx.system.remark("0x6c6564676572");
  await tx.signAsync(pair.address, { signer, nonce: 0, era: 64, mode: 1, metadataHash: hash });
  const at = (await api.rpc.chain.getFinalizedHead()).toHex();
  const raw = await api.rpc.state.call("TaggedTransactionQueue_validate_transaction", "0x02" + tx.toHex().slice(2) + at.slice(2), at);
  return JSON.stringify(api.registry.createType("TransactionValidity", raw).toHuman());
}

const good = await validateWith(metadataHash);
const bad = await validateWith("0x" + "11".repeat(32));
console.log(`correct hash -> ${good}\nwrong hash   -> ${bad}`);
await api.disconnect();

if (/Payment/.test(good) && /BadProof/.test(bad)) {
  console.log("\nREADY: the runtime enforces CheckMetadataHash and matches its system_properties token.");
  process.exit(0);
}
if (/CannotLookup/.test(good)) console.log("\nNOT READY: runtime built without the metadata hash — needs a runtime upgrade built from this template.");
else if (/BadProof/.test(good)) console.log("\nNOT READY: hash mismatch — the runtime was hashed with a different token than system_properties reports (check the chain manifest's [token]).");
else console.log("\nINCONCLUSIVE: unexpected results above.");
process.exit(1);
