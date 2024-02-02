// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use crate::{pallet::Event, *};
use super::*;
use frame_support::assert_ok;
use mock::{EraIndex, *};
use sp_runtime::traits::{AccountIdConversion, Zero};
use subsocial_support::SpaceId;
use crate::CreatorInfo;
use crate::tests::tests::Rewards;

/// Helper struct used to store information relevant to era/creator/backer combination.
pub(super) struct MemorySnapshot {
    pub(super) era_info: EraInfo<Balance>,
    pub(super) creator_info: CreatorInfo<AccountId>,
    pub(super) backer_stakes: StakesInfo<Balance, MaxEraStakeItems>,
    pub(super) creator_stakes_info: CreatorStakeInfo<Balance>,
    pub(super) free_balance: Balance,
    pub(super) backer_locks: BackerLocks<Balance, MaxUnbondingChunks>,
}

impl MemorySnapshot {
    /// Prepares a new `MemorySnapshot` struct based on the given arguments.
    pub(super) fn all(
        era: EraIndex,
        creator_id: SpaceId,
        account: AccountId,
    ) -> Self {
        Self {
            era_info: CreatorStaking::general_era_info(era).unwrap(),
            creator_info: RegisteredCreators::<TestRuntime>::get(creator_id).unwrap(),
            backer_stakes: BackerStakesByCreator::<TestRuntime>::get(&account, creator_id),
            creator_stakes_info: CreatorStaking::creator_stake_info(creator_id, era).unwrap_or_default(),
            backer_locks: CreatorStaking::backer_locks(&account),
            free_balance: <TestRuntime as Config>::Currency::free_balance(&account),
        }
    }

    /// Prepares a new `MemorySnapshot` struct but only with creator-related info
    /// (no info specific for individual backer).
    pub(super) fn creator(era: EraIndex, creator_id: SpaceId) -> Self {
        Self {
            era_info: CreatorStaking::general_era_info(era).unwrap(),
            creator_info: RegisteredCreators::<TestRuntime>::get(creator_id).unwrap(),
            backer_stakes: Default::default(),
            creator_stakes_info: CreatorStaking::creator_stake_info(creator_id, era).unwrap_or_default(),
            backer_locks: Default::default(),
            free_balance: Default::default(),
        }
    }
}

/// Used to fetch the free balance of creator stakingr rewards pot account
pub(super) fn free_balance_of_rewards_pot_account() -> Balance {
    <TestRuntime as Config>::Currency::free_balance(&account_id())
}

/// Used to fetch pallet account Id
pub(super) fn account_id() -> AccountId {
    <TestRuntime as Config>::PalletId::get().into_account_truncating()
}

/// Used to get total creators reward for an era.
pub(super) fn get_total_reward_per_era() -> Balance {
    Rewards::joint_block_reward() * BLOCKS_PER_ERA as Balance
}

/// Used to register creator for staking and assert success.
pub(super) fn assert_register(stakeholder: AccountId, creator_id: SpaceId) {
    let _m = use_static_mock();
    let space_owner_ctx = MockSpaces::get_space_owner_context();
    space_owner_ctx.expect().return_const(Ok(stakeholder)).times(1);

    let init_reserved_balance = <TestRuntime as Config>::Currency::reserved_balance(&stakeholder);

    // Creator shouldn't exist.
    assert!(!RegisteredCreators::<TestRuntime>::contains_key(creator_id));

    // Verify op is successful
    assert_ok!(CreatorStaking::force_register_creator(
        RuntimeOrigin::root(),
        creator_id.clone()
    ));

    let creator_info = RegisteredCreators::<TestRuntime>::get(creator_id).unwrap();
    assert_eq!(creator_info.status, CreatorStatus::Active);
    assert_eq!(creator_info.stakeholder, stakeholder);

    let final_reserved_balance = <TestRuntime as Config>::Currency::reserved_balance(&stakeholder);
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance + <TestRuntime as Config>::CreatorRegistrationDeposit::get()
    );
}

/// Perform `unregister` with all the accompanied checks including before/after storage comparison.
pub(super) fn assert_unregister(stakeholder: AccountId, creator_id: SpaceId) {
    let current_era = CreatorStaking::current_era();
    let init_state = MemorySnapshot::creator(current_era, creator_id);
    let init_reserved_balance = <TestRuntime as Config>::Currency::reserved_balance(&stakeholder);
    let registration_deposit = <TestRuntime as Config>::CreatorRegistrationDeposit::get();

    // creator should be registered prior to unregistering it
    assert_eq!(init_state.creator_info.status, CreatorStatus::Active);

    // Ensure that creator can be unregistered
    assert_ok!(CreatorStaking::force_unregister_creator(
        RuntimeOrigin::root(),
        creator_id
    ));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::CreatorUnregisteredWithSlash {
        creator_id,
        slash_amount: registration_deposit,
    }));

    let final_state = MemorySnapshot::creator(current_era, creator_id);
    let final_reserved_balance = <TestRuntime as Config>::Currency::reserved_balance(&stakeholder);
    assert_eq!(
        final_reserved_balance,
        init_reserved_balance - registration_deposit
    );

    assert_eq!(final_state.era_info.staked, init_state.era_info.staked);

    assert_eq!(
        final_state.creator_stakes_info.total_staked,
        init_state.creator_stakes_info.total_staked
    );
    assert_eq!(
        final_state.creator_stakes_info.backers_count,
        init_state.creator_stakes_info.backers_count
    );

    assert_eq!(
        final_state.creator_info.status,
        CreatorStatus::Inactive(current_era)
    );
    assert_eq!(final_state.creator_info.stakeholder, stakeholder);
}

/// Perform `withdraw_from_inactive_creator` with all the accompanied checks including before/after storage comparison.
pub(super) fn assert_withdraw_from_inactive_creator(
    backer: AccountId,
    creator_id: SpaceId,
) {
    let current_era = CreatorStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, creator_id, backer);

    // Initial checks
    if let CreatorStatus::Inactive(era) = init_state.creator_info.status {
        assert!(era <= CreatorStaking::current_era());
    } else {
        panic!("Creator should be unregistered.")
    };

    let staked_value = init_state.backer_stakes.current_stake();
    assert!(staked_value > 0);

    // Op with verification
    assert_ok!(CreatorStaking::withdraw_from_inactive_creator(
        RuntimeOrigin::signed(backer.clone()),
        creator_id
    ));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(
        Event::StakeWithdrawnFromInactiveCreator {
            who: backer.clone(),
            creator_id,
            amount: staked_value,
        },
    ));

    let final_state = MemorySnapshot::all(current_era, creator_id, backer);

    // Verify that all final states are as expected
    assert_eq!(
        init_state.era_info.staked,
        final_state.era_info.staked + staked_value
    );
    assert_eq!(
        init_state.era_info.locked,
        final_state.era_info.locked + staked_value
    );
    assert_eq!(init_state.creator_info, final_state.creator_info);
    assert_eq!(
        init_state.backer_locks.total_locked,
        final_state.backer_locks.total_locked + staked_value
    );
    assert_eq!(
        init_state.backer_locks.unbonding_info.vec(),
        final_state.backer_locks.unbonding_info.vec(),
    );

    assert!(final_state.backer_stakes.current_stake().is_zero());
    assert!(!BackerStakesByCreator::<TestRuntime>::contains_key(
        &backer,
        creator_id
    ));
}

/// Perform `bond_and_stake` with all the accompanied checks including before/after storage comparison.
pub(super) fn assert_stake(
    backer: AccountId,
    creator_id: SpaceId,
    value: Balance,
) {
    let current_era = CreatorStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, creator_id, backer);

    // Calculate the expected value that will be staked.
    let available_for_staking = init_state.free_balance
        - init_state.backer_locks.total_locked
        - <TestRuntime as Config>::MinimumRemainingFreeBalance::get();
    let staking_value = available_for_staking.min(value);

    // Perform op and verify everything is as expected
    assert_ok!(CreatorStaking::stake(
        RuntimeOrigin::signed(backer),
        creator_id,
        value,
    ));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::Staked {
        who: backer.clone(),
        creator_id,
        era: current_era,
        amount: staking_value,
    }));

    let final_state = MemorySnapshot::all(current_era, creator_id, backer);

    // In case backer hasn't been staking this creator until now
    if init_state.backer_stakes.current_stake() == 0 {
        assert!(BackerStakesByCreator::<TestRuntime>::contains_key(
            &backer,
            creator_id
        ));
        assert_eq!(
            final_state.creator_stakes_info.backers_count,
            init_state.creator_stakes_info.backers_count + 1
        );
    }

    // Verify the remaining states
    assert_eq!(
        final_state.era_info.staked,
        init_state.era_info.staked + staking_value
    );
    assert_eq!(
        final_state.era_info.locked,
        init_state.era_info.locked + staking_value
    );
    assert_eq!(
        final_state.creator_stakes_info.total_staked,
        init_state.creator_stakes_info.total_staked + staking_value
    );
    assert_eq!(
        final_state.backer_stakes.current_stake(),
        init_state.backer_stakes.current_stake() + staking_value
    );
    assert_eq!(
        final_state.backer_locks.total_locked,
        init_state.backer_locks.total_locked + staking_value
    );
}

/// Used to perform start_unbonding with success and storage assertions.
pub(super) fn assert_unstake(
    backer: AccountId,
    creator_id: SpaceId,
    value: Balance,
) {
    // Get latest staking info
    let current_era = CreatorStaking::current_era();
    let init_state = MemorySnapshot::all(current_era, creator_id, backer);

    // Calculate the expected resulting unbonding amount
    let remaining_staked = init_state
        .backer_stakes
        .current_stake()
        .saturating_sub(value);
    let expected_unbond_amount = if remaining_staked < MINIMUM_TOTAL_STAKE {
        init_state.backer_stakes.current_stake()
    } else {
        value
    };
    let remaining_staked = init_state.backer_stakes.current_stake() - expected_unbond_amount;

    // Ensure call executed successfully and event is emitted
    assert_ok!(CreatorStaking::unstake(
        RuntimeOrigin::signed(backer),
        creator_id,
        value
    ));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::Unstaked{
        who: backer.clone(),
        creator_id,
        era: current_era,
        amount: expected_unbond_amount,
    }));

    // Fetch the latest unbonding info so we can compare it to initial unbonding info
    let final_state = MemorySnapshot::all(current_era, creator_id, backer);
    let expected_unlock_era = current_era + UNBONDING_PERIOD_IN_ERAS;
    match init_state
        .backer_locks
        .unbonding_info
        .vec()
        .binary_search_by(|x| x.unlock_era.cmp(&expected_unlock_era))
    {
        Ok(_) => assert_eq!(
            init_state.backer_locks.unbonding_info.len(),
            final_state.backer_locks.unbonding_info.len()
        ),
        Err(_) => assert_eq!(
            init_state.backer_locks.unbonding_info.len() + 1,
            final_state.backer_locks.unbonding_info.len()
        ),
    }
    assert_eq!(
        init_state.backer_locks.unbonding_info.sum() + expected_unbond_amount,
        final_state.backer_locks.unbonding_info.sum()
    );

    // Push the unbonding chunk we expect to have at the end and compare two structs
    let unbonding_chunks = init_state.backer_locks.unbonding_info.vec();
    let mut unbonding_info = UnbondingInfo { unbonding_chunks };
    let _ = unbonding_info.add(UnbondingChunk {
        amount: expected_unbond_amount,
        unlock_era: current_era + UNBONDING_PERIOD_IN_ERAS,
    });
    assert_eq!(unbonding_info.vec(), final_state.backer_locks.unbonding_info.vec());

    // Ensure that total locked value for backer hasn't been changed.
    assert_eq!(init_state.backer_locks.total_locked, final_state.backer_locks.total_locked);
    if final_state.backer_locks.is_empty() {
        assert!(!BackerLocksByAccount::<TestRuntime>::contains_key(&backer));
    }

    // Ensure that total staked amount has been decreased for creator and staking points are updated
    assert_eq!(
        init_state.creator_stakes_info.total_staked - expected_unbond_amount,
        final_state.creator_stakes_info.total_staked
    );
    assert_eq!(
        init_state.backer_stakes.current_stake() - expected_unbond_amount,
        final_state.backer_stakes.current_stake()
    );

    // Ensure that the number of backers is as expected
    let delta = if remaining_staked > 0 { 0 } else { 1 };
    assert_eq!(
        init_state.creator_stakes_info.backers_count - delta,
        final_state.creator_stakes_info.backers_count
    );

    // Ensure that total staked value has been decreased
    assert_eq!(
        init_state.era_info.staked - expected_unbond_amount,
        final_state.era_info.staked
    );
    // Ensure that locked amount is the same since this will only start the unbonding period
    assert_eq!(init_state.era_info.locked, final_state.era_info.locked);
}

/// Used to perform start_unbonding with success and storage assertions.
pub(super) fn assert_withdraw_unbonded(backer: AccountId) {
    let current_era = CreatorStaking::current_era();

    let init_era_info = GeneralEraInfo::<TestRuntime>::get(current_era).unwrap();
    let init_backer_locks = BackerLocksByAccount::<TestRuntime>::get(&backer);

    // Get the current unbonding chunks
    let (valid_info, remaining_info) = init_backer_locks.unbonding_info.partition(current_era);
    let expected_unbond_amount = valid_info.sum();

    // Ensure call executed successfully and event is emitted
    assert_ok!(CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(
        backer
    ),));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::StakeWithdrawn {
        who: backer.clone(),
        amount: expected_unbond_amount,
    }));

    // Fetch the latest unbonding info so we can compare it to expected remainder
    let final_backer_locks = BackerLocksByAccount::<TestRuntime>::get(&backer);
    assert_eq!(remaining_info.vec(), final_backer_locks.unbonding_info.vec());
    if final_backer_locks.unbonding_info.is_empty() && final_backer_locks.total_locked == 0 {
        assert!(!BackerLocksByAccount::<TestRuntime>::contains_key(&backer));
    }

    // Compare the backer locks and total staked value
    let final_locks_and_stakes = GeneralEraInfo::<TestRuntime>::get(current_era).unwrap();
    assert_eq!(final_locks_and_stakes.staked, init_era_info.staked);
    assert_eq!(
        final_locks_and_stakes.locked,
        init_era_info.locked - expected_unbond_amount
    );
    assert_eq!(
        final_backer_locks.total_locked,
        init_backer_locks.total_locked - expected_unbond_amount
    );
}

/// Used to perform claim for backers with success assertion
pub(super) fn assert_claim_backer(claimer: AccountId, creator_id: SpaceId, restake: bool) {
    let (claim_era, _) = CreatorStaking::backer_stakes(&claimer, creator_id).claim();
    let current_era = CreatorStaking::current_era();

    //clean up possible leftover events
    System::reset_events();

    let init_state_claim_era = MemorySnapshot::all(claim_era, creator_id, claimer);
    let mut init_state_current_era = MemorySnapshot::all(current_era, creator_id, claimer);

    // Calculate backer's portion of the reward
    let (claim_era, staked) = init_state_claim_era.backer_stakes.clone().claim();
    assert!(claim_era > 0); // Sanity check - if this fails, method is being used incorrectly

    // Cannot claim rewards post unregister era, this indicates a bug!
    if let CreatorStatus::Inactive(unregistration_era) = init_state_claim_era.creator_info.status {
        assert!(unregistration_era > claim_era);
    }

    let calculated_reward = CreatorStaking::calculate_reward_for_backer_in_era(
        staked,
        claim_era,
    );
    let issuance_before_claim = <TestRuntime as Config>::Currency::total_issuance();

    assert_ok!(CreatorStaking::claim_backer_reward(
        RuntimeOrigin::signed(claimer),
        creator_id.clone(),
        restake,
    ));

    let final_state_current_era = MemorySnapshot::all(current_era, creator_id, claimer);

    // assert staked and free balances depending on restake check,
    assert_restake_reward(
        restake,
        current_era,
        &init_state_current_era,
        &final_state_current_era,
        calculated_reward,
    );

    // check for stake event if restaking is performed
    if CreatorStaking::ensure_can_restake_reward(
        restake,
        init_state_current_era.creator_info.status,
        &mut init_state_current_era.backer_stakes,
        current_era,
        calculated_reward,
    ).map_or(false, |should_restake| should_restake) {
        // There should be at least 2 events, Reward and BondAndStake.
        // if there's less, panic is acceptable
        let events = creator_staking_events();
        let second_last_event = &events[events.len() - 2];
        assert_eq!(
            second_last_event.clone(),
            Event::<TestRuntime>::Staked {
                who: claimer.clone(),
                creator_id,
                era: current_era,
                amount: calculated_reward,
            }
        );
    }

    // last event should be Reward, regardless of restaking
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::BackerRewardsClaimed {
        who: claimer.clone(),
        creator_id,
        amount: calculated_reward,
    }));

    let (new_era, _) = final_state_current_era.backer_stakes.clone().claim();
    if final_state_current_era.backer_stakes.is_empty() {
        assert!(new_era.is_zero());
        assert!(!BackerStakesByCreator::<TestRuntime>::contains_key(
            &claimer,
            creator_id
        ));
    } else {
        assert!(new_era > claim_era);
    }
    assert!(new_era.is_zero() || new_era > claim_era);

    // Claim shouldn't mint new tokens, instead it should just transfer from the creators staking pallet account
    let issuance_after_claim = <TestRuntime as Config>::Currency::total_issuance();
    assert_eq!(issuance_before_claim, issuance_after_claim);

    // Old `claim_era` creator info should never be changed
    let final_state_claim_era = MemorySnapshot::all(claim_era, creator_id, claimer);
    assert_eq!(
        init_state_claim_era.creator_stakes_info,
        final_state_claim_era.creator_stakes_info
    );
}

// assert staked and locked states depending on should_restake_reward
// returns should_restake_reward result so further checks can be made
fn assert_restake_reward(
    restake: bool,
    current_era: EraIndex,
    init_state_current_era: &MemorySnapshot,
    final_state_current_era: &MemorySnapshot,
    reward: Balance,
) {
    let mut init_backer_stakes = init_state_current_era.backer_stakes.clone();
    if CreatorStaking::ensure_can_restake_reward(
        restake,
        init_state_current_era.clone().creator_info.status,
        &mut init_backer_stakes,
        current_era,
        reward,
    ).map_or(false, |should_restake| should_restake) {
        // staked values should increase
        assert_eq!(
            init_state_current_era.backer_stakes.current_stake() + reward,
            final_state_current_era.backer_stakes.current_stake()
        );
        assert_eq!(
            init_state_current_era.era_info.staked + reward,
            final_state_current_era.era_info.staked
        );
        assert_eq!(
            init_state_current_era.era_info.locked + reward,
            final_state_current_era.era_info.locked
        );
        assert_eq!(
            init_state_current_era.creator_stakes_info.total_staked + reward,
            final_state_current_era.creator_stakes_info.total_staked
        );
    } else {
        // staked values should remain the same, and free balance increase
        assert_eq!(
            init_state_current_era.free_balance + reward,
            final_state_current_era.free_balance
        );
        assert_eq!(
            init_state_current_era.era_info.staked,
            final_state_current_era.era_info.staked
        );
        assert_eq!(
            init_state_current_era.era_info.locked,
            final_state_current_era.era_info.locked
        );
        assert_eq!(
            init_state_current_era.creator_stakes_info,
            final_state_current_era.creator_stakes_info
        );
    }
}

/// Used to perform claim for creator reward with success assertion
pub(super) fn assert_claim_creator(creator_id: SpaceId, claim_era: EraIndex) {
    let stakeholder = CreatorStaking::registered_creator(creator_id).unwrap().stakeholder;
    let init_state = MemorySnapshot::all(claim_era, creator_id, stakeholder);
    assert!(!init_state.creator_stakes_info.rewards_claimed);

    // Cannot claim rewards post unregister era
    if let CreatorStatus::Inactive(unregistration_era) = init_state.creator_info.status {
        assert!(unregistration_era > claim_era);
    }

    // Calculate creator portion of the reward
    let creator_reward =
        CreatorStaking::calculate_creator_reward(
            &init_state.creator_stakes_info, &init_state.era_info
        );

    assert_ok!(CreatorStaking::claim_creator_reward(
        RuntimeOrigin::signed(stakeholder),
        creator_id.clone(),
        claim_era,
    ));
    System::assert_last_event(mock::RuntimeEvent::CreatorStaking(Event::CreatorRewardsClaimed {
        who: stakeholder.clone(),
        amount: creator_reward,
    }));

    let final_state = MemorySnapshot::all(claim_era, creator_id, stakeholder);
    assert_eq!(
        init_state.free_balance + creator_reward,
        final_state.free_balance
    );

    assert!(final_state.creator_stakes_info.rewards_claimed);

    // Just in case creator is also a backer - this shouldn't cause any change in StakesInfo or BackerLocksByAccount
    assert_eq!(init_state.backer_stakes.stakes, final_state.backer_stakes.stakes);
    assert_eq!(init_state.backer_locks.total_locked, final_state.backer_locks.total_locked);
    assert_eq!(init_state.backer_locks.unbonding_info.vec(), final_state.backer_locks.unbonding_info.vec());
}

/// Used to perform move stake with success and storage assertions.
pub(crate) fn assert_move_stake(
    backer: AccountId,
    from_creator_id: CreatorId,
    to_creator_id: CreatorId,
    amount: Balance,
) {
    // Get latest staking info
    let current_era = CreatorStaking::current_era();
    let source_creator_init_state = MemorySnapshot::all(current_era, from_creator_id, backer);
    let target_creator_init_state = MemorySnapshot::all(current_era, to_creator_id, backer);

    // Calculate value which will actually be transferred
    let source_creator_init_stake_amount = source_creator_init_state.backer_stakes.current_stake();
    let expected_amount_to_move = if source_creator_init_stake_amount - amount > 0 {
        amount
    } else {
        source_creator_init_stake_amount
    };

    // Ensure op is successful and event is emitted
    assert_ok!(CreatorStaking::move_stake(
        RuntimeOrigin::signed(backer),
        from_creator_id,
        to_creator_id,
        amount,
    ));
    System::assert_last_event(RuntimeEvent::CreatorStaking(Event::StakeMoved {
        who: backer.clone(),
        from_creator_id,
        to_creator_id,
        amount: expected_amount_to_move,
    }));

    let source_creator_final_state = MemorySnapshot::all(current_era, from_creator_id, backer);
    let target_creator_final_state = MemorySnapshot::all(current_era, to_creator_id, backer);

    // Ensure backer stakes has increased/decreased staked amount
    assert_eq!(
        source_creator_final_state.backer_stakes.current_stake(),
        source_creator_init_stake_amount - expected_amount_to_move
    );
    assert_eq!(
        target_creator_final_state.backer_stakes.current_stake(),
        target_creator_init_state.backer_stakes.current_stake() + expected_amount_to_move
    );

    // Ensure total value staked on creators has appropriately increased/decreased
    assert_eq!(
        source_creator_final_state.creator_stakes_info.total_staked,
        source_creator_init_state.creator_stakes_info.total_staked - expected_amount_to_move
    );
    assert_eq!(
        target_creator_final_state.creator_stakes_info.total_staked,
        target_creator_init_state.creator_stakes_info.total_staked + expected_amount_to_move
    );

    // Ensure number of backers has been reduced on source creator if it is fully unstaked
    let is_source_creator_fully_unstaked = source_creator_init_stake_amount == expected_amount_to_move;
    if is_source_creator_fully_unstaked {
        assert_eq!(
            source_creator_final_state.creator_stakes_info.backers_count,
            source_creator_init_state.creator_stakes_info.backers_count - 1,
        );
    }

    // Ensure the number of backers has been increased on the target creator
    // if it is first stake by the backer
    let no_init_stake_on_target_creator = target_creator_init_state
        .backer_stakes
        .current_stake()
        .is_zero();
    if no_init_stake_on_target_creator {
        assert_eq!(
            target_creator_final_state.creator_stakes_info.backers_count,
            target_creator_init_state.creator_stakes_info.backers_count + 1
        );
    }

    // Ensure DB entry has been removed if era stake vector is empty
    let fully_unstaked_and_nothing_to_claim =
        is_source_creator_fully_unstaked && source_creator_final_state.backer_stakes.clone().claim() == (0, 0);
    if fully_unstaked_and_nothing_to_claim {
        assert!(!BackerStakesByCreator::<TestRuntime>::contains_key(
            &backer,
            &from_creator_id
        ));
    }
}
