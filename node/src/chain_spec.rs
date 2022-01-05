use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use sp_runtime::traits::{IdentifyAccount, Verify, Zero};
use hex_literal::hex;

use subsocial_parachain_runtime::{AccountId, AuraId, EXISTENTIAL_DEPOSIT, Signature, Balance, UNIT};
use crate::command::DEFAULT_PARA_ID;

pub const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "subx";

const TESTNET_DEFAULT_ENDOWMENT: Balance = 1_000_000;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<subsocial_parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
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

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn subsocial_session_keys(keys: AuraId) -> subsocial_parachain_runtime::SessionKeys {
	subsocial_parachain_runtime::SessionKeys { aura: keys }
}

pub fn development_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			parachain_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Ferdie"), TESTNET_DEFAULT_ENDOWMENT),
				],
				id,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			)
		},
		vec![],
		None,
		None,
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn local_testnet_config(id: ParaId, relay_chain: String) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Local Subsocial Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			parachain_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Ferdie"), TESTNET_DEFAULT_ENDOWMENT),
				],
				id,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		// Properties
		Some(subsocial_properties()),
		// Extensions
		Extensions {
			relay_chain,
			para_id: id.into(),
		},
	)
}

pub fn rococo_local_testnet_config(id: ParaId) -> ChainSpec {
	local_testnet_config(id, "rococo-local".into())
}

pub fn kusama_local_testnet_config(id: ParaId) -> ChainSpec {
	local_testnet_config(id, "kusama-local".into())
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial.json")[..])
}

pub fn staging_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// TODO: make it different from a Standalone chain
		"Subsocial",
		// TODO: make it different from a Standalone chain
		"subsocial",
		ChainType::Live,
		move || {
			let mut total_allocated: Balance = Zero::zero();

			let initial_authorities: Vec<(AccountId, AuraId)> = vec![
				// TODO: fill with `(AccountId, AuraId)`
			];

			let initial_allocation = vec![
				// TODO: fill with `(who, how_much)`
			];

			// TODO: put expected `Sudo` account here
			//	FIXME: Alice now
			let root_key: AccountId = hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"].into();

			const EXISTENTIAL_DEPOSIT_VALUE: Balance = EXISTENTIAL_DEPOSIT / UNIT;
			let unique_allocation_accounts = initial_allocation
				.iter()
				.map(|(account_id, amount)| {
					assert!(*amount >= EXISTENTIAL_DEPOSIT_VALUE, "allocation amount must gte ED");
					total_allocated = total_allocated
						.checked_add(*amount)
						.expect("shouldn't overflow when building genesis");

					account_id
				})
				.cloned()
				.collect::<std::collections::BTreeSet<_>>();

			assert!(
				unique_allocation_accounts.len() == initial_allocation.len(),
				"duplicate allocation accounts in genesis."
			);

			assert_eq!(
				total_allocated,
				100_000_000, // 100 million SUB
				"total allocation must be equal to 100 million SUB"
			);

			parachain_genesis(
				initial_authorities,
				initial_allocation,
				DEFAULT_PARA_ID.into(),
				root_key,
			)
		},
		vec![],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some(DEFAULT_PROTOCOL_ID),
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: DEFAULT_PARA_ID,
		},
	)
}

fn parachain_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
	root_key: AccountId,
) -> subsocial_parachain_runtime::GenesisConfig {
	subsocial_parachain_runtime::GenesisConfig {
		system: subsocial_parachain_runtime::SystemConfig {
			code: subsocial_parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: subsocial_parachain_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|(account, balance)| {
				(account, balance.saturating_mul(UNIT))
			}).collect(),
		},
		parachain_info: subsocial_parachain_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: subsocial_parachain_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: subsocial_parachain_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                       // account id
						acc,                 	        			 // validator id
						subsocial_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		sudo: subsocial_parachain_runtime::SudoConfig {
			key: root_key,
		},
	}
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 11.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}