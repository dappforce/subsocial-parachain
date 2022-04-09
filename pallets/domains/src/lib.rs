//! # Module for storing registered domains.
//!
//! Pallet that allows a trusted bridge account to store the user's registered domains.

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

pub use pallet_parachain_utils::{SpaceId, PostId, Content};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use types::*;

    use frame_system::Pallet as System;

    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};

    use frame_system::pallet_prelude::*;

    use sp_runtime::traits::{Saturating, StaticLookup, Zero};
    use sp_std::{cmp::Ordering, convert::TryInto, vec::Vec};

    use pallet_parachain_utils::ensure_content_is_valid;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<<Self as frame_system::Config>::AccountId>;

        /// Domain's minimum length.
        #[pallet::constant]
        type MinDomainLength: Get<u32>;

        /// Domain's maximum length.
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;

        /// Maximum number of domains that can be registered per account.
        #[pallet::constant]
        type MaxDomainsPerAccount: Get<u32>;

        /// The maximum domains amount can be inserted to a storage at once.
        #[pallet::constant]
        type DomainsInsertLimit: Get<u32>;

        /// The maximum amount of time the domain may be held for.
        #[pallet::constant]
        type RegistrationPeriodLimit: Get<<Self as frame_system::pallet::Config>::BlockNumber>;

        /// The length limit for the domains meta outer value.
        #[pallet::constant]
        type MaxOuterValueLength: Get<u32>;

        /// The amount held on deposit for storing the domains structure.
        #[pallet::constant]
        type BaseDomainDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per byte within the domains outer value.
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
    #[pallet::getter(fn is_reserved_word)]
    pub(super) type ReservedWords<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_domain)]
    pub(super) type RegisteredDomains<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, DomainMeta<T>>;

    #[pallet::storage]
    #[pallet::getter(fn domains_by_owner)]
    pub(super) type DomainsByOwner<T: Config> =
        StorageMap<_,
            Twox64Concat,
            T::AccountId,
            BoundedVec<DomainName<T>, T::MaxDomainsPerAccount>,
            ValueQuery,
        >;

    #[pallet::storage]
    pub(super) type DomainByInnerValue<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainInnerLink<T::AccountId>, DomainName<T>>;

    #[pallet::storage]
    pub(super) type SupportedTlds<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The domain name was successfully registered and stored.
        DomainRegistered(<T as frame_system::pallet::Config>::AccountId, DomainName<T>/*, BalanceOf<T>*/),
        /// The domain meta was successfully updated.
        DomainUpdated(<T as frame_system::pallet::Config>::AccountId, DomainName<T>),
        /// The domains list was successfully added to the reserved list.
        DomainsReserved(u16),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The content stored in domain metadata was not changed.
        DomainContentNotChanged,
        /// Cannot insert that many domains to a storage at once.
        DomainsInsertLimitReached,
        /// Cannot register more than `MaxDomainsPerAccount` domains.
        TooManyDomainsPerAccount,
        /// The domain label may contain only A-Z, 0-9 and hyphen characters.
        DomainContainsInvalidChar,
        /// The domain label length must be between 7 and 63 characters, inclusive.
        DomainNameIsTooShort,
        /// The domain has expired.
        DomainHasExpired,
        /// Domain was not found by either custom domain name or top level domain.
        DomainNotFound,
        /// This domain cannot be registered yet, because it is reserved.
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
        /// Cannot store a domain for that long period of time.
        TooBigRegistrationPeriod,
        /// Top level domain must be specified.
        TopLevelDomainNotSpecified,
        /// Top-level domain is not supported.
        TldNotSupported,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registers a domain ([full_domain]) using origin with [content],
        /// and set the domain to expire in [expires_in] number of blocks.
        /// [full_domain] is a full domain name including a dot (.) and TLD.
        /// Example of a [full_domain]: `mytoken.ksm`
        #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: <T as frame_system::pallet::Config>::BlockNumber,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            Self::do_register_domain(owner, full_domain, content, expires_in, ReserveDeposit::Yes)?;

            Ok(())
        }

        /// Registers a domain ([full_domain]) using root on behalf of a [target] with [content],
        /// and set the domain to expire in [expires_in].
        #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        pub fn force_register_domain(
            origin: OriginFor<T>,
            target: <<T as frame_system::pallet::Config>::Lookup as StaticLookup>::Source,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: <T as frame_system::pallet::Config>::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let owner = <T as frame_system::pallet::Config>::Lookup::lookup(target)?;

            Self::do_register_domain(owner, full_domain, content, expires_in, ReserveDeposit::No)?;

            Ok(Pays::No.into())
        }

        /// Sets the domain inner_value to be one of subsocial account, space, or post.
        #[pallet::weight(<T as Config>::WeightInfo::set_inner_value())]
        pub fn set_inner_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: Option<InnerValue<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(domain.clone());
            let mut meta = Self::require_domain(domain_lc.clone())?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.inner_value != value_opt, Error::<T>::InnerValueNotChanged);

            if let Some(inner_value) = &meta.inner_value {
                DomainByInnerValue::<T>::remove(inner_value);
            }

            if let Some(new_value) = &value_opt {
                DomainByInnerValue::<T>::insert(new_value, meta.screen_domain.clone());
            }

            meta.inner_value = value_opt;
            RegisteredDomains::<T>::insert(&domain_lc, meta);

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        /// Sets the domain outer_value to be a custom string.
        #[pallet::weight(<T as Config>::WeightInfo::set_outer_value())]
        pub fn set_outer_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: Option<OuterValue<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(domain.clone());
            let mut meta = Self::require_domain(domain_lc.clone())?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.outer_value != value_opt, Error::<T>::OuterValueNotChanged);

            let mut new_deposit = Zero::zero();
            if let Some(value) = &value_opt {
                new_deposit = T::OuterValueByteDeposit::get() * <BalanceOf<T>>::from(value.len() as u32);
                Self::try_reserve_deposit(&sender, &mut meta.outer_value_deposit, new_deposit)?;
            } else {
                Self::try_unreserve_deposit(&sender, &mut meta.outer_value_deposit)?;
            }

            if meta.outer_value_deposit != new_deposit {
                meta.outer_value_deposit = new_deposit;
            }

            meta.outer_value = value_opt;
            RegisteredDomains::<T>::insert(&domain_lc, meta);

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        /// Sets the domain content to be an outside link.
        #[pallet::weight(<T as Config>::WeightInfo::set_domain_content())]
        pub fn set_domain_content(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            new_content: Content,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(domain.clone());
            let mut meta = Self::require_domain(domain_lc.clone())?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.content != new_content, Error::<T>::DomainContentNotChanged);
            ensure_content_is_valid(new_content.clone())?;

            meta.content = new_content;
            RegisteredDomains::<T>::insert(&domain_lc, meta);

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        /// Mark set of domains as not reservable by users.
        #[pallet::weight(<T as Config>::WeightInfo::reserve_domains(T::DomainsInsertLimit::get()))]
        pub fn reserve_domains(
            origin: OriginFor<T>,
            domains: Vec<DomainName<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let domains_len = domains.len();
            Self::ensure_domains_insert_limit_not_reached(domains_len)?;

            let inserted_domains_count = Self::insert_domains(
                domains,
                Self::ensure_valid_domain,
                |domain| ReservedDomains::<T>::insert(domain, true),
            )?;

            Self::deposit_event(Event::DomainsReserved(domains_len as u16));
            Ok((
                Some(<T as Config>::WeightInfo::reserve_domains(inserted_domains_count)),
                Pays::No,
            ).into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn do_register_domain(
            owner: T::AccountId,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: <T as frame_system::pallet::Config>::BlockNumber,
            reserve_deposit: ReserveDeposit,
        ) -> DispatchResult {
            ensure!(!expires_in.is_zero(), Error::<T>::ZeroReservationPeriod);
            ensure!(
                expires_in <= T::RegistrationPeriodLimit::get(),
                Error::<T>::TooBigRegistrationPeriod,
            );

            // Note that while upper and lower case letters are allowed in domain
            // names, domain names are not case-sensitive. That is, two names with
            // the same spelling but different cases will be treated as identical.
            let domain_lc = Self::lower_domain_then_bound(full_domain.clone());

            ensure!(!Self::is_domain_reserved(&domain_lc), Error::<T>::DomainIsReserved);

            ensure_content_is_valid(content.clone())?;

            Self::ensure_valid_domain(&domain_lc)?;

            ensure!(
                Self::registered_domain(&domain_lc).is_none(),
                Error::<T>::DomainAlreadyOwned,
            );

            let domains_per_account = Self::domains_by_owner(&owner).len();
            ensure!(
                domains_per_account < T::MaxDomainsPerAccount::get() as usize,
                Error::<T>::TooManyDomainsPerAccount,
            );

            let expires_at = expires_in.saturating_add(System::<T>::block_number());

            let deposit = T::BaseDomainDeposit::get();
            let domain_meta = DomainMeta::new(
                full_domain.clone(),
                owner.clone(),
                content,
                expires_at,
                deposit,
            );

            if let ReserveDeposit::Yes = reserve_deposit {
                <T as Config>::Currency::reserve(&owner, deposit)?;
            }

            // TODO: withdraw balance

            RegisteredDomains::<T>::insert(domain_lc.clone(), domain_meta);
            DomainsByOwner::<T>::mutate(
                &owner, |domains| {
                    domains.try_push(domain_lc.clone()).expect("qed; too many domains per account")
                }
            );

            Self::deposit_event(Event::DomainRegistered(owner, full_domain));
            Ok(())
        }

        /// Throws an error if domain contains invalid character.
        fn ensure_domain_contains_valid_chars(domain: &[u8], error: Error<T>) -> DispatchResult {
            let is_char_alphanumeric = |c: &&u8| (**c).is_ascii_alphanumeric();

            let first_char_alphanumeric = domain.first().filter(is_char_alphanumeric).is_some();
            let last_char_alphanumeric = domain.last().filter(is_char_alphanumeric).is_some();

            let mut prev_char_hyphen = false;
            let domain_correct = domain.iter().all(|c| {
                let curr_char_hyphen = *c == b'-';

                // It is not allowed to have two or more sequential hyphen in a domain name.
                // Valid example: a-b-c.ksm
                // Invalid example: a--b.ksm
                if prev_char_hyphen && curr_char_hyphen {
                    return false;
                }

                prev_char_hyphen = curr_char_hyphen;
                c.is_ascii_alphanumeric() || curr_char_hyphen
            });

            ensure!(first_char_alphanumeric && last_char_alphanumeric && domain_correct, error);

            Ok(())
        }

        /// The domain must match the recommended IETF conventions:
        /// https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.1
        ///
        /// The domains must start with a letter, end with a letter or digit,
        /// and have as interior characters only letters, digits, and/or hyphens.
        /// There are also some restrictions on the length:
        /// Domains length must be between 3 and 63 characters.
        pub fn ensure_valid_domain(domain: &[u8]) -> DispatchResult {
            let mut split = domain.split(|c| *c == b'.');
            let dots = split.clone().count().saturating_sub(1);

            ensure!(dots <= 1, Error::<T>::LowerLevelDomainsNotAllowed);
            ensure!(!dots.is_zero(), Error::<T>::TopLevelDomainNotSpecified);

            let domain = split.next().unwrap();
            ensure!(split.next().unwrap() == TOP_LEVEL_DOMAIN, Error::<T>::TopLevelDomainNotAllowed);

            // No need to check max length, because we use BoundedVec as input value.
            ensure!(
                domain.len() >= T::MinDomainLength::get() as usize,
                Error::<T>::DomainNameIsTooShort,
            );

            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::DomainContainsInvalidChar
            )?;

            Ok(())
        }

        pub fn lower_domain_then_bound(domain: DomainName<T>) -> DomainName<T> {
            domain.to_ascii_lowercase().try_into().expect("domain exceeds max length")
        }

        pub fn insert_domains<F, S>(
            domains: Vec<DomainName<T>>,
            check_fn: F,
            insert_storage_fn: S,
        ) -> Result<u32, DispatchError>
        where
            F: Fn(&[u8]) -> DispatchResult,
            S: FnMut(&DomainName<T>),
        {
            for domain in domains.iter() {
                check_fn(domain)?;
            }

            domains.iter().for_each(insert_storage_fn);
            Ok(domains.len() as u32)
        }

        /// Try to get domain meta by it's custom and top level domain names.
        pub fn require_domain(domain: DomainName<T>) -> Result<DomainMeta<T>, DispatchError> {
            Ok(Self::registered_domain(&domain).ok_or(Error::<T>::DomainNotFound)?)
        }

        pub fn ensure_domains_insert_limit_not_reached(
            domains_len: usize,
        ) -> DispatchResultWithPostInfo {
            let domains_insert_limit = T::DomainsInsertLimit::get() as usize;
            ensure!(domains_len <= domains_insert_limit, Error::<T>::DomainsInsertLimitReached);

            Ok(Default::default())
        }

        pub fn ensure_allowed_to_update_domain(
            domain_meta: &DomainMeta<T>,
            sender: &<T as frame_system::pallet::Config>::AccountId,
        ) -> DispatchResult {
            let DomainMeta { owner, expires_at, .. } = domain_meta;

            ensure!(&System::<T>::block_number() < expires_at, Error::<T>::DomainHasExpired);
            ensure!(sender == owner, Error::<T>::NotDomainOwner);
            Ok(())
        }

        pub fn try_reserve_deposit(
            depositor: &<T as frame_system::pallet::Config>::AccountId,
            stored_value: &mut BalanceOf<T>,
            new_deposit: BalanceOf<T>,
        ) -> DispatchResult {
            let old_deposit = stored_value.clone();

            match old_deposit.cmp(&new_deposit) {
                Ordering::Less =>
                    <T as Config>::Currency::reserve(depositor, new_deposit - old_deposit)?,
                Ordering::Greater => {
                    let err_amount = <T as Config>::Currency::unreserve(
                        depositor, new_deposit - old_deposit,
                    );
                    debug_assert!(err_amount.is_zero());
                },
                _ => (),
            }

            *stored_value = new_deposit;
            Ok(())
        }

        pub fn try_unreserve_deposit(
            depositor: &<T as frame_system::pallet::Config>::AccountId,
            stored_value: &mut BalanceOf<T>,
        ) -> DispatchResult {
            let old_deposit = *stored_value;
            *stored_value = Zero::zero();

            <T as Config>::Currency::unreserve(depositor, old_deposit);

            Ok(())
        }
    }
}
