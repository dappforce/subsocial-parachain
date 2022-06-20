use frame_support::pallet_prelude::*;

use crate::mock::*;
use crate::utils::{ACCOUNT1, ACCOUNT2};

pub(crate) fn _default_follow_account() -> DispatchResult {
    _follow_account(None, None)
}

pub(crate) fn _follow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
    ProfileFollows::follow_account(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        account.unwrap_or(ACCOUNT1),
    )
}

pub(crate) fn _default_unfollow_account() -> DispatchResult {
    _unfollow_account(None, None)
}

pub(crate) fn _unfollow_account(origin: Option<Origin>, account: Option<AccountId>) -> DispatchResult {
    ProfileFollows::unfollow_account(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        account.unwrap_or(ACCOUNT1),
    )
}