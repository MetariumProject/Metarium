# Metarium

Metarium is a [Substrate](https://github.com/paritytech/polkadot-sdk)-based blockchain. Its runtime adds
**`pallet-metarium`** — a content-anchoring pallet built around:

- **channels** ("mnembooks") with an **actant** (writer) / **listener** (reader) access-control model;
- **arikuris** — content pointers (self-describing content hashes, e.g. `blake3://…`) attached to a channel;
- a **custodian-metadata commit thread** that serializes concurrent writers behind an on-chain lock;
- node reachability (`NodeInfoMap`), a bookUUID ↔ channel binding, and a membership reverse-index.

It is the chain the Metarium / mnem ecosystem runs: a sudo-governed solochain with AURA authoring + GRANDPA
finality, validators managed via `pallet-validator-set`.

This repository is a **fork of the [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)** with the
Metarium runtime and pallet added. Anyone may clone, build, run, and fork it.

## Layout
- **`templates/solochain/`** — the **Metarium solochain**: the runtime (live `spec_version` 102, wiring
  `pallet-metarium`) and the `solochain-template-node`.
- **`templates/parachain/pallets/metarium/`** — **`pallet-metarium`**, the core pallet.
- **`genesis/`** — a canonical genesis (raw chain spec + runtime wasm) you can boot a node from, with
  verification instructions (see [`genesis/GENESIS.md`](genesis/GENESIS.md)).
- **`substrate/`, `polkadot/`, `cumulus/`, `bridges/`** — the vendored Polkadot SDK this builds on.

## Build & run
```bash
# the toolchain is pinned by rust-toolchain.toml
cargo build --release -p solochain-template-node

# a throwaway dev node
./target/release/solochain-template-node --dev

# or boot from the canonical genesis (block 0 is fully defined by the spec)
./target/release/solochain-template-node --chain genesis/chainspec-raw.json
```
See [`templates/solochain/README.md`](templates/solochain/README.md) for node details.

## License
- The **Metarium additions** — `pallet-metarium` and the Metarium runtime configuration — are licensed
  **GPL-3.0-only** (see [`LICENSE`](LICENSE)).
- The **vendored Polkadot SDK** retains its upstream licensing (GPL-3.0 / Apache-2.0, per crate); see
  [`NOTICE`](NOTICE) and the per-crate `Cargo.toml` / license headers.
