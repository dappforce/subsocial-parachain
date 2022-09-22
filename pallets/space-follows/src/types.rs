use codec::{Decode, Encode};
use frame_support::{dispatch::TypeInfo, RuntimeDebug};

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
/// Settings for following a space.
pub struct SpaceFollowSettings<Balance> {
    /// The balance required to subscribe to a space.
    pub subscription: Option<Balance>,
}

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
/// Information about a space subscriber.
pub struct SpaceSubscriberInfo<Balance, BlockNumber> {
    /// The block number where the subscription becomes valid.
    pub subscribed_on: BlockNumber,
    /// The block number where the subscription becomes invalid.
    pub expires_on: Option<BlockNumber>,
    /// The amount paid for this subscription.
    pub subscription: Balance,
}
