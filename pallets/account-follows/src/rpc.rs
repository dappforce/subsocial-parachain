// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use sp_std::prelude::*;

use crate::{Module, Config};

impl<T: Config> Module<T> {
    pub fn filter_followed_accounts(account: T::AccountId, maybe_following: Vec<T::AccountId>) -> Vec<T::AccountId> {
        maybe_following.iter()
            .filter(|maybe_following| Self::account_followed_by_account((&account, maybe_following)))
            .cloned().collect()
    }
}