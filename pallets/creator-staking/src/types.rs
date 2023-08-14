use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::traits::Currency;
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{traits::Zero, RuntimeDebug};
use sp_std::{ops::Add, prelude::*};

use super::*;

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
//
// Since dApps staking has been stable for long time and there are plans to redesign & refactor it,
// doing neither of the above makes sense, timewise. So we use an assumption that vec length
// won't go over the following constant.
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

impl<AccountId> CreatorInfo<AccountId> {
    /// Create new `CreatorInfo` struct instance with the given developer and state `Registered`
    pub(super) fn new(stakeholder: AccountId) -> Self {
        Self { stakeholder, state: CreatorState::Registered }
    }
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

impl<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> AccountLedger<Balance> {
    /// `true` if ledger is empty (no locked funds, no unbonding chunks), `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.locked.is_zero() && self.unbonding_info.is_empty()
    }
}

/// Used to represent how much was staked in a particular era.
/// E.g. `{staked: 1000, era: 5}` means that in era `5`, staked amount was 1000.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EraStake<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> {
    /// Staked amount in era
    #[codec(compact)]
    staked: Balance,
    /// Staked era
    #[codec(compact)]
    era: EraIndex,
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> EraStake<Balance> {
    /// Create a new instance of `EraStake` with given values
    fn new(staked: Balance, era: EraIndex) -> Self {
        Self { staked, era }
    }
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
    stakes: Vec<EraStake<Balance>>,
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> MaxEncodedLen for StakerInfo<Balance> {
    // This is just an assumption, will be calculated properly in the future. See the comment for
    // `MAX_ASSUMED_VEC_LEN`.
    fn max_encoded_len() -> usize {
        codec::Compact(MAX_ASSUMED_VEC_LEN).encoded_size().saturating_add(
            (MAX_ASSUMED_VEC_LEN as usize).saturating_mul(EraStake::<Balance>::max_encoded_len()),
        )
    }
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> StakerInfo<Balance> {
    /// `true` if no active stakes and unclaimed eras exist, `false` otherwise
    pub(super) fn is_empty(&self) -> bool {
        self.stakes.is_empty()
    }

    /// number of `EraStake` chunks
    pub(super) fn len(&self) -> u32 {
        self.stakes.len() as u32
    }

    /// Stakes some value in the specified era.
    ///
    /// User should ensure that given era is either equal or greater than the
    /// latest available era in the staking info.
    ///
    /// # Example
    ///
    /// The following example demonstrates how internal vector changes when `stake` is called:
    ///
    /// `stakes: [<5, 1000>, <7, 1300>]`
    /// * `stake(7, 100)` will result in `[<5, 1000>, <7, 1400>]`
    /// * `stake(9, 200)` will result in `[<5, 1000>, <7, 1400>, <9, 1600>]`
    pub(super) fn stake(&mut self, current_era: EraIndex, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("Unexpected era")
            }

            let new_stake_value = era_stake.staked.saturating_add(value);

            if current_era == era_stake.era {
                *era_stake = EraStake::new(new_stake_value, current_era)
            } else {
                self.stakes.push(EraStake::new(new_stake_value, current_era))
            }
        } else {
            self.stakes.push(EraStake::new(value, current_era));
        }

        Ok(())
    }

    /// Unstakes some value in the specified era.
    ///
    /// User should ensure that given era is either equal or greater than the
    /// latest available era in the staking info.
    ///
    /// # Example 1
    ///
    /// `stakes: [<5, 1000>, <7, 1300>]`
    /// * `unstake(7, 100)` will result in `[<5, 1000>, <7, 1200>]`
    /// * `unstake(9, 400)` will result in `[<5, 1000>, <7, 1200>, <9, 800>]`
    /// * `unstake(10, 800)` will result in `[<5, 1000>, <7, 1200>, <9, 800>, <10, 0>]`
    ///
    /// # Example 2
    ///
    /// `stakes: [<5, 1000>]`
    /// * `unstake(5, 1000)` will result in `[]`
    ///
    /// Note that if no unclaimed eras remain, vector will be cleared.
    pub(super) fn unstake(&mut self, current_era: EraIndex, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("Unexpected era")
            }

            let new_stake_value = era_stake.staked.saturating_sub(value);
            if current_era == era_stake.era {
                *era_stake = EraStake::new(new_stake_value, current_era)
            } else {
                self.stakes.push(EraStake::new(new_stake_value, current_era))
            }

            // Removes unstaked values if they're no longer valid for comprehension
            if !self.stakes.is_empty() && self.stakes[0].staked.is_zero() {
                self.stakes.remove(0);
            }
        }

        Ok(())
    }

    /// `Claims` the oldest era available for claiming.
    /// In case valid era exists, returns `(claim era, staked amount)` tuple.
    /// If no valid era exists, returns `(0, 0)` tuple.
    ///
    /// # Example
    ///
    /// The following example will demonstrate how the internal vec changes when `claim` is called
    /// consecutively.
    ///
    /// `stakes: [<5, 1000>, <7, 1300>, <8, 0>, <15, 3000>]`
    ///
    /// 1. `claim()` will return `(5, 1000)`
    ///     Internal vector is modified to `[<6, 1000>, <7, 1300>, <8, 0>, <15, 3000>]`
    ///
    /// 2. `claim()` will return `(6, 1000)`.
    ///    Internal vector is modified to `[<7, 1300>, <8, 0>, <15, 3000>]`
    ///
    /// 3. `claim()` will return `(7, 1300)`.
    ///    Internal vector is modified to `[<15, 3000>]`
    ///    Note that `0` staked period is discarded since nothing can be claimed there.
    ///
    /// 4. `claim()` will return `(15, 3000)`.
    ///    Internal vector is modified to `[16, 3000]`
    ///
    /// Repeated calls would continue to modify vector following the same rule as in *4.*
    pub(super) fn claim(&mut self) -> (EraIndex, Balance) {
        if let Some(era_stake) = self.stakes.first() {
            let era_stake = *era_stake;

            if self.stakes.len() == 1 || self.stakes[1].era > era_stake.era + 1 {
                self.stakes[0] =
                    EraStake { staked: era_stake.staked, era: era_stake.era.saturating_add(1) }
            } else {
                // in case: self.stakes[1].era == era_stake.era + 1
                self.stakes.remove(0);
            }

            // Removes unstaked values if they're no longer valid for comprehension
            if !self.stakes.is_empty() && self.stakes[0].staked.is_zero() {
                self.stakes.remove(0);
            }

            (era_stake.era, era_stake.staked)
        } else {
            (0, Zero::zero())
        }
    }

    /// Latest staked value.
    /// E.g. if staker is fully unstaked, this will return `Zero`.
    /// Otherwise returns a non-zero balance.
    pub fn latest_staked_value(&self) -> Balance {
        self.stakes.last().map_or(Zero::zero(), |x| x.staked)
    }
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

impl<Balance> UnlockingChunk<Balance>
where
    Balance: Add<Output = Balance> + Copy + MaxEncodedLen,
{
    // Adds the specified amount to this chunk
    fn add_amount(&mut self, amount: Balance) {
        self.amount = self.amount + amount
    }
}

/// Contains unlocking chunks.
/// This is a convenience struct that provides various utility methods to help with unbonding
/// handling.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct UnbondingInfo<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> {
    // Vector of unlocking chunks. Sorted in ascending order in respect to unlock_era.
    unlocking_chunks: Vec<UnlockingChunk<Balance>>,
}

impl<Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen> MaxEncodedLen
    for UnbondingInfo<Balance>
{
    // This is just an assumption, will be calculated properly in the future. See the comment for
    // `MAX_ASSUMED_VEC_LEN`.
    fn max_encoded_len() -> usize {
        codec::Compact(MAX_ASSUMED_VEC_LEN).encoded_size().saturating_add(
            (MAX_ASSUMED_VEC_LEN as usize)
                .saturating_mul(UnlockingChunk::<Balance>::max_encoded_len()),
        )
    }
}

impl<Balance> UnbondingInfo<Balance>
where
    Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen,
{
    /// Returns total number of unlocking chunks.
    pub(super) fn len(&self) -> u32 {
        self.unlocking_chunks.len() as u32
    }

    /// True if no unlocking chunks exist, false otherwise.
    fn is_empty(&self) -> bool {
        self.unlocking_chunks.is_empty()
    }

    /// Returns sum of all unlocking chunks.
    pub(super) fn sum(&self) -> Balance {
        self.unlocking_chunks
            .iter()
            .map(|chunk| chunk.amount)
            .reduce(|c1, c2| c1 + c2)
            .unwrap_or_default()
    }

    /// Adds a new unlocking chunk to the vector, preserving the unlock_era based ordering.
    pub(super) fn add(&mut self, chunk: UnlockingChunk<Balance>) {
        // It is possible that the unbonding period changes so we need to account for that
        match self.unlocking_chunks.binary_search_by(|x| x.unlock_era.cmp(&chunk.unlock_era)) {
            // Merge with existing chunk if unlock_eras match
            Ok(pos) => self.unlocking_chunks[pos].add_amount(chunk.amount),
            // Otherwise insert where it should go. Note that this will in almost all cases return
            // the last index.
            Err(pos) => self.unlocking_chunks.insert(pos, chunk),
        }
    }

    /// Partitions the unlocking chunks into two groups:
    ///
    /// First group includes all chunks which have unlock era lesser or equal to the specified era.
    /// Second group includes all the rest.
    ///
    /// Order of chunks is preserved in the two new structs.
    pub(super) fn partition(self, era: EraIndex) -> (Self, Self) {
        let (matching_chunks, other_chunks): (
            Vec<UnlockingChunk<Balance>>,
            Vec<UnlockingChunk<Balance>>,
        ) = self.unlocking_chunks.iter().partition(|chunk| chunk.unlock_era <= era);

        (Self { unlocking_chunks: matching_chunks }, Self { unlocking_chunks: other_chunks })
    }
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
