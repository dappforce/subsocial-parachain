//! Runtime API definition for domains pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use sp_std::vec::Vec;

use pallet_creator_staking::{CreatorId, EraIndex};

sp_api::decl_runtime_apis! {
	pub trait CreatorStakingApi<AccountId, Balance> where
		AccountId: Codec + MaybeDisplay,
		Balance: Codec + MaybeDisplay,
	{
		fn estimated_backer_rewards_by_creators(
			backer: AccountId,
			creators: Vec<CreatorId>
		) -> Vec<(CreatorId, Balance)>;

		fn withdrawable_amounts_from_inactive_creators(
			backer: AccountId
		) -> Vec<(CreatorId, Balance)>;

		fn available_claims_by_backer(backer: AccountId) -> Vec<(CreatorId, u32)>;

		fn estimated_creator_rewards(creator_id: CreatorId) -> Balance;

		fn available_claims_by_creator(creator_id: CreatorId) -> Vec<EraIndex>;
	}
}
