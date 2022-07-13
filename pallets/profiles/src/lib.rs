#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct SocialAccount {
        pub followers_count: u32,
        pub following_accounts_count: u16,
        pub following_spaces_count: u16,
    }

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn social_account_by_id)]
    pub type SocialAccountById<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SocialAccount>;

    #[pallet::error]
    pub enum Error<T> {
        /// Social account was not found by id.
        SocialAccountNotFound,
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

    impl<T: Config> Pallet<T> {
        pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount {
            Self::social_account_by_id(account).unwrap_or(SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
            })
        }
    }
}
