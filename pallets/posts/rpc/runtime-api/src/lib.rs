//! Runtime API definition for posts pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;

use subsocial_support::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait PostsApi<AccountId>
        where
            AccountId: Codec + MaybeDisplay,
    {
        fn check_account_can_create_post(account: AccountId, space_id: SpaceId) -> bool;
    }
}
