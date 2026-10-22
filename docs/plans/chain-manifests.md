# One codebase, many chains: per-chain manifests

Status: **proposed, 2026-10-22.** Nothing below is built.

## Why

Every Metarium chain mnem.com runs, or hosts as a tenant, runs the same pallets. What differs per chain is a
handful of values: the runtime's name, its version counter, the token, a few constants, and its genesis.
Today each chain gets those values by copying the template into its own repo or branch and editing them by
hand. That has two costs, and both have already been paid:

- **Code goes missing.** mummai's runtime was upgraded 102 → 105 from source that was never pushed to any
  branch. When mummai's 1 GiB commit-size cap had to be raised (2026-10-22), the source had to be
  reconstructed by matching the live metadata against another chain's.
- **A fix lands N times.** The cap change went into the Metarium template (#10), into mummai's branch, and
  would have to go into each other chain by hand.

A chain cannot simply run the template's build: the runtime's `spec_name` is compiled into the Wasm and a
chain refuses a runtime whose name differs (`can_set_code`), and each chain's `spec_version` counts up
independently (mummai 106, mnemchain 105, the template 105). So each chain needs its **own build** — but
not its own codebase.

## What changes

**One source tree (this repo), and a manifest per chain.** A build names a chain; the manifest supplies
everything that chain-specific; the same commit builds every chain.

```
chains/
  metarium.toml      # the template's own identity (today's defaults)
  mnemchain.toml
  mummai.toml
  <tenant>.toml
```

```toml
# chains/mummai.toml
spec_name    = "mummai"
impl_name    = "mummai"
spec_version = 106            # the LIVE version; a build for an upgrade bumps it
impl_version = 6

[token]
symbol   = "MAI"
decimals = 12
ss58     = 42

[constants]                   # only the ones a chain overrides; everything else is the template default
max_commit_size = 1_099_511_627_776

[live]                        # filled in when a build is enacted; the provenance record
commit       = "04e12990"
wasm_blake2  = "0xbdf813ba…b8d3"
enacted_at   = "2026-10-22, block 0xaa20f9b4…"
```

**How the build reads it.** `templates/solochain/runtime/build.rs` reads `chains/$METARIUM_CHAIN.toml`
(default `metarium`) and writes a generated `version.rs` and `chain_constants.rs` into `OUT_DIR`, which
`lib.rs` and `configs/mod.rs` `include!`. The same manifest feeds:

- the `RuntimeVersion` (spec/impl name and version);
- the pallet constants it overrides (`MaxCommitSize` first; the same pattern for the others);
- `enable_metadata_hash(symbol, decimals)` — today hard-coded as `"UNIT", 12`, which does not match MAI and
  would break Ledger signing on a production build with `metadata-hash`;
- the node's chain-spec properties (token symbol, decimals, SS58), today hard-coded in `chain_spec.rs`.

One command builds a chain:

```bash
METARIUM_CHAIN=mummai cargo build --release -p solochain-template-runtime
```

**Guards in the build.** It refuses an unknown chain name, a manifest missing a required field, and a
`spec_version` lower than `[live].spec_version` (an upgrade must go up). It prints the chain name and
version it built, so a build for the wrong chain is visible before anything is uploaded.

**Each tenant's archive keeps its record.** After an upgrade is enacted, the tenant's code book (as done
for mummai in `mummai-chain`, ch55) receives the source at that commit plus the manifest with `[live]`
filled in. The chain's code is then recoverable from the archive alone, even if GitHub is not.

## What does not change

- The pallets and runtime logic: one copy, here.
- How an upgrade is enacted: a `spec_version` bump, a build, and `sudo(system.set_code)` signed by the
  chain's sudo key, approved by the operator.
- A chain imported from outside mnem.com (mummai today) may keep its own repo. Its manifest here is enough
  for mnem.com to build it; its own repo becomes optional.

## PRs

| # | Scope | Acceptance |
|---|---|---|
| **S0** | **Spike.** Confirm `#[sp_version::runtime_version]` accepts a `RuntimeVersion` generated into `OUT_DIR` and `include!`d (the macro reads the item to emit the `runtime_version` custom section). If it needs literals, generate the whole item with literals — the build script writes Rust. | A throwaway build for two names produces two Wasms whose embedded version (read with `subwasm` or by decoding the custom section) carries the right `spec_name`/`spec_version`. |
| **M1** | `chains/` manifests for `metarium` (today's defaults), `mnemchain`, `mummai`; `build.rs` codegen for `RuntimeVersion`; the build guards. | `METARIUM_CHAIN=metarium` builds a runtime **byte-identical** to today's (same blake2), so nothing changes for the template. `mummai` builds spec 106 with name `mummai`. |
| **M2** | Constants from the manifest (`MaxCommitSize`; the pattern for the rest), and the token into `enable_metadata_hash` and the node's chain spec. | A manifest override changes the constant in the built runtime; omitted constants keep the template defaults; MAI/12/42 reach both the metadata hash and the chain spec. |
| **M3** | Release tooling: a script that builds a named chain, prints size + sha256 + blake2, and writes `[live]` after enactment; the code-book push of the source + manifest. | Building mummai from this repo reproduces the live spec-106 Wasm's behaviour (same metadata); the record lands in the tenant's code book and reads back byte-identical. |
| **M4** | Move mnemchain and mummai onto manifests: their next upgrades are built from here. | Their next `set_code` is built with `METARIUM_CHAIN=<name>`; their own repos/branches become history. |

## Risks

- **A build for the wrong chain.** The chain refuses a wrong `spec_name`, and the build prints which chain
  it built; the release script (M3) also checks the target chain's live `spec_name` before anything is
  signed — the same checks `mummai-set-code.py` makes today.
- **Template behaviour drifting.** M1's acceptance is a byte-identical default build, so the change is
  proven neutral before any chain depends on it.
- **Per-chain logic beyond constants.** If a chain ever needs different pallets, that is a Cargo feature per
  chain, still in this one tree — out of scope until a chain needs it.

## Gates

Every runtime upgrade stays operator-approved: the build and the code-book push are routine; the
`set_code` on a live chain is the operator's signature, as it is today.
