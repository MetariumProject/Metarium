# Metarium

Metarium is a [Substrate](https://github.com/paritytech/polkadot-sdk)-based blockchain. Its runtime adds
**`pallet-metarium`** — a content-anchoring pallet built around:

- **channels** ("mnembooks") with an **actant** (writer) / **listener** (reader) access-control model;
- **arikuris** — content pointers (self-describing content hashes, e.g. `blake3://…`) attached to a channel;
- a **custodian-metadata commit thread** that serializes concurrent writers behind an on-chain lock;
- node reachability (`NodeInfoMap`), a bookUUID ↔ channel binding, and a membership reverse-index.

It is a **template**: a sudo-governed solochain (AURA authoring + GRANDPA finality, validators managed via
`pallet-validator-set`) that you **fork, name as your own chain, then compile and launch**. Nothing here is a
live network — you generate your own genesis.

This repository is a **fork of the [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)** with the
Metarium runtime and pallet added. Anyone may clone, rename, build, run, and fork it.

## Layout
- **`templates/solochain/`** — the **Metarium solochain template**: the runtime (wiring `pallet-metarium`)
  and the `solochain-template-node`.
- **`templates/parachain/pallets/metarium/`** — **`pallet-metarium`**, the core pallet. (It lives under
  the `parachain/` path but is a **shared crate the solochain runtime depends on** — its location does
  not make Metarium a parachain. See [`AGENTS.md`](AGENTS.md).)
- **`substrate/`, `polkadot/`, `cumulus/`, `bridges/`** — the vendored Polkadot SDK this builds on.

## Make it your own chain
1. **Name it.** Set your chain's identity in `templates/solochain/runtime/src/lib.rs` — `spec_name` /
   `impl_name` (they default to `"metarium"`) — and set your token/SS58 in the runtime as you like.
2. **Build.** First install the native build prereqs (Substrate needs a C/C++ toolchain, LLVM/`libclang`
   for `librocksdb-sys` bindgen, protobuf, and cmake):
   ```bash
   # Debian/Ubuntu
   sudo apt-get install -y build-essential clang libclang-dev llvm protobuf-compiler cmake
   # macOS (Apple clang lacks a bindgen-discoverable libclang, so install LLVM and point at it)
   xcode-select --install 2>/dev/null || true
   brew install llvm protobuf cmake
   export LIBCLANG_PATH="$(brew --prefix llvm)/lib"   # add to your shell profile
   ```
   ```bash
   # the toolchain is pinned by rust-toolchain.toml
   cargo build --release -p solochain-template-node
   ```
3. **Run a dev node** to try it:
   ```bash
   ./target/release/solochain-template-node --dev
   ```
4. **Generate your own genesis** for a real network (this template ships none):
   ```bash
   ./target/release/solochain-template-node build-spec --chain local > my-chain.json
   # edit validators / sudo / balances, then convert to a raw spec:
   ./target/release/solochain-template-node build-spec --chain my-chain.json --raw > my-chain-raw.json
   ./target/release/solochain-template-node --chain my-chain-raw.json
   ```
See [`templates/solochain/README.md`](templates/solochain/README.md) for node details.

## License
- The **Metarium additions** — `pallet-metarium` and the Metarium runtime configuration — are licensed
  **GPL-3.0-only** (see [`LICENSE`](LICENSE)).
- The **vendored Polkadot SDK** retains its upstream licensing (GPL-3.0 / Apache-2.0, per crate); see
  [`NOTICE`](NOTICE) and the per-crate `Cargo.toml` / license headers.
