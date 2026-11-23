use sc_service::ChainType;
use solochain_template_runtime::{TOKEN_DECIMALS, TOKEN_SYMBOL, WASM_BINARY};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Chain properties: the gas token comes from the chain manifest (`chains/metarium.toml`,
/// MAI / 12 decimals by default) via the runtime's `TOKEN_SYMBOL` / `TOKEN_DECIMALS` — the
/// same values baked into the runtime's metadata hash — and SS58 prefix 42.
/// Genesis issuance is intentionally tiny (100 MAI to sudo); supply is expanded
/// later via sudo. Keep this in sync with `genesis_config_presets.rs`.
fn mai_properties() -> sc_service::Properties {
	let mut p = sc_service::Properties::new();
	p.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
	p.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	p.insert("ss58Format".into(), 42.into());
	p
}

pub fn development_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_properties(mai_properties())
	.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
	.build())
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_properties(mai_properties())
	.with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
	.build())
}
