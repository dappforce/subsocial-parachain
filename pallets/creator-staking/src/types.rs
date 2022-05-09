use std::collections::BTreeMap;
use sp_std::ops::Add;
use frame_support::pallet_prelude::*;
use sp_std::prelude::*;
use sp_runtime::{ArithmeticError, Perbill, traits::CheckedAdd};
use frame_support::sp_std;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedSub, Zero};
use crate::{BalanceOf, Config, Error};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RewardConfigInfo {
    pub creators_percentage: Perbill,
    pub stakers_percentage: Perbill,
}

impl RewardConfigInfo {
    pub fn new(creator_percentage: Perbill, stakers_percentage: Perbill) -> Option<Self> {
        let config = RewardConfigInfo::new_unchecked(creator_percentage, stakers_percentage);
        if config.is_valid() {
            Some(config)
        } else {
            None
        }
    }

    pub fn new_unchecked(creator_percentage: Perbill, stakers_percentage: Perbill) -> Self {
        Self {
            creators_percentage: creator_percentage,
            stakers_percentage,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.creators_percentage.checked_add(&self.stakers_percentage) {
            None => false,
            Some(x) if x != Perbill::one() => false,
            _ => true,
        }
    }
}

impl Default for RewardConfigInfo {
    fn default() -> Self {
        Self {
            creators_percentage: Perbill::from_percent(50),
            stakers_percentage: Perbill::from_percent(50),
        }
    }
}


/// Type to index staking rounds.
pub(crate) type RoundIndex = u32;

/// The current round index and transition information
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Round<BlockNumber> {
    /// Current round index
    pub index: RoundIndex,
    /// The first block of the current round
    pub first: BlockNumber,
    /// The length of the current round in number of blocks
    pub length: u32,
}

impl<
    B: Copy + Add<Output=B> + sp_std::ops::Sub<Output=B> + From<u32> + PartialOrd,
> Default for Round<B>
{
    fn default() -> Round<B> {
        Round::new(1u32, 1u32.into(), 20u32)
    }
}

impl<
    B: Copy + Add<Output=B> + sp_std::ops::Sub<Output=B> + From<u32> + PartialOrd,
> Round<B>
{
    pub fn new(current: RoundIndex, first: B, length: u32) -> Round<B> {
        Round {
            index: current,
            first,
            length,
        }
    }
    /// Check if the round should be updated
    pub fn should_update(&self, now: B) -> bool {
        now - self.first >= self.length.into()
    }
    /// New round
    pub fn update(&mut self, now: B) {
        self.index = self.index.saturating_add(1u32);
        self.first = now;
    }
}

/// A record of rewards allocated for stakers and creators
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RewardInfo<Balance> {
    /// Total amount of rewards for stakers in a round
    pub stakers: Balance,
    /// Total amount of rewards for creators in a round
    pub creators: Balance,
}

/// A record for total rewards and total amount staked for a round
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RoundInfo<Balance> {
    /// Total amount of earned rewards for a round
    pub rewards: RewardInfo<Balance>,
    /// Total staked amount in a round
    pub staked: Balance,
    /// Total locked amount in a round
    pub locked: Balance,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CreatorInfo<T: Config> {
    /// Staker account
    pub id: T::AccountId,

    /// The deposit used for registration.
    pub deposit: BalanceOf<T>,

    /// The total number of stakers to this creator.
    pub stakers_count: u32,

    /// The total amount of tokens staked to this creator.
    pub staked_amount: BalanceOf<T>,
}

impl<T: Config> CreatorInfo<T> {
    pub fn from_account(account: T::AccountId, deposit: BalanceOf<T>) -> Self {
        Self {
            id: account,
            deposit,
            stakers_count: 0,
            staked_amount: Zero::zero(),
        }
    }
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum StakerStatus {
    /// Active with no scheduled exit
    Active,
    /// Schedule exit to revoke all ongoing delegations
    Leaving(RoundIndex),
}


#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub(crate) struct StakerInfo<T: Config> {
    /// Staker account
    pub id: T::AccountId,

    /// Total balance that is locked. (active + unlocking)
    pub total: BalanceOf<T>,

    /// The total amount of balance that will be in stake in the next rounds.
    pub active: BalanceOf<T>,

    /// Amount of balance staked for each creator.
    pub stake_per_creator: BTreeMap<T::AccountId, StakeState<T>>,

    /// Any balance that's in the process of being unlocked.
    pub unlocking: BoundedVec<UnlockChunk<BalanceOf<T>>, T::MaxUnlockingChunks>,

    /// The status of the staker.
    pub status: StakerStatus,
}

impl<T: Config> StakerInfo<T> {
    pub fn from_account(account: T::AccountId, stake: BalanceOf<T>) -> Self {
        Self {
            id: account,
            total: stake,
            active: stake,
            unlocking: Default::default(),
            status: StakerStatus::Active,
            stake_per_creator: Default::default(),
        }
    }
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UnlockChunk<Balance> {
    /// Amount of funds to be unlocked.
    pub value: Balance,

    /// Era number at which point it'll be unlocked.
    pub round: RoundIndex,
}

/// Used to represent how much was staked in a particular era.
/// E.g. `{staked: 1000, era: 5}` means that in era `5`, staked amount was 1000.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct RoundBond<Balance: AtLeast32BitUnsigned + Copy> {
    /// Staked amount in era
    #[codec(compact)]
    bond: Balance,
    /// Staked era
    #[codec(compact)]
    round: RoundIndex,
}

impl<Balance: AtLeast32BitUnsigned + Copy> RoundBond<Balance> {
    /// Create a new instance of `EraStake` with given values
    fn new(staked: Balance, era: RoundIndex) -> Self {
        Self { bond: staked, round: era }
    }
}

/// Used to provide a compact and bounded storage for information about stakes in unclaimed eras.
///
/// In order to avoid creating a separate storage entry for each `(staker, contract, era)` triplet,
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
/// 5. In era **9**, staker unstaked everything from the contract (interpreted from `<9, 0>`)
/// 6. No changes were made in era **10** so we can interpret this same as the previous entry which means **0** staked amount.
/// 7. In era **11**, staker staked **500** on the contract, making his stake active again after 2 eras of inactivity.
///
/// **NOTE:** It is important to understand that staker **DID NOT** claim any rewards during this period.
///
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub(crate) struct StakeState<T: Config> {
    // Size of this list would be limited by a configurable constant
    stakes: Vec<RoundBond<BalanceOf<T>>>,
}

impl<T: Config> StakeState<T> {
    pub(crate) fn default() -> Self {
        Self { stakes: Default::default() }
    }

    #[cfg(test)]
    pub(crate) fn new(mut stakes: Vec<RoundBond<BalanceOf<T>>>) -> Self {
        stakes.sort_by(|a, b| a.round.cmp(&b.round));
        Self { stakes }
    }

    /// `true` if no active stakes and unclaimed eras exist, `false` otherwise
    pub(crate) fn is_empty(&self) -> bool {
        self.stakes.is_empty()
    }

    /// number of `EraStake` chunks
    pub(crate) fn len(&self) -> u32 {
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
    ///
    pub(crate) fn stake(&mut self, current_era: RoundIndex, value: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
        let era_stake = match self.stakes.last_mut() {
            None => {
                self.stakes.push(RoundBond::new(value, current_era));
                return Ok(value);
            }
            Some(era_stake) => era_stake,
        };

        ensure!(current_era >= era_stake.round, Error::<T>::RoundNumberOutOfBounds);

        let new_stake_value = era_stake.bond.checked_add(&value)
            .ok_or(ArithmeticError::Overflow)?;

        if current_era == era_stake.round {
            *era_stake = RoundBond::new(new_stake_value, current_era)
        } else {
            self.stakes.push(RoundBond::new(new_stake_value, current_era))
        }

        Ok(new_stake_value)
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
    ///
    pub(crate) fn unstake(&mut self, current_era: RoundIndex, value: BalanceOf<T>) -> DispatchResult {
        let era_stake = match self.stakes.last_mut() {
            None => return Ok(()),
            Some(era_stake) => era_stake,
        };

        ensure!(current_era >= era_stake.round, Error::<T>::RoundNumberOutOfBounds);

        let new_stake_value = era_stake.bond.checked_sub(&value)
            .ok_or(ArithmeticError::Underflow)?;

        if current_era == era_stake.round {
            *era_stake = RoundBond::new(new_stake_value, current_era)
        } else {
            self.stakes.push(RoundBond::new(new_stake_value, current_era))
        }

        // Removes unstaked values if they're no longer valid for comprehension
        if !self.stakes.is_empty() && self.stakes[0].bond.is_zero() {
            self.stakes.remove(0);
        }

        Ok(())
    }

    /// `Claims` the oldest era available for claiming.
    /// In case valid era exists, returns `(claim era, staked amount)` tuple.
    /// If no valid era exists, returns `(0, 0)` tuple.
    ///
    /// # Example
    ///
    /// The following example will demonstrate how the internal vec changes when `claim` is called consecutively.
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
    ///
    pub(crate) fn claim(&mut self) -> Option<RoundBond<BalanceOf<T>>> {
        let era_stake = match self.stakes.first() {
            None => return None,
            Some(era_stake) => era_stake,
        };
        let era_stake = *era_stake;

        if self.stakes.len() == 1 || self.stakes[1].round > era_stake.round + 1 {
            self.stakes[0] = RoundBond {
                bond: era_stake.bond,
                round: era_stake.round.saturating_add(1),
            }
        } else {
            // in case: self.stakes[1].era == era_stake.era + 1
            self.stakes.remove(0);
        }

        // Removes unstaked values if they're no longer valid for comprehension
        if !self.stakes.is_empty() && self.stakes[0].bond.is_zero() {
            self.stakes.remove(0);
        }

        return Some(era_stake);
    }

    /// Latest staked value.
    pub(crate) fn latest_staked_value(&self) -> BalanceOf<T> {
        self.stakes.last()
            .map_or(Zero::zero(), |x| x.bond)
    }
}

#[cfg(test)]
mod staking_state_tests {
    use super::*;
    use crate::mock::*;
    use rstest::rstest;

    type SimpleRoundBond = (RoundIndex, BalanceOf<Test>);

    impl From<SimpleRoundBond> for RoundBond<BalanceOf<Test>> {
        fn from(x: SimpleRoundBond) -> Self {
            RoundBond {
                round: x.0,
                bond: x.1,
            }
        }
    }

    fn option_from_round_bond(rb: RoundBond<BalanceOf<Test>>) -> Option<RoundBond<BalanceOf<Test>>> {
        if rb.bond.is_zero() || rb.round.is_zero() {
            return None
        }

        Some(rb)
    }


    fn create_round_bond_vec(vec: Vec<SimpleRoundBond>) -> Vec<RoundBond<BalanceOf<Test>>> {
        vec.into_iter()
            .map(|(round, bond)| RoundBond { round, bond })
            .collect()
    }

    #[rstest]
    #[case(
        vec![(5, 1000), (7, 1300), (8, 0), (15, 3000)],
        vec![
            ((5,1000), vec![(6, 1000), (7, 1300), (8, 0), (15, 3000)]),
            ((6,1000), vec![(7, 1300), (8, 0), (15, 3000)]),
            ((7,1300), vec![(15, 3000)]),
            ((15,3000), vec![(16, 3000)]),
        ],
    )]
    #[case(
        vec![(34, 1000), (36, 500), (40, 0), (150, 3000), (155, 0)],
        vec![
            ((34,1000), vec![(35, 1000), (36, 500), (40, 0), (150, 3000), (155, 0)]),
            ((35, 1000), vec![(36, 500), (40, 0), (150, 3000), (155, 0)]),
            ((36, 500), vec![(37, 500), (40, 0), (150, 3000), (155, 0)]),
            ((37, 500), vec![(38, 500), (40, 0), (150, 3000), (155, 0)]),
            ((38, 500), vec![(39, 500), (40, 0), (150, 3000), (155, 0)]),
            ((39, 500), vec![(150, 3000), (155, 0)]),
            ((150, 3000), vec![(151, 3000), (155, 0)]),
            ((151, 3000), vec![(152, 3000), (155, 0)]),
            ((152, 3000), vec![(153, 3000), (155, 0)]),
            ((153, 3000), vec![(154, 3000), (155, 0)]),
            ((154, 3000), vec![]),
            ((0,0), vec![]),
            ((0,0), vec![]),
            ((0,0), vec![]),
        ],
    )]
    fn claim_test(
        #[case] initial_state: Vec<SimpleRoundBond>,
        #[case] action_result_state: Vec<(SimpleRoundBond, Vec<SimpleRoundBond>)>,
    ) {
        let stakes = create_round_bond_vec(initial_state);

        let mut s = StakeState::<Test>::new(stakes);

        for (i, (expected_result, expected_state)) in action_result_state.into_iter().enumerate() {
            let expected_state = create_round_bond_vec(expected_state);
            let expected_result = option_from_round_bond(expected_result.into());
            let result = s.claim();
            assert_eq!(result, expected_result, "expected_result is wrong in action {}", i);
            assert_eq!(s.stakes, expected_state, "expected_state is wrong in action {}", i);
        }
    }
}