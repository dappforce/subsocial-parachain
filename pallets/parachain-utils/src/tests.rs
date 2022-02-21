use crate::{mock::*, remove_from_vec};

#[test]
fn remove_from_vec_should_work_with_zero_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![];

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn remove_from_vec_should_work_with_last_element() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2]);

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    });
}

#[test]
fn remove_from_vec_should_work_with_two_elements() {
    ExtBuilder::build().execute_with(|| {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2, 7];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2, 7]);

        remove_from_vec(vector, element);
        assert_eq!(vector, &mut vec![7]);
    });
}
