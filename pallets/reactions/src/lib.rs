// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use frame_system::ensure_signed;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::Deserialize;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::prelude::*;

use pallet_permissions::SpacePermission;
use pallet_posts::{Pallet as Posts, PostById};
use pallet_spaces::Pallet as Spaces;
use subsocial_support::{
    new_who_and_when, remove_from_vec, traits::IsAccountBlocked, ModerationError, PostId,
    WhoAndWhenOf,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

// pub mod rpc;

pub type ReactionId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum ReactionKind {
    Upvote,
    Downvote,
}

impl Default for ReactionKind {
    fn default() -> Self {
        ReactionKind::Upvote
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Reaction<T: Config> {
    /// Unique sequential identifier of a reaction. Examples of reaction ids: `1`, `2`, `3`,
    /// and so on.
    pub id: ReactionId,

    pub created: WhoAndWhenOf<T>,
    pub kind: ReactionKind,
}

pub const FIRST_REACTION_ID: u64 = 1;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use subsocial_support::WhoAndWhen;

    use crate::weights::WeightInfo;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config + pallet_spaces::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::type_value]
    pub fn DefaultForNextReactionId() -> ReactionId {
        FIRST_REACTION_ID
    }

    /// The next reaction id.
    #[pallet::storage]
    #[pallet::getter(fn next_reaction_id)]
    pub type NextReactionId<T: Config> =
        StorageValue<_, ReactionId, ValueQuery, DefaultForNextReactionId>;

    #[pallet::storage]
    #[pallet::getter(fn reaction_by_id)]
    pub type ReactionById<T: Config> = StorageMap<_, Twox64Concat, ReactionId, Reaction<T>>;

    #[pallet::storage]
    #[pallet::getter(fn reaction_ids_by_post_id)]
    pub type ReactionIdsByPostId<T: Config> =
        StorageMap<_, Twox64Concat, PostId, Vec<ReactionId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn post_reaction_id_by_account)]
    pub type PostReactionIdByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, PostId), ReactionId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PostReactionCreated {
            account: T::AccountId,
            post_id: PostId,
            reaction_id: ReactionId,
            reaction_kind: ReactionKind,
        },
        PostReactionUpdated {
            account: T::AccountId,
            post_id: PostId,
            reaction_id: ReactionId,
            reaction_kind: ReactionKind,
        },
        PostReactionDeleted {
            account: T::AccountId,
            post_id: PostId,
            reaction_id: ReactionId,
            reaction_kind: ReactionKind,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Reaction was not found by id.
        ReactionNotFound,
        /// Account has already reacted to this post/comment.
        AccountAlreadyReacted,
        /// There is no reaction by account on this post/comment.
        ReactionByAccountNotFound,
        /// Only reaction owner can update their reaction.
        NotReactionOwner,
        /// New reaction kind is the same as old one on this post/comment.
        SameReaction,
        /// `force_create_post_reaction` failed, because reaction already exists.
        /// Consider removing reaction first with `force_delete_post_reaction`.
        ReactionAlreadyExists,
        /// Reaction not found on post by provided [post_id] and [reaction_id].
        ReactionNotFoundOnPost,

        /// Not allowed to react on a post/comment in a hidden space.
        CannotReactWhenSpaceHidden,
        /// Not allowed to react on a post/comment if a root post is hidden.
        CannotReactWhenPostHidden,

        /// User has no permission to upvote posts/comments in this space.
        NoPermissionToUpvote,
        /// User has no permission to downvote posts/comments in this space.
        NoPermissionToDownvote,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(< T as Config >::WeightInfo::create_post_reaction())]
        pub fn create_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            kind: ReactionKind,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            let post = &mut Posts::require_post(post_id)?;
            ensure!(
                !<PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::AccountAlreadyReacted
            );

            let space = post.get_space()?;
            ensure!(!space.hidden, Error::<T>::CannotReactWhenSpaceHidden);
            ensure!(
                Posts::<T>::is_root_post_visible(post_id)?,
                Error::<T>::CannotReactWhenPostHidden
            );

            ensure!(
                T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id),
                ModerationError::AccountIsBlocked
            );

            match kind {
                ReactionKind::Upvote => {
                    Spaces::ensure_account_has_space_permission(
                        owner.clone(),
                        &post.get_space()?,
                        SpacePermission::Upvote,
                        Error::<T>::NoPermissionToUpvote.into(),
                    )?;
                    post.inc_upvotes();
                },
                ReactionKind::Downvote => {
                    Spaces::ensure_account_has_space_permission(
                        owner.clone(),
                        &post.get_space()?,
                        SpacePermission::Downvote,
                        Error::<T>::NoPermissionToDownvote.into(),
                    )?;
                    post.inc_downvotes();
                },
            }

            PostById::<T>::insert(post_id, post.clone());
            let reaction_id = Self::insert_new_reaction(owner.clone(), kind);
            ReactionIdsByPostId::<T>::mutate(post.id, |ids| ids.push(reaction_id));
            PostReactionIdByAccount::<T>::insert((owner.clone(), post_id), reaction_id);

            Self::deposit_event(Event::PostReactionCreated {
                account: owner,
                post_id,
                reaction_id,
                reaction_kind: kind,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(< T as Config >::WeightInfo::update_post_reaction())]
        pub fn update_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            reaction_id: ReactionId,
            new_kind: ReactionKind,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(
                <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::ReactionByAccountNotFound
            );

            let mut reaction = Self::require_reaction(reaction_id)?;
            let post = &mut Posts::require_post(post_id)?;

            ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
            ensure!(reaction.kind != new_kind, Error::<T>::SameReaction);

            if let Some(space_id) = post.try_get_space_id() {
                ensure!(
                    T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id),
                    ModerationError::AccountIsBlocked
                );
            }

            reaction.kind = new_kind;

            match new_kind {
                ReactionKind::Upvote => {
                    post.inc_upvotes();
                    post.dec_downvotes();
                },
                ReactionKind::Downvote => {
                    post.inc_downvotes();
                    post.dec_upvotes();
                },
            }

            ReactionById::<T>::insert(reaction_id, reaction);
            PostById::<T>::insert(post_id, post);

            Self::deposit_event(Event::PostReactionUpdated {
                account: owner,
                post_id,
                reaction_id,
                reaction_kind: new_kind,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(< T as Config >::WeightInfo::delete_post_reaction())]
        pub fn delete_post_reaction(
            origin: OriginFor<T>,
            post_id: PostId,
            reaction_id: ReactionId,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(
                <PostReactionIdByAccount<T>>::contains_key((owner.clone(), post_id)),
                Error::<T>::ReactionByAccountNotFound
            );

            // TODO extract Self::require_reaction(reaction_id)?;
            let reaction = Self::require_reaction(reaction_id)?;
            let post = &mut Posts::require_post(post_id)?;

            ensure!(owner == reaction.created.account, Error::<T>::NotReactionOwner);
            if let Some(space_id) = post.try_get_space_id() {
                ensure!(
                    T::IsAccountBlocked::is_allowed_account(owner.clone(), space_id),
                    ModerationError::AccountIsBlocked
                );
            }

            match reaction.kind {
                ReactionKind::Upvote => post.dec_upvotes(),
                ReactionKind::Downvote => post.dec_downvotes(),
            }

            PostById::<T>::insert(post_id, post.clone());
            ReactionById::<T>::remove(reaction_id);
            ReactionIdsByPostId::<T>::mutate(post.id, |ids| remove_from_vec(ids, reaction_id));
            PostReactionIdByAccount::<T>::remove((owner.clone(), post_id));

            Self::deposit_event(Event::PostReactionDeleted {
                account: owner,
                post_id,
                reaction_id,
                reaction_kind: reaction.kind,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((
            Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(2, 3),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_create_post_reaction(
            origin: OriginFor<T>,
            who: T::AccountId,
            post_id: PostId,
            reaction_id: ReactionId,
            created: WhoAndWhenOf<T>,
            reaction_kind: ReactionKind,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(Self::reaction_by_id(reaction_id).is_none(), Error::<T>::ReactionAlreadyExists);

            let WhoAndWhen { account, time, .. } = created;
            let new_who_and_when =
                WhoAndWhen { account, block: frame_system::Pallet::<T>::block_number(), time };

            let reaction =
                Reaction { id: reaction_id, created: new_who_and_when, kind: reaction_kind };
            ReactionById::<T>::insert(reaction_id, reaction);
            ReactionIdsByPostId::<T>::mutate(post_id, |ids| ids.push(reaction_id));
            PostReactionIdByAccount::<T>::insert((who.clone(), post_id), reaction_id);

            Self::deposit_event(Event::PostReactionCreated {
                account: who,
                post_id,
                reaction_id,
                reaction_kind,
            });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().reads_writes(3, 3),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_delete_post_reaction(
            origin: OriginFor<T>,
            reaction_id: ReactionId,
            post_id: PostId,
            who: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(Self::reaction_by_id(&reaction_id).is_some(), Error::<T>::ReactionNotFound);

            let post_reaction_id_by_account =
                Self::post_reaction_id_by_account((who.clone(), post_id));
            ensure!(
                post_reaction_id_by_account == reaction_id,
                Error::<T>::ReactionByAccountNotFound
            );

            ReactionIdsByPostId::<T>::try_mutate(post_id, |ids| -> DispatchResultWithPostInfo {
                ensure!(ids.contains(&reaction_id), Error::<T>::ReactionNotFoundOnPost);
                remove_from_vec(ids, reaction_id);
                Ok(Pays::No.into())
            })?;
            ReactionById::<T>::remove(reaction_id);
            PostReactionIdByAccount::<T>::remove((who, post_id));

            Ok(Pays::No.into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_set_next_reaction_id(
            origin: OriginFor<T>,
            reaction_id: PostId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            NextReactionId::<T>::put(reaction_id);
            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction { id, created: new_who_and_when::<T>(account), kind };

        ReactionById::<T>::insert(id, reaction);
        NextReactionId::<T>::mutate(|n| {
            *n += 1;
        });

        id
    }

    /// Get `Reaction` by id from the storage or return `ReactionNotFound` error.
    pub fn require_reaction(reaction_id: ReactionId) -> Result<Reaction<T>, DispatchError> {
        Ok(Self::reaction_by_id(reaction_id).ok_or(Error::<T>::ReactionNotFound)?)
    }
}
