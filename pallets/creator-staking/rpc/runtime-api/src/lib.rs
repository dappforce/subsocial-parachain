//! Runtime API definition for domains pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use subsocial_support::SpaceId;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait CreatorStakingApi<AccountId, Balance> where
		AccountId: Codec + MaybeDisplay,
		Balance: Codec + MaybeDisplay,
	{
		fn estimated_backer_rewards_by_creators(
			backer: AccountId,
			creators: Vec<SpaceId>
		) -> Vec<(SpaceId, Balance)>;

		fn withdrawable_amounts_from_inactive_creators(
			backer: AccountId
		) -> Vec<(SpaceId, Balance)>;

		fn available_claims_by_backer(backer: AccountId) -> Vec<(SpaceId, u32)>;
	}
}