use frame_support::storage::IterableStorageMap;
use sp_runtime::traits::Saturating;

use pallet_utils as Utils;

use super::*;

pub fn fix_corrupted_handles_storage<T: Config>() -> frame_support::weights::Weight {
    let mut handles_iterated = 0;
    let mut should_remove;
    let mut removed = 0;

    for (handle, space_id) in SpaceIdByHandle::iter() {
        handles_iterated += 1;
        should_remove = false;

        if let Some(space) = Module::<T>::space_by_id(&space_id) {
            let space_handle_lc = space.handle.map(Utils::Module::<T>::lowercase_handle);

            if space_handle_lc.is_none() || space_handle_lc.as_ref() != Some(&handle) {
                should_remove = true;
            }
        } else {
            should_remove = true;
        }

        if should_remove {
            SpaceIdByHandle::remove(handle);
            removed += 1;
        }
    }

    SpaceIdByHandleStorageFixed::put(true);

    T::DbWeight::get().reads_writes(
        handles_iterated.saturating_mul(2),
        removed + 1,
    )
}
