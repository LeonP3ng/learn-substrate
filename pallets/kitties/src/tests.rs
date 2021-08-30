use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

use super::*;

//创建Kitty的成功用例
#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(||{
        //let dna = KittiesModule::random_value(Origin::Signed(1));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_eq!(
			Owner::<Test>::get(1),
            Some(1)
		);
	});
}

//创建Kitty，当Kitty数量溢出的失败用例
#[test]
fn create_kitty_when_count_is_overflow() {
    new_test_ext().execute_with(|| {
		KittiesCount::<Test>::put(u32::max_value());
		assert_noop!(
			KittiesModule::create(Origin::signed(1)), 
			Error::<Test>::KittiesCountOverflow
		);
	});
}

//创建Kitty，当账户余额小于质押金额的失败用例
#[test]
fn create_kitty_when_money_is_not_enough() {
	new_test_ext().execute_with(||{
		assert_noop!(
			KittiesModule::create(Origin::signed(5)),
			Error::<Test>::MoneyIsNotEnough
		);
	});
}
//转移Kitty的成功用例
#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(
			KittiesModule::transfer(Origin::signed(1), 2, 1),
		);
		assert_eq!(
			Owner::<Test>::get(1),
			Some(2)
		);
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 3, 1),
			Error::<Test>::NotOwner
		);

	});
}
//转移Kitty，当调用者不是Kitty拥有者的失败用例
#[test]
fn transfer_kitty_when_is_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(2), 2, 1),
			Error::<Test>::NotOwner
		);
	});
}

//出售Kitty的成功用例
#[test]
fn sell_kitty_works() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell_kitty(Origin::signed(1), 1, 10_000_000_000));
	});
}
//出售Kitty,当KittyId拥有者的失败用例
#[test]
fn sell_kitty_when_kitty_id_is_invalid() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::sell_kitty(Origin::signed(1), 2, 10_000_000_000),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

//出售Kitty,当调用者不是Kitty拥有者的失败用例
#[test]
fn sell_kitty_when_is_not_owner() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::sell_kitty(Origin::signed(2), 1, 10_000_000_000),
			Error::<Test>::NotOwner
		);
	});
}

//购买Kitty的成功用例
#[test]
fn buy_kitty_works() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell_kitty(Origin::signed(1), 1, 10_000_000_000));
		assert_ok!(KittiesModule::buy_kitty(Origin::signed(2), 1, 20_000_000_000));
		assert_eq!(
			Owner::<Test>::get(1),
			Some(2)
		);
	});
}

//购买Kitty，当KittyId无效的失败用例
#[test]
fn buy_kitty_when_kitty_id_is_invalid() {
	new_test_ext().execute_with(||{
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(2), 1, 20_000_000_000),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

//购买Kitty，当买方和Kitty拥有者为同一人的失败用例
#[test]
fn buy_kitty_when_buyer_is_same_to_owner() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(1), 1, 20_000_000_000),
			Error::<Test>::InvalidKittyBuyer
		);
	});
}
//购买Kitty，当Kitty不处于售卖状态时的失败用例
#[test]
fn buy_kitty_when_kitty_is_not_on_sale(){
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(2), 1, 20_000_000_000),
			Error::<Test>::KittyIsNotOnSale
		);
	});
}

//购买Kitty，当付款金额小于售价的失败用例
#[test]
fn buy_kitty_when_pay_money_is_not_enough() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell_kitty(Origin::signed(1), 1, 10_000_000_000));
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(2), 1, 1_000_000_000),
			Error::<Test>::MoneyIsNotEnough
		);
	});
}

//购买Kitty，当付款金额大于售价，但小于账户余额的失败用例
#[test]
fn buy_kitty_when_banlance_money_is_not_enough() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::sell_kitty(Origin::signed(1), 1, 10_000_000_000));
		assert_noop!(
			KittiesModule::buy_kitty(Origin::signed(5), 1, 10_000_000_000),
			Error::<Test>::MoneyIsNotEnough
		);
	});
}

//繁殖Kitty的成功用例
#[test]
fn breed_kitty_works () {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::breed(Origin::signed(1), 1, 2));
		assert_eq!(KittiesCount::<Test>::get(), Some(4));
		assert_eq!(Owner::<Test>::get(3), Some(1));
	});
}

//繁殖Kitty，当父母为同一Kitty的失败用例
#[test]
fn breed_kitty_when_the_same_parent() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 1),
			Error::<Test>::SameParentIndex
		);
	});
}

//繁殖Kitty，当KittyId不存在的失败用例
#[test]
fn breed_kitty_when_kitty_id_is_invalid() {
	new_test_ext().execute_with(||{
		assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 2),
			Error::<Test>::InvalidKittyIndex
		);
	});
}

//繁殖Kitty，当Kitty数量溢出的失败用例
#[test]
fn breed_kitty_when_kitty_count_is_overflow() {
	new_test_ext().execute_with(||{
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		KittiesCount::<Test>::put(u32::max_value());
		assert_noop!(
			KittiesModule::breed(Origin::signed(1), 1, 2),
			Error::<Test>::KittiesCountOverflow
		);
	
	});
}