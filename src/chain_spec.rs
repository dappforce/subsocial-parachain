use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use rococo_parachain_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<rococo_parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_chain_spec(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
				],
				vec![
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
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	)
}

pub fn staging_test_net(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				vec![
					// $secret//one
					hex!["aad9fa2249f87a210a0f93400b7f90e47b810c6d65caa0ca3f5af982904c2a33"]
						.unchecked_into(),
					// $secret//two
					hex!["d47753f0cca9dd8da00c70e82ec4fc5501a69c49a5952a643d18802837c88212"]
						.unchecked_into(),
				],
				vec![
					hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
				],
				id,
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> rococo_parachain_runtime::GenesisConfig {
	rococo_parachain_runtime::GenesisConfig {
		system: rococo_parachain_runtime::SystemConfig {
			code: rococo_parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: rococo_parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		sudo: rococo_parachain_runtime::SudoConfig { key: root_key },
		parachain_info: rococo_parachain_runtime::ParachainInfoConfig { parachain_id: id },
		aura: rococo_parachain_runtime::AuraConfig {
			authorities: initial_authorities,
		},
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}

use subsocial_common::Balance as SubsocialBalance;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type KusocialChainSpec = sc_service::GenericChainSpec<kusocial_runtime::GenesisConfig, Extensions>;
pub type WestsocialChainSpec = sc_service::GenericChainSpec<westsocial_runtime::GenesisConfig, Extensions>;

const KUSOCIAL_ED: SubsocialBalance = kusocial_runtime::constants::currency::EXISTENTIAL_DEPOSIT;
const WESTSOCIAL_ED: SubsocialBalance = westsocial_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn kusocial_session_keys(keys: AuraId) -> kusocial_runtime::opaque::SessionKeys {
	kusocial_runtime::opaque::SessionKeys { aura: keys }
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn westsocial_session_keys(keys: AuraId) -> westsocial_runtime::opaque::SessionKeys {
	westsocial_runtime::opaque::SessionKeys { aura: keys }
}

pub fn kusocial_development_config(id: ParaId) -> KusocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	KusocialChainSpec::from_genesis(
		// Name
		"Kusocial Development",
		// ID
		"kusocial_dev",
		ChainType::Local,
		move || {
			kusocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					)
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn kusocial_local_config(id: ParaId) -> KusocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	KusocialChainSpec::from_genesis(
		// Name
		"Kusocial Local",
		// ID
		"kusocial_local",
		ChainType::Local,
		move || {
			kusocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob")
					),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
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
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama-local".into(),
			para_id: id.into(),
		},
	)
}

pub fn kusocial_config(id: ParaId) -> KusocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "KSM".into());
	properties.insert("tokenDecimals".into(), 12.into());

	KusocialChainSpec::from_genesis(
		// Name
		"Kusocial",
		// ID
		"kusocial",
		ChainType::Live,
		move || {
			kusocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_collator_keys_from_seed("Charlie")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_collator_keys_from_seed("Dave")
					),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![],
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	)
}

fn kusocial_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> kusocial_runtime::GenesisConfig {
	kusocial_runtime::GenesisConfig {
		system: kusocial_runtime::SystemConfig {
			code: kusocial_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: kusocial_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, KUSOCIAL_ED * 4096))
				.collect(),
		},
		parachain_info: kusocial_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: kusocial_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: KUSOCIAL_ED * 16,
			..Default::default()
		},
		session: kusocial_runtime::SessionConfig {
			keys: invulnerables.iter().cloned().map(|(acc, aura)| (
				acc.clone(), // account id
				acc.clone(), // validator id
				kusocial_session_keys(aura), // session keys
			)).collect()
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		spaces: kusocial_runtime::SpacesConfig {
			endowed_account: root_key,
		},
	}
}

pub fn westsocial_development_config(id: ParaId) -> WestsocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "WND".into());
	properties.insert("tokenDecimals".into(), 12.into());

	WestsocialChainSpec::from_genesis(
		// Name
		"Westsocial Development",
		// ID
		"westsocial_dev",
		ChainType::Local,
		move || {
			westsocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					)
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	)
}

pub fn westsocial_local_config(id: ParaId) -> WestsocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "WND".into());
	properties.insert("tokenDecimals".into(), 12.into());

	WestsocialChainSpec::from_genesis(
		// Name
		"Westsocial Local",
		// ID
		"westsocial_local",
		ChainType::Local,
		move || {
			westsocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob")
					),
				],
				vec![
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
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend-local".into(),
			para_id: id.into(),
		},
	)
}

pub fn westsocial_config(id: ParaId) -> WestsocialChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "WND".into());
	properties.insert("tokenDecimals".into(), 12.into());

	WestsocialChainSpec::from_genesis(
		// Name
		"Westsocial",
		// ID
		"westsocial",
		ChainType::Live,
		move || {
			westsocial_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_collator_keys_from_seed("Charlie")
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_collator_keys_from_seed("Dave")
					),
				],
				vec![],
				// re-use the Westend sudo key
				hex!("6648d7f3382690650c681aba1b993cd11e54deb4df21a3a18c3e2177de9f7342").into(),
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	)
}

fn westsocial_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	root_key: AccountId,
	id: ParaId,
) -> westsocial_runtime::GenesisConfig {
	westsocial_runtime::GenesisConfig {
		system: westsocial_runtime::SystemConfig {
			code: westsocial_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: westsocial_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, WESTSOCIAL_ED * 4096))
				.collect(),
		},
		sudo: westsocial_runtime::SudoConfig { key: root_key.clone() },
		parachain_info: westsocial_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: westsocial_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: WESTSOCIAL_ED * 16,
			..Default::default()
		},
		session: westsocial_runtime::SessionConfig {
			keys: invulnerables.iter().cloned().map(|(acc, aura)| (
				acc.clone(), // account id
				acc.clone(), // validator id
				westsocial_session_keys(aura), // session keys
			)).collect()
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		spaces: westsocial_runtime::SpacesConfig {
			endowed_account: root_key,
		},
	}
}
