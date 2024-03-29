#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


use codec::Codec;
use sp_std::vec::Vec;
use sp_std::collections::btree_map::BTreeMap;

use pallet_reactions::{
    ReactionId,
    ReactionKind,
    rpc::FlatReaction,
};
use pallet_utils::PostId;

sp_api::decl_runtime_apis! {
    pub trait ReactionsApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_reactions_by_ids(reaction_ids: Vec<ReactionId>) -> Vec<FlatReaction<AccountId, BlockNumber>>;

        fn get_reactions_by_post_id(
            post_id: PostId,
            limit: u64,
            offset: u64
        ) -> Vec<FlatReaction<AccountId, BlockNumber>>;

        fn get_reaction_kinds_by_post_ids_and_reactor(
            post_ids: Vec<PostId>,
            reactor: AccountId,
        ) -> BTreeMap<PostId, ReactionKind>;
    }
}
