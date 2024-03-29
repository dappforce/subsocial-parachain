#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


use frame_support::dispatch::{DispatchError, DispatchResult};

use crate::{Content, PostId, SpaceId};

pub trait SpacePermissionsProvider<AccountId, SpacePermissionsInfo> {
    fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfo, DispatchError>;

    fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
}

pub trait SpaceFollowsProvider {
    type AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}

pub trait PostFollowsProvider {
    type AccountId;

    fn is_post_follower(account: Self::AccountId, post_id: PostId) -> bool;
}

pub trait ProfileManager<AccountId> {
    fn unlink_space_from_profile(account: &AccountId, space_id: SpaceId);
}

pub trait SpacesProvider<AccountId, SpaceId> {
    
    fn get_space_owner(space_id: SpaceId) -> Result<AccountId, DispatchError>;
    
    fn do_update_space_owner(space_id: SpaceId, new_owner: AccountId) -> DispatchResult;

    fn create_space(owner: &AccountId, content: Content) -> Result<SpaceId, DispatchError>;
}

pub trait CreatorStakingProvider<AccountId> {
    fn is_creator_active(
        creator_id: SpaceId,
    ) -> bool;
}

impl<AccountId> CreatorStakingProvider<AccountId> for () {
    fn is_creator_active(
        _creator_id: SpaceId,
    ) -> bool {
        false
    }
}

pub trait DomainsProvider<AccountId> {
    type MaxDomainLength: frame_support::traits::Get<u32>;
    
    fn get_domain_owner(domain: &[u8]) -> Result<AccountId, DispatchError>;
    
    fn ensure_domain_owner(domain: &[u8], account: &AccountId) -> DispatchResult;
    
    fn do_update_domain_owner(domain: &[u8], new_owner: &AccountId) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn register_domain(owner: &AccountId, domain: &[u8]) -> Result<sp_std::vec::Vec<u8>, DispatchError>;
}

pub trait PostsProvider<AccountId> {
    fn get_post_owner(post_id: PostId) -> Result<AccountId, DispatchError>;
    
    fn ensure_post_owner(post_id: PostId, account: &AccountId) -> DispatchResult;
    
    fn do_update_post_owner(post_id: PostId, new_owner: &AccountId) -> DispatchResult;

    #[cfg(feature = "runtime-benchmarks")]
    fn create_post(owner: &AccountId, space_id: SpaceId, content: Content) -> Result<PostId, DispatchError>;
}
