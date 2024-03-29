// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use crate::*;
use frame_support::traits::{Currency, Get, OnTimestampSet, Imbalance};
use sp_runtime::traits::Saturating;

type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl<Moment, T: Config> OnTimestampSet<Moment> for Pallet<T> {
    fn on_timestamp_set(_moment: Moment) {
        if PalletDisabled::<T>::get() {
            return;
        }

        let new_tokens_per_block: BalanceOf<T> = Self::per_block_reward();
        let inflation = T::Currency::issue(new_tokens_per_block);

        Self::distribute_rewards(inflation);
    }
}

impl<T: Config> Pallet<T> {
    fn distribute_rewards(block_reward: NegativeImbalanceOf<T>) {
        let distro_params = Self::reward_config();

        // Pre-calculate the balance that will be deposited for each beneficiary
        let backers_balance = distro_params.backers_percent * block_reward.peek();
        let creators_balance = distro_params.creators_percent * block_reward.peek();
        let treasury_balance = distro_params.treasury_percent * block_reward.peek();

        // Prepare imbalances
        let (creators_imbalance, remainder) = block_reward.split(creators_balance);
        let (backers_imbalance, remainder) = remainder.split(backers_balance);
        let (treasury_imbalance, leftover_imbalance) = remainder.split(treasury_balance);

        // Payout beneficiaries
        Self::add_to_reward_pool(backers_imbalance, creators_imbalance);

        T::Currency::resolve_creating(&T::TreasuryAccount::get(), treasury_imbalance.merge(leftover_imbalance));
    }

    pub fn add_to_reward_pool(backers: NegativeImbalanceOf<T>, creators: NegativeImbalanceOf<T>) {
        BlockRewardAccumulator::<T>::mutate(|accumulated_reward| {
            accumulated_reward.creators = accumulated_reward.creators.saturating_add(creators.peek());
            accumulated_reward.backers =
                accumulated_reward.backers.saturating_add(backers.peek());
        });

        T::Currency::resolve_creating(&Self::rewards_pot_account(), creators.merge(backers));
    }
}
