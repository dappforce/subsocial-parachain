use codec::{Decode, Encode};
use frame_support::{dispatch::TypeInfo, RuntimeDebug};

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
/// Settings for following a space.
pub struct SpaceFollowSettings<Balance> {
    pub subscription: Option<Balance>,
}
