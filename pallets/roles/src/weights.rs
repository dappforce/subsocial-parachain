// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


//! Autogenerated weights for pallet_roles
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
    // pallet_roles
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // pallets/roles/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_roles.
pub trait WeightInfo {
    fn create_role() -> Weight;
    fn update_role() -> Weight;
    fn delete_role(x: u32, ) -> Weight;
    fn grant_role(x: u32, ) -> Weight;
    fn revoke_role(x: u32, ) -> Weight;
}

/// Weights for pallet_roles using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles NextRoleId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Roles RoleIdsBySpaceId (r:1 w:1)
            // Storage: Roles RoleById (r:0 w:1)
        fn create_role() -> Weight {
        // Minimum execution time: 52_528 nanoseconds.
        Weight::from_ref_time(53_688_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
        }
            // Storage: Roles RoleById (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
        fn update_role() -> Weight {
        // Minimum execution time: 48_647 nanoseconds.
        Weight::from_ref_time(50_219_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
        }
            // Storage: Roles RoleById (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsBySpaceId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[0, 40]`.
        fn delete_role(x: u32, ) -> Weight {
        // Minimum execution time: 57_007 nanoseconds.
        Weight::from_ref_time(64_783_236)
            // Standard Error: 26_706
            .saturating_add(Weight::from_ref_time(8_701_651).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
            // Storage: Roles RoleById (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[1, 500]`.
        fn grant_role(x: u32, ) -> Weight {
        // Minimum execution time: 56_274 nanoseconds.
        Weight::from_ref_time(56_612_000)
            // Standard Error: 131_120
            .saturating_add(Weight::from_ref_time(21_272_680).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
            // Storage: Roles RoleById (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[1, 500]`.
        fn revoke_role(x: u32, ) -> Weight {
        // Minimum execution time: 61_065 nanoseconds.
        Weight::from_ref_time(61_492_000)
            // Standard Error: 13_040
            .saturating_add(Weight::from_ref_time(9_647_540).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles NextRoleId (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
            // Storage: Roles RoleIdsBySpaceId (r:1 w:1)
            // Storage: Roles RoleById (r:0 w:1)
        fn create_role() -> Weight {
        // Minimum execution time: 52_528 nanoseconds.
        Weight::from_ref_time(53_688_000)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(3))
        }
            // Storage: Roles RoleById (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
        fn update_role() -> Weight {
        // Minimum execution time: 48_647 nanoseconds.
        Weight::from_ref_time(50_219_000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(1))
        }
            // Storage: Roles RoleById (r:1 w:1)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsBySpaceId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[0, 40]`.
        fn delete_role(x: u32, ) -> Weight {
        // Minimum execution time: 57_007 nanoseconds.
        Weight::from_ref_time(64_783_236)
            // Standard Error: 26_706
            .saturating_add(Weight::from_ref_time(8_701_651).saturating_mul(x.into()))
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(RocksDbWeight::get().writes(3))
            .saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
            // Storage: Roles RoleById (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[1, 500]`.
        fn grant_role(x: u32, ) -> Weight {
        // Minimum execution time: 56_274 nanoseconds.
        Weight::from_ref_time(56_612_000)
            // Standard Error: 131_120
            .saturating_add(Weight::from_ref_time(21_272_680).saturating_mul(x.into()))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(RocksDbWeight::get().writes(1))
            .saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
            // Storage: Roles RoleById (r:1 w:0)
            // Storage: Spaces SpaceById (r:1 w:0)
            // Storage: Roles UsersByRoleId (r:1 w:1)
            // Storage: Roles RoleIdsByUserInSpace (r:1 w:1)
            /// The range of component `x` is `[1, 500]`.
        fn revoke_role(x: u32, ) -> Weight {
        // Minimum execution time: 61_065 nanoseconds.
        Weight::from_ref_time(61_492_000)
            // Standard Error: 13_040
            .saturating_add(Weight::from_ref_time(9_647_540).saturating_mul(x.into()))
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(x.into())))
            .saturating_add(RocksDbWeight::get().writes(1))
            .saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(x.into())))
        }
    }
