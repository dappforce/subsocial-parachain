#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;
use pallet_utils::SpaceId;
use pallet_permissions::SpacePermission;

sp_api::decl_runtime_apis! {
    pub trait RolesApi<AccountId> where
        AccountId: Codec
    {
        fn get_space_permissions_by_account(account: AccountId, space_id: SpaceId) -> Vec<SpacePermission>;

        fn get_accounts_with_any_role_in_space(space_id: SpaceId) -> Vec<AccountId>;

        fn get_space_ids_for_account_with_any_role(account_id: AccountId) -> Vec<SpaceId>;
    }
}
