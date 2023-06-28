// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2023 DAPPFORCE PTE. Ltd., dappforce@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use pallet_reactions::ReactionId;
use subsocial_support::{PostId, SpaceId};
use crate::mock::AccountId;

pub(crate) mod spaces_utils;
pub(crate) mod permissions_utils;
pub(crate) mod moderation_utils;
pub(crate) mod roles_utils;
pub(crate) mod space_ownership_utils;
pub(crate) mod reactions_utils;
pub(crate) mod space_follows_utils;
pub(crate) mod posts_utils;


pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;
pub(crate) const SPACE2: SpaceId = 1002;

pub(crate) const POST1: PostId = 1;
pub(crate) const POST2: PostId = 2;
pub(crate) const POST3: PostId = 3;

pub(crate) const REACTION1: ReactionId = 1;
pub(crate) const REACTION2: ReactionId = 2;
