//! Runtime build script.
//!
//! Every Wasm runtime built from this template carries a **metadata hash**
//! (`CheckMetadataHash`), always — there is no build without it. A runtime
//! without one rejects every transaction from the Ledger Polkadot Generic app
//! with `Unknown: CannotLookup`, and nothing else notices: software signers keep
//! working. It happened on a live Metarium chain (mummai): a Ledger-ready
//! runtime (spec 103) was followed by plain `cargo build --release` upgrades
//! that quietly dropped the hash, so it is no longer opt-in.
//!
//! The token symbol and decimals are part of the hashed metadata, so they come
//! from the chain's manifest — `chains/metarium.toml` by default, or the file
//! named by `METARIUM_MANIFEST` (absolute, or relative to the repository root):
//!
//! ```toml
//! [token]
//! symbol   = "MAI"
//! decimals = 12
//! ```
//!
//! The same values are exported to the runtime as `TOKEN_SYMBOL` /
//! `TOKEN_DECIMALS`, and the node's chain-spec properties use those, so the
//! hash and what wallets read from `system_properties` cannot drift apart.

use std::{
	env, fs,
	path::{Path, PathBuf},
};

/// Repository root, from this crate's directory (`templates/solochain/runtime`).
fn repo_root() -> PathBuf {
	let crate_dir =
		PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"));
	crate_dir.join("../../..")
}

fn manifest_path() -> PathBuf {
	match env::var("METARIUM_MANIFEST") {
		Ok(p) if !p.trim().is_empty() => {
			let p = PathBuf::from(p.trim());
			if p.is_absolute() {
				p
			} else {
				repo_root().join(p)
			}
		},
		_ => repo_root().join("chains/metarium.toml"),
	}
}

/// Reads `[token] symbol` and `[token] decimals` from the manifest. A deliberately
/// tiny parser (no build dependency): `key = value` lines, `#` comments.
fn read_token(path: &Path) -> (String, u8) {
	let text = fs::read_to_string(path)
		.unwrap_or_else(|e| panic!("cannot read chain manifest {}: {e}", path.display()));
	let (mut in_token, mut symbol, mut decimals) = (false, None, None);
	for raw in text.lines() {
		let line = raw.split('#').next().unwrap_or("").trim();
		if line.is_empty() {
			continue;
		}
		if line.starts_with('[') {
			in_token = line == "[token]";
			continue;
		}
		if !in_token {
			continue;
		}
		if let Some((key, value)) = line.split_once('=') {
			let value = value.trim();
			match key.trim() {
				"symbol" => symbol = Some(value.trim_matches('"').to_string()),
				"decimals" =>
					decimals = Some(value.replace('_', "").parse::<u8>().unwrap_or_else(|_| {
						panic!(
							"chain manifest {}: [token] decimals must be 0-255, got {value}",
							path.display()
						)
					})),
				_ => {},
			}
		}
	}
	let symbol = symbol
		.filter(|s| {
			!s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
		})
		.unwrap_or_else(|| {
			panic!("chain manifest {}: [token] symbol is missing or invalid", path.display())
		});
	let decimals = decimals
		.unwrap_or_else(|| panic!("chain manifest {}: [token] decimals is missing", path.display()));
	(symbol, decimals)
}

/// Writes `TOKEN_SYMBOL` / `TOKEN_DECIMALS` for `lib.rs` to `include!`. Runs for both
/// the native (std) build and the inner Wasm build, which must agree.
fn export_token() -> (String, u8) {
	let path = manifest_path();
	println!("cargo:rerun-if-env-changed=METARIUM_MANIFEST");
	println!("cargo:rerun-if-changed={}", path.display());
	let (symbol, decimals) = read_token(&path);
	let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo")).join("token.rs");
	fs::write(
		&out,
		format!(
			"/// The chain's token symbol, from its manifest. Baked into the metadata hash.\n\
			 pub const TOKEN_SYMBOL: &str = \"{symbol}\";\n\
			 /// The chain's token decimals, from its manifest. Baked into the metadata hash.\n\
			 pub const TOKEN_DECIMALS: u8 = {decimals};\n"
		),
	)
	.expect("write token.rs");
	(symbol, decimals)
}

#[cfg(feature = "std")]
fn main() {
	let (symbol, decimals) = export_token();
	println!("cargo:warning=runtime metadata hash: token {symbol}, {decimals} decimals (Ledger-ready)");
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(&symbol, decimals)
		.build();
}

/// The Wasm builder is not run when compiling this crate for Wasm, but the token
/// constants still have to exist for `lib.rs`.
#[cfg(not(feature = "std"))]
fn main() {
	export_token();
}
