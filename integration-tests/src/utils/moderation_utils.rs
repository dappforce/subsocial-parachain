use frame_support::assert_ok;
use frame_support::pallet_prelude::*;

use pallet_moderation::{EntityId, EntityStatus, ReportId};
use pallet_utils::{Content, SpaceId};
use pallet_utils::mock_functions::valid_content_ipfs;

use crate::mock::*;
use crate::utils::{ACCOUNT1, POST1, SPACE1};

// Moderation pallet mocks
// FIXME: remove when linter error is fixed
#[allow(dead_code)]
const REPORT1: ReportId = 1;

pub(crate) fn _report_default_post() -> DispatchResult {
    _report_entity(None, None, None, None)
}

pub(crate) fn _report_entity(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    reason: Option<Content>,
) -> DispatchResult {
    Moderation::report_entity(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        reason.unwrap_or_else(valid_content_ipfs),
    )
}

pub(crate) fn _suggest_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status: Option<Option<EntityStatus>>,
    report_id_opt: Option<Option<ReportId>>,
) -> DispatchResult {
    Moderation::suggest_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status.unwrap_or(Some(EntityStatus::Blocked)),
        report_id_opt.unwrap_or(Some(REPORT1)),
    )
}

pub(crate) fn _update_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status_opt: Option<Option<EntityStatus>>,
) -> DispatchResult {
    Moderation::update_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status_opt.unwrap_or(Some(EntityStatus::Allowed)),
    )
}

pub(crate) fn _delete_entity_status(
    origin: Option<Origin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
) -> DispatchResult {
    Moderation::delete_entity_status(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
    )
}

/*------------------------------------------------------------------------------------------------*/
// Moderation tests

pub(crate) fn block_account_in_space_1() {
    assert_ok!(
            _update_entity_status(
                None,
                Some(EntityId::Account(ACCOUNT1)),
                Some(SPACE1),
                Some(Some(EntityStatus::Blocked))
            )
        );
}

pub(crate) fn block_content_in_space_1() {
    assert_ok!(
            _update_entity_status(
                None,
                Some(EntityId::Content(valid_content_ipfs())),
                Some(SPACE1),
                Some(Some(EntityStatus::Blocked))
            )
        );
}