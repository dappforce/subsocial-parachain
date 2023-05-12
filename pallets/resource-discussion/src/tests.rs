use frame_support::assert_noop;
use sp_runtime::DispatchError::BadOrigin;
use sp_std::convert::TryInto;

use crate::mock::*;

// fn account(v)
//
// #[test]
// fn create_resource_post_should_fail_if_not_signed() {
//     ExtBuilder::default().build().execute_with(|| {
//         assert_noop!(
//             ResourceCommenting::create_resource_post(
//                 RuntimeOrigin::none(),
//                 b"test".to_vec().try_into().unwrap()
//             ),
//             BadOrigin,
//         );
//     });
// }
