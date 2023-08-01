// Copyright (C) DAPPFORCE PTE. LTD., dappforce@gmail.com.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use pallet_profiles::rpc::FlatSocialAccount;

sp_api::decl_runtime_apis! {
    pub trait ProfilesApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_social_accounts_by_ids(
            account_ids: Vec<AccountId>
        ) -> Vec<FlatSocialAccount<AccountId, BlockNumber>>;
    }
}
