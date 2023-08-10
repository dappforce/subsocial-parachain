//! Runtime API definition for domains pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait DomainsApi<Balance> where
		Balance: Codec + MaybeDisplay,
	{
		fn calculate_price(subdomain: Vec<u8>) -> Option<Balance>;
	}
}
