// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{pallet_prelude::BoundedVec, traits::{Currency, Get}};
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{Perbill, traits::Zero, RuntimeDebug};
use sp_std::{fmt::Debug, ops::Add, prelude::*};

use subsocial_support::SpaceId;

use super::*;

pub mod impls;

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

pub type CreatorId = SpaceId;

/// Convenience type for `BackerLocks` usage.
pub(crate) type BackerLocksOf<T> = BackerLocks<BalanceOf<T>, <T as Config>::MaxUnbondingChunks>;

/// Convenience type fo `StakesInfo` usage.
pub(crate) type StakesInfoOf<T> = StakesInfo<BalanceOf<T>, <T as Config>::MaxEraStakeItems>;

/// This enum is used to determine who is calling the `unregister_creator` function.
pub(crate) enum UnregistrationAuthority<AccountId> {
    Root,
    Creator(AccountId),
}

/// `CreatorStatus` is an enumeration that represents the current status of a creator in the system.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub(crate) enum CreatorStatus {
    /// Creator is registered and active.
    Active,
    /// Creator has been unregistered and is inactive.
    /// Claim for past eras and unbonding is still possible but no additional staking can be done.
    Inactive(EraIndex),
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CreatorInfo<AccountId> {
    /// Space owner account
    pub(crate) stakeholder: AccountId,
    /// Current Creator State
    pub(crate) status: CreatorStatus,
}

/// Used to split total EraPayout among creators.
/// Each tuple (creator, era) has this structure.
/// This will be used to reward creators and their backers.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CreatorStakeInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total amount staked on a creator.
    #[codec(compact)]
    pub total_staked: Balance,
    /// Total number of active backers staking towards a creator.
    #[codec(compact)]
    pub(crate) backers_count: u32,
    /// Indicates whether rewards were claimed for this era or not
    pub(crate) rewards_claimed: bool,
}

/// Contains information about an account's locked & unbonding balances.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxUnbondingChunks))]
pub struct BackerLocks<
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
    MaxUnbondingChunks: Get<u32>,
> {
    /// Total balance locked.
    #[codec(compact)]
    pub total_locked: Balance,
    /// Information about unbonding chunks.
    pub(crate) unbonding_info: UnbondingInfo<Balance, MaxUnbondingChunks>,
}

/// Used to represent how many total tokens were staked on the chain in a particular era.
/// E.g. `{staked: 1000, era: 5}` means that in era `5`, staked amount was 1000.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraStake<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    /// Staked amount in era
    #[codec(compact)]
    pub(crate) staked: Balance,
    /// Staked era
    #[codec(compact)]
    pub(super) era: EraIndex,
}

/// Used to provide a compact and bounded storage for information about stakes in unclaimed eras.
///
/// In order to avoid creating a separate storage entry for each `(backer, creator, era)` triplet,
/// this struct is used to provide a more memory efficient solution.
///
/// Basic idea is to store `EraStake` structs into a vector from which a complete
/// picture of **unclaimed eras** and stakes can be constructed.
///
/// # Example
/// For simplicity, the following example will represent `EraStake` using `<era, stake>` notation.
/// Let us assume we have the following vector in `StakesInfoOf` struct.
///
/// `[<5, 1000>, <6, 1500>, <8, 2100>, <9, 0>, <11, 500>]`
///
/// This tells us which eras are unclaimed and how much was staked in each era.
/// The interpretation is the following:
/// 1. In era **5**, staked amount was **1000** (interpreted from `<5, 1000>`)
/// 2. In era **6**, backer staked additional **500**, increasing total staked amount to **1500**
/// 3. No entry for era **7** exists which means there were no changes from the former entry.
///    This means that in era **7**, staked amount was also **1500**
/// 4. In era **8**, backer staked an additional **600**, increasing total stake to **2100**
/// 5. In era **9**, backer unstaked everything from the creator (interpreted from `<9, 0>`)
/// 6. No changes were made in era **10** so we can interpret this same as the previous entry which
/// means **0** staked amount. 7. In era **11**, backer staked **500** on the creator, making his
/// stake active again after 2 eras of inactivity.
///
/// **NOTE:** It is important to understand that backer **DID NOT** claim any rewards during this
/// period.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxEraStakeItems))]
pub struct StakesInfo<
    Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen,
    MaxEraStakeItems: Get<u32>,
> {
    pub(crate) stakes: BoundedVec<EraStake<Balance>, MaxEraStakeItems>,
}

/// Represents a balance amount that is currently unbonding.
/// Since unbonding takes time, it's important to keep track of when and how much was unbonded.
#[derive(
Clone, Copy, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct UnbondingChunk<Balance: MaxEncodedLen> {
    /// Amount being unbonded
    #[codec(compact)]
    pub(crate) amount: Balance,
    /// Era in which the amount will become unlocked and can be withdrawn.
    #[codec(compact)]
    pub(crate) unlock_era: EraIndex,
}

/// Contains unbonding chunks.
/// This is a convenience struct that provides various utility methods to help with unbonding
/// handling.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxUnbondingChunks))]
pub struct UnbondingInfo<
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen,
    MaxUnbondingChunks: Get<u32>,
> {
    // Vector of unbonding chunks. Sorted in ascending order in respect to unlock_era.
    pub(crate) unbonding_chunks: BoundedVec<UnbondingChunk<Balance>, MaxUnbondingChunks>,
}

/// A record of rewards allocated for backers and creators
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total amount of rewards for backers in an era
    #[codec(compact)]
    pub backers: Balance,
    /// Total amount of rewards for creators in an era
    #[codec(compact)]
    pub creators: Balance,
}

/// A record of total rewards and total amount staked in a particular era
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraInfo<Balance: HasCompact + MaxEncodedLen> {
    /// Total amount of earned rewards in an era
    pub rewards: RewardInfo<Balance>,
    /// Total staked amount in an era
    #[codec(compact)]
    pub staked: Balance,
    /// Total locked amount in an era: stake locks + unbonding locks.
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
    ForceNew,
}

/// A list of configuration parameters used to calculate reward distribution portions.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct RewardDistributionConfig {
    /// Base percentage of reward that goes to backers
    #[codec(compact)]
    pub backers_percent: Perbill,
    /// Percentage of rewards that goes to creators
    #[codec(compact)]
    pub creators_percent: Perbill,
    /// Percentage of rewards that goes to the treasury
    #[codec(compact)]
    pub treasury_percent: Perbill,
}
