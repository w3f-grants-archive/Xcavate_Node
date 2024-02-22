use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::Currency;

use pallet_balances::Error as BalancesError;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn add_letting_agent_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		));
		assert_eq!(PropertyManagement::letting_info::<AccountId>([0; 32].into()).is_some(), true);
	});
}

#[test]
fn add_letting_agent_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		), Error::<Test>::LocationUnknown);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		));
		assert_eq!(PropertyManagement::letting_info::<AccountId>([0; 32].into()).is_some(), true);
	});
}

 #[test]
fn let_letting_agent_deposit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
	});
}

#[test]
fn let_letting_agent_deposit_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())));
		assert_noop!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())), Error::<Test>::AlreadyDeposited);
		assert_noop!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([1; 32].into())), Error::<Test>::NoPermission);
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
		for x in 1..100 {
			assert_ok!(PropertyManagement::add_letting_agent(
				RuntimeOrigin::root(),
				0,
				[x; 32].into(),
			));
			Balances::make_free_balance_be(&[x; 32].into(), 200);
			assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([x; 32].into())));
		}
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[100; 32].into(),
		));
		Balances::make_free_balance_be(&[100; 32].into(), 200);
		assert_noop!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([100; 32].into())), Error::<Test>::TooManyLettingAgents);
	});
}

#[test]
fn let_letting_agent_deposit_not_enough_funds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[5; 32].into(),
		));
		assert_noop!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([5; 32].into())), BalancesError::<Test, _>::InsufficientBalance);
	});
}

#[test]
fn set_letting_agent_works() {
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
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 100));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 2, 100));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[2; 32].into(),
		));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[3; 32].into(),
		));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[4; 32].into(),
		));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([2; 32].into())));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([3; 32].into())));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([4; 32].into())));
		assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 0));
		assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 1));
		assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 2));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 3, 100));
		assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 3));
		assert_eq!(PropertyManagement::letting_storage(0).unwrap(), [2; 32].into());
		assert_eq!(PropertyManagement::letting_agent_locations(0).len(), 3);
		assert_eq!(PropertyManagement::letting_info::<AccountId>([2; 32].into()).unwrap().assigned_properties.len(), 2);
		assert_eq!(PropertyManagement::letting_info::<AccountId>([3; 32].into()).unwrap().assigned_properties.len(), 1);
		assert_eq!(PropertyManagement::letting_info::<AccountId>([4; 32].into()).unwrap().assigned_properties.len(), 1);
	});
}

#[test]
fn set_letting_agent_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(PropertyManagement::add_letting_agent(
			RuntimeOrigin::root(),
			0,
			[0; 32].into(),
		));
		assert_ok!(PropertyManagement::letting_agent_deposit(RuntimeOrigin::signed([0; 32].into())));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 19_999_900);
		assert_noop!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 0), Error::<Test>::NoNftFound);
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			100,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 0));
		assert_noop!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 0), Error::<Test>::LettingAgentAlreadySet);
		for x in 1..100 {
			assert_ok!(NftMarketplace::list_object(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				10_000,
				bvec![22, 22]
			));
			assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [(x+1); 32].into()));
			Balances::make_free_balance_be(&[x; 32].into(), 100_000);
			assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([x; 32].into()), (x as u32).into(), 100));
			assert_ok!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, x.into()));
		}
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			10_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 100, 100));
		assert_noop!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 100), Error::<Test>::TooManyAssignedProperties);
	});
}

#[test]
fn set_letting_agent_no_letting_agent() {
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
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 20));
		assert_noop!(PropertyManagement::set_letting_agent(RuntimeOrigin::signed([0; 32].into()), 0, 0), Error::<Test>::NoLettingAgentFound);
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
fn distribute_income_fails() {
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
		assert_noop!(PropertyManagement::distribute_income(RuntimeOrigin::signed([5; 32].into()), 0, 200), Error::<Test>::NotEnoughFunds);
		assert_eq!(PropertyManagement::stored_funds::<AccountId>([1; 32].into()), 0);
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

#[test]
fn withdraw_funds_fails() {
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
		assert_noop!(PropertyManagement::withdraw_funds(RuntimeOrigin::signed([2; 32].into())), Error::<Test>::UserHasNoFundsStored);
	});
}

