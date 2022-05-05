use std::collections::BTreeMap;
use sp_std::ops::Add;
use frame_support::pallet_prelude::*;
use sp_std::prelude::*;
use sp_runtime::{Perbill, traits::CheckedAdd};
use frame_support::sp_std;
use sp_runtime::traits::Zero;
use crate::{BalanceOf, Config};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RewardSplitConfig {
    pub influencer_percentage: Perbill,
    pub stakers_percentage: Perbill,
}

impl RewardSplitConfig {
    pub fn new(influencer_percentage: Perbill, stakers_percentage: Perbill) -> Option<Self> {
        let config = RewardSplitConfig::new_unchecked(influencer_percentage, stakers_percentage);
        if config.is_valid() {
            Some(config)
        } else {
            None
        }
    }

    pub fn new_unchecked(influencer_percentage: Perbill, stakers_percentage: Perbill) -> Self {
        Self {
            influencer_percentage,
            stakers_percentage,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.influencer_percentage.checked_add(&self.stakers_percentage) {
            None => false,
            Some(x) if x != Perbill::one() => false,
            _ => true,
        }
    }
}

impl Default for RewardSplitConfig {
    fn default() -> Self {
        Self {
            influencer_percentage: Perbill::from_percent(50),
            stakers_percentage: Perbill::from_percent(50),
        }
    }
}


/// Type to index staking rounds.
pub(crate) type RoundIndex = u32;

/// The current round index and transition information
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Round<BlockNumber> {
    /// Current round index
    pub current: RoundIndex,
    /// The first block of the current round
    pub first: BlockNumber,
    /// The length of the current round in number of blocks
    pub length: u32,
}

impl<
    B: Copy + Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
> Default for Round<B>
{
    fn default() -> Round<B> {
        Round::new(1u32, 1u32.into(), 20u32)
    }
}

impl<
    B: Copy + Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
> Round<B>
{
    pub fn new(current: RoundIndex, first: B, length: u32) -> Round<B> {
        Round {
            current,
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
        self.current = self.current.saturating_add(1u32);
        self.first = now;
    }
}


#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct InfluencerInfo<T: Config> {
    /// Staker account
    pub id: T::AccountId,

    /// The deposit used for registration.
    pub deposit: BalanceOf<T>,

    /// The total number of stakers to this influencer.
    pub stakers_count: u32,

    /// The total amount of tokens staked to this influencer.
    pub staked_amount: BalanceOf<T>,
}

impl<T: Config> InfluencerInfo<T> {
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
pub struct StakerInfo<T: Config> {
    /// Staker account
    pub id: T::AccountId,

    /// Total balance that is locked. (active + unlocking)
    pub total: BalanceOf<T>,

    /// The total amount of balance that will be in stake in the next rounds.
    pub active: BalanceOf<T>,

    /// Amount of balance staked for each influencer.
    pub staked_per_influencer: BTreeMap<T::AccountId, BalanceOf<T>>,

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
            staked_per_influencer: Default::default(),
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