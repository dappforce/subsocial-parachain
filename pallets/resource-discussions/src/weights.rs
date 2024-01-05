
//! Weights placeholder for for pallet_resource_discussions

// Executed Command:
// ./scripts/../target/release/subsocial-collator
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet
// pallet_resource_discussions
// --extrinsic
// *
// --execution=wasm
// --wasm-execution=Compiled
// --heap-pages=4096
// --output=pallets/resource-discussions/src/weights.rs
// --template=./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_resource_discussions.
pub trait WeightInfo {
    fn link_post_to_resource() -> Weight;
    fn create_resource_discussion() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    /// Storage: Posts PostById (r:1 w:0)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    /// Storage: ResourceDiscussions ResourceDiscussion (r:0 w:1)
    /// Proof: ResourceDiscussions ResourceDiscussion (max_values: None, max_size: Some(75), added: 2550, mode: MaxEncodedLen)
    fn link_post_to_resource() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `252`
        //  Estimated: `3717`
        // Minimum execution time: 17_000_000 picoseconds.
        Weight::from_parts(18_000_000, 3717)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    /// Storage: ResourceDiscussions ResourceDiscussion (r:1 w:1)
    /// Proof: ResourceDiscussions ResourceDiscussion (max_values: None, max_size: Some(75), added: 2550, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: Posts NextPostId (r:1 w:1)
    /// Proof Skipped: Posts NextPostId (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: Timestamp Now (r:1 w:0)
    /// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
    /// Proof Skipped: SpaceFollows SpaceFollowedByAccount (max_values: None, max_size: None, mode: Measured)
    /// Storage: Posts PostIdsBySpaceId (r:1 w:1)
    /// Proof Skipped: Posts PostIdsBySpaceId (max_values: None, max_size: None, mode: Measured)
    /// Storage: Posts PostById (r:0 w:1)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    fn create_resource_discussion() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1569`
        //  Estimated: `24758`
        // Minimum execution time: 49_000_000 picoseconds.
        Weight::from_parts(50_000_000, 24758)
            .saturating_add(RocksDbWeight::get().reads(6_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
    }
}