use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{pallet_prelude::BoundedVec, traits::{Currency, Get}};
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{Perbill, traits::Zero, RuntimeDebug};
use sp_std::{fmt::Debug, ops::Add, prelude::*};

use super::*;

pub mod impls;

pub(crate) type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Convenience type for `StakerLedger` usage.
pub(crate) type BackerLocksOf<T> = BackerLocks<BalanceOf<T>, <T as Config>::MaxUnlockingChunks>;

/// Convenience type fo `StakesInfo` usage.
pub(crate) type StakesInfoOf<T> = StakesInfo<BalanceOf<T>, <T as Config>::MaxEraStakeItems>;

/// This enum is used to determine who is calling the `unregister_creator` function.
pub(super) enum UnregistrationAuthority<AccountId> {
    Root,
    Creator(AccountId),
}

/// `CreatorStatus` is an enumeration that represents the current status of a creator in the system.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub(super) enum CreatorStatus {
    /// Creator is registered and active.
    Active,
    /// Creator has been unregistered and is inactive.
    /// Claim for past eras and unbonding is still possible but no additional staking can be done.
    Inactive(EraIndex),
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CreatorInfo<AccountId> {
    /// Space owner account
    pub(super) stakeholder: AccountId,
    /// Current Creator State
    pub(super) status: CreatorStatus,
}

/// Used to split total EraPayout among creators.
/// Each tuple (creator, era) has this structure.
/// This will be used to reward creators developer and his stakers.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CreatorStakeInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total staked amount.
    #[codec(compact)]
    pub total: Balance,
    /// Total number of active stakers
    #[codec(compact)]
    pub(super) stakers_count: u32,
    /// Indicates whether rewards were claimed for this era or not
    pub(super) rewards_claimed: bool,
}

/// Contains information about account's locked & unbonding balances.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxUnlockingChunks))]
pub struct BackerLocks<
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
    MaxUnlockingChunks: Get<u32>,
> {
    /// Total balance locked.
    #[codec(compact)]
    pub locked: Balance,
    /// Information about unbonding chunks.
    pub(super) unbonding_info: UnbondingInfo<Balance, MaxUnlockingChunks>,
}

/// Used to represent how much was staked in a particular era.
/// E.g. `{staked: 1000, era: 5}` means that in era `5`, staked amount was 1000.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraStake<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    /// Staked amount in era
    #[codec(compact)]
    pub(super) staked: Balance,
    /// Staked era
    #[codec(compact)]
    era: EraIndex,
}

/// Used to provide a compact and bounded storage for information about stakes in unclaimed eras.
///
/// In order to avoid creating a separate storage entry for each `(staker, creator, era)` triplet,
/// this struct is used to provide a more memory efficient solution.
///
/// Basic idea is to store `EraStake` structs into a vector from which a complete
/// picture of **unclaimed eras** and stakes can be constructed.
///
/// # Example
/// For simplicity, the following example will represent `EraStake` using `<era, stake>` notation.
/// Let us assume we have the following vector in `StakerInfo` struct.
///
/// `[<5, 1000>, <6, 1500>, <8, 2100>, <9, 0>, <11, 500>]`
///
/// This tells us which eras are unclaimed and how much it was staked in each era.
/// The interpretation is the following:
/// 1. In era **5**, staked amount was **1000** (interpreted from `<5, 1000>`)
/// 2. In era **6**, staker staked additional **500**, increasing total staked amount to **1500**
/// 3. No entry for era **7** exists which means there were no changes from the former entry.
///    This means that in era **7**, staked amount was also **1500**
/// 4. In era **8**, staker staked an additional **600**, increasing total stake to **2100**
/// 5. In era **9**, staker unstaked everything from the creator (interpreted from `<9, 0>`)
/// 6. No changes were made in era **10** so we can interpret this same as the previous entry which
/// means **0** staked amount. 7. In era **11**, staker staked **500** on the creator, making his
/// stake active again after 2 eras of inactivity.
///
/// **NOTE:** It is important to understand that staker **DID NOT** claim any rewards during this
/// period.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxEraStakeValues))]
pub struct StakesInfo<
    Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen,
    MaxEraStakeValues: Get<u32>,
> {
    pub(super) stakes: BoundedVec<EraStake<Balance>, MaxEraStakeValues>,
}

/// Represents an balance amount undergoing the unbonding process.
/// Since unbonding takes time, it's important to keep track of when and how much was unbonded.
#[derive(
Clone, Copy, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct UnlockingChunk<Balance: MaxEncodedLen> {
    /// Amount being unlocked
    #[codec(compact)]
    pub(super) amount: Balance,
    /// Era in which the amount will become unlocked and can be withdrawn.
    #[codec(compact)]
    pub(super) unlock_era: EraIndex,
}

/// Contains unlocking chunks.
/// This is a convenience struct that provides various utility methods to help with unbonding
/// handling.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxUnlockingChunks))]
pub struct UnbondingInfo<
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen,
    MaxUnlockingChunks: Get<u32>,
> {
    // Vector of unlocking chunks. Sorted in ascending order in respect to unlock_era.
    unlocking_chunks: BoundedVec<UnlockingChunk<Balance>, MaxUnlockingChunks>,
}

/// A record of rewards allocated for stakers and creators
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total amount of rewards for stakers in an era
    #[codec(compact)]
    pub stakers: Balance,
    /// Total amount of rewards for creators in an era
    #[codec(compact)]
    pub creators: Balance,
}

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total amount of earned rewards for an era
    pub rewards: RewardInfo<Balance>,
    /// Total staked amount in an era
    #[codec(compact)]
    pub staked: Balance,
    /// Total locked amount in an era
    #[codec(compact)]
    pub locked: Balance,
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    #[default]
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    /// Note that this will force to trigger an election until a new era is triggered, if the
    /// election failed, the next session end will trigger a new election again, until success.
    ForceNew,
}

/// A list of configuration parameters used to calculate reward distribution portions.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct RewardsDistributionConfig {
    /// Base percentage of reward that goes to stakers
    #[codec(compact)]
    pub stakers_percent: Perbill,
    /// Percentage of rewards that goes to creators
    #[codec(compact)]
    pub creators_percent: Perbill,
    /// Percentage of rewards that goes to the treasury
    #[codec(compact)]
    pub treasury_percent: Perbill,
}
