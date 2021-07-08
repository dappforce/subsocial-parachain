use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use subsocial_parachain_primitives::{AccountId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_telemetry::TelemetryEndpoints;

use parachain_runtime::constants::currency::SUBS;

pub const SUBSOCIAL_PARACHAIN_ID: u32 = 28;
// Note this is the URL for the telemetry server
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn local_testnet_config(id: ParaId) -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			let endowed_accounts = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			];

			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				endowed_accounts.iter().cloned().map(|k| (k, 10_000)).collect(),
				id,
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			)
		},
		vec![],
		None,
		None,
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "local_testnet".into(),
			para_id: id.into(),
		},
	))
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocial.json")[..])
}

pub fn staging_test_net(id: ParaId) -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		"Subsocial PC",
		"subsocial_parachain",
		ChainType::Live,
		move || testnet_genesis(
			/* Sudo Account */
			hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(),
			vec![
				(
					/* Sudo Account */
					hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(),
					/* Balance */
					1_000
				),
				(
					/* Account X1 */
					hex!["24d6d996a8bb42a63904afc36d610986e8d502f65898da62cb281cfe7f23b02f"].into(),
					/* Balance */
					2_499_000
				),
				(
					/* Account X2 */
					hex!["24d6d8fc5d051fd471e275f14c83e95287d2b863e4cc802de1f78dea06c6ca78"].into(),
					/* Balance */
					2_500_000
				),
				(
					/* Account X3 */
					hex!["24d6d901fb0531124040630e52cfd746ef7d037922c4baf290f513dbc3d47d66"].into(),
					/* Balance */
					2_500_000
				),
				(
					/* Account X4 */
					hex!["24d6d22d63313e82f9461281cb69aacad1828dc74273274751fd24333b182c68"].into(),
					/* Balance */
					2_500_000
				),
			],
			// Treasury
			id,
			hex!["24d6d683750c4c10e90dd81430efec95133e1ec1f5be781d3267390d03174706"].into(),
		),
		vec![],
		Some(TelemetryEndpoints::new(
			vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
		).expect("Staging telemetry url is valid; qed")),
		Some(DEFAULT_PROTOCOL_ID),
		Some(subsocial_properties()),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	))
}

fn testnet_genesis(
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	id: ParaId,
	treasury_account_id: AccountId,
) -> parachain_runtime::GenesisConfig {
	parachain_runtime::GenesisConfig {
		frame_system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|(k, b)| (k, b * SUBS))
				.collect(),
		},
		pallet_sudo: parachain_runtime::SudoConfig { key: root_key.clone() },
		parachain_info: parachain_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_utils: parachain_runtime::UtilsConfig {
			treasury_account: treasury_account_id,
		},
		pallet_spaces: parachain_runtime::SpacesConfig {
			endowed_account: root_key,
		},
	}
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "SUB".into());

	properties
}
