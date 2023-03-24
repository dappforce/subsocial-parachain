use frame_support::{assert_ok, pallet_prelude::*};
use pallet_permissions::SpacePermissions;
use pallet_spaces::{types::SpaceUpdate, SpaceById};
use sp_core::storage::Storage;
use sp_io::TestExternalities;
use subsocial_support::{Content, SpaceId};

use crate::mock::*;

////// Ext Builder

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let mut accounts = Vec::new();
        for account in ACCOUNT1..=ACCOUNT3 {
            accounts.push(account);
        }

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: accounts.iter().cloned().map(|k| (k, 100)).collect(),
        }
        .assimilate_storage(storage);
    }

    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    fn add_default_space() {
        assert_ok!(_create_default_space());
    }

    /// Custom ext configuration with SpaceId 1 and BlockNumber 1
    pub fn build_with_space() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_default_space);
        ext
    }

    /// Custom ext configuration with space follow without Space
    pub fn build_with_space_follow_no_space() -> TestExternalities {
        let mut ext = Self::build_with_space();

        ext.execute_with(|| {
            assert_ok!(_default_follow_space());
            <SpaceById<Test>>::remove(SPACE1);
        });

        ext
    }
}

////// Consts

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;

///////////// Space Utils

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn space_update(content: Option<Content>, hidden: Option<bool>) -> SpaceUpdate {
    SpaceUpdate { content, hidden, permissions: None }
}

pub(crate) fn _create_default_space() -> DispatchResult {
    _create_space(None, None, None, None)
}

pub(crate) fn _create_space(
    origin: Option<RuntimeOrigin>,
    // FIXME: we don't have handles anymore
    _handle: Option<Option<Vec<u8>>>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>,
) -> DispatchResult {
    Spaces::create_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        content.unwrap_or_else(space_content_ipfs),
        permissions.unwrap_or_default(),
    )
}


pub(crate) fn _update_space(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    update: Option<SpaceUpdate>,
) -> DispatchResult {
    Spaces::update_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        update.unwrap_or_else(|| space_update(None, None)),
    )
}

//// Space follows utils

pub(crate) fn _default_follow_space() -> DispatchResult {
    _follow_space(None, None)
}

pub(crate) fn _follow_space(origin: Option<RuntimeOrigin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceFollows::follow_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

pub(crate) fn _default_unfollow_space() -> DispatchResult {
    _unfollow_space(None, None)
}

pub(crate) fn _unfollow_space(origin: Option<RuntimeOrigin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceFollows::unfollow_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}
