// Copyright (C) DAPPFORCE PTE. LTD., dappforce@gmail.com.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait ProfileFollowsApi<AccountId> where
        AccountId: Codec
    {
        fn filter_followed_accounts(account: AccountId, maybe_following: Vec<AccountId>) -> Vec<AccountId>;
    }
}
