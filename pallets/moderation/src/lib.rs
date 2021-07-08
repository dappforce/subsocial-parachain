#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure,
    dispatch::DispatchResult,
    traits::Get,
};
use frame_system::{self as system, ensure_signed};

use pallet_utils::{Content, WhoAndWhen, SpaceId, Module as Utils};
use pallet_posts::PostId;
use pallet_spaces::Pallet as Spaces;

/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
*/

pub mod functions;

pub type ReportId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityId<AccountId> {
    Content(Content),
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityStatus {
    Allowed,
    Blocked,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Report<T: Config> {
    id: ReportId,
    created: WhoAndWhen<T>,
    reported_entity: EntityId<T::AccountId>,
    reported_within: SpaceId,
    reason: Content,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SuggestedStatus<T: Config> {
    suggested: WhoAndWhen<T>,
    status: Option<EntityStatus>,
    report_id: Option<ReportId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceModerationSettings {
    autoblock_threshold: Option<u16>
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceModerationSettingsUpdate {
    autoblock_threshold: Option<Option<u16>>
}

/// The pallet's configuration trait.
pub trait Config: system::Config
    + pallet_posts::Config
    + pallet_spaces::Config
    + pallet_space_follows::Config
    + pallet_utils::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    type DefaultAutoblockThreshold: Get<u16>;
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Config> as ModerationModule {
        NextReportId get(fn next_report_id): ReportId = 1;

        pub ReportById get(fn report_by_id):
            map hasher(twox_64_concat) ReportId => Option<Report<T>>;

        pub ReportIdByAccount get(fn report_id_by_account):
            map hasher(twox_64_concat) (EntityId<T::AccountId>, T::AccountId) => Option<ReportId>;

        pub ReportIdsBySpaceId: map hasher(twox_64_concat) SpaceId => Vec<ReportId>;

        pub ReportIdsByEntityInSpace get(fn report_ids_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
                => Vec<ReportId>;

        pub StatusByEntityInSpace get(fn status_by_entity_in_space): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
                => Option<EntityStatus>;

        pub SuggestedStatusesByEntityInSpace get(fn suggested_statuses): double_map
            hasher(twox_64_concat) EntityId<T::AccountId>,
            hasher(twox_64_concat) SpaceId
             => Vec<SuggestedStatus<T>>;

        pub SpaceSettings get(fn space_settings):
            map hasher(twox_64_concat) SpaceId => Option<SpaceModerationSettings>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Config>::AccountId,
        EntityId = EntityId<<T as system::Config>::AccountId>
    {
        EntityReported(AccountId, SpaceId, EntityId, ReportId),
        EntityStatusSuggested(AccountId, SpaceId, EntityId, Option<EntityStatus>),
        EntityStatusUpdated(AccountId, SpaceId, EntityId, Option<EntityStatus>),
        EntityStatusDeleted(AccountId, SpaceId, EntityId),
        SpaceSettingsUpdated(AccountId, SpaceId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Config> {
        /// The account has already made a report on this entity.
        AlreadyReported,
        /// Entity status in this space is not specified. Nothing to delete.
        EntityHasNoAnyStatusInScope,
        /// Entity scope differs from the scope provided.
        EntityIsNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity status is already as suggested one
        EntityStatusDoNotDiffer,
        /// Entity scope provided doesn't exist.
        InvalidScope,
        /// Account has no permission to suggest a new entity status.
        NoPermissionToSuggestEntityStatus,
        /// Account has no permission to update entity status.
        NoPermissionToUpdateEntityStatus,
        /// Account has no permission to update space settings.
        NoPermissionToUpdateSpaceSettings,
        /// No any updates provided for space settings.
        NoUpdatesForSpaceSettings,
        /// Report reason shouldn't be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
        /// The specified scope differs from ones within report was created
        ScopeDiffersFromReport,
        /// Entity status update is already suggested by this account
        SuggestionAlreadyCreated,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        const DefaultAutoblockThreshold: u16 = T::DefaultAutoblockThreshold::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Report any entity by any person with mandatory reason.
        /// `entity` scope and the `scope` provided mustn't differ
        #[weight = 10_000 + T::DbWeight::get().reads_writes(6, 5)]
        pub fn report_entity(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            reason: Content
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Utils::<T>::ensure_content_is_some(&reason).map_err(|_| Error::<T>::ReasonIsEmpty)?;
            Utils::<T>::is_valid_content(reason.clone())?;

            ensure!(Spaces::<T>::require_space(scope).is_ok(), Error::<T>::InvalidScope);
            Self::ensure_entity_in_scope(&entity, scope)?;

            ensure!(Self::report_id_by_account((&entity, &who)).is_none(), Error::<T>::AlreadyReported);

            let report_id = Self::next_report_id();
            let new_report = Report::<T>::new(report_id, who.clone(), entity.clone(), scope, reason);

            ReportById::<T>::insert(report_id, new_report);
            ReportIdByAccount::<T>::insert((&entity, &who), report_id);
            ReportIdsBySpaceId::mutate(scope, |ids| ids.push(report_id));
            ReportIdsByEntityInSpace::<T>::mutate(&entity, scope, |ids| ids.push(report_id));
            NextReportId::mutate(|n| { *n += 1; });

            Self::deposit_event(RawEvent::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

        /// Leave a feedback on the report either it's confirmation or ignore.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        #[weight = 10_000]
        pub fn suggest_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            status: Option<EntityStatus>,
            report_id_opt: Option<ReportId>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if let Some(report_id) = report_id_opt {
                let report = Self::require_report(report_id)?;
                ensure!(scope == report.reported_within, Error::<T>::ScopeDiffersFromReport);
            }

            let entity_status = StatusByEntityInSpace::<T>::get(&entity, scope);
            ensure!(!(entity_status.is_some() && status == entity_status), Error::<T>::EntityStatusDoNotDiffer);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::SuggestEntityStatus,
                Error::<T>::NoPermissionToSuggestEntityStatus.into(),
            )?;

            let mut suggestions = SuggestedStatusesByEntityInSpace::<T>::get(&entity, scope);
            let is_already_suggested = suggestions.iter().any(|suggestion| suggestion.suggested.account == who);
            ensure!(!is_already_suggested, Error::<T>::SuggestionAlreadyCreated);
            suggestions.push(SuggestedStatus::new(who.clone(), status.clone(), report_id_opt));

            let block_suggestions_total = suggestions.iter()
                .filter(|suggestion| suggestion.status == Some(EntityStatus::Blocked))
                .count();

            let autoblock_threshold_opt = Self::space_settings(scope)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings)
                .autoblock_threshold;

            if let Some(autoblock_threshold) = autoblock_threshold_opt {
                if block_suggestions_total >= autoblock_threshold as usize {
                    Self::block_entity_in_scope(&entity, scope)?;
                }
            }

            Self::deposit_event(RawEvent::EntityStatusSuggested(who, scope, entity.clone(), status));
            SuggestedStatusesByEntityInSpace::<T>::insert(entity, scope, suggestions);
            Ok(())
        }

        /// Block any `entity` provided.
        /// `origin` - any permitted account (e.g. Space owner or moderator that's set via role)
        #[weight = 10_000]
        pub fn update_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            status_opt: Option<EntityStatus>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: add `forbid_content` parameter and track entity Content blocking via OCW
            //  - `forbid_content` - whether to block `Content` provided with entity.

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
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

            Self::deposit_event(RawEvent::EntityStatusUpdated(who, scope, entity, status_opt));
            Ok(())
        }

        #[weight = 10_000]
        pub fn delete_entity_status(origin, entity: EntityId<T::AccountId>, scope: SpaceId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let status = Self::status_by_entity_in_space(&entity, scope);
            ensure!(status.is_some(), Error::<T>::EntityHasNoAnyStatusInScope);

            let space = Spaces::<T>::require_space(scope).map_err(|_| Error::<T>::InvalidScope)?;
            Self::ensure_account_status_manager(who.clone(), &space)?;

            StatusByEntityInSpace::<T>::remove(&entity, scope);

            Self::deposit_event(RawEvent::EntityStatusDeleted(who, scope, entity));
            Ok(())
        }

        // todo: add ability to delete report_ids

        #[weight = 10_000]
        fn update_space_settings(origin, space_id: SpaceId, update: SpaceModerationSettingsUpdate) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let has_updates = update.autoblock_threshold.is_some();
            ensure!(has_updates, Error::<T>::NoUpdatesForSpaceSettings);

            let space = Spaces::<T>::require_space(space_id)?;

            Spaces::<T>::ensure_account_has_space_permission(
                who.clone(),
                &space,
                pallet_permissions::SpacePermission::UpdateSpaceSettings,
                Error::<T>::NoPermissionToUpdateSpaceSettings.into(),
            )?;

            let mut is_updated = false;

            let mut space_settings = Self::space_settings(space_id)
                .unwrap_or_else(Self::default_autoblock_threshold_as_settings);

            if let Some(autoblock_threshold) = update.autoblock_threshold {
                if autoblock_threshold != space_settings.autoblock_threshold {
                    space_settings.autoblock_threshold = autoblock_threshold;
                    is_updated = true;
                }
            }

            if is_updated {
                SpaceSettings::insert(space_id, space_settings);
                Self::deposit_event(RawEvent::SpaceSettingsUpdated(who, space_id));
            }
            Ok(())
        }
    }
}
