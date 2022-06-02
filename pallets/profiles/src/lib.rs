#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::{decl_error, decl_module, decl_storage};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

// pub mod rpc;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SocialAccount {
    pub followers_count: u32,
    pub following_accounts_count: u16,
    pub following_spaces_count: u16,
}

/// The pallet's configuration trait.
pub trait Config: frame_system::Config {}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as Profiles {
        pub SocialAccountById get(fn social_account_by_id):
            map hasher(blake2_128_concat) T::AccountId => Option<SocialAccount>;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Social account was not found by id.
        SocialAccountNotFound,
    }
}

decl_module! {
  pub struct Module<T: Config> for enum Call where origin: T::Origin {
    // Initializing errors
    type Error = Error<T>;
  }
}

impl SocialAccount {
    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn inc_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_add(1);
    }

    pub fn dec_following_accounts(&mut self) {
        self.following_accounts_count = self.following_accounts_count.saturating_sub(1);
    }

    pub fn inc_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_add(1);
    }

    pub fn dec_following_spaces(&mut self) {
        self.following_spaces_count = self.following_spaces_count.saturating_sub(1);
    }
}

impl<T: Config> Module<T> {
    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount {
        Self::social_account_by_id(account).unwrap_or(
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
            }
        )
    }
}
