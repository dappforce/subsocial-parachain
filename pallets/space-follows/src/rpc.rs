use sp_std::prelude::*;

use pallet_utils::SpaceId;

use crate::{Module, Config};

impl<T: Config> Module<T> {
    pub fn get_space_ids_followed_by_account(account: T::AccountId) -> Vec<SpaceId> {
        Self::spaces_followed_by_account(account)
    }

    pub fn filter_followed_space_ids(account: T::AccountId, space_ids: Vec<SpaceId>) -> Vec<SpaceId> {
        space_ids.iter()
            .filter(|space_id| Self::space_followed_by_account((&account, space_id)))
            .cloned().collect()
    }
}