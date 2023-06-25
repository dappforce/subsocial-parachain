// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpecExtension, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use sp_runtime::traits::{IdentifyAccount, Verify, Zero};
use hex_literal::hex;

use subsocial_parachain_runtime::{AccountId, AuraId, EXISTENTIAL_DEPOSIT, Signature, Balance, UNIT};

pub const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "subx";
const DEFAULT_PARA_ID: u32 = 2100;

const TESTNET_DEFAULT_ENDOWMENT: Balance = 1_000_000;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<subsocial_parachain_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<polkadot_primitives::v2::Block>,
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
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn subsocial_session_keys(keys: AuraId) -> subsocial_parachain_runtime::SessionKeys {
	subsocial_parachain_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
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
					)
				],
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), TESTNET_DEFAULT_ENDOWMENT),
					(get_account_id_from_seed::<sr25519::Public>("Ferdie"), TESTNET_DEFAULT_ENDOWMENT),
				],
				DEFAULT_PARA_ID.into(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: DEFAULT_PARA_ID,
			bad_blocks: None,
		},
	)
}

pub fn local_testnet_config(relay_chain: String) -> ChainSpec {
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
				DEFAULT_PARA_ID.into(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(subsocial_properties()),
		// Extensions
		Extensions {
			relay_chain,
			para_id: DEFAULT_PARA_ID,
			bad_blocks: None,
		},
	)
}

pub fn rococo_local_testnet_config() -> ChainSpec {
	local_testnet_config("rococo-local".into())
}

pub fn kusama_local_testnet_config() -> ChainSpec {
	local_testnet_config("kusama-local".into())
}

pub fn subsocialx_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial-kusama.json")[..])
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial-polkadot.json")[..])
}

pub fn staging_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"SubsocialX",
		"subsocialx",
		ChainType::Live,
		move || {
			let mut total_allocated: Balance = Zero::zero();

			let initial_authorities: Vec<(AccountId, AuraId)> = vec![
				(
					// Collator 1
					hex!["467d5f51e8ba14e840009bcc00bafb5de1dff2d2e7263632e0a261217d51ba02"].into(),
					hex!["467d5f51e8ba14e840009bcc00bafb5de1dff2d2e7263632e0a261217d51ba02"].unchecked_into()
				),
				(
					// Collator 2
					hex!["22f17e92302cd511dd9c0c6cd3ef2912d195a0db33d586eeb77713fa17535672"].into(),
					hex!["22f17e92302cd511dd9c0c6cd3ef2912d195a0db33d586eeb77713fa17535672"].unchecked_into()
				)
			];

			let initial_allocation = vec![
				(hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(), 100_000_000)
			];

			let root_key: AccountId = hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into();

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
		None,
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: DEFAULT_PARA_ID,
			bad_blocks: None,
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
		vesting: subsocial_parachain_runtime::VestingConfig { vesting: vec![] },
		polkadot_xcm: subsocial_parachain_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		sudo: subsocial_parachain_runtime::SudoConfig {
			key: Some(root_key.clone()),
		},
		spaces: subsocial_parachain_runtime::SpacesConfig {
			endowed_account: Some(root_key),
		},
	}
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
