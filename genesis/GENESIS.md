# mnemchain genesis (canonical, version-controlled)

This directory is the **single source of truth for block 0**. Anyone with access
to this GitHub repo can bootstrap or recover the chain from here — no other state
is required to reach genesis.

| | |
|---|---|
| Chain id | `mnemchain` |
| Genesis hash | `0x4e2b321ea509d94bb5aa4b9d192541ca047a903928e860c85d2325531f904df6` |
| Token | MAI, 12 decimals, SS58 42 |
| Genesis issuance | 100 MAI, all to sudo |
| Consensus | AURA (authoring) + GRANDPA (finality), validators via sudo `pallet-validator-set` |

## Files
- **`chainspec-raw.json`** — the raw chain spec. Its `genesis.raw.top` includes the
  genesis runtime under the `:code` key, so this file alone fully defines block 0.
  Start any node with `--chain genesis/chainspec-raw.json`.
- **`mnemchain-genesis.wasm`** — the genesis runtime wasm, extracted for convenience
  / verification (`:code` from the raw spec).
  `sha256 = ad5791abec282335145ad45ff93b1641597d72cccf9d5adee1bfd15123ddf37c`.

## Verifying the wasm matches the spec
```bash
python3 - <<'PY'
import json,binascii,hashlib
d=json.load(open("chainspec-raw.json"))
code=binascii.unhexlify(d["genesis"]["raw"]["top"]["0x3a636f6465"][2:])
print(hashlib.sha256(code).hexdigest())   # must equal the sha256 above
PY
```

## Recovery contract
The chain is fully reconstructable from two durable sources:
1. **This directory (in GitHub)** → block 0 + the genesis wasm.
2. **Block history** → blocks 1..N, replayed via `import-blocks` from a block dump
   and/or served by a block explorer indexing the chain.

> Regenerate this directory whenever genesis changes (new token / issuance / validators).
