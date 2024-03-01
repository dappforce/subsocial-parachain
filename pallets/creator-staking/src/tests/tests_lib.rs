// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use crate::*;
use super::*;
use frame_support::assert_ok;
use mock::{Balance, MaxUnbondingChunks};
use sp_runtime::traits::Zero;
use crate::tests::mock::MaxEraStakeItems;

#[test]
fn unbonding_info_test() {
    let mut unbonding_info = UnbondingInfo::<Balance, MaxUnbondingChunks>::default();

    // assert basic ops on empty info
    assert!(unbonding_info.is_empty());
    assert!(unbonding_info.len().is_zero());
    let unbonding_chunks = unbonding_info.vec();
    let (first_info, second_info) = UnbondingInfo { unbonding_chunks }.partition(2);
    assert!(first_info.is_empty());
    assert!(second_info.is_empty());

    // Prepare unbonding chunks.
    let count = 5;
    let base_amount: Balance = 100;
    let base_unlock_era = 4 * count;
    let mut chunks = vec![];
    for x in 1_u32..=count as u32 {
        chunks.push(UnbondingChunk {
            amount: base_amount * x as Balance,
            unlock_era: base_unlock_era - 3 * x,
        });
    }

    // Add one unbonding chunk and verify basic ops.
    let _ = unbonding_info.add(chunks[0 as usize]);

    assert!(!unbonding_info.is_empty());
    assert_eq!(1, unbonding_info.len());
    assert_eq!(chunks[0 as usize].amount, unbonding_info.sum());

    let unbonding_chunks = unbonding_info.vec();
    let (first_info, second_info) = UnbondingInfo { unbonding_chunks }.partition(base_unlock_era);
    assert_eq!(1, first_info.len());
    assert_eq!(chunks[0 as usize].amount, first_info.sum());
    assert!(second_info.is_empty());

    // Add remainder and verify basic ops
    for x in unbonding_info.len() as usize..chunks.len() {
        let _ = unbonding_info.add(chunks[x]);
        // Ensure internal vec is sorted
        assert!(unbonding_info
            .vec()
            .windows(2)
            .all(|w| w[0].unlock_era <= w[1].unlock_era));
    }
    assert_eq!(chunks.len(), unbonding_info.len() as usize);
    let total: Balance = chunks.iter().map(|c| c.amount).sum();
    assert_eq!(total, unbonding_info.sum());

    let partition_era = chunks[2].unlock_era + 1;
    let unbonding_chunks = unbonding_info.vec();
    let (first_info, second_info) = UnbondingInfo { unbonding_chunks }.partition(partition_era);
    assert_eq!(3, first_info.len());
    assert_eq!(2, second_info.len());
    assert_eq!(unbonding_info.sum(), first_info.sum() + second_info.sum());
}

#[test]
fn stakes_info_basic() {
    let backer_stakes = StakesInfo::<Balance, MaxEraStakeItems>::default();

    assert!(backer_stakes.is_empty());
    assert_eq!(backer_stakes.len(), 0);
    assert_eq!(backer_stakes.current_stake(), 0);
}

#[test]
fn stakes_info_stake_ops() {
    let mut backer_stakes = StakesInfo::<Balance, MaxEraStakeItems>::default();

    // Do first stake and verify it
    let first_era = 1;
    let first_stake = 100;
    assert_ok!(backer_stakes.increase_stake(first_era, first_stake));
    assert!(!backer_stakes.is_empty());
    assert_eq!(backer_stakes.len(), 1);
    assert_eq!(backer_stakes.current_stake(), first_stake);

    // Do second stake and verify it
    let second_era = first_era + 1;
    let second_stake = 200;
    assert_ok!(backer_stakes.increase_stake(second_era, second_stake));
    assert_eq!(backer_stakes.len(), 2);
    assert_eq!(
        backer_stakes.current_stake(),
        first_stake + second_stake
    );

    // Do third stake and verify it
    let third_era = second_era + 2; // must be greater than 1 so a `hole` is present
    let third_stake = 333;
    assert_ok!(backer_stakes.increase_stake(third_era, third_stake));
    assert_eq!(
        backer_stakes.current_stake(),
        first_stake + second_stake + third_stake
    );
    assert_eq!(backer_stakes.len(), 3);

    // Do fourth stake and verify it
    let fourth_era = third_era; // ensure that multi-stake in same era works
    let fourth_stake = 444;
    assert_ok!(backer_stakes.increase_stake(fourth_era, fourth_stake));
    assert_eq!(backer_stakes.len(), 3);
    assert_eq!(
        backer_stakes.current_stake(),
        first_stake + second_stake + third_stake + fourth_stake
    );
}

#[test]
fn stakes_info_stake_error() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();
    assert_ok!(backer_stakes.increase_stake(5, 100));
    if let Err(_) = backer_stakes.increase_stake(4, 100) {
    } else {
        panic!("Mustn't be able to stake with past era.");
    }
}

#[test]
fn stakes_info_unstake_ops() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();

    // Unstake on empty backer_stakes
    assert!(backer_stakes.is_empty());
    assert_ok!(backer_stakes.decrease_stake(1, 100));
    assert!(backer_stakes.is_empty());

    // Prepare some stakes
    let (first_era, second_era) = (1, 3);
    let (first_stake, second_stake) = (110, 222);
    let total_staked = first_stake + second_stake;
    assert_ok!(backer_stakes.increase_stake(first_era, first_stake));
    assert_ok!(backer_stakes.increase_stake(second_era, second_stake));

    // Unstake an existing EraStake
    let first_unstake_era = second_era;
    let first_unstake = 55;
    assert_ok!(backer_stakes.decrease_stake(first_unstake_era, first_unstake));
    assert_eq!(backer_stakes.len(), 2);
    assert_eq!(
        backer_stakes.current_stake(),
        total_staked - first_unstake
    );
    let total_staked = total_staked - first_unstake;

    // Unstake an non-existing EraStake
    let second_unstake_era = first_unstake_era + 2;
    let second_unstake = 37;
    assert_ok!(backer_stakes.decrease_stake(second_unstake_era, second_unstake));
    assert_eq!(backer_stakes.len(), 3);
    assert_eq!(
        backer_stakes.current_stake(),
        total_staked - second_unstake
    );
    let total_staked = total_staked - second_unstake;

    // Save this for later
    let stakes = backer_stakes.stakes.clone();
    let temp_backer_stakes = StakesInfo { stakes, staked: backer_stakes.staked };

    // Fully unstake existing EraStake
    assert_ok!(backer_stakes.decrease_stake(second_unstake_era, total_staked));
    assert_eq!(backer_stakes.len(), 3);
    assert_eq!(backer_stakes.current_stake(), 0);

    // Fully unstake non-existing EraStake
    let mut backer_stakes = temp_backer_stakes; // restore
    assert_ok!(backer_stakes.decrease_stake(second_unstake_era + 1, total_staked));
    assert_eq!(backer_stakes.len(), 4);
    assert_eq!(backer_stakes.current_stake(), 0);
}

#[test]
fn stake_after_full_unstake() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();

    // Stake some amount
    let first_era = 1;
    let first_stake = 100;
    assert_ok!(backer_stakes.increase_stake(first_era, first_stake));
    assert_eq!(backer_stakes.current_stake(), first_stake);

    // Unstake all in next era
    let unstake_era = first_era + 1;
    assert_ok!(backer_stakes.decrease_stake(unstake_era, first_stake));
    assert!(backer_stakes.current_stake().is_zero());
    assert_eq!(backer_stakes.len(), 2);

    // Stake again in the next era
    let restake_era = unstake_era + 2;
    let restake_value = 57;
    assert_ok!(backer_stakes.increase_stake(restake_era, restake_value));
    assert_eq!(backer_stakes.current_stake(), restake_value);
    assert_eq!(backer_stakes.len(), 3);
}

#[test]
fn stakes_info_unstake_error() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();
    assert_ok!(backer_stakes.increase_stake(5, 100));
    if let Err(_) = backer_stakes.decrease_stake(4, 100) {
    } else {
        panic!("Mustn't be able to unstake with past era.");
    }
}

#[test]
fn stakes_info_claim_ops_basic() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();

    // Empty backer info
    assert!(backer_stakes.is_empty());
    assert_eq!(backer_stakes.claim(), (0, 0));
    assert!(backer_stakes.is_empty());

    // Only one unstaked exists
    assert_ok!(backer_stakes.increase_stake(1, 100));
    assert_ok!(backer_stakes.decrease_stake(1, 100));
    assert!(backer_stakes.is_empty());
    assert_eq!(backer_stakes.claim(), (0, 0));
    assert!(backer_stakes.is_empty());

    // Only one staked exists
    backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();
    let stake_era = 1;
    let stake_value = 123;
    assert_ok!(backer_stakes.increase_stake(stake_era, stake_value));
    assert_eq!(backer_stakes.len(), 1);
    assert_eq!(backer_stakes.claim(), (stake_era, stake_value));
    assert_eq!(backer_stakes.len(), 1);
}

#[test]
fn stakes_info_claim_ops_advanced() {
    let mut backer_stakes = StakesInfo::<Balance, MaxUnbondingChunks>::default();

    // Two consecutive eras staked, third era contains a gap with the second one
    let (first_stake_era, second_stake_era, third_stake_era) = (1, 2, 4);
    let (first_stake_value, second_stake_value, third_stake_value) = (123, 456, 789);

    assert_ok!(backer_stakes.increase_stake(first_stake_era, first_stake_value));
    assert_ok!(backer_stakes.increase_stake(second_stake_era, second_stake_value));
    assert_ok!(backer_stakes.increase_stake(third_stake_era, third_stake_value));

    // First claim
    assert_eq!(backer_stakes.len(), 3);
    assert_eq!(backer_stakes.claim(), (first_stake_era, first_stake_value));
    assert_eq!(backer_stakes.len(), 2);

    // Second claim
    assert_eq!(
        backer_stakes.claim(),
        (second_stake_era, first_stake_value + second_stake_value)
    );
    assert_eq!(backer_stakes.len(), 2);

    // Third claim, expect that 3rd era stake is the same as second
    assert_eq!(
        backer_stakes.claim(),
        (3, first_stake_value + second_stake_value)
    );
    assert_eq!(backer_stakes.len(), 1);

    // Fully unstake 5th era
    let total_staked = first_stake_value + second_stake_value + third_stake_value;
    assert_ok!(backer_stakes.decrease_stake(5, total_staked));
    assert_eq!(backer_stakes.len(), 2);

    // Stake 7th era (so after it was unstaked)
    let seventh_era = 7;
    let seventh_stake_value = 147;
    assert_ok!(backer_stakes.increase_stake(seventh_era, seventh_stake_value));
    assert_eq!(backer_stakes.len(), 3);

    // Claim 4th era
    assert_eq!(backer_stakes.claim(), (third_stake_era, total_staked));
    assert_eq!(backer_stakes.len(), 1);

    // Claim 7th era
    assert_eq!(backer_stakes.claim(), (seventh_era, seventh_stake_value));
    assert_eq!(backer_stakes.len(), 1);
    assert_eq!(backer_stakes.current_stake(), seventh_stake_value);

    // Claim future eras
    for x in 1..10 {
        assert_eq!(backer_stakes.claim(), (seventh_era + x, seventh_stake_value));
        assert_eq!(backer_stakes.len(), 1);
        assert_eq!(backer_stakes.current_stake(), seventh_stake_value);
    }
}
