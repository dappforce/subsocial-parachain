//! # Module for registering decentralized domain names

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub use pallet::*;
pub use subsocial_support::{PostId, SpaceId};

pub use crate::weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub mod types;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{ReservableCurrency, Currency, ExistenceRequirement::KeepAlive, tokens::WithdrawReasons}, dispatch::DispatchClass};    use frame_system::{pallet_prelude::*, Pallet as System};
    use sp_runtime::traits::{Saturating, StaticLookup, Zero};
    use sp_std::{convert::TryInto, vec::Vec};

    use types::*;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Domain's minimum length.
        #[pallet::constant]
        type MinDomainLength: Get<DomainLength>;

        /// Domain's maximum length.
        #[pallet::constant]
        type MaxDomainLength: Get<DomainLength>;

        /// Maximum number of domains that can be registered per account.
        #[pallet::constant]
        type MaxDomainsPerAccount: Get<u32>;

        /// The maximum number of domains that can be inserted into a storage at once.
        #[pallet::constant]
        type DomainsInsertLimit: Get<u32>;

        /// The period of time the domain may be held for.
        #[pallet::constant]
        type RegistrationPeriod: Get<Self::BlockNumber>;

        /// The maximum length of the domain's outer value.
        #[pallet::constant]
        type MaxOuterValueLength: Get<u32>;

        /// The maximum length of the domain's record key.
        #[pallet::constant]
        type MaxRecordKeyLength: Get<u32>;

        /// The maximum length of the domain's record value.
        #[pallet::constant]
        type MaxRecordValueLength: Get<u32>;

        /// The amount held on deposit for storing the domain's structure.
        #[pallet::constant]
        type BaseDomainDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per byte of the domain's record key and its value.
        #[pallet::constant]
        type RecordByteDeposit: Get<BalanceOf<Self>>;

        /// The governance origin to control this pallet.
        type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Account that receives funds spent for domain purchase.
        /// Used only once, when the pallet is initialized.
        type InitialPaymentBeneficiary: Get<Self::AccountId>;

        /// A set of prices according to a domain length.
        /// Used only once, when the pallet is initialized.
        #[pallet::constant]
        type InitialPrices: Get<Vec<(DomainLength, BalanceOf<Self>)>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Reserved words that can not be used in the domain name, making domains with these words unavailable
    /// for registeration.
    #[pallet::storage]
    #[pallet::getter(fn is_word_reserved)]
    pub(super) type ReservedWords<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    /// Records associated per domain.
    #[pallet::storage]
    #[pallet::getter(fn domain_record)]
    pub(super) type DomainRecords<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        DomainName<T>,
        Blake2_128Concat,
        DomainRecordKey<T>,
        RecordValueWithDepositInfo<T>,
    >;

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
    pub(super) type DomainsByOwner<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<DomainName<T>, T::MaxDomainsPerAccount>,
        ValueQuery,
    >;

    /// TWOX-NOTE: Safe as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    #[deprecated] // remove after migration to new style of record key/value
    pub(super) type DomainByInnerValue<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Blake2_128Concat,
        InnerValue<T::AccountId>,
        DomainName<T>,
    >;

    /// Supported TLDs available for registration.
    #[pallet::storage]
    #[pallet::getter(fn is_tld_supported)]
    pub(super) type SupportedTlds<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, bool, ValueQuery>;

    /// Price configuration domains, depending on the length of the domain the price would change.
    ///
    /// **NOTE:** the stored array must be sorted in ascending order based on the length of the domain
    /// (first element of the tuple).
    ///
    /// ## Example:
    /// assume we have the following setup for price config.
    /// ```rust
    /// let price_config = [
    ///     (0, 200),
    ///     (3, 170),
    ///     (5, 100),
    ///     (9, 30),
    /// ];
    ///```
    /// would make a relationship between price and domain length described by the following graph.
    /// ```
    /// //    ▲
    /// //    │ price
    /// // 200├────────┐
    /// //    │        │
    /// // 180│        │
    /// //    │        └─────┐
    /// // 160│        ^     │
    /// //    │        ^     │
    /// // 140│        ^     │
    /// //    │        ^     │
    /// // 120│        ^     │
    /// //    │        ^     │
    /// // 100│        ^     └───────────┐
    /// //    │        ^     ^           │
    /// //  80│        ^     ^           │
    /// //    │        ^     ^           │
    /// //  60│        ^     ^           │
    /// //    │        ^     ^           │
    /// //  40│        ^     ^           │
    /// //    │        ^     ^           └───────────────────
    /// //  20│        ^     ^           ^            domain
    /// //    │        ^     ^           ^            length
    /// //   0└─────────────────────────────────────────────►
    /// //    0  1  2  3  4  5  6  7  8  9  10 11 12 13 14
    ///
    /// ```
    ///
    ///
    #[pallet::storage]
    #[pallet::getter(fn prices_config)]
    pub(super) type PricesConfig<T: Config> =
        StorageValue<_, PricesConfigVec<T>, ValueQuery, T::InitialPrices>;

    /// The default value for [PaymentBeneficiary]
    #[pallet::type_value]
    pub(super) fn DefaultPaymentBeneficiary<T: Config>() -> T::AccountId {
        T::InitialPaymentBeneficiary::get()
    }

    /// Account that receives payment for the domain registration.
    #[pallet::storage]
    #[pallet::getter(fn payment_beneficiary)]
    pub(super) type PaymentBeneficiary<T: Config> =
        StorageValue<_, T::AccountId, ValueQuery, DefaultPaymentBeneficiary<T>>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_prices: Vec<(DomainLength, BalanceOf<T>)>,
        pub initial_payment_beneficiary: T::AccountId,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_prices: T::InitialPrices::get(),
                initial_payment_beneficiary: T::InitialPaymentBeneficiary::get(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::init_pallet(&self.initial_prices);
            PaymentBeneficiary::<T>::set(self.initial_payment_beneficiary.clone());
        }
    }

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
        /// The domain record has been updated
        DomainRecordUpdated {
            account: T::AccountId,
            domain: DomainName<T>,
            key: DomainRecordKey<T>,
            value: Option<DomainRecordValue<T>>,
            deposit: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Cannot register more than `MaxDomainsPerAccount` domains.
        TooManyDomainsPerAccount,
        /// This domain label may contain only a-z, 0-9 and hyphen characters.
        DomainContainsInvalidChar,
        /// This domain label length must be withing the limits defined with
        /// [`Config::MinDomainLength`] and [`Config::MaxDomainLength`] characters, inclusive.
        DomainIsTooShort,
        /// This domain has expired.
        DomainHasExpired,
        /// Domain was not found by the domain name.
        DomainNotFound,
        /// This domain cannot be registered yet, because this word is reserved.
        DomainIsReserved,
        /// This domain is already held by another account.
        DomainAlreadyOwned,
        /// Lower than the second-level domains are not allowed.
        SubdomainsNotAllowed,
        /// This account is not allowed to update the domain metadata.
        NotDomainOwner,
        /// The reservation period cannot be a zero value.
        ZeroReservationPeriod,
        /// Cannot store a domain for such a long period of time.
        TooBigRegistrationPeriod,
        /// Top-level domain must be specified.
        TldNotSpecified,
        /// Top-level domain is not supported.
        TldNotSupported,
        /// The domain price cannot be calculated.
        CannotCalculatePrice,
        /// There are not enough funds to reserve domain deposit.
        InsufficientBalanceToReserveDeposit,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Registers a domain ([full_domain]) using origin.
        /// [full_domain] is a full domain name including a dot (.) and TLD.
        /// Example of a [full_domain]: `mytoken.ksm`
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            owner_target: AccountIdLookupOf<T>,
            full_domain: DomainName<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let owner = T::Lookup::lookup(owner_target)?;
            let domain_data = DomainRegisterData::new(owner, full_domain);

            Self::do_register_domain(domain_data, DomainPayer::<T>::Account(caller))
        }

        /// Registers a domain ([full_domain]) using root on behalf of a [recipient].
        #[pallet::call_index(1)]
        #[pallet::weight((
            <T as Config>::WeightInfo::force_register_domain(),
            DispatchClass::Operational,
        ))]
        pub fn force_register_domain(
            origin: OriginFor<T>,
            recipient: <T::Lookup as StaticLookup>::Source,
            full_domain: DomainName<T>,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;

            let recipient = T::Lookup::lookup(recipient)?;
            let domain_data = DomainRegisterData::new(recipient, full_domain);

            Self::do_register_domain(domain_data, DomainPayer::<T>::ForceOrigin)
        }

        /// Change the record associated with a domain name.
        ///
        /// **Record Deposit:**
        ///
        /// Deposit value is calculated based on how many bytes are in the key+value.
        /// If the new value is bigger than the old value, the reminder will be reserved. And if the
        /// new value is smaller than the old value, part of the deposit will be refunded
        /// back to the domain owner. While if [value_opt] is None, the record will be
        /// deleted and the whole deposit will be refunded.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_record())]
        pub fn set_record(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            key: DomainRecordKey<T>,
            value_opt: Option<DomainRecordValue<T>>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::do_set_record(domain, key, value_opt, Some(sender))?;

            Ok(())
        }

        /// Calls [Pallet::set_record] with the [Config::ForceOrigin].
        #[pallet::call_index(3)]
        #[pallet::weight((
            <T as Config>::WeightInfo::force_set_record(),
            DispatchClass::Operational,
        ))]
        pub fn force_set_record(
            origin: OriginFor<T>,
            domain: DomainName<T>,
            key: DomainRecordKey<T>,
            value_opt: Option<DomainRecordValue<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_set_record(domain, key, value_opt, None)?;

            Ok(Pays::No.into())
        }

        /// Mark a set of domains as not reservable by users.
        #[pallet::call_index(4)]
        #[pallet::weight((
            <T as Config>::WeightInfo::reserve_words(T::DomainsInsertLimit::get()),
            DispatchClass::Operational,
        ))]
        pub fn reserve_words(
            origin: OriginFor<T>,
            words: BoundedDomainsVec<T>,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            let inserted_words_count =
                Self::insert_domains(&words, Self::ensure_domain_contains_valid_chars, |domain| {
                    ReservedWords::<T>::insert(Self::lower_domain_then_bound(domain), true)
                })?;

            Self::deposit_event(Event::NewWordsReserved { count: inserted_words_count });
            Ok(Some(<T as Config>::WeightInfo::reserve_words(inserted_words_count)).into())
        }

        /// Add support for a set of top-level domains.
        #[pallet::call_index(5)]
        #[pallet::weight((
            <T as Config>::WeightInfo::support_tlds(T::DomainsInsertLimit::get()),
            DispatchClass::Operational,
        ))]
        pub fn support_tlds(
            origin: OriginFor<T>,
            tlds: BoundedDomainsVec<T>,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            let inserted_tlds_count =
                Self::insert_domains(&tlds, Self::ensure_ascii_alphanumeric, |domain| {
                    SupportedTlds::<T>::insert(Self::lower_domain_then_bound(domain), true)
                })?;

            Self::deposit_event(Event::NewTldsSupported { count: inserted_tlds_count });
            Ok(Some(<T as Config>::WeightInfo::support_tlds(inserted_tlds_count)).into())
        }

        /// Change [PaymentBeneficiary], only callable from [Config::ForceOrigin].
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)]
        pub fn set_payment_beneficiary(
            origin: OriginFor<T>,
            payment_beneficiary: T::AccountId,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;
            PaymentBeneficiary::<T>::set(payment_beneficiary);
            Ok(())
        }

        /// Change [PricesConfig], only callable from [Config::ForceOrigin].
        ///
        /// This call must ensure that provided prices configs are sorted by the first element in the tuple.
        #[pallet::call_index(7)]
        #[pallet::weight(
            T::DbWeight::get().writes(1).ref_time() + (100_000 * new_prices_config.len() as u64 * 2)
        )]
        pub fn set_price_config(
            origin: OriginFor<T>,
            mut new_prices_config: PricesConfigVec<T>,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;

            new_prices_config.sort_by_key(|(length, _)| *length);
            new_prices_config.dedup_by_key(|(length, _)| *length);

            PricesConfig::<T>::set(new_prices_config);
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // TODO: refactor long method
        fn do_register_domain(domain_data: DomainRegisterData<T>, payer: DomainPayer<T>) -> DispatchResult {
            let is_forced = match payer {
                DomainPayer::ForceOrigin => IsForced::Yes,
                DomainPayer::Account(_) => IsForced::No,
            };

            let expires_in = T::RegistrationPeriod::get();
            let DomainRegisterData { owner, full_domain} = domain_data;

            // Note that while upper and lower case letters are allowed in domain
            // names, domain names are not case-sensitive. That is, two names with
            // the same spelling but different cases will be treated as identical.
            let domain_lc = Self::lower_domain_then_bound(&full_domain);
            let domain_parts = Self::split_domain_by_dot(&domain_lc);

            // Perform checks that require storage access.
            Self::ensure_valid_domain(&domain_parts)?;

            let (subdomain, tld) = Self::get_domain_subset(&domain_parts);

            ensure!(Self::is_tld_supported(tld), Error::<T>::TldNotSupported);
            Self::ensure_domain_is_free(&domain_lc)?;
            Self::ensure_can_reserve_deposit(&owner, &is_forced)?;

            Self::ensure_within_domains_limit(&owner)?;
            let price = Self::calculate_price(&subdomain);

            if let DomainPayer::Account(payer) = payer {
                // TODO: this check is duplicating one, which happens in one of the functions above
                Self::ensure_word_is_not_reserved(&subdomain)?;
                Self::ensure_can_pay_for_domain(&payer, price)?;

                // Perform write operations.
                <T as Config>::Currency::transfer(
                    &payer,
                    &Self::payment_beneficiary(),
                    price,
                    KeepAlive,
                )?;
            }

            let deposit = Self::try_reserve_domain_deposit(&owner, &is_forced)?;
            let expires_at = expires_in.saturating_add(System::<T>::block_number());
            let domain_meta = DomainMeta::new(expires_at, owner.clone(), deposit);

            RegisteredDomains::<T>::insert(&domain_lc, domain_meta);
            DomainsByOwner::<T>::mutate(
                &owner, |domains| {
                    domains.try_push(domain_lc).expect("qed; too many domains per account")
                }
            );

            Self::deposit_event(Event::DomainRegistered { who: owner, domain: full_domain });
            Ok(())
        }

        fn do_set_record(
            domain: DomainName<T>,
            key: DomainRecordKey<T>,
            value_opt: Option<DomainRecordValue<T>>,
            check_ownership: Option<T::AccountId>,
        ) -> DispatchResult {
            let domain_lc = Self::lower_domain_then_bound(&domain);
            let meta = Self::require_domain(domain_lc.clone())?;
            let owner = meta.owner.clone();
            let should_reserve_deposit = check_ownership.is_some();

            if let Some(should_be_owner) = check_ownership {
                Self::ensure_allowed_to_update_domain(&meta, &should_be_owner)?;
            }

            let current_record = DomainRecords::<T>::get(domain_lc.clone(), key.clone());

            let (old_depositor, old_deposit) =
                current_record.map_or((owner.clone(), 0u32.into()), |r| (r.depositor, r.deposit));

            let new_deposit = should_reserve_deposit
                .then(|| Self::calc_record_deposit(key.clone(), value_opt.clone()))
                .unwrap_or_default();

            Self::try_reserve_deposit(&old_depositor, old_deposit, &owner, new_deposit)?;

            DomainRecords::<T>::mutate_exists(domain_lc.clone(), key.clone(), |current_opt| {
                *current_opt = value_opt.clone().map(|value| RecordValueWithDepositInfo::<T> {
                    record_value: value,
                    depositor: owner.clone(),
                    deposit: new_deposit,
                });
            });

            Self::deposit_event(Event::DomainRecordUpdated {
                account: owner,
                domain: domain_lc,
                key,
                value: value_opt,
                deposit: new_deposit,
            });
            Ok(())
        }

        pub(crate) fn calc_record_deposit(
            key: DomainRecordKey<T>,
            value_opt: Option<DomainRecordValue<T>>,
        ) -> BalanceOf<T> {
            let num_of_bytes: u32 = if let Some(value) = value_opt {
                key.len().saturating_add(value.len()) as u32
            } else {
                return 0u32.into()
            };

            T::RecordByteDeposit::get().saturating_mul(num_of_bytes.into())
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

            ensure!(
                first_char_alphanumeric && last_char_alphanumeric,
                Error::<T>::DomainContainsInvalidChar
            );

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

        /// lowercase the given domain then bound it to [DomainName] type.
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
            Ok(Self::registered_domain(domain).ok_or(Error::<T>::DomainNotFound)?)
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

        /// Reserve new_deposit from new_depositor, and refunds the old_deposit to old_depositor.
        // TODO: move to subsocial-support crate
        pub fn try_reserve_deposit(
            old_depositor: &T::AccountId,
            old_deposit: BalanceOf<T>,
            new_depositor: &T::AccountId,
            new_deposit: BalanceOf<T>,
        ) -> DispatchResult {
            let (balance_to_reserve, balance_to_unreserve) = if old_depositor == new_depositor {
                (
                    new_deposit.saturating_sub(old_deposit), /* will result in a zero
                                                              * old_deposit is bigger than
                                                              * new_deposit. */
                    old_deposit.saturating_sub(new_deposit), /* will result in a zero
                                                              * new_deposit is bigger than
                                                              * old_deposit. */
                )
            } else {
                (
                    new_deposit, /* since the new_depositor didn't reserve anything, the whole
                                  * new deposit should be reserved. */
                    old_deposit, /* since the old_depositor is no longer maintaining the
                                  * deposit, the whole previous deposit should be refunded. */
                )
            };

            if !balance_to_reserve.is_zero() {
                <T as Config>::Currency::reserve(new_depositor, balance_to_reserve)?;
            }

            if !balance_to_unreserve.is_zero() {
                let err_amount =
                    <T as Config>::Currency::unreserve(old_depositor, balance_to_unreserve);
                debug_assert!(err_amount.is_zero());
            }

            Ok(())
        }

        pub(crate) fn split_domain_by_dot(full_domain: &DomainName<T>) -> Vec<DomainName<T>> {
            full_domain.split(|c| *c == b'.').map(Self::lower_domain_then_bound).collect()
        }

        pub(crate) fn get_domain_subset(parts: &[DomainName<T>]) -> DomainParts<T> {
            let tld = parts.last().unwrap().clone();
            let subdomain = parts.first().unwrap().clone();
            (subdomain, tld)
        }

        pub(crate) fn ensure_within_domains_limit(owner: &T::AccountId) -> DispatchResult {
            let domains_per_account = DomainsByOwner::<T>::decode_len(owner)
                .unwrap_or(Zero::zero());

            ensure!(
                domains_per_account < T::MaxDomainsPerAccount::get() as usize,
                Error::<T>::TooManyDomainsPerAccount,
            );
            Ok(())
        }

        pub(crate) fn ensure_domain_is_free(domain_lc: &DomainName<T>) -> DispatchResult {
            ensure!(
                !RegisteredDomains::<T>::contains_key(domain_lc),
                Error::<T>::DomainAlreadyOwned,
            );
            Ok(())
        }

        pub(crate) fn try_reserve_domain_deposit(
            depositor: &T::AccountId,
            is_forced: &IsForced,
        ) -> Result<BalanceOf<T>, DispatchError> {
            match is_forced {
                IsForced::No => {
                    // TODO: unreserve the balance for expired domains
                    let deposit = T::BaseDomainDeposit::get();
                    Self::try_reserve_deposit(
                        depositor,
                        Zero::zero(),
                        depositor,
                        deposit,
                    )?;
                    Ok(deposit)
                },
                IsForced::Yes => Ok(Zero::zero()),
            }
        }

        fn ensure_can_reserve_deposit(depositor: &T::AccountId, is_forced: &IsForced) -> DispatchResult {
            match is_forced {
                IsForced::No => {
                    ensure!(
                        <T as Config>::Currency::can_reserve(depositor, T::BaseDomainDeposit::get()),
                        Error::<T>::InsufficientBalanceToReserveDeposit,
                    );
                },
                IsForced::Yes => (),
            }
            Ok(())
        }

        fn ensure_word_is_not_reserved(word: &DomainName<T>) -> DispatchResult {
            let word_without_hyphens =
                Self::bound_domain(word.iter().filter(|c| **c != b'-').cloned().collect());

            ensure!(!Self::is_word_reserved(word_without_hyphens), Error::<T>::DomainIsReserved);
            Ok(())
        }

        pub(crate) fn calculate_price(subdomain: &DomainName<T>) -> BalanceOf<T> {
            let price_config = Self::prices_config();
            let subdomain_len = subdomain.len() as u32;

            let partition_point = price_config.partition_point(|(l, _)| l <= &subdomain_len);
            let (_, price) = price_config[partition_point.saturating_sub(1)];

            price
        }

        fn ensure_can_pay_for_domain(owner: &T::AccountId, price: BalanceOf<T>) -> DispatchResult {
            let balance = <T as Config>::Currency::free_balance(owner);
            <T as Config>::Currency::ensure_can_withdraw(
                &owner, price, WithdrawReasons::TRANSFER, balance.saturating_sub(price)
            )
        }

        pub fn init_pallet(default_prices: &[(DomainLength, BalanceOf<T>)]) {
            PricesConfig::<T>::mutate(|prices| {
                default_prices.iter().for_each(|(length, price)| {
                    match prices.binary_search_by_key(length, |(l, _)| *l) {
                        Ok(_) => (),
                        Err(index) => prices.insert(index, (*length, *price)),
                    }
                });
            });
        }
    }
}
