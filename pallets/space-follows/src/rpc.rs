// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use sp_std::prelude::*;

use pallet_utils::SpaceId;

use crate::{Config, Pallet};

impl<T: Config> Pallet<T> {
    pub fn get_space_ids_followed_by_account(account: T::AccountId) -> Vec<SpaceId> {
        Self::spaces_followed_by_account(account)
    }

    pub fn filter_followed_space_ids(account: T::AccountId, space_ids: Vec<SpaceId>) -> Vec<SpaceId> {
        space_ids.iter()
            .filter(|space_id| Self::space_followed_by_account((&account, space_id)))
            .cloned().collect()
    }
}