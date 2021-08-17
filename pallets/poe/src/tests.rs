use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

//创建存证的成功用例
#[test]
fn create_claim_works() {
	new_test_ext().execute_with(||{
		let claim = vec![0; ClaimSize::get()];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((1,frame_system::Pallet::<Test>::block_number())
		));
	});
}


//创建已存在的存证的失败用例
#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

//吊销存证的成功用例
#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), None);
	})

}

//吊销存证当存证不存在的失败用例
#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

//吊销存证当调用者不是拥有者的失败用例
#[test]
fn revoke_claim_failed_when_origin_is_not_owner() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		let _  = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotProofOwner
		);
	})
}

//转移存证的成功用例
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::NotProofOwner
		);
	})	
}

//转移存证，当存证不存在的失败用例
#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
			Error::<Test>::ClaimNotExist
		);
	})
}

//转移存证，当调用者不是拥有者的失败用例
#[test]
fn transfer_claim_failed_when_origin_is_not_owner() {
	new_test_ext().execute_with(||{
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(3), claim.clone(), 2),
			Error::<Test>::NotProofOwner
		);
	})
}


//创建存证，当存证内容长度超过上限的失败用例
#[test]
fn create_claim_failed_when_size_too_large() {
	new_test_ext().execute_with(||{
		let claim = vec![0; ClaimSize::get() + 1];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimSizeTooLarge
		);
	});	
}