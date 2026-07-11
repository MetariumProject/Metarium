#[cfg(all(feature = "std", feature = "metadata-hash"))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		// Set to the chain's REAL registered tokenSymbol/tokenDecimals before a
		// production build: these values are part of the hashed metadata, and a
		// mismatch — or building without the `metadata-hash` feature at all —
		// makes the Ledger Polkadot Generic App unable to sign on the chain.
		.enable_metadata_hash("UNIT", 12)
		.build();
}

#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
	substrate_wasm_builder::WasmBuilder::build_using_defaults();
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
