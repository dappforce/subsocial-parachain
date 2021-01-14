use crate::{SpaceId};

pub trait IsAccountBlocked {
    type AccountId;

    fn is_account_blocked(account: Self::AccountId, scope: SpaceId) -> bool;
}

pub trait IsSpaceBlocked {
    type SpaceId;

    fn is_space_blocked(space_id: Self::SpaceId, scope: SpaceId) -> bool;
}

pub trait IsPostBlocked {
    type PostId;

    fn is_post_blocked(post_id: Self::PostId, scope: SpaceId) -> bool;
}

pub trait IsContentBlocked {
    type Content;

    fn is_content_blocked(content: Self::Content, scope: SpaceId) -> bool;
}
