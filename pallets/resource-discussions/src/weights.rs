
//! Autogenerated weights for pallet_resource_discussions
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Tarek-M1-Pro.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
    // ./scripts/../target/release/subsocial-collator
    // benchmark
    // pallet
    // --chain
    // dev
    // --execution
    // wasm
    // --wasm-execution
    // Compiled
    // --pallet
    // pallet_resource_discussions
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // ./pallets/resource-discussions/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

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

/// Weights for pallet_resource_discussions using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Posts PostById (r:1 w:0)
            // Storage: ResourceDiscussions ResourceDiscussion (r:0 w:1)
        fn link_post_to_resource() -> Weight {
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
        }
            // Storage: ResourceDiscussions ResourceDiscussion (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
            // Storage: Posts PostById (r:0 w:1)
        fn create_resource_discussion() -> Weight {
        // Minimum execution time: 46_000 nanoseconds.
        Weight::from_ref_time(46_000_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Posts PostById (r:1 w:0)
            // Storage: ResourceDiscussions ResourceDiscussion (r:0 w:1)
        fn link_post_to_resource() -> Weight {
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
        }
            // Storage: ResourceDiscussions ResourceDiscussion (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
            // Storage: Posts PostById (r:0 w:1)
        fn create_resource_discussion() -> Weight {
        // Minimum execution time: 46_000 nanoseconds.
        Weight::from_ref_time(46_000_000)
            .saturating_add(RocksDbWeight::get().reads(6))
            .saturating_add(RocksDbWeight::get().writes(4))
        }
    }
