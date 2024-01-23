//! Runtime API definition for posts pallet.

#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


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
