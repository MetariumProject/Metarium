//! Runtime build script — one source, a manifest per chain.
//!
//! Every Metarium chain builds from this same source. What differs per chain lives in its
//! **manifest** (docs/plans/chain-manifests.md): `chains/metarium.toml` by default, or the file
//! named by `METARIUM_MANIFEST` (absolute, or relative to the repository root). A chain keeps its
//! manifest in its own data book and points the build at it:
//!
//! ```toml
//! spec_name    = "mummai"
//! impl_name    = "mummai"
//! spec_version = 108          # bump it for an upgrade
//! impl_version = 6
//!
//! [token]
//! symbol   = "MAI"
//! decimals = 12
//! ss58     = 42
//! ```
//!
//! From it this script generates, into `OUT_DIR`:
//! - `version.rs` — the runtime's `VERSION` (`include!`d by `lib.rs`);
//! - `token.rs` — `TOKEN_SYMBOL` / `TOKEN_DECIMALS`, which the node's chain-spec properties use.
//!
//! It also ALWAYS builds the Wasm with the **metadata hash** for that token (`CheckMetadataHash`).
//! A runtime without one rejects every Ledger Polkadot Generic app transaction with
//! `Unknown: CannotLookup` while software signers keep working — that happened on a live Metarium
//! chain (mummai: spec 103 was Ledger-ready, plain builds 104–106 silently dropped it), so there is
//! no build without it.
//!
//! The build prints the chain, version and token it built for. It re-runs when `METARIUM_MANIFEST`
//! or the manifest file changes, so a build for one chain is never reused for another.

use std::{
	collections::BTreeMap,
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

/// The manifest's `key = value` pairs, keyed by `section.key` (top-level keys have no prefix).
/// A deliberately tiny parser (no build dependency): `#` comments, `[section]` headers,
/// "quoted" strings and integers with `_` separators.
struct Manifest {
	path: PathBuf,
	values: BTreeMap<String, String>,
}

impl Manifest {
	fn read(path: &Path) -> Self {
		let text = fs::read_to_string(path)
			.unwrap_or_else(|e| panic!("cannot read chain manifest {}: {e}", path.display()));
		let (mut section, mut values) = (String::new(), BTreeMap::new());
		for (n, raw) in text.lines().enumerate() {
			let line = raw.split('#').next().unwrap_or("").trim();
			if line.is_empty() {
				continue;
			}
			if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
				section = name.trim().to_string();
				continue;
			}
			let (key, value) = line.split_once('=').unwrap_or_else(|| {
				panic!("chain manifest {} line {}: expected `key = value`", path.display(), n + 1)
			});
			let key = if section.is_empty() {
				key.trim().to_string()
			} else {
				format!("{section}.{}", key.trim())
			};
			let value = value.trim();
			let value = value
				.strip_prefix('"')
				.and_then(|v| v.strip_suffix('"'))
				.map(str::to_string)
				.unwrap_or_else(|| value.replace('_', ""));
			if values.insert(key.clone(), value).is_some() {
				panic!("chain manifest {}: `{key}` is set twice", path.display());
			}
		}
		Manifest { path: path.to_path_buf(), values }
	}

	fn get(&self, key: &str) -> Option<&str> {
		self.values.get(key).map(String::as_str)
	}

	fn name(&self, key: &str) -> String {
		let v = self
			.get(key)
			.unwrap_or_else(|| panic!("chain manifest {}: `{key}` is missing", self.path.display()));
		assert!(
			!v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
			"chain manifest {}: `{key}` = {v:?} must be non-empty [A-Za-z0-9_-]",
			self.path.display()
		);
		v.to_string()
	}

	fn number<T: std::str::FromStr>(&self, key: &str) -> T {
		let v = self
			.get(key)
			.unwrap_or_else(|| panic!("chain manifest {}: `{key}` is missing", self.path.display()));
		v.parse().unwrap_or_else(|_| {
			panic!("chain manifest {}: `{key}` = {v:?} is not a valid number", self.path.display())
		})
	}
}

struct Chain {
	spec_name: String,
	impl_name: String,
	spec_version: u32,
	impl_version: u32,
	symbol: String,
	decimals: u8,
}

/// Reads the manifest and writes `version.rs` + `token.rs` for `lib.rs` to `include!`. Runs for
/// both the native (std) build and the inner Wasm build, which therefore always agree.
fn generate() -> Chain {
	let path = manifest_path();
	println!("cargo:rerun-if-env-changed=METARIUM_MANIFEST");
	println!("cargo:rerun-if-changed={}", path.display());
	let m = Manifest::read(&path);

	let spec_name = m.name("spec_name");
	let chain = Chain {
		impl_name: m.get("impl_name").map(|_| m.name("impl_name")).unwrap_or_else(|| spec_name.clone()),
		spec_name,
		spec_version: m.number("spec_version"),
		impl_version: m.number("impl_version"),
		symbol: m.name("token.symbol"),
		decimals: m.number("token.decimals"),
	};
	// The SS58 prefix is still a runtime constant (configs: `SS58Prefix = 42`); refuse a manifest
	// that expects anything else rather than build a runtime that silently disagrees with it.
	if let Some(ss58) = m.get("token.ss58") {
		assert!(
			ss58 == "42",
			"chain manifest {}: token.ss58 = {ss58} — only 42 is supported (runtime SS58Prefix)",
			path.display()
		);
	}
	for key in m.values.keys() {
		assert!(
			matches!(
				key.as_str(),
				"spec_name" |
					"impl_name" | "spec_version" |
					"impl_version" | "token.symbol" |
					"token.decimals" | "token.ss58"
			),
			"chain manifest {}: unknown key `{key}` (not supported by this source yet)",
			path.display()
		);
	}

	let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
	let Chain { spec_name, impl_name, spec_version, impl_version, symbol, decimals } = &chain;
	fs::write(
		out.join("version.rs"),
		format!(
			"#[sp_version::runtime_version]\n\
			 pub const VERSION: RuntimeVersion = RuntimeVersion {{\n\
			 \tspec_name: alloc::borrow::Cow::Borrowed({spec_name:?}),\n\
			 \timpl_name: alloc::borrow::Cow::Borrowed({impl_name:?}),\n\
			 \tauthoring_version: 1,\n\
			 \tspec_version: {spec_version},\n\
			 \timpl_version: {impl_version},\n\
			 \tapis: apis::RUNTIME_API_VERSIONS,\n\
			 \ttransaction_version: 1,\n\
			 \tsystem_version: 1,\n\
			 }};\n"
		),
	)
	.expect("write version.rs");
	fs::write(
		out.join("token.rs"),
		format!(
			"/// The chain's token symbol, from its manifest. Baked into the metadata hash.\n\
			 pub const TOKEN_SYMBOL: &str = {symbol:?};\n\
			 /// The chain's token decimals, from its manifest. Baked into the metadata hash.\n\
			 pub const TOKEN_DECIMALS: u8 = {decimals};\n"
		),
	)
	.expect("write token.rs");
	chain
}

/// Pin the Wasm build to THIS repository's `Cargo.lock`.
///
/// `substrate-wasm-builder` looks for the lock by walking UP from `OUT_DIR` — i.e. from the
/// target directory, not from the workspace being built. With an external `CARGO_TARGET_DIR`
/// (how chain runtimes are built) it found some other checkout's lock, or none at all and
/// resolved dependencies fresh: mummai spec 107 was built against an unrelated checkout's lock,
/// and a rebuild of the same commit elsewhere pulled newer crates and came out different
/// (2026-11-23). Pointing `WASM_BUILD_WORKSPACE_HINT` at the repo root makes every build use the
/// committed lock, so a runtime is reproducible from its source commit alone.
#[cfg(feature = "std")]
fn pin_cargo_lock() {
	println!("cargo:rerun-if-env-changed=WASM_BUILD_WORKSPACE_HINT");
	if env::var_os("WASM_BUILD_WORKSPACE_HINT").is_none() {
		let root = repo_root().canonicalize().expect("repository root exists");
		assert!(
			root.join("Cargo.lock").exists(),
			"no Cargo.lock at the repository root {} — refusing an unpinned runtime build",
			root.display()
		);
		// Build scripts are single-threaded; the Wasm builder below reads this in-process.
		env::set_var("WASM_BUILD_WORKSPACE_HINT", &root);
	}
}

#[cfg(feature = "std")]
fn main() {
	pin_cargo_lock();
	let c = generate();
	println!(
		"cargo:warning=runtime {} spec {} (impl {} {}) · metadata hash: token {}, {} decimals (Ledger-ready)",
		c.spec_name, c.spec_version, c.impl_name, c.impl_version, c.symbol, c.decimals
	);
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(&c.symbol, c.decimals)
		.build();
}

/// The Wasm builder is not run when compiling this crate for Wasm, but the generated files still
/// have to exist for `lib.rs`.
#[cfg(not(feature = "std"))]
fn main() {
	generate();
}
