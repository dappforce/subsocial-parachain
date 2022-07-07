//! Autogenerated weights for pallet_posts
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-07, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    // pallet_posts
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // ./pallets/posts/src//weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_posts.
pub trait WeightInfo {
    fn create_post__regular() -> Weight;
    fn create_post__shared() -> Weight;
    fn create_post__comment() -> Weight;
}

/// Weights for pallet_posts using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
            // Storage: Posts PostById (r:0 w:1)
        fn create_post__regular() -> Weight {
        (59_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
        }
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostById (r:1 w:2)
            // Storage: Posts SharedPostIdsByOriginalPostId (r:1 w:1)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
        fn create_post__shared() -> Weight {
        (88_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
        }
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Posts PostById (r:2 w:3)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts ReplyIdsByPostId (r:1 w:1)
        fn create_post__comment() -> Weight {
        (79_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
            // Storage: Posts PostById (r:0 w:1)
        fn create_post__regular() -> Weight {
        (59_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
        }
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:1)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts PostById (r:1 w:2)
            // Storage: Posts SharedPostIdsByOriginalPostId (r:1 w:1)
            // Storage: Posts PostIdsBySpaceId (r:1 w:1)
        fn create_post__shared() -> Weight {
        (88_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
        }
            // Storage: Posts NextPostId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Posts PostById (r:2 w:3)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Posts ReplyIdsByPostId (r:1 w:1)
        fn create_post__comment() -> Weight {
        (79_000_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
        }
    }
