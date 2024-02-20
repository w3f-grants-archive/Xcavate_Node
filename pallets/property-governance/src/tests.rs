use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn propose_works() {
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
		assert_ok!(PropertyGovernance::propose(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_eq!(PropertyGovernance::proposals(1).unwrap().asset_id, 0);
		assert_eq!(PropertyGovernance::ongoing_votes(1).is_some(), true);
	});
}

#[test]
fn propose_fails() {
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
		assert_noop!(PropertyGovernance::propose(RuntimeOrigin::signed([2; 32].into()), 0), Error::<Test>::NoPermission);
	});
}

#[test]
fn inquery_agains_letting_agent_works() {
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
		assert_ok!(PropertyGovernance::inquery_agains_letting_agent(RuntimeOrigin::signed([1; 32].into()), 0));
	});
}

#[test]
fn vote_on_proposal_works() {
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
		assert_ok!(PropertyGovernance::propose(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_ok!(PropertyGovernance::vote_on_proposal(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes));
	});
}

#[test]
fn vote_on_proposal_fails() {
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
		assert_noop!(PropertyGovernance::vote_on_proposal(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes), Error::<Test>::NotOngoing);
		assert_ok!(PropertyGovernance::propose(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_ok!(PropertyGovernance::vote_on_proposal(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes));
		assert_noop!(PropertyGovernance::vote_on_proposal(RuntimeOrigin::signed([2; 32].into()), 1, crate::Vote::Yes), Error::<Test>::NoPermission);
		assert_noop!(PropertyGovernance::vote_on_proposal(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes), Error::<Test>::AlreadyVoted);
	});
}

#[test]
fn vote_on_inquery_works() {
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
		assert_ok!(PropertyGovernance::inquery_agains_letting_agent(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_ok!(PropertyGovernance::vote_on_letting_agent_inquery(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes));
	});
}

#[test]
fn vote_on_inquery_fails() {
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
		assert_noop!(PropertyGovernance::vote_on_letting_agent_inquery(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes), Error::<Test>::NotOngoing);
		assert_ok!(PropertyGovernance::inquery_agains_letting_agent(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_ok!(PropertyGovernance::vote_on_letting_agent_inquery(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes));
		assert_noop!(PropertyGovernance::vote_on_letting_agent_inquery(RuntimeOrigin::signed([2; 32].into()), 1, crate::Vote::Yes), Error::<Test>::NoPermission);
		assert_noop!(PropertyGovernance::vote_on_letting_agent_inquery(RuntimeOrigin::signed([1; 32].into()), 1, crate::Vote::Yes), Error::<Test>::AlreadyVoted);
	});
}

