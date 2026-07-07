# pallet-metarium
Substrate pallet for Metarium — the content-anchoring pallet at the heart of the Metarium solochain.

Regenerate weights (requires a `--features runtime-benchmarks` build):

./target/release/solochain-template-node benchmark pallet --chain dev --pallet pallet_metarium --extrinsic "*" --output templates/solochain/pallets/metarium/src/weights.rs

./target/release/solochain-template-node benchmark pallet --chain dev --list
