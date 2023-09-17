use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		System::assert_last_event(Event::Proposed { proposal_index: 1 }.into());
	})
}

#[test]
fn propose_doesnt_work_not_enough_userbalance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			CommunityLoanPool::propose(RuntimeOrigin::signed(DAVE), 100, BOB),
			Error::<Test>::InsufficientProposersBalance
		);
	})
}

#[test]
fn add_committee_member_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_eq!(CommunityLoanPool::voting_committee()[0], ALICE);
	})
}

#[test]
fn add_committee_member_fails_when_member_is_two_times_added() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_eq!(CommunityLoanPool::voting_committee()[0], ALICE);
		assert_noop!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE), Error::<Test>::AlreadyMember);
	})
}

#[test]
fn voting_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(BOB), 100, BOB));
		assert_ok!(CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Yes));
		assert_eq!(CommunityLoanPool::ongoing_votes(1).unwrap().yes_votes, 1);
	})
}

#[test]
fn voting_works_only_for_members() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(BOB), 100, BOB));
		assert_noop!(CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed(DAVE), 1, crate::Vote::Yes), Error::<Test>::InsufficientPermission);
	})
}

#[test]
fn reject_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		assert_ok!(CommunityLoanPool::reject_proposal(RuntimeOrigin::root(), 1));
		System::assert_last_event(Event::Rejected { proposal_index: 1 }.into());
	})
}

/* #[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE), 100, BOB));
		assert_ok!(CommunityLoanPool::approve_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			0,
			100,
			100,
			0,
			10,
			BOB,
			ALICE,
			None,
			5000000000.into(),
		));
	})
} */

#[test]
fn approve_fails_invalid_index() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			CommunityLoanPool::approve_proposal(
				RuntimeOrigin::signed(ALICE),
				0,
				0,
				100,
				100,
				0,
				10,
				BOB,
				ALICE,
				None,
				5000000000.into(),
			),
			Error::<Test>::InvalidIndex
		);
	})
}
