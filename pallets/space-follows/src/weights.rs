
//! Autogenerated weights for pallet_space_follows
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `benchmarks-ci`, CPU: `Intel(R) Xeon(R) Platinum 8280 CPU @ 2.70GHz`
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
    // pallet_space_follows
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // pallets/space-follows/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_space_follows.
pub trait WeightInfo {
    fn follow_space() -> Weight;
    fn unfollow_space() -> Weight;
}

/// Weights for pallet_space_follows using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowers (r:1 w:1)
            // Storage: SpaceFollows SpacesFollowedByAccount (r:1 w:1)
        fn follow_space() -> Weight {
        // Minimum execution time: 48_140 nanoseconds.
        Weight::from_ref_time(48_862_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
        }
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:1)
            // Storage: SpaceFollows SpacesFollowedByAccount (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowers (r:1 w:1)
        fn unfollow_space() -> Weight {
        // Minimum execution time: 55_112 nanoseconds.
        Weight::from_ref_time(55_868_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowers (r:1 w:1)
            // Storage: SpaceFollows SpacesFollowedByAccount (r:1 w:1)
        fn follow_space() -> Weight {
        // Minimum execution time: 48_140 nanoseconds.
        Weight::from_ref_time(48_862_000)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
        }
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:1)
            // Storage: SpaceFollows SpacesFollowedByAccount (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowers (r:1 w:1)
        fn unfollow_space() -> Weight {
        // Minimum execution time: 55_112 nanoseconds.
        Weight::from_ref_time(55_868_000)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
        }
    }
