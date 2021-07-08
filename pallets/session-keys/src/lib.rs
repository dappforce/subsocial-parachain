#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::boxed_local)]

use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Zero, Dispatchable, Saturating, SaturatedConversion};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    weights::{
        GetDispatchInfo, DispatchClass, WeighData,
        Weight, ClassifyDispatch, PaysFee, Pays,
    },
    dispatch::{DispatchError, DispatchResult, PostDispatchInfo},
    traits::{
        Currency, Get, ExistenceRequirement,
        OriginConfig, IsType, Filter,
    },
    Parameter,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::WhoAndWhen;

struct CalculateProxyWeight<T: Config>(Box<<T as Config>::Call>);
impl<T: Config> WeighData<(&Box<<T as Config>::Call>,)> for CalculateProxyWeight<T> {
    fn weigh_data(&self, target: (&Box<<T as Config>::Call>,)) -> Weight {
        let call_dispatch_info = target.0.get_dispatch_info();
        let db_weight = T::DbWeight::get();
        let mut weight = call_dispatch_info.weight;

        match call_dispatch_info.pays_fee {
            Pays::Yes => weight = weight.saturating_add(db_weight.reads_writes(1, 1)),
            Pays::No => weight = weight.saturating_add(db_weight.reads_writes(1, 0)),
        }

        weight
    }
}

impl<T: Config> ClassifyDispatch<(&Box<<T as Config>::Call>,)> for CalculateProxyWeight<T> {
    fn classify_dispatch(&self, _target: (&Box<<T as Config>::Call>,)) -> DispatchClass {
        DispatchClass::Normal
    }
}

impl<T: Config> PaysFee<(&Box<<T as Config>::Call>,)> for CalculateProxyWeight<T> {
    fn pays_fee(&self, _target: (&Box<<T as Config>::Call>,)) -> Pays {
        Pays::Yes
    }
}

type BalanceOf<T> = <<T as pallet_utils::Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

// TODO define session key permissions

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SessionKey<T: Config> {
    /// Who and when created this session key.
    pub created: WhoAndWhen<T>,

    /// The last time this session key was used or updated by its owner.
    pub updated: Option<WhoAndWhen<T>>,

    /// A block number when this session key should be expired.
    pub expires_at: T::BlockNumber,

    /// Max amount of tokens allowed to spend with this session key.
    pub limit: Option<BalanceOf<T>>,

    /// How much tokens this session key already spent.
    pub spent: BalanceOf<T>,

    // TODO allowed_actions: ...
}

/// The pallet's configuration trait.
pub trait Config: system::Config + pallet_utils::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    /// The overarching call type.
    type Call: Parameter
        + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo>
        + GetDispatchInfo + From<frame_system::Call<Self>>
        + IsType<<Self as frame_system::Config>::Call>;

    /// The maximum amount of session keys allowed for a single account.
    type MaxSessionKeysPerAccount: Get<u16>;

    /// Base Call filter for the session keys' proxy
    type BaseFilter: Filter<<Self as Config>::Call>;
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId
    {
        SessionKeyAdded(/* owner */ AccountId, /* session key */ AccountId),
        SessionKeyRemoved(/* session key */ AccountId),
        AllSessionKeysRemoved(/* owner */ AccountId),
        /// A proxy was executed correctly, with the given result.
		ProxyExecuted(DispatchResult),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Session key details was not found by its account id.
        SessionKeyNotFound,
        /// Account already added as a session key.
        SessionKeyAlreadyAdded,
        /// There are too many session keys registered to this account.
        TooManySessionKeys,
        /// Time to live (TTL) of a session key cannot be zero.
        ZeroTimeToLive,
        /// Limit of a session key cannot be zero.
        ZeroLimit,
        /// Session key is expired.
        SessionKeyExpired,
        /// Reached the limit of tokens this session key can spend.
        SessionKeyLimitReached,
        /// Only a session key owner can manage their keys.
        NeitherSessionKeyOwnerNorExpired,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as SessionKeysModule {

        /// Session key details by its account id (key).
        pub KeyDetails get(fn key_details):
            map hasher(blake2_128_concat)/* session key */ T::AccountId
            => Option<SessionKey<T>>;

        /// A binary-sorted list of all session keys owned by the account.
        pub KeysByOwner get(fn keys_by_owner):
            map hasher(twox_64_concat) /* primary owner */ T::AccountId
            => /* session keys */ Vec<T::AccountId>;

        /// List of session keys and their owner by expiration block number
        /// Vec<(KeyOwner, SessionKey)>
        SessionKeysByExpireBlock:
            map hasher(twox_64_concat)/* expiration_block_number */ T::BlockNumber
            => /* (key owner, session key) */ Vec<(T::AccountId, T::AccountId)>;
    }
}

// The pallet's dispatchable functions.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        const MaxSessionKeysPerAccount: u16 = T::MaxSessionKeysPerAccount::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Add a new SessionKey for `origin` bonding 2 * Existential Deposit to keep session alive
        #[weight = 10_000 + T::DbWeight::get().reads_writes(3, 3)]
        fn add_key(origin,
            key_account: T::AccountId,
            time_to_live: T::BlockNumber,
            limit: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(time_to_live > Zero::zero(), Error::<T>::ZeroTimeToLive);
            ensure!(limit != Some(Zero::zero()), Error::<T>::ZeroLimit);
            ensure!(!KeyDetails::<T>::contains_key(key_account.clone()), Error::<T>::SessionKeyAlreadyAdded);

            let mut keys = KeysByOwner::<T>::get(who.clone());
            ensure!(keys.len() < T::MaxSessionKeysPerAccount::get() as usize, Error::<T>::TooManySessionKeys);

            Self::keep_session_alive(&who, &key_account)?;

            let i = keys.binary_search(&key_account).err().ok_or(Error::<T>::SessionKeyAlreadyAdded)?;
            keys.insert(i, key_account.clone());
            KeysByOwner::<T>::insert(&who, keys);

            let details = SessionKey::<T>::new(who.clone(), time_to_live, limit);
            KeyDetails::<T>::insert(key_account.clone(), details);

            let current_block = system::Pallet::<T>::block_number();
            let expiration_block = current_block.saturating_add(time_to_live);

            SessionKeysByExpireBlock::<T>::mutate(
                expiration_block,
                |keys| keys.push((who.clone(), key_account.clone()))
            );

            Self::deposit_event(RawEvent::SessionKeyAdded(who, key_account));
            Ok(())
        }

        /// A key could be removed either the origin is an owner or key is expired.
        #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 2)]
        fn remove_key(origin, key_account: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let key = Self::require_key(key_account.clone())?;
            key.ensure_owner_or_expired(who.clone())?;

            // Deposits event on success
            Self::try_remove_key(who, key_account)?;
            Ok(())
        }

        /// Unregister all session keys for the sender.
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 2) * T::MaxSessionKeysPerAccount::get() as u64]
        fn remove_keys(origin) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let keys = KeysByOwner::<T>::take(&who);
            for key in keys {
                KeyDetails::<T>::remove(&key);
                Self::withdraw_key_account_to_owner(&key, &who, None)?;
            }

            Self::deposit_event(RawEvent::AllSessionKeysRemoved(who));
            Ok(())
        }

        /// `origin` is a session key
        #[weight = CalculateProxyWeight::<T>(call.clone())]
        fn proxy(origin, call: Box<<T as Config>::Call>) -> DispatchResult {
            let key = ensure_signed(origin)?;

            let mut details = Self::require_key(key.clone())?;

            if details.is_expired() {
                Self::try_remove_key(details.created.account, key)?;
                return Err(Error::<T>::SessionKeyExpired.into());
            }

            let real = details.owner();
            let can_spend: BalanceOf<T>;

            // TODO get limit from account settings

            if let Some(limit) = details.limit {
                can_spend = limit.saturating_sub(details.spent);
                ensure!(can_spend > Zero::zero(), Error::<T>::SessionKeyLimitReached);
            }

            let call_dispatch_info = call.get_dispatch_info();
            if call_dispatch_info.pays_fee == Pays::Yes {
                let spent_on_call = BalanceOf::<T>::saturated_from(call_dispatch_info.weight.into());
                T::Currency::transfer(&real, &key, spent_on_call, ExistenceRequirement::KeepAlive)?;

                // TODO: what if balance left is less than InclusionFee on the next call?

                details.spent = details.spent.saturating_add(spent_on_call);
                details.updated = Some(WhoAndWhen::<T>::new(key.clone()));

                KeyDetails::<T>::insert(key, details);
            }

            // TODO check that this call is among allowed calls per this account/session key.
            let mut origin: T::Origin = frame_system::RawOrigin::Signed(real).into();
			origin.add_filter(move |c: &<T as frame_system::Config>::Call| {
				let c = <T as Config>::Call::from_ref(c);
				T::BaseFilter::filter(c)
			});

            let e = call.dispatch(origin);
            Self::deposit_event(RawEvent::ProxyExecuted(e.map(|_| ()).map_err(|e| e.error)));

            Ok(())
        }

        fn on_finalize(block_number: T::BlockNumber) {
            let keys_to_remove = SessionKeysByExpireBlock::<T>::take(block_number);
            for key in keys_to_remove {
                let (owner, key_account) = key;
                let _ = Self::try_remove_key(owner, key_account).ok();
            }
        }
    }
}

impl<T: Config> SessionKey<T> {
    pub fn new(
        created_by: T::AccountId,
        time_to_live: T::BlockNumber,
        limit: Option<BalanceOf<T>>,
    ) -> Self {
        SessionKey::<T> {
            created: WhoAndWhen::new(created_by),
            updated: None,
            expires_at: time_to_live + <system::Pallet<T>>::block_number(),
            limit,
            spent: Zero::zero(),
        }
    }

    pub fn owner(&self) -> T::AccountId {
        self.created.account.clone()
    }

    pub fn is_owner(&self, maybe_owner: &T::AccountId) -> bool {
        self.owner() == *maybe_owner
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= <system::Pallet<T>>::block_number()
    }

    pub fn ensure_owner_or_expired(&self, maybe_owner: T::AccountId) -> DispatchResult {
        ensure!(
            self.is_owner(&maybe_owner) || self.is_expired(),
            Error::<T>::NeitherSessionKeyOwnerNorExpired
        );
        Ok(())
    }
}

impl<T: Config> Module<T> {
    /// Get `SessionKey` details by `key_account` from the storage
    /// or return `SessionKeyNotFound` error.
    pub fn require_key(key_account: T::AccountId) -> Result<SessionKey<T>, DispatchError> {
        Ok(Self::key_details(key_account).ok_or(Error::<T>::SessionKeyNotFound)?)
    }

    /// Remove `SessionKey` data from storages if found
    fn try_remove_key(owner: T::AccountId, key_account: T::AccountId) -> DispatchResult {
        KeyDetails::<T>::remove(key_account.clone());

        let mut keys = KeysByOwner::<T>::get(owner.clone());
        let i = keys.binary_search(&key_account).ok().ok_or(Error::<T>::SessionKeyNotFound)?;
        keys.remove(i);
        KeysByOwner::<T>::insert(&owner, keys);

        Self::withdraw_key_account_to_owner(&key_account, &owner, None)?;

        Self::deposit_event(RawEvent::SessionKeyRemoved(key_account));
        Ok(())
    }

    /// Transfer tokens amount/entire free balance (if amount is `None`) from key account to owner
    fn withdraw_key_account_to_owner(
        key_account: &T::AccountId,
        owner: &T::AccountId,
        amount: Option<BalanceOf<T>>
    ) -> DispatchResult {
        T::Currency::transfer(
            key_account,
            owner,
            amount.unwrap_or_else(|| T::Currency::free_balance(key_account)),
            ExistenceRequirement::AllowDeath
        )
    }

    fn keep_session_alive(source: &T::AccountId, key_account: &T::AccountId) -> DispatchResult {
        T::Currency::transfer(
            source,
            key_account,
            T::Currency::minimum_balance().saturating_mul(BalanceOf::<T>::from(2)),
            ExistenceRequirement::KeepAlive
        )
    }
}
