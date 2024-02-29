use node_template_runtime::{
	constants::currency::DOLLARS, AccountId, RuntimeGenesisConfig, Signature, WASM_BINARY, 
	BABE_GENESIS_EPOCH_CONFIG, opaque::SessionKeys, Balance,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use hex_literal::hex;
use sp_core::crypto::UncheckedInto;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, AuraId, GrandpaId, ImOnlineId, AuthorityDiscoveryId) {
	(get_account_id_from_seed::<sr25519::Public>(s), get_account_id_from_seed::<sr25519::Public>(s), get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s), get_from_seed::<ImOnlineId>(s), get_from_seed::<AuthorityDiscoveryId>(s))
}

fn session_keys(
	aura: AuraId,
	grandpa: GrandpaId,
 	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId, 
) -> SessionKeys {
	SessionKeys { aura, grandpa, im_online, authority_discovery}
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		get_endowed_accounts_with_balance(),
		true,
	))
	.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		get_endowed_accounts_with_balance(),
		true,
	))
	.build())
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		AuraId,
		GrandpaId,
 		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			// Configure endowed accounts with initial balance of 1 << 60.
			"balances": endowed_accounts.iter().cloned().map(|x| (x.0, 1u64 << 60)).collect::<Vec<_>>(),
		},
		// "aura": {
		// 	"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		// },
		// "grandpa": {
		// 	"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		// },
/* 		"assets" : {
			"assets": vec![(1, root_key.clone(), true, 1)], // Genesis assets: id, owner, is_sufficient, min_balance
			//"metadata": vec![(1, "XUSD".into(), "XUSD".into(), 0)], // Genesis metadata: id, name, symbol, decimals
			"accounts": endowed_accounts.iter().cloned().map(|x| (1, x.0.clone(), 1_000_000)).collect::<Vec<_>>(),
		}, */
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key),
		},
		"staking": {
			"validatorCount": initial_authorities.len() as u32,
			"minimumValidatorCount": initial_authorities.len() as u32,
			"invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
			"slashRewardFraction": Perbill::from_percent(10),
		},
		"babe": {
			"epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		"session": {
			"keys": initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone() , x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
	})
}

pub fn get_endowed_accounts_with_balance() -> Vec<(AccountId, u128)> {
	let accounts: Vec<AccountId> = vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
	];

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	let accounts_with_balance: Vec<(AccountId, u128)> =
		accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect();
	let json_data = &include_bytes!("../../seed/balances.json")[..];
	let additional_accounts_with_balance: Vec<(AccountId, u128)> =
		serde_json::from_slice(json_data).unwrap_or_default();

	let mut accounts = additional_accounts_with_balance.clone();

	accounts_with_balance.iter().for_each(|tup1| {
		for tup2 in additional_accounts_with_balance.iter() {
			if tup1.0 == tup2.0 {
				return;
			}
		}
		accounts.push(tup1.to_owned());
	});

	accounts
}