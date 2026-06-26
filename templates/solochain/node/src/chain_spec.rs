use sc_service::ChainType;
use solochain_template_runtime::WASM_BINARY;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Chain properties: the gas token is MAI (12 decimals), SS58 prefix 42.
/// Genesis issuance is intentionally tiny (100 MAI to sudo); supply is expanded
/// later via sudo. Keep this in sync with `genesis_config_presets.rs`.
fn mai_properties() -> sc_service::Properties {
	let mut p = sc_service::Properties::new();
	p.insert("tokenSymbol".into(), "MAI".into());
	p.insert("tokenDecimals".into(), 12.into());
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
