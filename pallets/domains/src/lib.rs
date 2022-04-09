//! # Module for storing registered domains.
//!
//! Pallet that allows a trusted bridge account to store the user's registered domains.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// pub mod weights;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use frame_system::Pallet as System;
use pallet_timestamp::Pallet as Timestamp;

use frame_support::{
    dispatch::DispatchResult, traits::{Currency, ReservableCurrency},
};
use sp_runtime::traits::{Saturating, StaticLookup, Zero};
#[cfg(feature = "std")]
use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryInto, vec::Vec};

use pallet_parachain_utils::{SpaceId, PostId, WhoAndWhen, Content};

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // pub use crate::weights::WeightInfo;

    pub(crate) type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
    pub(crate) type DomainsVec<T> = BoundedVec<DomainName<T>, <T as Config>::MaxDomainsPerAccount>;
    // type InnerValue<T> = Option<DomainInnerLink<<T as frame_system::pallet::Config>::AccountId>>;
    pub(crate) type OuterValue = Option<Vec<u8>>;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

    pub(crate) type WhoAndWhenOf<T> =
        WhoAndWhen<
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::BlockNumber,
            <T as pallet_timestamp::Config>::Moment,
        >;

    pub fn new_who_and_when<T>(
        account: T::AccountId
    ) -> WhoAndWhen<T::AccountId, T::BlockNumber, T::Moment>
    where
        T: frame_system::Config + pallet_timestamp::Config
    {
        WhoAndWhen {
            account,
            block: System::<T>::block_number(),
            time: Timestamp::<T>::now(),
        }
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum DomainInnerLink<AccountId> {
        Account(AccountId),
        Space(SpaceId),
        Post(PostId),
    }

    // A domain metadata.
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct DomainMeta<T: Config> {
        // When the domain was created.
        created: WhoAndWhenOf<T>,
        // When the domain was updated.
        updated: Option<WhoAndWhenOf<T>>,

        // Specific block, when the domain will become unavailable.
        pub(super) expires_at: T::BlockNumber,

        // The domain owner.
        pub(super) owner: T::AccountId,

        // Some additional (custom) domain metadata.
        pub(super) content: Content,

        // The inner domain link (some Subsocial entity).
        // pub(super) inner_value: InnerValue<T>,
        // The outer domain link (any string).
        pub(super) outer_value: OuterValue,

        // The amount was held as a deposit for storing this structure.
        pub(super) domain_deposit: BalanceOf<T>,
        // The amount was held as a deposit for storing outer value.
        pub(super) outer_value_deposit: BalanceOf<T>,
    }

    impl<T: Config> DomainMeta<T> {
        fn new(
            expires_at: T::BlockNumber,
            owner: T::AccountId,
            content: Content,
            domain_deposit: BalanceOf<T>,
        ) -> Self {
            Self {
                created: new_who_and_when::<T>(owner.clone()),
                updated: None,
                expires_at,
                owner,
                content,
                // inner_value: None,
                outer_value: None,
                domain_deposit,
                outer_value_deposit: Zero::zero(),
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<<Self as frame_system::Config>::AccountId>;

        /// The loose coupled provider to get a space.
        // type SpacesProvider: SpacesProvider;

        /// Domains minimum length.
        #[pallet::constant]
        type MinDomainLength: Get<u32>;

        /// Domains maximum length.
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;

        #[pallet::constant]
        type MaxDomainsPerAccount: Get<u32>;

        /// The maximum domains amount can be inserted to a storage at once.
        #[pallet::constant]
        type DomainsInsertLimit: Get<u32>;

        /// The maximum amount of time the domain may be held for.
        #[pallet::constant]
        type ReservationPeriodLimit: Get<<Self as frame_system::pallet::Config>::BlockNumber>;

        /// The length limit for the domains meta outer value.
        #[pallet::constant]
        type OuterValueLimit: Get<u16>;

        /// The amount held on deposit for storing the domains structure.
        #[pallet::constant]
        type DomainDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per byte within the domains outer value.
        #[pallet::constant]
        type OuterValueByteDeposit: Get<BalanceOf<Self>>;

        // /// Weight information for extrinsics in this pallet.
        // type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn reserved_domain)]
    pub(super) type ReservedDomains<T: Config> =
        StorageMap<_, Twox64Concat, DomainName<T>, bool, ValueQuery>;

    // TODO: how to clean this when domain has expired?
    #[pallet::storage]
    #[pallet::getter(fn registered_domain)]
    pub(super) type RegisteredDomains<T: Config> =
        StorageMap<_,
            Blake2_128Concat,
            DomainName<T>, /* Domain */
            DomainMeta<T>,
        >;

    #[pallet::storage]
    pub(super) type RegisteredDomainsByOwner<T: Config> =
        StorageMap<_,
            Blake2_128Concat,
            T::AccountId,
            Vec<DomainName<T>>,
            ValueQuery,
        >;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The domain name was successfully registered and stored.
        DomainRegistered(<T as frame_system::pallet::Config>::AccountId, DomainName<T>, BalanceOf<T>),
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
        /// Lower than Second level domains are not allowed.
        LowerLevelDomainsNotAllowed,
        /// This account is not allowed to update the domain metadata.
        NotDomainOwner,
        /// Outer value exceeds its length limit.
        OuterValueOffLengthLimit,
        /// A new outer value is the same as the old one.
        OuterValueNotChanged,
        /// Reservation period cannot be a zero value.
        InvalidReservationPeriod,
        /// Cannot store a domain for that long period of time.
        TooBigReservationPeriod,
        /// The top level domain may contain only A-Z, 0-9 and hyphen characters.
        TopLevelDomainContainsInvalidChar,
        /// The top level domain length must be between 3 and 63 characters, inclusive.
        TopLevelDomainIsOffLengthLimits,
        /// This top level domain is not supported.
        TopLevelDomainNotSupported,
        /// This inner value is not supported yet.
        InnerValueNotSupported,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        #[pallet::weight(10000)]
        pub fn register_domain(
            origin: OriginFor<T>,
            owner: <<T as frame_system::pallet::Config>::Lookup as StaticLookup>::Source,
            full_domain: DomainName<T>,
            content: Content,
            expires_in: <T as frame_system::pallet::Config>::BlockNumber,
            #[pallet::compact] price: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let owner = <T as frame_system::pallet::Config>::Lookup::lookup(owner)?;

            ensure!(!expires_in.is_zero(), Error::<T>::InvalidReservationPeriod);
            ensure!(
                expires_in <= T::ReservationPeriodLimit::get(),
                Error::<T>::TooBigReservationPeriod,
            );

            // Note that while upper and lower case letters are allowed in domain
            // names, domain names are not case-sensitive. That is, two names with
            // the same spelling but different case are to be treated as if identical.
            let domain_lc = Self::lower_domain_then_bound(full_domain.clone());


            ensure!(!Self::reserved_domain(&domain_lc), Error::<T>::DomainIsReserved);

            // Utils::<T>::is_valid_content(content.clone())?;

            Self::ensure_valid_domain(&domain_lc)?;

            ensure!(
                Self::registered_domain(&domain_lc).is_none(),
                Error::<T>::DomainAlreadyOwned,
            );

            let expires_at = expires_in.saturating_add(System::<T>::block_number());

            let deposit = T::DomainDeposit::get();
            let domain_meta = DomainMeta::new(
                expires_at,
                owner.clone(),
                content,
                deposit,
            );

            <T as Config>::Currency::reserve(&owner, deposit)?;

            RegisteredDomains::<T>::insert(&domain_lc, domain_meta);
            RegisteredDomainsByOwner::<T>::mutate(&owner, |domains| domains.push(domain_lc));

            Self::deposit_event(Event::DomainRegistered(owner, full_domain, price));
            Ok(Pays::No.into())
        }

        // #[pallet::weight(<T as Config>::WeightInfo::set_inner_value())]
        // #[pallet::weight(10000)]
        // pub fn set_inner_value(
        //     origin: OriginFor<T>,
        //     domain: DomainName<T>,
        //     value: InnerValue<T>,
        // ) -> DispatchResult {
        //     let sender = ensure_signed(origin)?;
        //
        //     let domain_lc = domain.to_ascii_lowercase();
        //     let mut meta = Self::require_domain(&domain_lc)?;
        //
        //     Self::ensure_allowed_to_update_domain(&meta, &sender)?;
        //
        //     ensure!(meta.inner_value != value, Error::<T>::InnerValueNotChanged);
        //     Self::ensure_valid_inner_value(&value)?;
        //
        //     meta.inner_value = value;
        //     RegisteredDomains::<T>::insert(&domain_lc, meta);
        //
        //     Self::deposit_event(Event::DomainUpdated(sender, domain));
        //     Ok(())
        // }

        // #[pallet::weight(<T as Config>::WeightInfo::set_outer_value())]
        #[pallet::weight(10000)]
        pub fn set_outer_value(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            value_opt: OuterValue,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(domain.clone());
            let mut meta = Self::require_domain(&domain_lc)?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.outer_value != value_opt, Error::<T>::OuterValueNotChanged);
            Self::ensure_valid_outer_value(&value_opt)?;

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

        // #[pallet::weight(<T as Config>::WeightInfo::set_domain_content())]
        #[pallet::weight(10000)]
        pub fn set_domain_content(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            new_content: Content,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let domain_lc = Self::lower_domain_then_bound(domain.clone());
            let mut meta = Self::require_domain(&domain_lc)?;

            Self::ensure_allowed_to_update_domain(&meta, &sender)?;

            ensure!(meta.content != new_content, Error::<T>::DomainContentNotChanged);
            // Utils::<T>::is_valid_content(new_content.clone())?;

            meta.content = new_content;
            RegisteredDomains::<T>::insert(&domain_lc, meta);

            Self::deposit_event(Event::DomainUpdated(sender, domain));
            Ok(())
        }

        // #[pallet::weight(<T as Config>::WeightInfo::reserve_domains(T::DomainsInsertLimit::get()))]
        #[pallet::weight(10000)]
        pub fn reserve_domains(
            origin: OriginFor<T>,
            domains: DomainsVec<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let domains_len = domains.len();
            Self::ensure_domains_insert_limit_not_reached(domains_len)?;

            let _inserted_domains_count = Self::insert_domains(
                domains,
                Self::ensure_valid_domain,
                |domain| ReservedDomains::<T>::insert(domain, true),
            )?;

            Self::deposit_event(Event::DomainsReserved(domains_len as u16));
            Ok((
                // Some(<T as Config>::WeightInfo::reserve_domains(inserted_domains_count)),
                Some(10000),
                Pays::No,
            ).into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Checks the length of the provided u8 array.
        fn ensure_domain_has_valid_length(
            string: &[u8],
            min: u32,
            max: u32,
            error: Error<T>,
        ) -> DispatchResult {
            let length = string.len();
            ensure!(length >= min as usize && length <= max as usize, error);

            Ok(())
        }

        /// Throws an error if domain contains invalid character.
        fn ensure_domain_contains_valid_chars(domain: &[u8], error: Error<T>) -> DispatchResult {
            let first_char_alpha = domain.first()
                .filter(|c| (**c).is_ascii_alphabetic());

            let last_char_not_hyphen = domain.last().filter(|c| **c != b'-');

            ensure!(
                first_char_alpha.is_some() && last_char_not_hyphen.is_some() &&
                domain.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-'),
                error
            );

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
            Self::ensure_domain_has_valid_length(
                domain,
                T::MinDomainLength::get(),
                T::MaxDomainLength::get(),
                Error::<T>::TopLevelDomainIsOffLengthLimits,
            )?;

            ensure!(
                domain.iter().all(|c| *c != b'.'),
                Error::<T>::LowerLevelDomainsNotAllowed,
            );

            Self::ensure_domain_contains_valid_chars(
                domain, Error::<T>::TopLevelDomainContainsInvalidChar
            )?;

            Ok(())
        }

        pub fn lower_domain_then_bound(domain: DomainName<T>) -> DomainName<T> {
            domain.to_ascii_lowercase().try_into().expect("domain exceeds max length")
        }

        // pub fn ensure_valid_inner_value(inner_value: &InnerValue<T>) -> DispatchResult {
        //     if inner_value.is_none() { return Ok(()) }
        //
        //     match inner_value.clone().unwrap() {
        //         // DomainInnerLink::Space(space_id) => T::SpacesProvider::ensure_space_exists(space_id),
        //         DomainInnerLink::Space(_) => Ok(()),
        //         DomainInnerLink::Account(_) => Ok(()),
        //         // TODO: support all inner values
        //         _ => Err(Error::<T>::InnerValueNotSupported.into()),
        //     }
        // }

        pub fn ensure_valid_outer_value(outer_value: &OuterValue) -> DispatchResult {
            if let Some(outer) = &outer_value {
                ensure!(
                    outer.len() <= T::OuterValueLimit::get().into(),
                    Error::<T>::OuterValueOffLengthLimit
                );
            }
            Ok(())
        }

        pub fn insert_domains<F, S>(
            domains: DomainsVec<T>,
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
        pub fn require_domain(domain: &DomainName<T>) -> Result<DomainMeta<T>, DispatchError> {
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

            ensure!(expires_at > &System::<T>::block_number(), Error::<T>::DomainHasExpired);
            ensure!(sender == owner, Error::<T>::NotDomainOwner);
            Ok(())
        }

        pub fn try_reserve_deposit(
            depositor: &<T as frame_system::pallet::Config>::AccountId,
            stored_value: &mut BalanceOf<T>,
            new_deposit: BalanceOf<T>,
        ) -> DispatchResult {
            let old_deposit = &mut stored_value.clone();
            *stored_value = new_deposit;

            use sp_std::cmp::Ordering;

            match stored_value.cmp(&old_deposit) {
                Ordering::Greater => <T as Config>::Currency::reserve(depositor, *stored_value - *old_deposit)?,
                Ordering::Less => {
                    let err_amount = <T as Config>::Currency::unreserve(
                        depositor, *old_deposit - *stored_value,
                    );
                    debug_assert!(err_amount.is_zero());
                },
                _ => (),
            }
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
