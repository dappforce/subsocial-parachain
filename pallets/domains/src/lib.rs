// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

//! # Module for registering decentralized domain names

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use crate::weights::WeightInfo;

pub mod types;

pub use subsocial_support::{SpaceId, PostId, Content};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use types::*;

    use frame_system::Pallet as System;

    use frame_support::{pallet_prelude::*, traits::ReservableCurrency, dispatch::DispatchClass};

    use frame_system::pallet_prelude::*;

    use sp_runtime::traits::{Saturating, StaticLookup, Zero};
    use sp_std::{cmp::Ordering, convert::TryInto, vec::Vec};

    use subsocial_support::ensure_content_is_valid;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Domain's minimum length.
        #[pallet::constant]
        type MinDomainLength: Get<u32>;

        /// Domain's maximum length.
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;

        /// Maximum number of domains that can be registered per account.
        #[pallet::constant]
        type MaxDomainsPerAccount: Get<u32>;

        /// Maximum number of promotional domains that can be registered per account.
        #[pallet::constant]
        type MaxPromoDomainsPerAccount: Get<u32>;

        /// The maximum number of domains that can be inserted into a storage at once.
        #[pallet::constant]
        type DomainsInsertLimit: Get<u32>;

        /// The maximum period of time the domain may be held for.
        #[pallet::constant]
        type RegistrationPeriodLimit: Get<Self::BlockNumber>;

        /// The maximum length of the domain's outer value.
        #[pallet::constant]
        type MaxOuterValueLength: Get<u32>;

        /// The amount held on deposit for storing the domain's structure.
        #[pallet::constant]
        type BaseDomainDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per byte of the domain's outer value.
        #[pallet::constant]
        type OuterValueByteDeposit: Get<BalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn is_word_reserved)]
    pub(super) type ReservedWords<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    /// Metadata associated per domain.
    #[pallet::storage]
    #[pallet::getter(fn registered_domain)]
    pub(super) type RegisteredDomains<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, DomainMeta<T>>;

    /// Domains owned per account.
    ///
    /// TWOX-NOTE: Safe as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    #[pallet::getter(fn domains_by_owner)]
    pub(super) type DomainsByOwner<T: Config> =
        StorageMap<_,
            Twox64Concat,
            T::AccountId,
            BoundedVec<DomainName<T>, T::MaxDomainsPerAccount>,
            ValueQuery,
        >;

    /// TWOX-NOTE: Safe as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub(super) type DomainByInnerValue<T: Config> =
        StorageDoubleMap<_,
            Twox64Concat,
            T::AccountId,
            Blake2_128Concat,
            InnerValue<T::AccountId>,
            DomainName<T>,
        >;

    #[pallet::storage]
    #[pallet::getter(fn is_tld_supported)]
    pub(super) type SupportedTlds<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The domain name was successfully registered.
        DomainRegistered { who: T::AccountId, domain: DomainName<T> },
        /// The domain meta was successfully updated.
        DomainMetaUpdated { who: T::AccountId, domain: DomainName<T> },
        /// New words have been reserved.
        NewWordsReserved { count: u32 },
        /// Added support for new TLDs (top-level domains).
        NewTldsSupported { count: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The content stored in a domain metadata was not changed.
        DomainContentNotChanged,
        /// Cannot register more than `MaxDomainsPerAccount` domains.
        TooManyDomainsPerAccount,
        /// This domain label may contain only a-z, 0-9 and hyphen characters.
        DomainContainsInvalidChar,
        /// This domain label length must be between `MinDomainLength` and 63 characters, inclusive.
        DomainIsTooShort,
        /// This domain has expired.
        DomainHasExpired,
        /// Domain was not found by the domain name.
        DomainNotFound,
        /// This domain cannot be registered yet, because this word is reserved.
        DomainIsReserved,
        /// This domain is already held by another account.
        DomainAlreadyOwned,
        /// A new inner value is the same as the old one.
        InnerValueNotChanged,
        /// Lower than the second-level domains are not allowed.
        SubdomainsNotAllowed,
        /// This account is not allowed to update the domain metadata.
        NotDomainOwner,
        /// A new outer value is the same as the old one.
        OuterValueNotChanged,
        /// Reservation period cannot be a zero value.
        ZeroReservationPeriod,
        /// Cannot store a domain for such a long period of time.
        TooBigRegistrationPeriod,
        /// Top-level domain must be specified.
        TldNotSpecified,
        /// Top-level domain is not supported.
        TldNotSupported,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registers a domain ([full_domain]) using origin with [content],
        /// and set the domain to expire in [expires_in] number of blocks.
        /// [full_domain] is a full domain name including a dot (.) and TLD.
        /// Example of a [full_domain]: `mytoken.ksm`
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: T::BlockNumber,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            Self::do_register_domain(owner, full_domain, content, expires_in, IsForced::No)
        }

        /// Registers a domain ([full_domain]) using root on behalf of a [target] with [content],
        /// and set the domain to expire in [expires_in] number of blocks.
        #[pallet::call_index(1)]
        #[pallet::weight((
            <T as Config>::WeightInfo::force_register_domain(),
            DispatchClass::Operational,
        ))]
        pub fn force_register_domain(
            origin: OriginFor<T>,
            target: <T::Lookup as StaticLookup>::Source,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: T::BlockNumber,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let owner = T::Lookup::lookup(target)?;

            Self::do_register_domain(owner, full_domain, content, expires_in, IsForced::Yes)
        }

        /// Sets the domain inner_value to be one of Subsocial account, space, or post.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_inner_value())]
        pub fn set_inner_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: Option<InnerValueOf<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::do_set_inner_value(domain, value_opt, Some(sender))?;

            Ok(())
        }

        /// Sets the domain inner_value to be one of subsocial account, space, or post.
        #[pallet::call_index(3)]
        #[pallet::weight((
            <T as Config>::WeightInfo::force_set_inner_value(),
            DispatchClass::Operational,
        ))]
        pub fn force_set_inner_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: Option<InnerValueOf<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::do_set_inner_value(domain, value_opt, None)?;

            Ok(())
        }

        /// Sets the domain outer_value to be a custom string.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_outer_value())]
        pub fn set_outer_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: Option<OuterValue<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(&domain);
            let meta = Self::require_domain(domain_lc.clone())?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.outer_value != value_opt, Error::<T>::OuterValueNotChanged);

            let mut new_deposit = Zero::zero();
            if let Some(value) = &value_opt {
                new_deposit = T::OuterValueByteDeposit::get()
                    .saturating_mul(<BalanceOf<T>>::from(value.len() as u32));

                Self::try_reserve_deposit(&sender, meta.outer_value_deposit, new_deposit)?;
            } else {
                <T as Config>::Currency::unreserve(&sender, meta.outer_value_deposit);
            }

            RegisteredDomains::<T>::mutate(&domain_lc, |meta_opt| {
                if let Some(stored_meta) = meta_opt {
                    if stored_meta.outer_value_deposit != new_deposit {
                        stored_meta.outer_value_deposit = new_deposit;
                    }

                    stored_meta.outer_value = value_opt;
                }
            });

            Self::deposit_event(Event::DomainMetaUpdated { who: sender, domain });
            Ok(())
        }

        /// Sets the domain content to be an outside link.
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::set_domain_content())]
        pub fn set_domain_content(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            new_content: Content,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(&domain);
            let mut meta = Self::require_domain(domain_lc.clone())?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.content != new_content, Error::<T>::DomainContentNotChanged);
            ensure_content_is_valid(new_content.clone())?;

            meta.content = new_content;
            RegisteredDomains::<T>::insert(&domain_lc, meta);

            Self::deposit_event(Event::DomainMetaUpdated { who: sender, domain });
            Ok(())
        }

        /// Mark set of domains as not reservable by users.
        #[pallet::call_index(6)]
        #[pallet::weight((
            <T as Config>::WeightInfo::reserve_words(T::DomainsInsertLimit::get()),
            DispatchClass::Operational,
        ))]
        pub fn reserve_words(
            origin: OriginFor<T>,
            words: BoundedDomainsVec<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let inserted_words_count = Self::insert_domains(
                &words,
                Self::ensure_domain_contains_valid_chars,
                |domain| ReservedWords::<T>::insert(Self::lower_domain_then_bound(domain), true),
            )?;

            Self::deposit_event(Event::NewWordsReserved { count: inserted_words_count });
            Ok(Some(<T as Config>::WeightInfo::reserve_words(inserted_words_count)).into())
        }

        /// Add support for a set of top-level domains.
        #[pallet::call_index(7)]
        #[pallet::weight((
            <T as Config>::WeightInfo::support_tlds(T::DomainsInsertLimit::get()),
            DispatchClass::Operational,
        ))]
        pub fn support_tlds(
            origin: OriginFor<T>,
            tlds: BoundedDomainsVec<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let inserted_tlds_count = Self::insert_domains(
                &tlds,
                Self::ensure_ascii_alphanumeric,
                |domain| SupportedTlds::<T>::insert(Self::lower_domain_then_bound(domain), true),
            )?;

            Self::deposit_event(Event::NewTldsSupported { count: inserted_tlds_count });
            Ok(Some(<T as Config>::WeightInfo::support_tlds(inserted_tlds_count)).into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn do_register_domain(
            owner: T::AccountId,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: T::BlockNumber,
            is_forced: IsForced,
        ) -> DispatchResult {
            ensure!(!expires_in.is_zero(), Error::<T>::ZeroReservationPeriod);
            ensure!(
                expires_in <= T::RegistrationPeriodLimit::get(),
                Error::<T>::TooBigRegistrationPeriod,
            );
            ensure_content_is_valid(content.clone())?;

            // Note that while upper and lower case letters are allowed in domain
            // names, domain names are not case-sensitive. That is, two names with
            // the same spelling but different cases will be treated as identical.
            let domain_lc = Self::lower_domain_then_bound(&full_domain);
            let domain_parts = Self::split_domain_by_dot(&domain_lc);

            Self::ensure_valid_domain(&domain_parts)?;

            let subdomain = domain_parts.first().unwrap();
            let tld = domain_parts.last().unwrap();

            // FIXME: this is hot fix, change asap
            // ensure!(Self::is_tld_supported(tld), Error::<T>::TldNotSupported);
            ensure!(tld.as_slice() == b"sub", Error::<T>::TldNotSupported);

            let domains_per_account = Self::domains_by_owner(&owner).len();

            if let IsForced::No = is_forced {
                ensure!(
                    domains_per_account < T::MaxPromoDomainsPerAccount::get() as usize,
                    Error::<T>::TooManyDomainsPerAccount,
                );
                Self::ensure_word_is_not_reserved(subdomain)?;
            }

            ensure!(
                Self::registered_domain(&domain_lc).is_none(),
                Error::<T>::DomainAlreadyOwned,
            );

            ensure!(
                domains_per_account < T::MaxDomainsPerAccount::get() as usize,
                Error::<T>::TooManyDomainsPerAccount,
            );

            let mut deposit = Zero::zero();
            if let IsForced::No = is_forced {
                // TODO: unreserve the balance for expired or sold domains
                deposit = T::BaseDomainDeposit::get();
                <T as Config>::Currency::reserve(&owner, deposit)?;
            }

            let expires_at = expires_in.saturating_add(System::<T>::block_number());
            let domain_meta = DomainMeta::new(
                expires_at,
                owner.clone(),
                content,
                deposit,
            );

            // TODO: withdraw balance when it will be possible to purchase domains.

            RegisteredDomains::<T>::insert(domain_lc.clone(), domain_meta);
            DomainsByOwner::<T>::mutate(
                &owner, |domains| {
                    domains.try_push(domain_lc.clone()).expect("qed; too many domains per account")
                }
            );

            Self::deposit_event(Event::DomainRegistered { who: owner, domain: full_domain });
            Ok(())
        }

        fn do_set_inner_value(
            full_domain: DomainName<T>,
            new_value_opt: Option<InnerValueOf<T>>,
            check_ownership: Option<T::AccountId>,
        ) -> DispatchResult {
            let domain_lc = Self::lower_domain_then_bound(&full_domain);
            let meta = Self::require_domain(domain_lc.clone())?;

            if let Some(should_be_owner) = check_ownership {
                Self::ensure_allowed_to_update_domain(&meta, &should_be_owner)?;
            }

            ensure!(meta.inner_value != new_value_opt, Error::<T>::InnerValueNotChanged);

            if let Some(old_value) = &meta.inner_value {
                DomainByInnerValue::<T>::remove(&meta.owner, old_value);
            }

            if let Some(new_value) = &new_value_opt {
                DomainByInnerValue::<T>::insert(&meta.owner, new_value, domain_lc.clone());
            }

            RegisteredDomains::<T>::mutate(&domain_lc, |meta_opt| {
                if let Some(stored_meta) = meta_opt {
                    stored_meta.inner_value = new_value_opt;
                }
            });

            Self::deposit_event(Event::DomainMetaUpdated { who: meta.owner, domain: domain_lc });
            Ok(())
        }

        fn ensure_ascii_alphanumeric(domain: &[u8]) -> DispatchResult {
            ensure!(
                !domain.iter().any(|c| !(*c).is_ascii_alphanumeric()),
                Error::<T>::DomainContainsInvalidChar
            );
            Ok(())
        }

        /// Throws an error if domain contains invalid character.
        fn ensure_domain_contains_valid_chars(domain: &[u8]) -> DispatchResult {
            let is_char_alphanumeric = |c: &&u8| (**c).is_ascii_alphanumeric();

            let first_char_alphanumeric = domain.first().filter(is_char_alphanumeric).is_some();
            let last_char_alphanumeric = domain.last().filter(is_char_alphanumeric).is_some();

            ensure!(first_char_alphanumeric && last_char_alphanumeric, Error::<T>::DomainContainsInvalidChar);

            let mut prev_char_hyphen = false;
            let domain_correct = domain.iter().all(|c| {
                let curr_char_hyphen = *c == b'-';

                // It is not allowed to have two or more sequential hyphens in a domain name.
                // Valid example: a-b-c.ksm
                // Invalid example: a--b.ksm
                if prev_char_hyphen && curr_char_hyphen {
                    return false;
                }

                prev_char_hyphen = curr_char_hyphen;
                c.is_ascii_alphanumeric() || curr_char_hyphen
            });

            ensure!(domain_correct, Error::<T>::DomainContainsInvalidChar);

            Ok(())
        }

        /// The domain must match the recommended IETF conventions:
        /// https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.1
        ///
        /// The domains must start with a letter, end with a letter or digit,
        /// and have as interior characters only letters, digits, and/or hyphens.
        /// There are also some restrictions on the length:
        /// Domains length must be between 3 and 63 characters.
        pub fn ensure_valid_domain(domain: &[DomainName<T>]) -> DispatchResult {
            let dots = domain.len().saturating_sub(1);

            ensure!(dots <= 1, Error::<T>::SubdomainsNotAllowed);
            ensure!(!dots.is_zero(), Error::<T>::TldNotSpecified);

            let domain = domain.first().unwrap();

            // No need to check max length, because we use BoundedVec as input value.
            ensure!(
                domain.len() >= T::MinDomainLength::get() as usize,
                Error::<T>::DomainIsTooShort,
            );

            Self::ensure_domain_contains_valid_chars(domain)?;

            Ok(())
        }

        pub(crate) fn bound_domain(domain: Vec<u8>) -> DomainName<T> {
            domain.try_into().expect("qed; domain exceeds max length")
        }

        pub fn lower_domain_then_bound(domain: &[u8]) -> DomainName<T> {
            Self::bound_domain(domain.to_ascii_lowercase())
        }

        /// A generic function to insert a list of reserved words or supported TLDs.
        pub fn insert_domains<F, S>(
            domains: &[DomainName<T>],
            validate_fn: F,
            insert_storage_fn: S,
        ) -> Result<u32, DispatchError>
        where
            F: Fn(&[u8]) -> DispatchResult,
            S: FnMut(&DomainName<T>),
        {
            for domain in domains.iter() {
                validate_fn(domain)?;
            }

            domains.iter().for_each(insert_storage_fn);
            Ok(domains.len() as u32)
        }

        /// Try to get domain meta by it's custom and top-level domain names.
        pub fn require_domain(domain: DomainName<T>) -> Result<DomainMeta<T>, DispatchError> {
            Ok(Self::registered_domain(&domain).ok_or(Error::<T>::DomainNotFound)?)
        }

        /// Check that the domain is not expired and [sender] is the current owner.
        pub fn ensure_allowed_to_update_domain(
            domain_meta: &DomainMeta<T>,
            sender: &T::AccountId,
        ) -> DispatchResult {
            let DomainMeta { owner, expires_at, .. } = domain_meta;

            ensure!(&System::<T>::block_number() < expires_at, Error::<T>::DomainHasExpired);
            ensure!(sender == owner, Error::<T>::NotDomainOwner);
            Ok(())
        }

        pub fn try_reserve_deposit(
            depositor: &T::AccountId,
            old_deposit: BalanceOf<T>,
            new_deposit: BalanceOf<T>,
        ) -> DispatchResult {
            match old_deposit.cmp(&new_deposit) {
                Ordering::Less =>
                    <T as Config>::Currency::reserve(depositor, new_deposit - old_deposit)?,
                Ordering::Greater => {
                    let err_amount = <T as Config>::Currency::unreserve(
                        depositor, old_deposit - new_deposit,
                    );
                    debug_assert!(err_amount.is_zero());
                },
                _ => (),
            }

            Ok(())
        }

        pub(crate) fn split_domain_by_dot(full_domain: &DomainName<T>) -> Vec<DomainName<T>> {
            full_domain.split(|c| *c == b'.').map(Self::lower_domain_then_bound).collect()
        }

        fn ensure_word_is_not_reserved(word: &DomainName<T>) -> DispatchResult {
            let word_without_hyphens = Self::bound_domain(
                word.iter().filter(|c| **c != b'-').cloned().collect()
            );

            ensure!(!Self::is_word_reserved(word_without_hyphens), Error::<T>::DomainIsReserved);
            Ok(())
        }
    }
}
