// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use pallet_spaces::rpc::FlatSpace;
use pallet_utils::SpaceId;

sp_api::decl_runtime_apis! {
    pub trait SpacesApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_next_space_id() -> SpaceId;

        fn get_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_spaces_by_ids(space_ids: Vec<SpaceId>) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_public_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_unlisted_spaces(start_id: u64, limit: u64) -> Vec<FlatSpace<AccountId, BlockNumber>>;

        fn get_public_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId>;

        fn get_unlisted_space_ids_by_owner(owner: AccountId) -> Vec<SpaceId>;

        fn get_space_by_handle(handle: Vec<u8>) -> Option<FlatSpace<AccountId, BlockNumber>>;

        fn get_space_id_by_handle(handle: Vec<u8>) -> Option<SpaceId>;
    }
}
