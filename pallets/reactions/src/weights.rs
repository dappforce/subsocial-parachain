// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


//! Autogenerated weights for pallet_reactions
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
    // pallet_reactions
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // pallets/reactions/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_reactions.
pub trait WeightInfo {
    fn create_post_reaction() -> Weight;
    fn update_post_reaction() -> Weight;
    fn delete_post_reaction() -> Weight;
}

/// Weights for pallet_reactions using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Posts PostById (r:1 w:1)
            // Storage: Reactions PostReactionIdByAccount (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Reactions NextReactionId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Reactions ReactionIdsByPostId (r:1 w:1)
            // Storage: Reactions ReactionById (r:0 w:1)
        fn create_post_reaction() -> Weight {
        // Minimum execution time: 73_075 nanoseconds.
        Weight::from_parts(74_198_000, 0)
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(5))
        }
            // Storage: Reactions PostReactionIdByAccount (r:1 w:0)
            // Storage: Reactions ReactionById (r:1 w:1)
            // Storage: Posts PostById (r:1 w:1)
        fn update_post_reaction() -> Weight {
        // Minimum execution time: 48_899 nanoseconds.
        Weight::from_parts(52_268_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
        }
            // Storage: Reactions PostReactionIdByAccount (r:1 w:1)
            // Storage: Reactions ReactionById (r:1 w:1)
            // Storage: Posts PostById (r:1 w:1)
            // Storage: Reactions ReactionIdsByPostId (r:1 w:1)
        fn delete_post_reaction() -> Weight {
        // Minimum execution time: 55_284 nanoseconds.
        Weight::from_parts(56_721_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Posts PostById (r:1 w:1)
            // Storage: Reactions PostReactionIdByAccount (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
            // Storage: Reactions NextReactionId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Reactions ReactionIdsByPostId (r:1 w:1)
            // Storage: Reactions ReactionById (r:0 w:1)
        fn create_post_reaction() -> Weight {
        // Minimum execution time: 73_075 nanoseconds.
        Weight::from_parts(74_198_000, 0)
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(5))
        }
            // Storage: Reactions PostReactionIdByAccount (r:1 w:0)
            // Storage: Reactions ReactionById (r:1 w:1)
            // Storage: Posts PostById (r:1 w:1)
        fn update_post_reaction() -> Weight {
        // Minimum execution time: 48_899 nanoseconds.
        Weight::from_parts(52_268_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
        }
            // Storage: Reactions PostReactionIdByAccount (r:1 w:1)
            // Storage: Reactions ReactionById (r:1 w:1)
            // Storage: Posts PostById (r:1 w:1)
            // Storage: Reactions ReactionIdsByPostId (r:1 w:1)
        fn delete_post_reaction() -> Weight {
        // Minimum execution time: 55_284 nanoseconds.
        Weight::from_parts(56_721_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
        }
    }
