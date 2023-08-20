use crate::{mock::*, Error, Event};
use frame_support::{assert_ok, BoundedVec, assert_noop};
use sp_core::ConstU32;

#[test]
fn propose_works() {
 	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		System::assert_last_event(Event::Proposed {
			proposal_index: 0
		}.into());
	}) 
}

fn propose_doesnt_work_not_enough_userbalance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(CommunityLoanPool::propose(RuntimeOrigin::signed(DAVE), 100, BOB), Error::<Test>::InsufficientProposersBalance);
	}) 
}

#[test]
fn reject_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		assert_ok!(CommunityLoanPool::reject_proposal(RuntimeOrigin::root(), 0));
		System::assert_last_event(Event::Rejected {
			proposal_index: 0
		}.into());
	})
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		assert_ok!(CommunityLoanPool::approve_proposal(RuntimeOrigin::root(), 0, 0, 100, 0, BOB, ALICE));
	})
}

#[test]
fn approve_fails_invalid_index() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(CommunityLoanPool::approve_proposal(RuntimeOrigin::root(), 0, 0, 100, 0, BOB, ALICE), Error::<Test>::InvalidIndex);
	})
}