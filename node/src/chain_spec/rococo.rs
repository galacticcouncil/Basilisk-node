use super::*;

const INITIAL_BALANCE: u128 = 10_000;

pub fn parachain_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	let genesis_json = parachain_genesis(
		// initial_authorities
		(
			vec![
				(
					// 5CcMLZnK8RNMfurDsRXHwtabSKt8ZmG3ry5G3sAeRXfj4QK2
					hex!["1822c7a002c35274bd5da15690e9d0027d9d189998990fcefd4458f768109a57"].into(),
					hex!["1822c7a002c35274bd5da15690e9d0027d9d189998990fcefd4458f768109a57"].unchecked_into(),
				),
				(
					// 5CfHZGU9iFpv2mRd9jBDu1VT6yNPFL3xsjnk971bsGBmuZ8x
					hex!["1a5fc9b99feaac2b2dcb8473b1b8e5d641296394233685499b7222edceb40327"].into(),
					hex!["1a5fc9b99feaac2b2dcb8473b1b8e5d641296394233685499b7222edceb40327"].unchecked_into(),
				),
			],
			// candidacy bond
			10_000 * UNITS,
		),
		// endowed_accounts
		vec![
			(
				hex!["3418b257de81886bef265495f3609def9a083869f32ef5a03f7351956497d41a"].into(),
				INITIAL_BALANCE,
			), // sudo
			(
				hex!["1822c7a002c35274bd5da15690e9d0027d9d189998990fcefd4458f768109a57"].into(),
				INITIAL_BALANCE,
			), // collator-01
			(
				hex!["1a5fc9b99feaac2b2dcb8473b1b8e5d641296394233685499b7222edceb40327"].into(),
				INITIAL_BALANCE,
			), // collator-02
		],
		// council_members
		vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
		// tech_committee_members
		vec![hex!["3418b257de81886bef265495f3609def9a083869f32ef5a03f7351956497d41a"].into()], // same as sudo
		// registered_assets
		vec![
			(b"KSM".to_vec(), 1_000u128, Some(1u32)),
			(b"KUSD".to_vec(), 1_000u128, Some(2u32)),
		],
		// accepted_assets
		vec![(1, Price::from_float(0.0000212)), (2, Price::from_float(0.000806))],
		// token_balances
		vec![],
		// elections
		vec![],
		// parachain ID
		PARA_ID.into(),
	);

	let chain_spec = ChainSpec::builder(
		wasm_binary,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	)
	.with_name("Basilisk Testnet (Rococo)")
	.with_id("testnet_local")
	.with_chain_type(ChainType::Local)
	.with_boot_nodes(vec![])
	.with_properties(properties)
	.with_protocol_id(PROTOCOL_ID)
	.with_genesis_config_patch(genesis_json)
	.build();

	Ok(chain_spec)
}
