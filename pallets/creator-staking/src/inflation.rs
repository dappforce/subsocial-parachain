use crate::*;
use frame_support::traits::{Currency, Get, OnTimestampSet, Imbalance};
use sp_runtime::traits::{Saturating, SaturatedConversion, UniqueSaturatedInto, Zero};

type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

impl<Moment, T: Config> OnTimestampSet<Moment> for Pallet<T> {
    fn on_timestamp_set(_moment: Moment) {
        let total_issuance = T::Currency::total_issuance();
        let reward_amount: BalanceOf<T> = T::CurrentAnnualInflation::get() * total_issuance
            / T::BlocksPerYear::get().saturated_into::<u32>().unique_saturated_into();

        let inflation = T::Currency::issue(reward_amount);
        Self::distribute_rewards(inflation);
    }
}

impl<T: Config> Pallet<T> {
    fn distribute_rewards(block_reward: NegativeImbalanceOf<T>) {
        let distro_params = Self::reward_config();

        // Pre-calculate balance which will be deposited for each beneficiary
        let stakers_balance = distro_params.stakers_percent * block_reward.peek();
        let creators_balance = distro_params.creators_percent * block_reward.peek();

        // Prepare imbalances
        let (creators_imbalance, remainder) = block_reward.split(creators_balance);
        let (stakers_imbalance, treasury_imbalance) = remainder.split(stakers_balance);

        // Payout beneficiaries
        BlockRewardAccumulator::<T>::mutate(|accumulated_reward| {
            accumulated_reward.creators = accumulated_reward.creators.saturating_add(creators_imbalance.peek());
            accumulated_reward.stakers =
                accumulated_reward.stakers.saturating_add(stakers_imbalance.peek());
        });

        T::Currency::resolve_creating(&Self::account_id(), creators_imbalance.merge(stakers_imbalance));
        if !treasury_imbalance.peek().is_zero() {
            T::Currency::resolve_creating(&T::TreasuryAccount::get(), treasury_imbalance);
        }
    }
}
