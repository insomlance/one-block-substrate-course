use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;

#[test]
fn create_claim_test(){
	new_test_ext().execute_with(|| {
		let claim=vec![6,6,6];
		let origin=Origin::signed(6);
		assert_ok!(PoeModule::create_claim(origin.clone(),claim.clone()));

		assert_noop!(PoeModule::create_claim(origin.clone(),claim.clone()),Error::<Test>::ProofExist);

		assert_eq!(Proofs::<Test>::get(&claim),
					Some((6,frame_system::Pallet::<Test>::block_number()))
				);
	});
}

#[test]
fn revoke_claim_test(){
	new_test_ext().execute_with(|| {
		let claim=vec![6,6,6];
		let origin=Origin::signed(6);
		let another_origin=Origin::signed(7);

		assert_noop!(PoeModule::revoke_claim(origin.clone(),claim.clone()),Error::<Test>::ProofNotExist);

		PoeModule::create_claim(origin.clone(),claim.clone());

		assert_noop!(PoeModule::revoke_claim(another_origin.clone(),claim.clone()),Error::<Test>::InconsistentOwner);
		assert_ok!(PoeModule::revoke_claim(origin.clone(),claim.clone()));
	});
}

#[test]
fn transfer_claim_test(){
	new_test_ext().execute_with(||{
		let claim=vec![7,7,7];
		let origin=Origin::signed(13);
		let error_origin=Origin::signed(14);
		let target= 14;
		PoeModule::create_claim(origin.clone(),claim.clone());
		assert_noop!(PoeModule::transfer_claim(error_origin,target.clone(),claim.clone()),
					Error::<Test>::InconsistentOwner);
		PoeModule::transfer_claim(origin,target,claim.clone());
		assert_eq!(Proofs::<Test>::get(&claim),
					Some((14,frame_system::Pallet::<Test>::block_number()))
		);

	});

#[test]
fn create_claim_with_check_test(){
	new_test_ext().execute_with(|| {
		let mut claim=vec![6,6,6,1,1,1,1,1,1,1];
		let origin=Origin::signed(6);
		assert_ok!(PoeModule::create_claim(origin.clone(),claim.clone()));

		claim.push(7);

		assert_noop!(PoeModule::create_claim_with_check(Origin::signed(7),claim),Error::<Test>::ProofLengthTooLong);

	});
}

}