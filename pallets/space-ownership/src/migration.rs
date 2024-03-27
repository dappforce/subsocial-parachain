// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{log, traits::OnRuntimeUpgrade};
#[cfg(feature = "try-runtime")]
use sp_runtime::traits::Zero;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use super::*;

const LOG_TARGET: &'static str = "runtime::ownership";

pub mod v1 {
    use frame_support::{
        pallet_prelude::*, storage_alias, weights::Weight,
    };
    use sp_io::hashing::twox_128;
    use sp_io::KillStorageResult;

    use subsocial_support::SpaceId;

    use super::*;

    #[storage_alias]
    pub type PendingSpaceOwner<T: Config> =
        StorageMap<Pallet<T>, Twox64Concat, SpaceId, <T as frame_system::Config>::AccountId>;

    pub struct MigrateToV1<T, P, N>(sp_std::marker::PhantomData<(T, P, N)>);

    impl<T: Config, P: GetStorageVersion + PalletInfoAccess, N: Get<&'static str>> OnRuntimeUpgrade
        for MigrateToV1<T, P, N>
    {
        fn on_runtime_upgrade() -> Weight {
            let current_version = Pallet::<T>::current_storage_version();

            let old_pallet_name = N::get();
            let old_pallet_prefix = twox_128(old_pallet_name.as_bytes());
            let old_pallet_has_data = sp_io::storage::next_key(&old_pallet_prefix).is_some();

            let new_pallet_name = <P as PalletInfoAccess>::name();

            log::info!(
                target: LOG_TARGET,
                "Running migration to clean-up the old pallet name {}",
                old_pallet_name,
            );

            if old_pallet_has_data {
                if new_pallet_name == old_pallet_name {
                    log::warn!(
                        target: LOG_TARGET,
                        "new ownership name is equal to the old one, migration won't run"
                    );
                    return T::DbWeight::get().reads(1)
                }

                current_version.put::<Pallet<T>>();

                match sp_io::storage::clear_prefix(&old_pallet_prefix, None) {
                    KillStorageResult::SomeRemaining(remaining) => {
                        log::warn!(
                            target: LOG_TARGET,
                            "Some records from the old pallet {} have not been removed: {:?}",
                            old_pallet_name,
                            remaining
                        );
                    }
                    KillStorageResult::AllRemoved(removed) => {
                        log::info!(
                            target: LOG_TARGET,
                            "Removed {} records from the old pallet {}",
                            removed,
                            old_pallet_name
                        );
                    },
                }

                <T as frame_system::Config>::BlockWeights::get().max_block
            } else {
                log::info!(
                    target: LOG_TARGET,
                    "Migration did not execute. v1 upgrade should be removed"
                );
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            let old_pallet_name = N::get().as_bytes();
            let old_pallet_prefix = twox_128(old_pallet_name);

            ensure!(
                sp_io::storage::next_key(&old_pallet_prefix).is_some(),
                "no data for the old pallet name has been detected; consider removing the migration"
            );

            Ok(Vec::new())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
            let old_pallet_name = N::get();
            let new_pallet_name = <P as PalletInfoAccess>::name();

            // skip storage prefix checks for the same pallet names
            if new_pallet_name == old_pallet_name {
                return Ok(());
            }

            // Assert that nothing remains at the old prefix.
            let old_pallet_prefix = twox_128(N::get().as_bytes());
            let old_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
                old_pallet_prefix.to_vec(),
                old_pallet_prefix.to_vec(),
                |_| Ok(()),
            );
            ensure!(
                old_pallet_prefix_iter.count().is_zero(),
                "old pallet data hasn't been removed"
            );

            // Assert nothing redundant is left in the new prefix.
            // NOTE: storage_version_key is already in the new prefix.
            let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
            let new_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
                new_pallet_prefix.to_vec(),
                new_pallet_prefix.to_vec(),
                |_| Ok(()),
            );
            assert_eq!(new_pallet_prefix_iter.count(), 1);

            Ok(())
        }
    }
}
