use frame_support::pallet_prelude::*;

use pallet_reactions::{ReactionId, ReactionKind};
use pallet_utils::PostId;

use crate::mock::*;
use crate::utils::{ACCOUNT1, POST1};

pub(crate) fn reaction_upvote() -> ReactionKind {
    ReactionKind::Upvote
}

pub(crate) fn reaction_downvote() -> ReactionKind {
    ReactionKind::Downvote
}


pub(crate) fn _create_default_post_reaction() -> DispatchResult {
    _create_post_reaction(None, None, None)
}

pub(crate) fn _create_default_comment_reaction() -> DispatchResult {
    _create_comment_reaction(None, None, None)
}

pub(crate) fn _create_post_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    Reactions::create_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        kind.unwrap_or_else(reaction_upvote),
    )
}

pub(crate) fn _create_comment_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    _create_post_reaction(origin, Some(post_id.unwrap_or(2)), kind)
}

pub(crate) fn _update_post_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    Reactions::update_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        reaction_id,
        kind.unwrap_or_else(reaction_upvote),
    )
}

pub(crate) fn _update_comment_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    _update_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id, kind)
}

pub(crate) fn _delete_post_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
) -> DispatchResult {
    Reactions::delete_post_reaction(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        reaction_id,
    )
}

pub(crate) fn _delete_comment_reaction(
    origin: Option<Origin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
) -> DispatchResult {
    _delete_post_reaction(origin, Some(post_id.unwrap_or(2)), reaction_id)
}