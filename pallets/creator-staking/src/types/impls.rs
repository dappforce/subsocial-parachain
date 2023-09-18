use super::types::*;
use sp_runtime::traits::CheckedAdd;

impl<AccountId> CreatorInfo<AccountId> {
    /// Create new `CreatorInfo` struct instance with the given developer and state `Registered`
    pub(crate) fn new(stakeholder: AccountId) -> Self {
        Self { stakeholder, status: CreatorStatus::Active }
    }
}

impl<Balance, MaxUnlockingChunks> Default for StakerLedger<Balance, MaxUnlockingChunks>
    where
        Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
        MaxUnlockingChunks: Get<u32>,
{
    fn default() -> Self {
        Self {
            locked: Balance::default(),
            unbonding_info: UnbondingInfo::default(),
        }
    }
}

impl<Balance, MaxUnlockingChunks> StakerLedger<Balance, MaxUnlockingChunks>
    where
        Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
        MaxUnlockingChunks: Get<u32>,
{
    /// `true` if ledger is empty (no locked funds, no unbonding chunks), `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.locked.is_zero() && self.unbonding_info.is_empty()
    }
}

impl<Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen> EraStake<Balance> {
    /// Create a new instance of `EraStake` with given values
    pub(crate) fn new(staked: Balance, era: EraIndex) -> Self {
        Self { staked, era }
    }
}

impl<Balance, MaxEraStakeValues> Default for StakesInfo<Balance, MaxEraStakeValues>
    where
        Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen,
        MaxEraStakeValues: Get<u32>,
{
    fn default() -> Self {
        Self {
            stakes: BoundedVec::<EraStake<Balance>, MaxEraStakeValues>::default(),
        }
    }
}

impl<Balance, MaxEraStakeValues> StakesInfo<Balance, MaxEraStakeValues>
    where
        Balance: AtLeast32BitUnsigned + Copy + MaxEncodedLen + Debug,
        MaxEraStakeValues: Get<u32>,
{
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
    pub(crate) fn increase_stake(&mut self, current_era: EraIndex, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("CannotStakeInPastEra")
            }

            let new_stake_value = era_stake.staked.saturating_add(value);

            if current_era == era_stake.era {
                *era_stake = EraStake::new(new_stake_value, current_era)
            } else {
                self.stakes.try_push(EraStake::new(new_stake_value, current_era))
                    .expect("qed; too many stakes in StakesInfo");
            }
        } else {
            self.stakes.try_push(EraStake::new(value, current_era))
                .expect("qed; too many stakes in StakesInfo");
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
    pub(crate) fn unstake(&mut self, current_era: EraIndex, value: Balance) -> Result<(), &str> {
        if let Some(era_stake) = self.stakes.last_mut() {
            if era_stake.era > current_era {
                return Err("Unexpected era")
            }

            let new_stake_value = era_stake.staked.saturating_sub(value);
            if current_era == era_stake.era {
                *era_stake = EraStake::new(new_stake_value, current_era)
            } else {
                self.stakes.try_push(EraStake::new(new_stake_value, current_era))
                    .expect("qed; too many stakes in StakesInfo");
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
    pub(crate) fn claim(&mut self) -> (EraIndex, Balance) {
        if let Some(oldest_era_stake) = self.stakes.first() {
            let oldest_era_stake = *oldest_era_stake;
            let has_no_stake_updates_for_the_next_era =
                oldest_era_stake.era + 2 <= self.stakes[1].era;

            if self.stakes.len() == 1 || has_no_stake_updates_for_the_next_era {
                // If there is a record from the next era and its stake has been changed:
                self.stakes[0] =
                    EraStake { staked: oldest_era_stake.staked, era: oldest_era_stake.era.saturating_add(1) }
            } else {
                // in case: self.stakes[1].era == era_stake.era + 1
                self.stakes.remove(0);
            }

            // Removes unstaked values if they're no longer valid for comprehension
            if self.stakes[0].staked.is_zero() {
                self.stakes.remove(0);
            }

            (oldest_era_stake.era, oldest_era_stake.staked)
        } else {
            (0, Zero::zero())
        }
    }

    /// Latest staked value.
    /// E.g. if staker is fully unstaked, this will return `Zero`.
    /// Otherwise returns a non-zero balance.
    pub fn current_stake(&self) -> Balance {
        self.stakes.last().map_or(Zero::zero(), |x| x.staked)
    }
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

impl<Balance, MaxUnlockingChunks> Default for UnbondingInfo<Balance, MaxUnlockingChunks>
    where
        Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
        MaxUnlockingChunks: Get<u32>,
{
    fn default() -> Self {
        Self { unlocking_chunks: BoundedVec::<UnlockingChunk<Balance>, MaxUnlockingChunks>::default() }
    }
}

impl<Balance, MaxUnlockingChunks> UnbondingInfo<Balance, MaxUnlockingChunks>
    where
        Balance: AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Debug,
        MaxUnlockingChunks: Get<u32>,
{
    /// Returns total number of unlocking chunks.
    pub(crate) fn len(&self) -> u32 {
        self.unlocking_chunks.len() as u32
    }

    /// True if no unlocking chunks exist, false otherwise.
    fn is_empty(&self) -> bool {
        self.unlocking_chunks.is_empty()
    }

    /// Returns sum of all unlocking chunks.
    pub(crate) fn sum(&self) -> Balance {
        self.unlocking_chunks
            .iter()
            .map(|chunk| chunk.amount)
            .reduce(|c1, c2| c1 + c2)
            .unwrap_or_default()
    }

    /// Adds a new unlocking chunk to the vector, preserving the unlock_era based ordering.
    pub(crate) fn add(&mut self, chunk: UnlockingChunk<Balance>) {
        // It is possible that the unbonding period changes so we need to account for that
        match self.unlocking_chunks.binary_search_by(|x| x.unlock_era.cmp(&chunk.unlock_era)) {
            // Merge with existing chunk if unlock_eras match
            Ok(pos) => self.unlocking_chunks[pos].add_amount(chunk.amount),
            // Otherwise insert where it should go. Note that this will in almost all cases return
            // the last index.
            Err(pos) => self.unlocking_chunks.try_insert(pos, chunk)
                .expect("qed; too many chunks in UnbondingInfo"),
        }
    }

    /// Partitions the unlocking chunks into two groups:
    ///
    /// First group includes all chunks which have unlock era lesser or equal to the specified era.
    /// Second group includes all the rest.
    ///
    /// Order of chunks is preserved in the two new structs.
    pub(crate) fn partition(self, era: EraIndex) -> (Self, Self) {
        let (matching_chunks, other_chunks): (
            Vec<UnlockingChunk<Balance>>,
            Vec<UnlockingChunk<Balance>>,
        ) = self.unlocking_chunks.iter().partition(|chunk| chunk.unlock_era <= era);

        let matching_chunks = matching_chunks.try_into().unwrap();
        let other_chunks = other_chunks.try_into().unwrap();

        (Self { unlocking_chunks: matching_chunks }, Self { unlocking_chunks: other_chunks })
    }
}

impl Default for RewardsDistributionConfig {
    /// `default` values based on configuration at the time of writing this code.
    /// Should be overridden by desired params.
    fn default() -> Self {
        RewardsDistributionConfig {
            stakers_percent: Perbill::from_percent(45),
            creators_percent: Perbill::from_percent(45),
            treasury_percent: Perbill::from_percent(10),
        }
    }
}

impl RewardsDistributionConfig {
    /// `true` if sum of all percentages is `one whole`, `false` otherwise.
    pub fn is_consistent(&self) -> bool {
        let variables = vec![
            &self.stakers_percent,
            &self.creators_percent,
            &self.treasury_percent,
        ];
        let accumulator = variables
            .iter()
            .try_fold(Perbill::zero(), |acc, &percent| acc.checked_add(percent))
            .unwrap_or_default();

        Perbill::one() == accumulator
    }
}
