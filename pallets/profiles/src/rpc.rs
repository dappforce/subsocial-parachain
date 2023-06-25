// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use pallet_utils::rpc::{FlatContent, FlatWhoAndWhen};

use frame_system::Pallet as SystemPallet;

use crate::{Module, Profile, SocialAccount, Config};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatProfile<AccountId, BlockNumber> {
    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,
    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatSocialAccount<AccountId, BlockNumber> {
    pub id: AccountId,
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
    pub reputation: u32,
    pub profile: Option<FlatProfile<AccountId, BlockNumber>>,
}

impl<T: Config> From<Profile<T>> for FlatProfile<T::AccountId, T::BlockNumber> {
    fn from(from: Profile<T>) -> Self {
        let Profile { created, updated, content } = from;

        Self {
            who_and_when: (created, updated).into(),
            content: content.into(),
        }
    }
}

impl<T: Config> From<SocialAccount<T>> for FlatSocialAccount<T::AccountId, T::BlockNumber> {
    fn from(from: SocialAccount<T>) -> Self {
        let SocialAccount {
            followers_count, following_accounts_count, following_spaces_count, reputation, profile
        } = from;

        Self {
            id: T::AccountId::default(),
            followers_count,
            following_accounts_count,
            following_spaces_count,
            reputation,
            profile: profile.map(|profile| profile.into()),
        }
    }
}

impl<T: Config> Module<T> {
    pub fn get_social_accounts_by_ids(
        account_ids: Vec<T::AccountId>
    ) -> Vec<FlatSocialAccount<T::AccountId, T::BlockNumber>> {
        account_ids.iter()
                   .filter_map(|account| {
                       Self::social_account_by_id(account)
                           .map(|social_account| social_account.into())
                           .map(|mut flat_social_account: FlatSocialAccount<T::AccountId, T::BlockNumber>| {
                               flat_social_account.id = account.clone();
                               flat_social_account
                           })
                   })
                   .collect()
    }

    pub fn get_account_data(account: T::AccountId) -> T::AccountData {
        SystemPallet::<T>::account(&account).data
    }
}
