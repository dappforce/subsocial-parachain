// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpaceFollowsApi<AccountId> where
        AccountId: Codec
    {
        fn get_space_ids_followed_by_account(account: AccountId) -> Vec<SpaceId>;

        fn filter_followed_space_ids(account: AccountId, space_ids: Vec<SpaceId>) -> Vec<SpaceId>;
    }
}
