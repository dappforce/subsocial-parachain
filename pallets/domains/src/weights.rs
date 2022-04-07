//! Autogenerated weights for pallet_domains
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-07, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
    // ./scripts/../target/release/subsocial-collator
    // benchmark
    // --chain
    // dev
    // --execution
    // wasm
    // --wasm-execution
    // Compiled
    // --pallet
    // pallet_domains
    // --extrinsic
    // *
    // --steps
    // 50
    // --repeat
    // 20
    // --heap-pages
    // 4096
    // --output
    // ./pallets/domains/src/weights.rs
    // --template
    // ./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_domains.
pub trait WeightInfo {
    fn register_domain() -> Weight;
    fn force_register_domain() -> Weight;
    fn set_inner_value() -> Weight;
    fn force_set_inner_value() -> Weight;
    fn set_outer_value() -> Weight;
    fn set_domain_content() -> Weight;
    fn reserve_words(s: u32, ) -> Weight;
    fn support_tlds(s: u32, ) -> Weight;
}

/// Weights for pallet_domains using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            // Storage: Domains SupportedTlds (r:1 w:0)
            // Storage: Domains ReservedWords (r:1 w:0)
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainsByOwner (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
        fn register_domain() -> Weight {
        (44_166_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
        }
            // Storage: Domains SupportedTlds (r:1 w:0)
            // Storage: Domains ReservedWords (r:1 w:0)
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainsByOwner (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
        fn force_register_domain() -> Weight {
        (32_806_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainByInnerValue (r:0 w:2)
        fn set_inner_value() -> Weight {
        (25_249_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainByInnerValue (r:0 w:2)
        fn force_set_inner_value() -> Weight {
        (24_413_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
        fn set_outer_value() -> Weight {
        (32_353_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
        fn set_domain_content() -> Weight {
        (20_776_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
        }
            // Storage: Domains ReservedWords (r:0 w:1)
        fn reserve_words(s: u32, ) -> Weight {
        (12_241_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((1_637_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
        }
            // Storage: Domains SupportedTlds (r:0 w:1)
        fn support_tlds(s: u32, ) -> Weight {
        (12_742_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((1_561_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            // Storage: Domains SupportedTlds (r:1 w:0)
            // Storage: Domains ReservedWords (r:1 w:0)
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainsByOwner (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
        fn register_domain() -> Weight {
        (44_166_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
        }
            // Storage: Domains SupportedTlds (r:1 w:0)
            // Storage: Domains ReservedWords (r:1 w:0)
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainsByOwner (r:1 w:1)
            // Storage: Timestamp Now (r:1 w:0)
        fn force_register_domain() -> Weight {
        (32_806_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainByInnerValue (r:0 w:2)
        fn set_inner_value() -> Weight {
        (25_249_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
            // Storage: Domains DomainByInnerValue (r:0 w:2)
        fn force_set_inner_value() -> Weight {
        (24_413_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
        fn set_outer_value() -> Weight {
        (32_353_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: Domains RegisteredDomains (r:1 w:1)
        fn set_domain_content() -> Weight {
        (20_776_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
        }
            // Storage: Domains ReservedWords (r:0 w:1)
        fn reserve_words(s: u32, ) -> Weight {
        (12_241_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((1_637_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
        }
            // Storage: Domains SupportedTlds (r:0 w:1)
        fn support_tlds(s: u32, ) -> Weight {
        (12_742_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((1_561_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
        }
    }
