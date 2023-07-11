use frame_support::{log, traits::OnRuntimeUpgrade};
use sp_runtime::{Saturating, traits::Zero};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use super::*;

pub mod v1 {
    use frame_support::{pallet_prelude::*, weights::Weight};

    use subsocial_support::WhoAndWhenOf;

    use crate::types::*;

    use super::*;

    // Old domain metadata
    #[derive(Decode)]
    pub struct OldDomainMeta<T: Config> {
        pub(super) created: WhoAndWhenOf<T>,
        pub(super) updated: Option<WhoAndWhenOf<T>>,
        pub(super) expires_at: T::BlockNumber,
        pub(super) owner: T::AccountId,
        pub(super) content: Content,
        pub(super) inner_value: Option<InnerValueOf<T>>,
        pub(super) outer_value: Option<OuterValue<T>>,
        pub(super) domain_deposit: BalanceOf<T>,
        pub(super) outer_value_deposit: BalanceOf<T>,
    }

    impl<T: Config> OldDomainMeta<T> {
        fn migrate_to_v1(self) -> DomainMeta<T> {
            let new_deposit = if self.domain_deposit.is_zero() {
                (self.owner.clone(), Zero::zero()).into()
            } else {
                (self.owner.clone(), self.domain_deposit).into()
            };

            DomainMeta {
                created: self.created,
                updated: self.updated,
                expires_at: self.expires_at,
                owner: self.owner,
                content: self.content,
                inner_value: self.inner_value,
                outer_value: self.outer_value,
                domain_deposit: new_deposit,
                outer_value_deposit: self.outer_value_deposit,
            }
        }
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        fn on_runtime_upgrade() -> Weight {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();

            log::info!(
				target: LOG_TARGET,
				"Running migration with current storage version {:?} / onchain {:?}",
				current_version,
				onchain_version
			);

            if onchain_version == 0 && current_version == 1 {
                let mut translated = 0u64;
                RegisteredDomains::<T>::translate::<
                    OldDomainMeta<T>,
                    _,
                >(|_key, old_value| {
                    translated.saturating_inc();
                    Some(old_value.migrate_to_v1())
                });

                current_version.put::<Pallet<T>>();

                log::info!(
					target: LOG_TARGET,
					"Upgraded {} records, storage to version {:?}",
					translated,
					current_version
				);
                T::DbWeight::get().reads_writes(translated + 1, translated + 1)
            } else {
                log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
                T::DbWeight::get().reads(1)
            }
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();
            ensure!(onchain_version == 0 && current_version == 1, "migration from version 0 to 1.");
            let prev_count = RegisteredDomains::<T>::iter().count();
            Ok((prev_count as u32).encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(prev_count: Vec<u8>) -> Result<(), &'static str> {
            let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
                "the state parameter should be something that was generated by pre_upgrade",
            );
            let post_count = RegisteredDomains::<T>::iter().count() as u32;
            ensure!(
				prev_count == post_count,
				"the records count before and after the migration should be the same"
			);

            ensure!(Pallet::<T>::on_chain_storage_version() == 1, "wrong storage version");

            Ok(())
        }
    }
}

