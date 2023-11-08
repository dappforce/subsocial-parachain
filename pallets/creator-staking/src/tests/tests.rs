// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use crate::{pallet::Error, pallet::Event, *};
use super::*;
use frame_support::{assert_noop, assert_ok, traits::{Currency, OnInitialize, OnTimestampSet}, weights::Weight};
use mock::{Balances, *};
use sp_runtime::{
    traits::{BadOrigin, Zero},
    Perbill, RuntimeDebug,
};

use testing_utils::*;

#[test]
fn on_initialize_when_creator_staking_enabled_in_mid_of_an_era_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        // Set a block number in mid of an era
        System::set_block_number(2);

        // Verify that current era is 0 since creator staking hasn't been initialized yet
        assert_eq!(0u32, CreatorStaking::current_era());

        // Call on initialize in the mid of an era (according to block number calculation)
        // but since no era was initialized before, it will trigger a new era init.
        CreatorStaking::on_initialize(System::block_number());
        assert_eq!(1u32, CreatorStaking::current_era());
    })
}

#[test]
fn rewards_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        // At the beginning, both should be 0
        assert_eq!(
            BlockRewardAccumulator::<TestRuntime>::get(),
            Default::default()
        );
        assert!(free_balance_of_rewards_pot_account().is_zero());

        // After handling imbalance, accumulator and account should be updated
        let creators_reward = 12345;
        let backers_reward = 9999;
        let total_reward = creators_reward + backers_reward;
        CreatorStaking::add_to_reward_pool(
            Balances::issue(backers_reward),
            Balances::issue(creators_reward),
        );

        assert_eq!(total_reward, free_balance_of_rewards_pot_account());
        let reward_accumulator = BlockRewardAccumulator::<TestRuntime>::get();
        assert_eq!(reward_accumulator.backers, backers_reward);
        assert_eq!(reward_accumulator.creators, creators_reward);

        // After triggering a new era, accumulator should be set to 0 but account shouldn't consume any new imbalance
        CreatorStaking::on_initialize(System::block_number());
        assert_eq!(
            BlockRewardAccumulator::<TestRuntime>::get(),
            Default::default()
        );
        assert_eq!(total_reward, free_balance_of_rewards_pot_account());
    })
}

#[test]
fn on_initialize_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        // Before we start, era is zero
        assert!(CreatorStaking::current_era().is_zero());

        // We initialize the first block and advance to second one. New era must be triggered.
        initialize_first_block();
        let current_era = CreatorStaking::current_era();
        assert_eq!(1, current_era);

        let previous_era = current_era;
        advance_to_era(previous_era + 10);

        // Check that all reward&stakes are as expected
        let current_era = CreatorStaking::current_era();
        for era in 1..current_era {
            let reward_info = GeneralEraInfo::<TestRuntime>::get(era).unwrap().rewards;
            assert_eq!(
                get_total_reward_per_era(),
                reward_info.backers + reward_info.creators
            );
        }
        // Current era rewards should be 0
        let era_rewards = GeneralEraInfo::<TestRuntime>::get(current_era).unwrap();
        assert_eq!(0, era_rewards.staked);
        assert_eq!(era_rewards.rewards, Default::default());
    })
}

#[test]
fn new_era_length_is_always_blocks_per_era() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        let blocks_per_era = BLOCKS_PER_ERA;

        // go to beginning of an era
        advance_to_era(CreatorStaking::current_era() + 1);

        // record era number and block number
        let start_era = CreatorStaking::current_era();
        let starting_block_number = System::block_number();

        // go to next era
        advance_to_era(CreatorStaking::current_era() + 1);
        let ending_block_number = System::block_number();

        // make sure block number difference is is blocks_per_era
        assert_eq!(CreatorStaking::current_era(), start_era + 1);
        assert_eq!(ending_block_number - starting_block_number, blocks_per_era);
    })
}

#[test]
fn new_era_is_handled_with_maintenance_mode() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        // enable maintenance mode
        assert_ok!(CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), true));
        assert!(PalletDisabled::<TestRuntime>::exists());
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::MaintenanceModeSet {
            enabled: true,
        }));

        // advance 9 blocks or 3 era lengths (advance_to_era() doesn't work in maintenance mode)
        run_for_blocks(BLOCKS_PER_ERA * 3);

        // verify that `current block > NextEraStartingBlock` but era hasn't changed
        assert!(System::block_number() > CreatorStaking::next_era_starting_block());
        assert_eq!(CreatorStaking::current_era(), 1);

        // disable maintenance mode
        assert_ok!(CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), false));
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::MaintenanceModeSet {
            enabled: false,
        }));

        // advance one era
        run_for_blocks(BLOCKS_PER_ERA);

        // verify we're at block 14
        assert_eq!(System::block_number(), (4 * BLOCKS_PER_ERA) + 2); // 2 from initialization, advanced 4 eras worth of blocks

        // verify era was updated and NextEraStartingBlock is 15
        assert_eq!(CreatorStaking::current_era(), 2);
        assert_eq!(
            CreatorStaking::next_era_starting_block(),
            (5 * BLOCKS_PER_ERA)
        );
    })
}

#[test]
fn new_forced_era_length_is_always_blocks_per_era() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        let blocks_per_era = BLOCKS_PER_ERA;

        // go to beginning of an era
        advance_to_era(CreatorStaking::current_era() + 1);

        // go to middle of era
        run_for_blocks(1); // can be any number between 0 and blocks_per_era

        // force new era
        <ForceEra<TestRuntime>>::put(Forcing::ForceNew);
        run_for_blocks(1); // calls on_initialize()

        // note the start block number of new (forced) era
        let start_block_number = System::block_number();

        // go to start of next era
        advance_to_era(CreatorStaking::current_era() + 1);

        // show the length of the forced era is equal to blocks_per_era
        let end_block_number = System::block_number();
        assert_eq!(end_block_number - start_block_number, blocks_per_era);
    })
}

#[test]
fn new_era_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        // set initial era index
        advance_to_era(CreatorStaking::current_era() + 10);
        let starting_era = CreatorStaking::current_era();

        // verify that block reward is zero at the beginning of an era
        assert_eq!(CreatorStaking::block_reward_accumulator(), Default::default());

        // Increment block by setting it to the first block in era value
        run_for_blocks(1);
        let current_era = CreatorStaking::current_era();
        assert_eq!(starting_era, current_era);

        // verify that block reward is added to the block_reward_accumulator
        let block_reward = CreatorStaking::block_reward_accumulator();
        assert_eq!(
            Rewards::joint_block_reward(),
            block_reward.backers + block_reward.creators
        );

        // register and bond to verify storage item
        let backer = 2;
        let stakeholder = 3;
        let staked_amount = 100;
        let creator = 1;

        assert_register(stakeholder, creator);
        assert_stake(backer, creator, staked_amount);

        // CurrentEra should be incremented
        // block_reward_accumulator should be reset to 0
        advance_to_era(CreatorStaking::current_era() + 1);

        let current_era = CreatorStaking::current_era();
        assert_eq!(starting_era + 1, current_era);
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::NewCreatorStakingEra {
            era: starting_era + 1,
        }));

        // verify that block reward accumulator is reset to 0
        let block_reward = CreatorStaking::block_reward_accumulator();
        assert_eq!(block_reward, Default::default());

        let Rewards { backers_reward: backer_reward, creators_reward, .. } =
            Rewards::calculate(&CreatorStaking::reward_config());

        let expected_era_reward = get_total_reward_per_era();
        let expected_creators_reward = creators_reward * BLOCKS_PER_ERA as Balance;
        let expected_backers_reward = backer_reward * BLOCKS_PER_ERA as Balance;

        // verify that .staked is copied and .reward is added
        let era_rewards = GeneralEraInfo::<TestRuntime>::get(starting_era).unwrap();
        assert_eq!(staked_amount, era_rewards.staked);
        assert_eq!(
            expected_era_reward,
            era_rewards.rewards.creators + era_rewards.rewards.backers
        );
        assert_eq!(expected_creators_reward, era_rewards.rewards.creators);
        assert_eq!(expected_backers_reward, era_rewards.rewards.backers);
    })
}

#[test]
fn new_era_forcing() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        advance_to_era(3);
        let starting_era = CreatorStaking::current_era();

        // call on_initialize. It is not last block in the era, but it should increment the era
        <ForceEra<TestRuntime>>::put(Forcing::ForceNew);
        run_for_blocks(1);

        // check that era is incremented
        let current = CreatorStaking::current_era();
        assert_eq!(starting_era + 1, current);

        // check that forcing is cleared
        assert_eq!(CreatorStaking::force_era(), Forcing::NotForcing);

        // check the event for the new era
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::NewCreatorStakingEra {
            era: starting_era + 1,
        }));
    })
}

#[test]
fn general_backer_stakes_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let first_stakeholder = 10;
        let first_creator_id = 1;

        assert_register(first_stakeholder, first_creator_id);

        let second_stakeholder = 11;
        let second_creator_id = 2;

        assert_register(second_stakeholder, second_creator_id);

        let (backer_1, backer_2, backer_3) = (1, 2, 3);
        let amount = 100;

        let starting_era = 3;
        advance_to_era(starting_era);
        assert_stake(backer_1, first_creator_id, amount);
        assert_stake(backer_2, first_creator_id, amount);

        let mid_era = 7;
        advance_to_era(mid_era);
        assert_unstake(backer_2, first_creator_id, amount);
        assert_stake(backer_3, first_creator_id, amount);
        assert_stake(backer_3, second_creator_id, amount);

        let final_era = 12;
        advance_to_era(final_era);

        // Check first interval
        let mut first_backer_stakes = CreatorStaking::backer_stakes(&backer_1, &first_creator_id);
        let mut second_backer_stakes = CreatorStaking::backer_stakes(&backer_2, &first_creator_id);
        let mut third_backer_stakes = CreatorStaking::backer_stakes(&backer_3, &first_creator_id);

        for era in starting_era..mid_era {
            let creator_info = CreatorStaking::creator_stake_info(&first_creator_id, era).unwrap();
            assert_eq!(2, creator_info.backers_count);

            assert_eq!((era, amount), first_backer_stakes.claim());
            assert_eq!((era, amount), second_backer_stakes.claim());

            assert!(!CreatorStakeInfoByEra::<TestRuntime>::contains_key(
                &second_creator_id,
                era
            ));
        }

        // Check second interval
        for era in mid_era..=final_era {
            let first_creator_info =
                CreatorStaking::creator_stake_info(&first_creator_id, era).unwrap();
            assert_eq!(2, first_creator_info.backers_count);

            assert_eq!((era, amount), first_backer_stakes.claim());
            assert_eq!((era, amount), third_backer_stakes.claim());

            assert_eq!(
                CreatorStaking::creator_stake_info(&second_creator_id, era)
                    .unwrap()
                    .backers_count,
                1
            );
        }

        // Check that before starting era nothing exists
        assert!(!CreatorStakeInfoByEra::<TestRuntime>::contains_key(
            &first_creator_id,
            starting_era - 1
        ));
        assert!(!CreatorStakeInfoByEra::<TestRuntime>::contains_key(
            &second_creator_id,
            starting_era - 1
        ));
    })
}

#[test]
fn register_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let ok_creator = 1;

        assert!(<TestRuntime as Config>::Currency::reserved_balance(&stakeholder).is_zero());
        assert_register(stakeholder, ok_creator);
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::CreatorRegistered {
            who: stakeholder.clone(),
            creator_id: ok_creator,
        }));

        assert_eq!(
            CreatorRegistrationDeposit::get(),
            <TestRuntime as Config>::Currency::reserved_balance(&stakeholder)
        );
    })
}

#[test]
fn register_with_non_root_fails() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let ok_creator = 1;

        assert_noop!(
            CreatorStaking::force_register_creator(RuntimeOrigin::signed(stakeholder), ok_creator),
            BadOrigin
        );
    })
}

#[test]
fn register_same_creator_twice_fails() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);

        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::CreatorRegistered {
            who: stakeholder,
            creator_id,
        }));

        // now register same creator by different stakeholder
        assert_noop!(
            CreatorStaking::force_register_creator(RuntimeOrigin::root(), creator_id),
            Error::<TestRuntime>::CreatorAlreadyRegistered
        );
    })
}

#[test]
fn force_unregister_after_register_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);
        assert_unregister(stakeholder, creator_id);
        assert!(<TestRuntime as Config>::Currency::reserved_balance(&stakeholder).is_zero());

        // Not possible to unregister a creator twice
        assert_noop!(
            CreatorStaking::force_unregister_creator(RuntimeOrigin::root(), creator_id.clone()),
            Error::<TestRuntime>::InactiveCreator
        );
    })
}

#[test]
fn force_unregister_with_non_root() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);

        // Not possible to unregister if caller isn't root
        assert_noop!(
            CreatorStaking::force_unregister_creator(RuntimeOrigin::signed(stakeholder), creator_id.clone()),
            BadOrigin
        );
    })
}

#[test]
fn unregister_stake_and_unstake_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        // Register creator, stake it, unstake a bit
        assert_register(stakeholder, creator_id);
        assert_stake(backer, creator_id, 100);
        assert_unstake(backer, creator_id, 10);

        // Unregister creator and verify that stake & unstake no longer work
        assert_unregister(stakeholder, creator_id);

        assert_noop!(
            CreatorStaking::stake(RuntimeOrigin::signed(backer), creator_id.clone(), 100),
            Error::<TestRuntime>::InactiveCreator,
        );
        assert_noop!(
            CreatorStaking::unstake(
                RuntimeOrigin::signed(backer),
                creator_id.clone(),
                100
            ),
            Error::<TestRuntime>::InactiveCreator,
        );
    })
}

#[test]
fn withdraw_from_inactive_creator_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let dummy_stakeholder = 2;
        let backer_1 = 3;
        let backer_2 = 4;
        let staked_value_1 = 150;
        let staked_value_2 = 330;
        let creator_id = 1;
        let dummy_creator_id = 5;

        // Register both creators and stake them
        assert_register(stakeholder, creator_id);
        assert_register(dummy_stakeholder, dummy_creator_id);
        assert_stake(backer_1, creator_id, staked_value_1);
        assert_stake(backer_2, creator_id, staked_value_2);

        // This creator will just exist so it helps us with testing BackerLocksByAccount content
        assert_stake(backer_1, dummy_creator_id, staked_value_1);

        // Advance eras. This will accumulate some rewards.
        advance_to_era(5);

        assert_unregister(stakeholder, creator_id);

        // Claim all past rewards
        for era in 1..CreatorStaking::current_era() {
            assert_claim_backer(backer_1, creator_id, false);
            assert_claim_backer(backer_2, creator_id, false);
            assert_claim_creator(creator_id, era);
        }

        // Unbond everything from the creator.
        assert_withdraw_from_inactive_creator(backer_1, creator_id);
        assert_withdraw_from_inactive_creator(backer_2, creator_id);

        // No additional claim ops should be possible
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer_1), creator_id.clone(), false),
            Error::<TestRuntime>::NotStakedCreator
        );
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer_2), creator_id.clone(), false),
            Error::<TestRuntime>::NotStakedCreator
        );
        assert_noop!(
            CreatorStaking::claim_creator_reward(
                RuntimeOrigin::signed(stakeholder),
                creator_id.clone(),
                CreatorStaking::current_era()
            ),
            Error::<TestRuntime>::InactiveCreator,
        );
    })
}

#[test]
fn withdraw_from_inactive_creator_when_creator_doesnt_exist() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_noop!(
            CreatorStaking::withdraw_from_inactive_creator(RuntimeOrigin::signed(1), creator_id),
            Error::<TestRuntime>::CreatorNotFound,
        );
    })
}

#[test]
fn withdraw_from_inactive_creator_when_creator_is_still_registered() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;
        assert_register(stakeholder, creator_id);

        assert_noop!(
            CreatorStaking::withdraw_from_inactive_creator(RuntimeOrigin::signed(1), creator_id),
            Error::<TestRuntime>::CreatorIsActive
        );
    })
}

#[test]
fn withdraw_from_inactive_creator_when_nothing_is_staked() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;
        assert_register(stakeholder, creator_id);

        let backer = 2;
        let not_a_backer = 3;
        assert_stake(backer, creator_id, 100);

        assert_unregister(stakeholder, creator_id);

        // No staked amount so call should fail.
        assert_noop!(
            CreatorStaking::withdraw_from_inactive_creator(RuntimeOrigin::signed(not_a_backer), creator_id),
            Error::<TestRuntime>::NotStakedCreator
        );

        // Call should fail if called twice since no staked funds remain.
        assert_withdraw_from_inactive_creator(backer, creator_id);
        assert_noop!(
            CreatorStaking::withdraw_from_inactive_creator(RuntimeOrigin::signed(backer), creator_id),
            Error::<TestRuntime>::NotStakedCreator
        );
    })
}

#[test]
fn withdraw_from_inactive_creator_when_unclaimed_rewards_remaining() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let creator_id = 1;
        assert_register(stakeholder, creator_id);

        let backer = 2;
        assert_stake(backer, creator_id, 100);

        // Advance eras. This will accumulate some rewards.
        advance_to_era(5);

        assert_unregister(stakeholder, creator_id);

        for _ in 1..CreatorStaking::current_era() {
            assert_noop!(
                CreatorStaking::withdraw_from_inactive_creator(
                    RuntimeOrigin::signed(backer),
                    creator_id
                ),
                Error::<TestRuntime>::UnclaimedRewardsRemaining
            );
            assert_claim_backer(backer, creator_id, false);
        }

        // Withdraw should work after all rewards have been claimed
        assert_withdraw_from_inactive_creator(backer, creator_id);
    })
}

#[test]
fn stake_different_eras_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 20;
        let backer_id = 1;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);

        // initially, storage values should be None
        let current_era = CreatorStaking::current_era();
        assert!(CreatorStaking::creator_stake_info(creator_id, current_era).is_none());

        assert_stake(backer_id, creator_id, 100);

        advance_to_era(current_era + 2);

        // Stake and bond again on the same creator but using a different amount.
        assert_stake(backer_id, creator_id, 300);
    })
}

#[test]
fn stake_two_different_creators_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let first_stakeholder = 5;
        let second_stakeholder = 6;
        let backer_id = 1;
        let first_creator_id = 1;
        let second_creator_id = 2;

        // Insert creators under registered creators. Don't use the backer Id.
        assert_register(first_stakeholder, first_creator_id);
        assert_register(second_stakeholder, second_creator_id);

        // Stake on both creators.
        assert_stake(backer_id, first_creator_id, 100);
        assert_stake(backer_id, second_creator_id, 300);
    })
}

#[test]
fn stake_two_backers_one_creator_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let first_backer_id = 1;
        let second_backer_id = 2;
        let first_stake_value = 50;
        let second_stake_value = 235;
        let creator_id = 1;

        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // Both backers stake on the same creator, expect a pass.
        assert_stake(first_backer_id, creator_id, first_stake_value);
        assert_stake(second_backer_id, creator_id, second_stake_value);
    })
}

#[test]
fn stake_different_value_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 20;
        let backer_id = 1;
        let creator_id = 1;

        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // stake almost the entire available balance of the backer.
        let backer_free_balance =
            Balances::free_balance(&backer_id).saturating_sub(MINIMUM_REMAINING_AMOUNT);
        assert_stake(backer_id, creator_id, backer_free_balance - 1);

        // stake again with less than existential deposit but this time expect a pass
        // since we're only increasing the already staked amount.
        assert_stake(backer_id, creator_id, 1);

        // stake more than what's available in funds. Verify that only what's available is staked.
        let backer_id = 2;
        let backer_free_balance = Balances::free_balance(&backer_id);
        assert_stake(backer_id, creator_id, backer_free_balance + 1);

        // Verify the minimum transferable amount of backers account
        let transferable_balance =
            Balances::free_balance(&backer_id) - BackerLocksByAccount::<TestRuntime>::get(backer_id).total_locked;
        assert_eq!(MINIMUM_REMAINING_AMOUNT, transferable_balance);

        // stake some amount, a bit less than free balance
        let backer_id = 3;
        let backer_free_balance =
            Balances::free_balance(&backer_id).saturating_sub(MINIMUM_REMAINING_AMOUNT);
        assert_stake(backer_id, creator_id, backer_free_balance - 200);

        // Try to stake more than we have available (since we already locked most of the free balance).
        // This doesn't fail, because in any case we calculate available_balance
        // and then get the minimum amount between available and requested.
        assert_stake(backer_id, creator_id, 500);
    })
}

#[test]
fn stake_on_inactive_creator_fails() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let backer_id = 1;
        let stake_value = 100;

        // Check not registered creator. Expect an error.
        let creator = 1;
        assert_noop!(
            CreatorStaking::stake(
                RuntimeOrigin::signed(backer_id),
                creator,
                stake_value
            ),
            Error::<TestRuntime>::InactiveCreator,
        );
    })
}

#[test]
fn stake_insufficient_value() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();
        let stakeholder = 20;
        let backer_id = 1;
        let creator_id = 1;

        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // If user tries to make an initial stake with less than minimum amount, raise an error.
        assert_noop!(
            CreatorStaking::stake(
                RuntimeOrigin::signed(backer_id),
                creator_id.clone(),
                MINIMUM_STAKING_AMOUNT - 1
            ),
            Error::<TestRuntime>::InsufficientStakingAmount
        );

        // Now stake the entire stash so we lock all the available funds.
        let backer_free_balance = Balances::free_balance(&backer_id);
        assert_stake(backer_id, creator_id, backer_free_balance);

        // Now try to stake some additional funds and expect an error since we cannot stake 0.
        assert_noop!(
            CreatorStaking::stake(RuntimeOrigin::signed(backer_id), creator_id.clone(), 1),
            Error::<TestRuntime>::CannotStakeZero
        );
    })
}

#[test]
fn stake_too_many_backers_per_creator() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let creator_id = 1;
        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // Stake with MAX_NUMBER_OF_BACKERS on the same creator. It must work.
        for backer_id in 1..=MAX_NUMBER_OF_BACKERS {
            assert_stake(backer_id.into(), creator_id, 100);
        }

        // Now try to stake with an additional backer and expect an error.
        assert_noop!(
            CreatorStaking::stake(
                RuntimeOrigin::signed((1 + MAX_NUMBER_OF_BACKERS).into()),
                creator_id.clone(),
                100
            ),
            Error::<TestRuntime>::MaxNumberOfBackersExceeded
        );
    })
}

#[test]
fn stake_too_many_era_stakes() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let backer_id = 1;
        let creator_id = 1;
        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // Stake with MAX_NUMBER_OF_BACKERS - 1 on the same creator. It must work.
        let start_era = CreatorStaking::current_era();
        for offset in 1..MAX_ERA_STAKE_ITEMS {
            assert_stake(backer_id, creator_id, 100);
            advance_to_era(start_era + offset);
        }

        // Now try to stake with an additional backer and expect an error.
        assert_noop!(
            CreatorStaking::stake(RuntimeOrigin::signed(backer_id), creator_id, 100),
            Error::<TestRuntime>::TooManyEraStakeValues
        );
    })
}

#[test]
fn unstake_multiple_time_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let backer_id = 1;
        let creator_id = 1;
        let original_staked_value = 300 + MINIMUM_STAKING_AMOUNT;
        let old_era = CreatorStaking::current_era();

        // Insert a creator under registered creators, stake it.
        assert_register(stakeholder, creator_id);
        assert_stake(backer_id, creator_id, original_staked_value);
        advance_to_era(old_era + 1);

        // Unstake such an amount so there will remain staked funds on the creator
        let unstaked_value = 100;
        assert_unstake(backer_id, creator_id, unstaked_value);

        // Unbond yet again, but don't advance era
        // Unstake such an amount so there will remain staked funds on the creator
        let unstaked_value = 50;
        assert_unstake(backer_id, creator_id, unstaked_value);
    })
}

#[test]
fn unstake_value_below_staking_threshold() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let backer_id = 1;
        let creator_id = 1;
        let first_value_to_unstake = 300;
        let staked_value = first_value_to_unstake + MINIMUM_STAKING_AMOUNT;

        // Insert a creator under registered creators, stake it.
        assert_register(stakeholder, creator_id);
        assert_stake(backer_id, creator_id, staked_value);

        // Unstake such an amount that exactly minimum staking amount will remain staked.
        assert_unstake(backer_id, creator_id, first_value_to_unstake);

        // Unstake 1 token and expect that the entire staked amount will be unstaked.
        assert_unstake(backer_id, creator_id, 1);
    })
}

#[test]
fn unstake_in_different_eras() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let (first_backer_id, second_backer_id) = (1, 2);
        let creator_id = 1;
        let staked_value = 500;

        // Insert a creator under registered creators, stake it with two different backers.
        assert_register(10, creator_id);
        assert_stake(first_backer_id, creator_id, staked_value);
        assert_stake(second_backer_id, creator_id, staked_value);

        // Advance era, unstake & withdraw with first backer, verify that it was successful
        advance_to_era(CreatorStaking::current_era() + 10);
        let current_era = CreatorStaking::current_era();
        assert_unstake(first_backer_id, creator_id, 100);

        // Advance era, unbond with second backer and verify storage values are as expected
        advance_to_era(current_era + 10);
        assert_unstake(second_backer_id, creator_id, 333);
    })
}

#[test]
fn unstake_calls_in_same_era_can_exceed_max_chunks() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_register(10, creator_id);

        let backer = 1;
        assert_stake(backer, creator_id, 200 * MAX_UNBONDING_CHUNKS as Balance);

        // Ensure that we can unbond up to a limited amount of time.
        for _ in 0..MAX_UNBONDING_CHUNKS * 2 {
            assert_unstake(1, creator_id, 10);
            assert_eq!(1, BackerLocksByAccount::<TestRuntime>::get(&backer).unbonding_info.len());
        }
    })
}

#[test]
fn unstake_with_zero_value_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_register(10, creator_id);

        assert_noop!(
            CreatorStaking::unstake(RuntimeOrigin::signed(1), creator_id, 0),
            Error::<TestRuntime>::CannotUnstakeZero
        );
    })
}

#[test]
fn unstake_on_not_operated_creator_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_noop!(
            CreatorStaking::unstake(RuntimeOrigin::signed(1), creator_id, 100),
            Error::<TestRuntime>::InactiveCreator
        );
    })
}

#[test]
fn unstake_too_many_unbonding_chunks_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_register(10, creator_id);

        let backer = 1;
        let unstake_amount = 10;
        let stake_amount =
            MINIMUM_STAKING_AMOUNT * 10 + unstake_amount * MAX_UNBONDING_CHUNKS as Balance;

        assert_stake(backer, creator_id, stake_amount);

        // Ensure that we can unbond up to a limited amount of time.
        for _ in 0..MAX_UNBONDING_CHUNKS {
            advance_to_era(CreatorStaking::current_era() + 1);
            assert_unstake(backer, creator_id, unstake_amount);
        }

        // Ensure that we're at the max but can still add new chunks since it should be merged with the existing one
        assert_eq!(
            MAX_UNBONDING_CHUNKS,
            CreatorStaking::backer_locks(&backer).unbonding_info.len()
        );
        assert_unstake(backer, creator_id, unstake_amount);

        // Ensure that further unbonding attempts result in an error.
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_noop!(
            CreatorStaking::unstake(
                RuntimeOrigin::signed(backer),
                creator_id.clone(),
                unstake_amount
            ),
            Error::<TestRuntime>::TooManyUnbondingChunks,
        );
    })
}

#[test]
fn unstake_on_not_staked_creator_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let creator_id = 1;
        assert_register(10, creator_id);

        assert_noop!(
            CreatorStaking::unstake(RuntimeOrigin::signed(1), creator_id, 10),
            Error::<TestRuntime>::NotStakedCreator,
        );
    })
}

#[test]
fn unstake_should_fail_when_too_many_era_stakes() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let backer_id = 1;
        let creator_id = 1;
        assert_register(10, creator_id);

        // Fill up the `EraStakes` vec
        let start_era = CreatorStaking::current_era();
        for offset in 1..MAX_ERA_STAKE_ITEMS {
            assert_stake(backer_id, creator_id, 100);
            advance_to_era(start_era + offset);
        }

        // At this point, we have max allowed amount of `EraStake` values so we cannot create
        // an additional one.
        assert_noop!(
            CreatorStaking::unstake(RuntimeOrigin::signed(backer_id), creator_id, 10),
            Error::<TestRuntime>::TooManyEraStakeValues
        );
    })
}

// Works, but requires mock modification.
#[ignore]
#[test]
fn unstake_with_no_chunks_allowed() {
    // UT can be used to verify situation when MaxUnbondingChunks = 0.
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        // Sanity check
        assert_eq!(<TestRuntime as Config>::MaxUnbondingChunks::get(), 0);

        let creator_id = 1;
        assert_register(10, creator_id);

        let backer_id = 1;
        assert_stake(backer_id, creator_id, 100);

        assert_noop!(
            CreatorStaking::unstake(
                RuntimeOrigin::signed(backer_id),
                creator_id.clone(),
                20
            ),
            Error::<TestRuntime>::TooManyUnbondingChunks,
        );
    })
}

#[test]
fn withdraw_unbonded_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);

        let backer_account = 1;
        assert_stake(backer_account, creator_id, 1000);

        let first_unstake_value = 75;
        let second_unstake_value = 39;
        let initial_era = CreatorStaking::current_era();

        // Unbond some amount in the initial era
        assert_unstake(backer_account, creator_id, first_unstake_value);

        // Advance one era and then unbond some more
        advance_to_era(initial_era + 1);
        assert_unstake(backer_account, creator_id, second_unstake_value);

        // Now advance one era before first chunks finishes the unbonding process
        advance_to_era(initial_era + UNBONDING_PERIOD_IN_ERAS - 1);
        assert_noop!(
            CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(backer_account)),
            Error::<TestRuntime>::NothingToWithdraw
        );

        // Advance one additional era and expect that the first chunk can be withdrawn
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_ok!(CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(
            backer_account
        ),));
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::StakeWithdrawn {
            who: backer_account,
            amount: first_unstake_value,
        }));

        // Advance one additional era and expect that the first chunk can be withdrawn
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_ok!(CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(
            backer_account
        ),));
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::StakeWithdrawn {
            who: backer_account,
            amount: second_unstake_value,
        }));

        // Advance one additional era but since we have nothing else to withdraw, expect an error
        advance_to_era(initial_era + UNBONDING_PERIOD_IN_ERAS - 1);
        assert_noop!(
            CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(backer_account)),
            Error::<TestRuntime>::NothingToWithdraw
        );
    })
}

fn withdraw_unbonded_full_vector(stakeholder: AccountId, creator_id: CreatorId, backer_id: AccountId) {
    initialize_first_block();

    assert_register(stakeholder, creator_id);
    assert_stake(backer_id, creator_id, 1000);

    // Repeatedly start unbonding and advance era to create unbonding chunks
    let init_unbonding_amount = 15;
    for x in 1..=MAX_UNBONDING_CHUNKS {
        assert_unstake(backer_id, creator_id, init_unbonding_amount * x as u128);
        advance_to_era(CreatorStaking::current_era() + 1);
    }

    // Now clean up all that are eligible for clean-up
    assert_withdraw_unbonded(backer_id);

    // This is a sanity check for the test. Some chunks should remain, otherwise test isn't testing realistic unbonding period.
    assert!(!BackerLocksByAccount::<TestRuntime>::get(&backer_id)
        .unbonding_info
        .is_empty());
}

#[test]
fn withdraw_unbonded_full_vector_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        let stakeholder = 10;
        let creator_id = 1;
        let backer_id = 1;

        withdraw_unbonded_full_vector(stakeholder, creator_id, backer_id);

        while !BackerLocksByAccount::<TestRuntime>::get(&backer_id)
            .unbonding_info
            .is_empty()
        {
            advance_to_era(CreatorStaking::current_era() + 1);
            assert_withdraw_unbonded(backer_id);
        }
    })
}

#[test]
fn withdraw_unbonded_full_vector_at_once_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        let stakeholder = 10;
        let creator_id = 1;
        let backer_id = 1;

        withdraw_unbonded_full_vector(stakeholder, creator_id, backer_id);

        let unbonding_chunks = BackerLocksByAccount::<TestRuntime>::get(&backer_id)
            .unbonding_info.vec().len() as u32;

        advance_to_era(CreatorStaking::current_era() + unbonding_chunks + 1);
        assert_withdraw_unbonded(backer_id);
    })
}

#[test]
fn withdraw_unbonded_no_value_is_not_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_noop!(
            CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(1)),
            Error::<TestRuntime>::NothingToWithdraw,
        );
    })
}

// Works, but requires mock modification.
#[ignore]
#[test]
fn withdraw_unbonded_no_unbonding_period() {
    // UT can be used to verify situation when UnbondingPeriodInEras = 0.
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        // Sanity check
        assert_eq!(<TestRuntime as Config>::UnbondingPeriodInEras::get(), 0);

        let creator_id = 1;
        assert_register(10, creator_id);

        let backer_id = 1;
        assert_stake(backer_id, creator_id, 100);
        assert_unstake(backer_id, creator_id, 20);

        // Try to withdraw but expect an error since current era hasn't passed yet
        assert_noop!(
            CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(backer_id)),
            Error::<TestRuntime>::NothingToWithdraw,
        );

        // Advance an era and expect successful withdrawal
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_withdraw_unbonded(backer_id);
    })
}

#[test]
fn claim_not_staked_creator() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);

        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer), creator_id, false),
            Error::<TestRuntime>::NotStakedCreator
        );

        advance_to_era(CreatorStaking::current_era() + 1);
        assert_noop!(
            CreatorStaking::claim_creator_reward(RuntimeOrigin::signed(stakeholder), creator_id, 1),
            Error::<TestRuntime>::NotStakedCreator
        );
    })
}

#[test]
fn claim_inactive_creator() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        assert_register(stakeholder, creator_id);
        assert_stake(backer, creator_id, 100);

        // Advance one era and unregister the creator
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_unregister(stakeholder, creator_id);

        // First claim should pass but second should fail because creator was unregistered
        assert_claim_backer(backer, creator_id, false);
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer), creator_id, false),
            Error::<TestRuntime>::InactiveCreator
        );

        assert_claim_creator(creator_id, 1);
        assert_noop!(
            CreatorStaking::claim_creator_reward(RuntimeOrigin::signed(stakeholder), creator_id, 2),
            Error::<TestRuntime>::InactiveCreator
        );
    })
}

#[test]
fn claim_in_future_era() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);
        assert_stake(backer, creator_id, 100);
        advance_to_era(start_era + 5);

        for era in start_era..CreatorStaking::current_era() {
            assert_claim_backer(backer, creator_id, false);
            assert_claim_creator(creator_id, era);
        }

        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer), creator_id, false),
            Error::<TestRuntime>::CannotClaimInFutureEra
        );
        assert_noop!(
            CreatorStaking::claim_creator_reward(
                RuntimeOrigin::signed(stakeholder),
                creator_id,
                CreatorStaking::current_era()
            ),
            Error::<TestRuntime>::CannotClaimInFutureEra
        );
    })
}

#[test]
fn claim_creator_same_era_twice() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);
        assert_stake(backer, creator_id, 100);
        advance_to_era(start_era + 1);

        assert_claim_creator(creator_id, start_era);
        assert_noop!(
            CreatorStaking::claim_creator_reward(RuntimeOrigin::signed(stakeholder), creator_id, start_era),
            Error::<TestRuntime>::AlreadyClaimedInThisEra
        );
    })
}

#[test]
fn claim_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let first_stakeholder = 1;
        let second_stakeholder = 2;
        let first_backer = 3;
        let second_backer = 4;
        let first_creator_id = 1;
        let second_creator_id = 2;

        let start_era = CreatorStaking::current_era();

        // Prepare a scenario with different stakes

        assert_register(first_stakeholder, first_creator_id);
        assert_register(second_stakeholder, second_creator_id);
        assert_stake(first_backer, first_creator_id, 100);
        assert_stake(second_backer, first_creator_id, 45);

        // Just so ratio isn't 100% in favor of the first creator
        assert_stake(first_backer, second_creator_id, 33);
        assert_stake(second_backer, second_creator_id, 22);

        let eras_advanced = 3;
        advance_to_era(start_era + eras_advanced);

        for x in 0..eras_advanced.into() {
            assert_stake(first_backer, first_creator_id, 20 + x * 3);
            assert_stake(second_backer, first_creator_id, 5 + x * 5);
            advance_to_era(CreatorStaking::current_era() + 1);
        }

        // Ensure that all past eras can be claimed
        let current_era = CreatorStaking::current_era();
        for era in start_era..current_era {
            assert_claim_backer(first_backer, first_creator_id, false);
            assert_claim_creator(first_creator_id, era);
            assert_claim_backer(second_backer, first_creator_id, false);
        }

        // Shouldn't be possible to claim current era.
        // Also, previous claim calls should have claimed everything prior to current era.
        assert_noop!(
            CreatorStaking::claim_backer_reward(
                RuntimeOrigin::signed(first_backer),
                first_creator_id.clone(),
                false,
            ),
            Error::<TestRuntime>::CannotClaimInFutureEra
        );
        assert_noop!(
            CreatorStaking::claim_creator_reward(
                RuntimeOrigin::signed(first_stakeholder),
                first_creator_id,
                current_era
            ),
            Error::<TestRuntime>::CannotClaimInFutureEra
        );
    })
}

#[test]
fn claim_after_unregister_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);
        let stake_value = 100;
        assert_stake(backer, creator_id, stake_value);

        // Advance few eras, then unstake everything
        advance_to_era(start_era + 5);
        assert_unstake(backer, creator_id, stake_value);
        let full_unstake_era = CreatorStaking::current_era();
        let number_of_staking_eras = full_unstake_era - start_era;

        // Few eras pass, then backer stakes again
        advance_to_era(CreatorStaking::current_era() + 3);
        let stake_value = 75;
        let restake_era = CreatorStaking::current_era();
        assert_stake(backer, creator_id, stake_value);

        // Again, few eras pass then creator is unregistered
        advance_to_era(CreatorStaking::current_era() + 3);
        assert_unregister(stakeholder, creator_id);
        let unregistration_era = CreatorStaking::current_era();
        let number_of_staking_eras = number_of_staking_eras + unregistration_era - restake_era;
        advance_to_era(CreatorStaking::current_era() + 2);

        // Ensure that backer can claim all the eras that he had an active stake
        for _ in 0..number_of_staking_eras {
            assert_claim_backer(backer, creator_id, false);
        }
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer), creator_id.clone(), false),
            Error::<TestRuntime>::InactiveCreator
        );

        // Ensure the same for creator reward
        for era in start_era..unregistration_era {
            if era >= full_unstake_era && era < restake_era {
                assert_noop!(
                    CreatorStaking::claim_creator_reward(
                        RuntimeOrigin::signed(stakeholder),
                        creator_id.clone(),
                        era
                    ),
                    Error::<TestRuntime>::NotStakedCreator
                );
            } else {
                assert_claim_creator(creator_id, era);
            }
        }
    })
}

#[test]
fn claim_only_payout_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        // stake some tokens
        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);
        let stake_value = 100;
        assert_stake(backer, creator_id, stake_value);

        // move to next era to be able to claim for the previous one
        advance_to_era(start_era + 1);

        // ensure it's claimed correctly
        assert_claim_backer(backer, creator_id, false);
    })
}

#[test]
fn claim_with_zero_staked_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;
        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);

        // stake some tokens and wait for an era
        let stake_value = 100;
        assert_stake(backer, creator_id, stake_value);
        advance_to_era(start_era + 1);

        // unstake all the tokens
        assert_unstake(backer, creator_id, stake_value);

        // ensure claimed value goes to claimer's free balance
        assert_claim_backer(backer, creator_id, false);
    })
}

// FIXME: how to be here?
#[ignore]
#[test]
fn claiming_when_stakes_full_without_compounding_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 10;
        let backer_id = 1;
        let creator_id = 1;
        // Insert a creator under registered creators.
        assert_register(stakeholder, creator_id);

        // Stake with MAX_ERA_STAKE_VALUES - 1 on the same creator. It must work.
        let start_era = CreatorStaking::current_era();
        for offset in 1..MAX_ERA_STAKE_ITEMS {
            assert_stake(backer_id, creator_id, 100);
            advance_to_era(start_era + offset * 5);
        }

        // claim and restake once, so there's a claim record for the current era in the stakes vec
        assert_claim_backer(backer_id, creator_id, true);

        // making another gap in eras and trying to claim and restake would exceed MAX_ERA_STAKE_VALUES
        advance_to_era(CreatorStaking::current_era() + 1);
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(backer_id), creator_id, true),
            Error::<TestRuntime>::TooManyEraStakeValues
        );

        // claiming should work again
        assert_claim_backer(backer_id, creator_id, false);
    })
}

#[test]
fn claim_creator_with_zero_stake_periods_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let stakeholder = 1;
        let backer = 2;
        let creator_id = 1;

        // Prepare scenario: <staked eras><not staked eras><staked eras><not staked eras>

        let start_era = CreatorStaking::current_era();
        assert_register(stakeholder, creator_id);
        let stake_value = 100;
        assert_stake(backer, creator_id, stake_value);

        advance_to_era(start_era + 5);
        let first_full_unstake_era = CreatorStaking::current_era();
        assert_unstake(backer, creator_id, stake_value);

        advance_to_era(CreatorStaking::current_era() + 7);
        let restake_era = CreatorStaking::current_era();
        assert_stake(backer, creator_id, stake_value);

        advance_to_era(CreatorStaking::current_era() + 4);
        let second_full_unstake_era = CreatorStaking::current_era();
        assert_unstake(backer, creator_id, stake_value);
        advance_to_era(CreatorStaking::current_era() + 10);

        // Ensure that first interval can be claimed
        for era in start_era..first_full_unstake_era {
            assert_claim_creator(creator_id, era);
        }

        // Ensure that the empty interval cannot be claimed
        for era in first_full_unstake_era..restake_era {
            assert_noop!(
                CreatorStaking::claim_creator_reward(
                    RuntimeOrigin::signed(stakeholder),
                    creator_id.clone(),
                    era
                ),
                Error::<TestRuntime>::NotStakedCreator
            );
        }

        // Ensure that second interval can be claimed
        for era in restake_era..second_full_unstake_era {
            assert_claim_creator(creator_id, era);
        }

        // Ensure no more claims are possible since creator was fully unstaked
        assert_noop!(
            CreatorStaking::claim_creator_reward(
                RuntimeOrigin::signed(stakeholder),
                creator_id.clone(),
                second_full_unstake_era
            ),
            Error::<TestRuntime>::NotStakedCreator
        );

        // Now stake again and ensure creator can once again be claimed
        let last_claim_era = CreatorStaking::current_era();
        assert_stake(backer, creator_id, stake_value);
        advance_to_era(last_claim_era + 1);
        assert_claim_creator(creator_id, last_claim_era);
    })
}

#[test]
fn rewards_are_independent_of_total_staked_amount_for_creators() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        let start_era = CreatorStaking::current_era();
        let registration_era = start_era + 1;
        let staking_era = start_era + 2;
        let claiming_era = start_era + 3;

        let stakeholder = 10;
        let first_creator_id = 1;
        let second_creator_id = 2;

        let dummy_backer_id = 1;
        let backer_id = 2;

        let stake_value = 100;

        // Register creators
        assert_register(stakeholder, first_creator_id);
        assert_register(stakeholder, second_creator_id);
        advance_to_era(registration_era);

        // Make creators have different total stakes
        assert_stake(dummy_backer_id, first_creator_id, 10);
        assert_stake(dummy_backer_id, second_creator_id, 10_000);
        advance_to_era(staking_era);

        // Stake some tokens (stake amount not to change total staked) for both creators
        assert_stake(backer_id, first_creator_id, stake_value);
        assert_stake(backer_id, second_creator_id, stake_value);
        advance_to_era(claiming_era);

        // Claim rewards for both creators
        let initial_backer_balance = Balances::free_balance(&backer_id);

        assert_claim_backer(backer_id, first_creator_id, false);
        let reward_for_first_creator = Balances::free_balance(&backer_id) - initial_backer_balance;

        assert_claim_backer(backer_id, second_creator_id, false);
        let reward_for_second_creator =
            Balances::free_balance(&backer_id) - reward_for_first_creator - initial_backer_balance;

        // Actual rewards should be equal since total staked amount doesn't affect reward
        assert_eq!(reward_for_first_creator, reward_for_second_creator);
    })
}

#[test]
fn maintenance_mode_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        assert_ok!(CreatorStaking::ensure_pallet_enabled());
        // TODO: revert when default PalletDisabled changed back to false
        // assert!(!PalletDisabled::<TestRuntime>::exists());
        assert!(!PalletDisabled::<TestRuntime>::get());

        assert_ok!(CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), true));
        // TODO: revert when default PalletDisabled changed back to false
        // assert!(PalletDisabled::<TestRuntime>::exists());
        assert!(PalletDisabled::<TestRuntime>::get());

        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::MaintenanceModeSet {
            enabled: true,
        }));

        let account = 1;
        let creator_id = 1;

        //
        // 1
        assert_noop!(
            CreatorStaking::force_register_creator(RuntimeOrigin::root(), creator_id),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::force_unregister_creator(RuntimeOrigin::root(), creator_id),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::withdraw_from_inactive_creator(RuntimeOrigin::signed(account), creator_id),
            Error::<TestRuntime>::PalletIsDisabled
        );

        //
        // 2
        assert_noop!(
            CreatorStaking::stake(RuntimeOrigin::signed(account), creator_id, 100),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::unstake(RuntimeOrigin::signed(account), creator_id, 100),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::claim_creator_reward(RuntimeOrigin::signed(account), creator_id, 5),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::claim_backer_reward(RuntimeOrigin::signed(account), creator_id, false),
            Error::<TestRuntime>::PalletIsDisabled
        );
        assert_noop!(
            CreatorStaking::withdraw_unstaked(RuntimeOrigin::signed(account)),
            Error::<TestRuntime>::PalletIsDisabled
        );

        //
        // 3
        // shouldn't do anything since we're in maintenance mode
        assert_eq!(CreatorStaking::on_initialize(3), Weight::zero());

        //
        // 4
        assert_ok!(CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), false));
        System::assert_last_event(RuntimeEvent::CreatorStaking(Event::MaintenanceModeSet {
            enabled: false,
        }));
        assert_register(account, creator_id);
    })
}

#[test]
fn maintenance_mode_no_change() {
    ExternalityBuilder::build().execute_with(|| {
        initialize_first_block();

        // Expect an error since maintenance mode is already disabled
        assert_ok!(CreatorStaking::ensure_pallet_enabled());
        assert_noop!(
            CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), false),
            Error::<TestRuntime>::MaintenanceModeNotChanged
        );

        // Same for the case when maintenance mode is already enabled
        assert_ok!(CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), true));
        assert_noop!(
            CreatorStaking::set_maintenance_mode(RuntimeOrigin::root(), true),
            Error::<TestRuntime>::MaintenanceModeNotChanged
        );
    })
}

#[test]
fn calculate_creator_reward_is_ok() {
    let base_backers_reward = 7 * 11 * 13 * 17;
    let base_creators_reward = 19 * 23 * 31;
    let staked_on_creator = 123456;
    let total_staked = staked_on_creator * 3;

    // Prepare structs
    let creator_info = CreatorStakeInfo::<Balance> {
        total_staked: staked_on_creator,
        backers_count: 10,
        rewards_claimed: false,
    };
    let era_info = EraInfo::<Balance> {
        rewards: RewardInfo {
            creators: base_creators_reward,
            backers: base_backers_reward,
        },
        staked: total_staked,
        locked: total_staked,
    };

    let actual_creator_reward =
        CreatorStaking::calculate_creator_reward(&creator_info, &era_info);

    let creator_stake_ratio = Perbill::from_rational(staked_on_creator, total_staked);
    let expected_creator_reward = creator_stake_ratio * base_creators_reward;

    assert_eq!(expected_creator_reward, actual_creator_reward);
}

#[test]
pub fn tvl_util_test() {
    ExternalityBuilder::build().execute_with(|| {
        // Ensure TVL is zero before first block and also after first block
        assert!(CreatorStaking::tvl().is_zero());
        initialize_first_block();
        assert!(CreatorStaking::tvl().is_zero());

        let stakeholder = 1;
        let creator_id = 1;
        assert_register(stakeholder, creator_id);

        // Expect TVL to change as we stake more
        let iterations = 10;
        let stake_value = 100;
        for x in 1..=iterations {
            assert_stake(stakeholder, creator_id, stake_value);
            assert_eq!(CreatorStaking::tvl(), stake_value * x);
        }

        // Era advancement should have no effect on TVL
        advance_to_era(5);
        assert_eq!(CreatorStaking::tvl(), stake_value * iterations);
    })
}

// Inflation tests
// ---------------

#[test]
fn default_reward_distribution_config_is_equal_to_one() {
    let reward_config = RewardDistributionConfig::default();
    assert!(reward_config.is_sum_equal_to_one());
}

#[test]
fn reward_distribution_config_is_equal_to_one() {
    // 1
    let reward_config = RewardDistributionConfig {
        backers_percent: Zero::zero(),
        creators_percent: Zero::zero(),
        treasury_percent: Perbill::from_percent(100),
    };
    assert!(reward_config.is_sum_equal_to_one());

    // 2
    let reward_config = RewardDistributionConfig {
        backers_percent: Zero::zero(),
        creators_percent: Perbill::from_percent(100),
        treasury_percent: Zero::zero(),
    };
    assert!(reward_config.is_sum_equal_to_one());

    // 3
    // Sum of 3 random numbers should equal to one.
    let reward_config = RewardDistributionConfig {
        backers_percent: Perbill::from_percent(34),
        creators_percent: Perbill::from_percent(51),
        treasury_percent: Perbill::from_percent(15),
    };
    assert!(reward_config.is_sum_equal_to_one());
}

#[test]
fn reward_distribution_config_not_equal_to_one() {
    // 1
    // 99%
    let reward_config = RewardDistributionConfig {
        backers_percent: Perbill::from_percent(34),
        creators_percent: Perbill::from_percent(51),
        // Here should be 15, then it'll be equal to 100%
        treasury_percent: Perbill::from_percent(14),
    };
    assert!(!reward_config.is_sum_equal_to_one());

    // 2
    // 101%
    let reward_config = RewardDistributionConfig {
        backers_percent: Perbill::from_percent(34),
        creators_percent: Perbill::from_percent(51),
        // Here should be 15, then it'll be equal to 100%
        treasury_percent: Perbill::from_percent(16),
    };
    assert!(!reward_config.is_sum_equal_to_one());
}

#[test]
pub fn set_configuration_fails() {
    ExternalityBuilder::build().execute_with(|| {
        // Not root cannot set any (even valid) config
        assert_noop!(
            CreatorStaking::set_reward_distribution_config(RuntimeOrigin::signed(1), Default::default()),
            BadOrigin
        );

        // Root cannot set invalid config
        let reward_config = RewardDistributionConfig {
            treasury_percent: Perbill::from_percent(100),
            ..Default::default()
        };
        assert!(!reward_config.is_sum_equal_to_one());
        assert_noop!(
            CreatorStaking::set_reward_distribution_config(RuntimeOrigin::root(), reward_config),
            Error::<TestRuntime>::InvalidSumOfRewardDistributionConfig,
        );
    })
}

#[test]
pub fn set_configuration_is_ok() {
    ExternalityBuilder::build().execute_with(|| {
        // custom config so it differs from the default one
        let new_config = RewardDistributionConfig {
            backers_percent: Perbill::from_percent(15),
            creators_percent: Perbill::from_percent(35),
            treasury_percent: Perbill::from_percent(50),
        };
        assert!(new_config.is_sum_equal_to_one());

        assert_ok!(CreatorStaking::set_reward_distribution_config(
            RuntimeOrigin::root(),
            new_config.clone()
        ));
        System::assert_last_event(RuntimeEvent::CreatorStaking(
            Event::RewardDistributionConfigChanged { new_config: new_config.clone() },
        ));

        assert_eq!(
            ActiveRewardDistributionConfig::<TestRuntime>::get(),
            new_config
        );
    })
}

#[test]
pub fn inflation_and_total_issuance_as_expected() {
    ExternalityBuilder::build().execute_with(|| {
        let init_issuance = <TestRuntime as Config>::Currency::total_issuance();
        let per_block_reward = Rewards::total_block_reward();

        for block in 0..10 {
            assert_eq!(
                <TestRuntime as Config>::Currency::total_issuance(),
                block * per_block_reward + init_issuance
            );
            CreatorStaking::on_timestamp_set(0);
            assert_eq!(
                <TestRuntime as Config>::Currency::total_issuance(),
                (block + 1) * per_block_reward + init_issuance
            );
        }
    })
}

#[test]
pub fn reward_distribution_as_expected() {
    ExternalityBuilder::build().execute_with(|| {
        // Ensure that initially, all beneficiaries have no free balance
        let init_balance_snapshot = FreeBalanceSnapshot::new();
        assert!(init_balance_snapshot.is_zero());

        // Prepare a custom config (easily discernible percentages for visual verification)
        let reward_config = RewardDistributionConfig {
            backers_percent: Perbill::from_percent(15),
            creators_percent: Perbill::from_percent(35),
            treasury_percent: Perbill::from_percent(50),
        };
        assert!(reward_config.is_sum_equal_to_one());
        assert_ok!(CreatorStaking::set_reward_distribution_config(
            RuntimeOrigin::root(),
            reward_config.clone()
        ));

        // Issue rewards a couple of times and verify distribution is as expected
        for _block in 1..=100 {
            let init_balance_state = FreeBalanceSnapshot::new();
            let rewards = Rewards::calculate(&reward_config);

            CreatorStaking::on_timestamp_set(0);

            let final_balance_state = FreeBalanceSnapshot::new();
            init_balance_state.assert_distribution(&final_balance_state, &rewards);
        }
    })
}

/// Represents free balance snapshot at a specific point in time
/// (i.e. before or after reward distribution).
///
/// Contains balances for all reward beneficiaries:
/// - treasury
/// - backers&creators (rewards pot account)
#[derive(PartialEq, Eq, Clone, RuntimeDebug)]
pub(super) struct FreeBalanceSnapshot {
    treasury: Balance,
    rewards_pot: Balance,
}

impl FreeBalanceSnapshot {
    /// Creates a new free balance snapshot using current balance state.
    ///
    /// Future balance changes won't be reflected in this instance.
    fn new() -> Self {
        Self {
            treasury: <TestRuntime as Config>::Currency::free_balance(
                TREASURY_ACCOUNT,
            ),
            rewards_pot: <TestRuntime as Config>::Currency::free_balance(
                CreatorStaking::rewards_pot_account(),
            ),
        }
    }

    /// `true` if all free balances equal `Zero`, `false` otherwise
    fn is_zero(&self) -> bool {
        self.treasury.is_zero()
            && self.rewards_pot.is_zero()
    }

    /// Asserts that post reward state is as expected.
    ///
    /// Increase in balances, based on `rewards` values, is verified.
    ///
    fn assert_distribution(&self, post_reward_state: &Self, rewards: &Rewards) {
        assert_eq!(
            self.treasury + rewards.treasury_reward,
            post_reward_state.treasury
        );
        assert_eq!(
            self.rewards_pot + (rewards.backers_reward + rewards.creators_reward),
            post_reward_state.rewards_pot
        );
    }
}

/// Represents reward distribution balances for a single distribution.
#[derive(PartialEq, Eq, Clone, RuntimeDebug)]
pub(super) struct Rewards {
    pub(super) treasury_reward: Balance,
    pub(super) backers_reward: Balance,
    pub(super) creators_reward: Balance,
}

impl Rewards {
    pub(super) fn total_block_reward() -> Balance {
        CreatorStaking::per_block_reward()
    }

    pub(super) fn joint_block_reward() -> Balance {
        let Rewards { treasury_reward, .. } = Self::calculate(&CreatorStaking::reward_config());

        Self::total_block_reward() - treasury_reward
    }

    /// Pre-calculates the reward distribution, using the provided `RewardDistributionConfig`.
    /// Method assumes that total issuance will be increased by `BLOCK_REWARD`.
    pub(super) fn calculate(reward_config: &RewardDistributionConfig) -> Self {
        let per_block_reward = Self::total_block_reward();

        // Calculate `tvl-independent` portions
        let treasury_reward = reward_config.treasury_percent * per_block_reward;
        let backer_reward = reward_config.backers_percent * per_block_reward;
        let creators_reward = reward_config.creators_percent * per_block_reward;

        Self {
            treasury_reward,
            backers_reward: backer_reward,
            creators_reward,
        }
    }
}
