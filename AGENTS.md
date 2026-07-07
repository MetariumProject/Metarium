# Agent notes — Metarium

**Metarium is a SOLOCHAIN.** Build and extend **`templates/solochain`** — a standalone L1 with AURA
authoring + GRANDPA finality, sudo-governed validators (`pallet-validator-set`). The node binary is
**`solochain-template-node`**.

## Don't get this wrong
- **`pallet-metarium`** (the core pallet: channels/mnembooks, arikuris, custodianship,
  `PrincipalChannelOf`, …) lives with the solochain at **`templates/solochain/pallets/metarium/`** —
  Metarium is a **solochain-first** project. The **solochain** runtime
  (`templates/solochain/runtime`) depends on it; edit the pallet there and it flows into the runtime.
- **Do NOT build or extend `templates/parachain` for the Metarium chain.** It is upstream Polkadot SDK
  and pulls `revm` (via `pallet-revive`) + cumulus, which won't build on the pinned
  `rust-toolchain.toml`. Only the solochain is the Metarium chain. (The parachain runtime still lists
  `pallet-metarium` as a workspace dependency, but it is not the Metarium chain — don't build it.)
- The `minimal` and `zombienet` templates are also upstream scaffolding, not the Metarium chain.

## Build
```bash
cargo build -p solochain-template-node      # the chain
cargo test  -p pallet-metarium              # the pallet's unit tests (fast, no native C++ deps)
```
macOS needs the native prereqs (see `README.md`): `brew install llvm protobuf cmake`,
`export LIBCLANG_PATH="$(brew --prefix llvm)/lib"` (for `librocksdb-sys`), and a working C++ sysroot
for `cxx`/`wasm-opt` (e.g. `export SDKROOT="$(xcrun --show-sdk-path)"`). Linux builds cleanly.

## Where things live
- Runtime config (pallet wiring): `templates/solochain/runtime/src/configs/mod.rs`
- Genesis presets (dev/local): `templates/solochain/runtime/src/genesis_config_presets.rs`
- The pallet: `templates/solochain/pallets/metarium/src/` (see above)
