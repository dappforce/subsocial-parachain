// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


//! Autogenerated weights for pallet_ownership
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-29, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MacBook-Pro-Vladislav.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./scripts/../target/release/subsocial-collator
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet
// pallet_ownership
// --extrinsic
// *
// --execution=wasm
// --wasm-execution=Compiled
// --heap-pages=4096
// --output=pallets/space-ownership/src/weights.rs
// --template=./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ownership.
pub trait WeightInfo {
    fn transfer_space_ownership() -> Weight;
    fn transfer_post_ownership() -> Weight;
    fn transfer_domain_ownership() -> Weight;
    fn accept_pending_space_ownership_transfer() -> Weight;
    fn accept_pending_post_ownership_transfer() -> Weight;
    fn accept_pending_domain_ownership_transfer() -> Weight;
    fn reject_pending_ownership() -> Weight;
}

/// Weights for pallet_ownership using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: CreatorStaking RegisteredCreators (r:1 w:0)
    /// Proof: CreatorStaking RegisteredCreators (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_space_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1447`
        //  Estimated: `8430`
        // Minimum execution time: 20_000_000 picoseconds.
        Weight::from_parts(21_000_000, 8430)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Posts PostById (r:1 w:0)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
    /// Proof Skipped: SpaceFollows SpaceFollowedByAccount (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_post_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1700`
        //  Estimated: `15495`
        // Minimum execution time: 27_000_000 picoseconds.
        Weight::from_parts(28_000_000, 15495)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Domains RegisteredDomains (r:1 w:0)
    /// Proof Skipped: Domains RegisteredDomains (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_domain_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `282`
        //  Estimated: `3747`
        // Minimum execution time: 15_000_000 picoseconds.
        Weight::from_parts(16_000_000, 3747)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceById (r:1 w:1)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: CreatorStaking RegisteredCreators (r:1 w:0)
    /// Proof: CreatorStaking RegisteredCreators (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceIdsByOwner (r:2 w:2)
    /// Proof Skipped: Spaces SpaceIdsByOwner (max_values: None, max_size: None, mode: Measured)
    /// Storage: Profiles ProfileSpaceIdByAccount (r:1 w:0)
    /// Proof: Profiles ProfileSpaceIdByAccount (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
    fn accept_pending_space_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1638`
        //  Estimated: `23290`
        // Minimum execution time: 36_000_000 picoseconds.
        Weight::from_parts(37_000_000, 23290)
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Posts PostById (r:1 w:1)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    fn accept_pending_post_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `403`
        //  Estimated: `7438`
        // Minimum execution time: 20_000_000 picoseconds.
        Weight::from_parts(20_000_000, 7438)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Domains RegisteredDomains (r:1 w:1)
    /// Proof Skipped: Domains RegisteredDomains (max_values: None, max_size: None, mode: Measured)
    /// Storage: Domains DomainsByOwner (r:2 w:2)
    /// Proof Skipped: Domains DomainsByOwner (max_values: None, max_size: None, mode: Measured)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    fn accept_pending_domain_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `688`
        //  Estimated: `20547`
        // Minimum execution time: 56_000_000 picoseconds.
        Weight::from_parts(57_000_000, 20547)
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(6_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    fn reject_pending_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1595`
        //  Estimated: `8630`
        // Minimum execution time: 21_000_000 picoseconds.
        Weight::from_parts(22_000_000, 8630)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: CreatorStaking RegisteredCreators (r:1 w:0)
    /// Proof: CreatorStaking RegisteredCreators (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_space_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1447`
        //  Estimated: `8430`
        // Minimum execution time: 20_000_000 picoseconds.
        Weight::from_parts(21_000_000, 8430)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    /// Storage: Posts PostById (r:1 w:0)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceFollows SpaceFollowedByAccount (r:1 w:0)
    /// Proof Skipped: SpaceFollows SpaceFollowedByAccount (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_post_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1700`
        //  Estimated: `15495`
        // Minimum execution time: 27_000_000 picoseconds.
        Weight::from_parts(28_000_000, 15495)
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    /// Storage: Domains RegisteredDomains (r:1 w:0)
    /// Proof Skipped: Domains RegisteredDomains (max_values: None, max_size: None, mode: Measured)
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:0 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    fn transfer_domain_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `282`
        //  Estimated: `3747`
        // Minimum execution time: 15_000_000 picoseconds.
        Weight::from_parts(16_000_000, 3747)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceById (r:1 w:1)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    /// Storage: CreatorStaking RegisteredCreators (r:1 w:0)
    /// Proof: CreatorStaking RegisteredCreators (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceIdsByOwner (r:2 w:2)
    /// Proof Skipped: Spaces SpaceIdsByOwner (max_values: None, max_size: None, mode: Measured)
    /// Storage: Profiles ProfileSpaceIdByAccount (r:1 w:0)
    /// Proof: Profiles ProfileSpaceIdByAccount (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
    fn accept_pending_space_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1638`
        //  Estimated: `23290`
        // Minimum execution time: 36_000_000 picoseconds.
        Weight::from_parts(37_000_000, 23290)
            .saturating_add(RocksDbWeight::get().reads(6_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Posts PostById (r:1 w:1)
    /// Proof Skipped: Posts PostById (max_values: None, max_size: None, mode: Measured)
    fn accept_pending_post_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `403`
        //  Estimated: `7438`
        // Minimum execution time: 20_000_000 picoseconds.
        Weight::from_parts(20_000_000, 7438)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Domains RegisteredDomains (r:1 w:1)
    /// Proof Skipped: Domains RegisteredDomains (max_values: None, max_size: None, mode: Measured)
    /// Storage: Domains DomainsByOwner (r:2 w:2)
    /// Proof Skipped: Domains DomainsByOwner (max_values: None, max_size: None, mode: Measured)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    fn accept_pending_domain_ownership_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `688`
        //  Estimated: `20547`
        // Minimum execution time: 56_000_000 picoseconds.
        Weight::from_parts(57_000_000, 20547)
            .saturating_add(RocksDbWeight::get().reads(6_u64))
            .saturating_add(RocksDbWeight::get().writes(6_u64))
    }
    /// Storage: SpaceOwnership PendingOwnershipTransfers (r:1 w:1)
    /// Proof: SpaceOwnership PendingOwnershipTransfers (max_values: None, max_size: Some(105), added: 2580, mode: MaxEncodedLen)
    /// Storage: Spaces SpaceById (r:1 w:0)
    /// Proof Skipped: Spaces SpaceById (max_values: None, max_size: None, mode: Measured)
    fn reject_pending_ownership() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1595`
        //  Estimated: `8630`
        // Minimum execution time: 21_000_000 picoseconds.
        Weight::from_parts(22_000_000, 8630)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}