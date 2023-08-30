use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::traits::Currency;
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{Perbill, traits::Zero, RuntimeDebug};
use sp_std::{ops::Add, prelude::*};

use super::*;

pub mod impls;

pub(crate) type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

// This represents the max assumed vector length that any storage item should have.
// In particular, this relates to `UnbondingInfo` and `StakerInfo`.
// In structs which are bound in size, `MaxEncodedLen` can just be derived but that's not the case
// for standard `vec`. To fix this 100% correctly, we'd need to do one of the following:
//
// - Use `BoundedVec` instead of `Vec` and do storage migration
// - Introduce a new type `S: Get<u32>` into the aforementioned structs and use it to inject max
//   allowed size, thus allowing us to correctly calculate max encoded len
//
// The issue with first approach is that it requires storage migration which we want to avoid
// unless it's really necessary. The issue with second approach is that it makes code much more
// difficult to work with since all of it will be ridden with injections of the `S` type.
const MAX_ASSUMED_VEC_LEN: u32 = 10;

// TODO: there are few more options to chose from:
//     - UnregistrationSource: This emphasizes where the unregistration is coming from.
//     - UnregisteringParty: This focuses on who is performing the unregistration.
//     - UnregistrationActor: This could signify the entity taking action in the unregistration process.
//     - UnregistrationInitiator: This implies the entity that started or initiated the unregistration.
/// This enum is used to determine who is calling the `unregister_creator` function.
pub(super) enum UnregistrationAuthority<AccountId> {
    Creator(AccountId),
    Root,
}

/// Creator State descriptor
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub(super) enum CreatorState {
    /// Creator is registered and active.
    Registered,
    /// Creator has been unregistered and is inactive.
    /// Claim for past eras and unbonding is still possible but no additional staking can be done.
    Unregistered(EraIndex),
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CreatorInfo<AccountId> {
    /// Space owner account
    pub(super) stakeholder: AccountId,
    /// Current Creator State
    pub(super) state: CreatorState,
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
    pub(super) number_of_stakers: u32,
    /// Indicates whether rewards were claimed for this era or not
    pub(super) creator_reward_claimed: bool,
}

/// Contains information about account's locked & unbonding balances.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AccountLedger<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> {
    /// Total balance locked.
    #[codec(compact)]
    pub locked: Balance,
    /// Information about unbonding chunks.
    pub(super) unbonding_info: UnbondingInfo<Balance>,
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
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct StakerInfo<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    // Size of this list would be limited by a configurable constant
    pub(super) stakes: Vec<EraStake<Balance>>,
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
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct UnbondingInfo<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> {
    // Vector of unlocking chunks. Sorted in ascending order in respect to unlock_era.
    unlocking_chunks: Vec<UnlockingChunk<Balance>>,
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
pub struct RewardDistributionConfig {
    /// Base percentage of reward that goes to stakers
    #[codec(compact)]
    pub stakers_percent: Perbill,
    /// Percentage of rewards that goes to dApps
    #[codec(compact)]
    pub creators_percent: Perbill,
}
