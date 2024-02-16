use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn setting_letting_agent_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PropertyManagement::set_letting_agent(
			RuntimeOrigin::root(),
			0,
			0,
			[0; 32].into(),
		));
		assert_eq!(PropertyManagement::letting_storage(0,0), Some([0; 32].into()));
	});
}

#[test]
fn let_letting_agent_stake() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
	});
}

#[test]
fn slash_letting_agent_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
		assert_ok!(PropertyManagement::slash_letting_agent(RuntimeOrigin::signed([1; 32].into()), 50, [0; 32].into()));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
	});
}

#[test]
fn distribute_income_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 20));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 30));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([3; 32].into()), 0, 50));
		assert_ok!(PropertyManagement::distribute_income(RuntimeOrigin::signed([4; 32].into()), 0, 200));
		assert_eq!(PropertyManagement::stored_funds::<AccountId>([1; 32].into()), 40);
		assert_eq!(PropertyManagement::stored_funds::<AccountId>([2; 32].into()), 60);
		assert_eq!(PropertyManagement::stored_funds::<AccountId>([3; 32].into()), 100);
		assert_eq!(Balances::free_balance(&([4; 32].into())), 4800);
	});
}

#[test]
fn withdraw_funds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(PropertyManagement::distribute_income(RuntimeOrigin::signed([4; 32].into()), 0, 200));
		assert_eq!(PropertyManagement::stored_funds::<AccountId>([1; 32].into()), 200);
		assert_eq!(Balances::free_balance(&([4; 32].into())), 4800);
		assert_eq!(Balances::free_balance(&PropertyManagement::account_id()), 5200);
		assert_ok!(PropertyManagement::withdraw_funds(RuntimeOrigin::signed([1; 32].into())));
		assert_eq!(Balances::free_balance(&PropertyManagement::account_id()), 5000);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_200);
	});
}

