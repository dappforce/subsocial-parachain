//! # Moderation Module
//!
//! The Moderation module allows any user (account) to report an account, space, post or even
//! IPFS CID, if they think it's a spam, abuse or inappropriate for a specific space.
//!
//! Moderators of a space can review reported entities and suggest a moderation status for them:
//! `Block` or `Allowed`. A space owner can make a final decision: either block or allow any entity
//! within the space they control.
//!
//! This pallet also has a setting to auto-block the content after a specific number of statuses
//! from moderators that suggest to block the entity. If the entity is added to allow list,
//! then the entity cannot be blocked.
//!
//! The next rules applied to the blocked entities:
//!
//! - A post cannot be added to a space if an IPFS CID of this post is blocked in this space.
//! - An account cannot create posts in a space if this account is blocked in this space.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure,
    dispatch::DispatchResult,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};

use subsocial_support::{Content, new_who_and_when, PostId, SpaceId, ensure_content_is_valid, ensure_content_is_some, WhoAndWhenOf};
use pallet_spaces::Module as Spaces;

pub use pallet::*;


// // TODO: move all tests to df-integration-tests
// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

pub mod functions;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;

    pub type ReportId = u64;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EntityId<AccountId> {
        Content(Content),
        Account(AccountId),
        Space(SpaceId),
        Post(PostId),
    }

    /// Entity status is used in two cases: when moderators suggest a moderation status
    /// for a reported entity; or when a space owner makes a final decision to either block
    /// or allow this entity within the space.
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum EntityStatus {
        Allowed,
        Blocked,
    }

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Report<T: Config> {
        pub(crate) id: ReportId,
        pub(crate) created: WhoAndWhenOf<T>,
        /// An id of reported entity: account, space, post or IPFS CID.
        pub(crate) reported_entity: EntityId<T::AccountId>,
        /// Within what space (scope) this entity has been reported.
        pub(crate) reported_within: SpaceId, // TODO rename: reported_in_space
        /// A reason should describe why this entity should be blocked in this space.
        pub(crate) reason: Content,
    }

    // TODO rename to SuggestedEntityStatus
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct SuggestedStatus<T: Config> {
        /// An account id of a moderator who suggested this status.
        pub(crate) suggested: WhoAndWhenOf<T>,
        /// `None` if a moderator wants to signal that they have reviewed the entity,
        /// but they are not sure about what status should be applied to it.
        pub(crate) status: Option<EntityStatus>,
        /// `None` if a suggested status is not based on any reports.
        pub(crate) report_id: Option<ReportId>,
    }

    // TODO rename to ModerationSettings?
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct SpaceModerationSettings {
        pub(crate) autoblock_threshold: Option<u16>
    }

    // TODO rename to ModerationSettingsUpdate?
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct SpaceModerationSettingsUpdate {
        pub autoblock_threshold: Option<Option<u16>>
    }

    pub const FIRST_REPORT_ID: u64 = 1;


    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_posts::Config
        + pallet_spaces::Config
        + pallet_space_follows::Config
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type DefaultAutoblockThreshold: Get<u16>;
    }


    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::type_value]
    pub fn DefaultForNextReportId() -> PostId {
        FIRST_REPORT_ID
    }

    /// The next moderation report id.
    #[pallet::storage]
    #[pallet::getter(fn next_report_id)]
    pub type NextReportId<T: Config> = StorageValue<_, ReportId, ValueQuery, DefaultForNextReportId>;

    /// Report details by its id (key).
    #[pallet::storage]
    #[pallet::getter(fn report_by_id)]
    pub type ReportById<T: Config> = StorageMap<_, Twox64Concat, ReportId, Report<T>>;

    /// Report id if entity (key 1) was reported by a specific account (key 2)
    #[pallet::storage]
    #[pallet::getter(fn report_id_by_account)]
    pub type ReportIdByAccount<T: Config> = StorageMap<_, Twox64Concat, (EntityId<T::AccountId>, T::AccountId), ReportId>;

    /// Ids of all reports in this space (key).
    #[pallet::storage]
    #[pallet::getter(fn report_ids_by_space_id)]
    pub type ReportIdsBySpaceId<T: Config> = StorageMap<_, Twox64Concat, SpaceId, Vec<ReportId>, ValueQuery>;

    /// Ids of all reports related to a specific entity (key 1) sent to this space (key 2).
    #[pallet::storage]
    #[pallet::getter(fn report_ids_by_entity_in_space)]
    pub type ReportIdsByEntityInSpace<T: Config> = StorageDoubleMap<_,
        Twox64Concat,
        EntityId<T::AccountId>,
        Twox64Concat,
        SpaceId,
        Vec<ReportId>,
        ValueQuery,
    >;

    /// An entity (key 1) status (`Blocked` or `Allowed`) in this space (key 2).
    #[pallet::storage]
    #[pallet::getter(fn status_by_entity_in_space)]
    pub type StatusByEntityInSpace<T: Config> = StorageDoubleMap<_,
        Twox64Concat,
        EntityId<T::AccountId>,
        Twox64Concat,
        SpaceId,
        EntityStatus,
    >;

    /// Entity (key 1) statuses suggested by space (key 2) moderators.
    #[pallet::storage]
    #[pallet::getter(fn suggested_statuses)]
    pub type SuggestedStatusesByEntityInSpace<T: Config> = StorageDoubleMap<_,
        Twox64Concat,
        EntityId<T::AccountId>,
        Twox64Concat,
        SpaceId,
        Vec<SuggestedStatus<T>>,
        ValueQuery,
    >;

    /// A custom moderation settings for a certain space (key).
    #[pallet::storage]
    #[pallet::getter(fn moderation_settings)]
    pub type ModerationSettings<T: Config> = StorageMap<_, Twox64Concat, SpaceId, SpaceModerationSettings>;

    // The pallet's events
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        EntityReported(T::AccountId, SpaceId, EntityId<T::AccountId>, ReportId),
        EntityStatusSuggested(T::AccountId, SpaceId, EntityId<T::AccountId>, Option<EntityStatus>),
        EntityStatusUpdated(T::AccountId, SpaceId, EntityId<T::AccountId>, Option<EntityStatus>),
        EntityStatusDeleted(T::AccountId, SpaceId, EntityId<T::AccountId>),
        ModerationSettingsUpdated(T::AccountId, SpaceId),
    }

    // The pallet's errors
    #[pallet::error]
    pub enum Error<T> {
        /// The account has already reported this entity.
        AlreadyReportedEntity,
        /// The entity has no status in this space. Nothing to delete.
        EntityHasNoStatusInScope,
        /// Entity scope differs from the scope provided.
        EntityNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity status is already as suggested one.
        SuggestedSameEntityStatus,
        /// Provided entity scope does not exist.
        ScopeNotFound,
        /// Account does not have a permission to suggest a new entity status.
        NoPermissionToSuggestEntityStatus,
        /// Account does not have a permission to update an entity status.
        NoPermissionToUpdateEntityStatus,
        /// Account does not have a permission to update the moderation settings.
        NoPermissionToUpdateModerationSettings,
        /// No updates provided for the space settings.
        NoUpdatesForModerationSettings,
        /// Report reason should not be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
        /// Trying to suggest an entity status in a scope that is different from the scope
        /// the entity was reported in.
        SuggestedStatusInWrongScope,
        /// Entity status has already been suggested by this moderator account.
        AlreadySuggestedEntityStatus,
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Report any entity by any person with mandatory reason.
        /// `entity` scope and the `scope` provided mustn't differ
        #[pallet::weight((Weight::from_ref_time(10_000) + T::DbWeight::get().reads_writes(6, 5)))]
        pub fn report_entity(
            origin: OriginFor<T>,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            reason: Content
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO check this func, if looks strange
            ensure_content_is_some(&reason).map_err(|_| Error::<T>::ReasonIsEmpty)?;

            ensure_content_is_valid(reason.clone())?;

            ensure!(Spaces::<T>::require_space(scope).is_ok(), Error::<T>::ScopeNotFound);
            Self::ensure_entity_in_scope(&entity, scope)?;

            let not_reported_yet = Self::report_id_by_account((&entity, &who)).is_none();
            ensure!(not_reported_yet, Error::<T>::AlreadyReportedEntity);

            let report_id = Self::next_report_id();
            let new_report = Report::<T>::new(report_id, who.clone(), entity.clone(), scope, reason);

            ReportById::<T>::insert(report_id, new_report);
            ReportIdByAccount::<T>::insert((&entity, &who), report_id);
            ReportIdsBySpaceId::<T>::mutate(scope, |ids| ids.push(report_id));
            ReportIdsByEntityInSpace::<T>::mutate(&entity, scope, |ids| ids.push(report_id));
            NextReportId::<T>::mutate(|n| { *n += 1; });

            Self::deposit_event(Event::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

        /// Leave a feedback on the report either it's confirmation or ignore.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        #[pallet::weight(Weight::from_ref_time(10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */))]
        pub fn suggest_entity_status(
            origin: OriginFor<T>,
            entity: EntityId<T::AccountId>,
            scope: SpaceId, // TODO make scope as Option, but either scope or report_id_opt should be Some
            status: Option<EntityStatus>,
            report_id_opt: Option<ReportId>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if let Some(report_id) = report_id_opt {
                let report = Self::require_report(report_id)?;
                ensure!(scope == report.reported_within, Error::<T>::SuggestedStatusInWrongScope);
            }

            let entity_status = StatusByEntityInSpace::<T>::get(&entity, scope);
            ensure!(!(entity_status.is_some() && status == entity_status), Error::<T>::SuggestedSameEntityStatus);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::SuggestEntityStatus,
                Error::<T>::NoPermissionToSuggestEntityStatus.into(),
            )?;

            let mut suggestions = SuggestedStatusesByEntityInSpace::<T>::get(&entity, scope);
            let is_already_suggested = suggestions.iter().any(|suggestion| suggestion.suggested.account == who);
            ensure!(!is_already_suggested, Error::<T>::AlreadySuggestedEntityStatus);
            suggestions.push(SuggestedStatus::new(who.clone(), status.clone(), report_id_opt));

            let block_suggestions_total = suggestions.iter()
                .filter(|suggestion| suggestion.status == Some(EntityStatus::Blocked))
                .count();

            let autoblock_threshold_opt = Self::moderation_settings(scope)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings)
                .autoblock_threshold;

            if let Some(autoblock_threshold) = autoblock_threshold_opt {
                if block_suggestions_total >= autoblock_threshold as usize {
                    Self::block_entity_in_scope(&entity, scope)?;
                }
            }

            SuggestedStatusesByEntityInSpace::<T>::insert(entity.clone(), scope, suggestions);

            Self::deposit_event(Event::EntityStatusSuggested(who, scope, entity, status));
            Ok(())
        }

        /// Allows a space owner/admin to update the final moderation status of a reported entity.
        #[pallet::weight(Weight::from_ref_time(10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */))]
        pub fn update_entity_status(
            origin: OriginFor<T>,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            status_opt: Option<EntityStatus>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: add `forbid_content` parameter and track entity Content blocking via OCW
            //  - `forbid_content` - whether to block `Content` provided with entity.

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Self::ensure_account_status_manager(who.clone(), &space)?;

            if let Some(status) = &status_opt {
                let is_entity_in_scope = Self::ensure_entity_in_scope(&entity, scope).is_ok();

                if is_entity_in_scope && status == &EntityStatus::Blocked {
                    Self::block_entity_in_scope(&entity, scope)?;
                } else {
                    StatusByEntityInSpace::<T>::insert(entity.clone(), scope, status);
                }
            } else {
                StatusByEntityInSpace::<T>::remove(entity.clone(), scope);
            }

            Self::deposit_event(Event::EntityStatusUpdated(who, scope, entity, status_opt));
            Ok(())
        }

        /// Allows a space owner/admin to delete a current status of a reported entity.
        #[pallet::weight(Weight::from_ref_time(10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */))]
        pub fn delete_entity_status(
            origin: OriginFor<T>,
            entity: EntityId<T::AccountId>,
            scope: SpaceId
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let status = Self::status_by_entity_in_space(&entity, scope);
            ensure!(status.is_some(), Error::<T>::EntityHasNoStatusInScope);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::ScopeNotFound)?;
            Self::ensure_account_status_manager(who.clone(), &space)?;

            StatusByEntityInSpace::<T>::remove(&entity, scope);

            Self::deposit_event(Event::EntityStatusDeleted(who, scope, entity));
            Ok(())
        }

        // todo: add ability to delete report_ids

        // TODO rename to update_settings?
        #[pallet::weight(Weight::from_ref_time(10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */))]
        pub fn update_moderation_settings(
            origin: OriginFor<T>,
            space_id: SpaceId,
            update: SpaceModerationSettingsUpdate
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let has_updates = update.autoblock_threshold.is_some();
            ensure!(has_updates, Error::<T>::NoUpdatesForModerationSettings);

            let space = Spaces::<T>::require_space(space_id)?;

            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::UpdateSpaceSettings,
                Error::<T>::NoPermissionToUpdateModerationSettings.into(),
            )?;

            // `true` if there is at least one updated field.
            let mut should_update = false;

            let mut settings = Self::moderation_settings(space_id)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings);

            if let Some(autoblock_threshold) = update.autoblock_threshold {
                if autoblock_threshold != settings.autoblock_threshold {
                    settings.autoblock_threshold = autoblock_threshold;
                    should_update = true;
                }
            }

            if should_update {
                ModerationSettings::<T>::insert(space_id, settings);
                Self::deposit_event(Event::ModerationSettingsUpdated(who, space_id));
            }
            Ok(())
        }
    }
}