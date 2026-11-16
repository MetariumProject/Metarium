# One codebase, many chains: per-chain manifests

Status: **proposed, revised 2026-11-16.** Nothing below is built.

## Why

Every Metarium chain runs the same pallets. What differs per chain is a handful of values: the runtime's
name, its version counter, the token and a few constants. Today each chain copies the template and edits
those by hand, which has already cost us twice:

- **Code went missing.** mummai's runtime was upgraded 102 → 105 from source that was never pushed anywhere.
- **A fix lands N times.** The commit-size cap change (#10) had to be made in the template and again in each
  chain's copy.

## The model

- **This repo is the central code** for every chain: pallets, runtime and node, one copy.
- **Each chain keeps its own code book** on its own archive. The book holds a snapshot of this repo plus the
  chain's **manifest**. Each chain's code book is what that chain is built from.
- **Now and then a chain pulls an upgrade** from this repo's `main` into its code book, then builds from there.
- Logic is never changed in one chain's copy only. A change goes here first, so the next pull keeps it.
  Only names, versions, the token and constants live in a manifest.

## The manifest

A small file in the chain's code book (this repo carries only `chains/metarium.toml`, the template's own
defaults):

```toml
spec_name    = "mummai"
impl_name    = "mummai"
spec_version = 106            # bump it for an upgrade
impl_version = 6

[token]
symbol   = "MAI"
decimals = 12
ss58     = 42

[constants]                   # only what this chain overrides
max_commit_size = 1_099_511_627_776
```

`build.rs` reads the file named by `METARIUM_MANIFEST` (default `chains/metarium.toml`) and generates the
`RuntimeVersion`, the overridden constants and the token values that are hard-coded today
(`enable_metadata_hash("UNIT", 12)`, `chain_spec.rs`). It re-runs when that variable or that file changes,
so a build for one chain is never reused for another.

## An upgrade

1. Pull `main` into the chain's code book and bump `spec_version` in its manifest.
2. Build: `METARIUM_MANIFEST=<path> cargo build --release -p solochain-template-runtime`.
3. Push the source and manifest to the chain's code book (on the operator's word).
4. `sudo(system.set_code)` with that build (on the operator's word). The release script first checks the
   target chain's live `spec_name` and that the new `spec_version` is higher.

Two rules keep a live chain safe:

- **The source is on chain before the upgrade.** Step 3 comes before step 4, never after.
- **A live chain's chain spec is frozen.** Its raw chainspec and genesis sit in the chain's data book and are
  never regenerated from a manifest (that would be a new chain).

## PRs

| # | Scope | Acceptance |
|---|---|---|
| **M1** | `build.rs` reads the manifest; generated `RuntimeVersion` and constants; `chains/metarium.toml`. | With the default manifest the runtime builds **byte-identical** to today's (the spike showed this works if `lib.rs` line numbers do not shift). |
| **M2** | The token into `enable_metadata_hash` and the node's chain-spec properties. | A manifest's token reaches both. |
| **M3** | A release script: build from a code book's manifest, print size + blake2, check the live chain, prepare `set_code`. | iyalkodi, built from its own code book (ch11) with its manifest, matches its live runtime. |

mummai and mnemchain move onto manifests at their next upgrade.
