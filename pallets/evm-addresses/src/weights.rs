
//! Weights placeholder for evm-addresses pallet
// Executed Command:
// ./scripts/../target/release/subsocial-collator
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet
// pallet_evm_addresses
// --extrinsic
// *
// --execution=wasm
// --wasm-execution=Compiled
// --heap-pages=4096
// --output=pallets/evm-addresses/src/weights.rs
// --template=./.maintain/weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_evm_addresses.
pub trait WeightInfo {
    fn link_evm_address() -> Weight;
    fn unlink_evm_address() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    /// Storage: EvmAddresses AccountsByEvmAddress (r:1 w:1)
    /// Proof Skipped: EvmAddresses AccountsByEvmAddress (max_values: None, max_size: None, mode: Measured)
    /// Storage: EvmAddresses EvmAddressByAccount (r:0 w:1)
    /// Proof Skipped: EvmAddresses EvmAddressByAccount (max_values: None, max_size: None, mode: Measured)
    fn link_evm_address() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `45`
        //  Estimated: `7148`
        // Minimum execution time: 52_000_000 picoseconds.
        Weight::from_parts(53_000_000, 7148)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    /// Storage: EvmAddresses AccountsByEvmAddress (r:1 w:1)
    /// Proof Skipped: EvmAddresses AccountsByEvmAddress (max_values: None, max_size: None, mode: Measured)
    /// Storage: EvmAddresses EvmAddressByAccount (r:0 w:1)
    /// Proof Skipped: EvmAddresses EvmAddressByAccount (max_values: None, max_size: None, mode: Measured)
    fn unlink_evm_address() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `135`
        //  Estimated: `3735`
        // Minimum execution time: 13_000_000 picoseconds.
        Weight::from_parts(14_000_000, 3735)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}