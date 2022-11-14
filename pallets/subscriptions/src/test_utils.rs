use frame_support::{traits::Get};
pub use paste::paste;
use sp_std::borrow::{Borrow, BorrowMut};
pub use frame_support::parameter_types;


#[macro_export]
macro_rules! clearable_parameter_type {
    ($vis:vis static $name:ident: $type:ty) => {
        paste::paste! {
			std::thread_local! { $vis static [<$name:snake:upper>]: std::cell::RefCell<Option<$type>> = std::cell::RefCell::new(None); }
			struct $name;
			impl $name {
				/// Returns the value of this parameter type.
				pub fn get() -> Option<$type> {
					[<$name:snake:upper>].with(|v| v.borrow().clone())
				}

				/// Clear the internal value.
				pub fn clear() {
					[<$name:snake:upper>].with(|v| *v.borrow_mut() = None);
				}

				/// Set the internal value.
				pub fn set(t: $type) {
					[<$name:snake:upper>].with(|v| *v.borrow_mut() = Some(t));
				}
			}
		}
    };
}


clearable_parameter_type!(pub static TestValue: u32);

#[test]
fn test() {
	assert_eq!(TestValue::get(), None);
	TestValue::set(121);
	assert_eq!(TestValue::get(), Some(121));
	TestValue::clear();
	assert_eq!(TestValue::get(), None);
}