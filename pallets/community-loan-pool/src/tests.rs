use crate::{mock::*, Error, Event};
use frame_support::sp_runtime::Percent;
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};

use crate::Config;
use crate::{BoundedProposedMilestones, ProposedMilestone};

fn get_milestones(mut n: u32) -> BoundedProposedMilestones<Test> {
	let max = <Test as Config>::MaxMilestonesPerProject::get();
	if n > max {
		n = max
	}
	(0..n)
		.map(|_| ProposedMilestone { percentage_to_unlock: Percent::from_percent((100 / n) as u8) })
		.collect::<Vec<ProposedMilestone>>()
		.try_into()
		.expect("bound is ensured; qed")
}

fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 0 {
			CommunityLoanPool::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		CommunityLoanPool::on_initialize(System::block_number());
	}
}

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(ALICE),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		System::assert_last_event(Event::Proposed { proposal_index: 1 }.into());
	})
}

#[test]
fn propose_doesnt_work_not_enough_userbalance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			CommunityLoanPool::propose(
				RuntimeOrigin::signed(DAVE),
				100,
				get_milestones(10),
				sp_runtime::MultiAddress::Id(BOB)
			),
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
		assert_noop!(
			CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE),
			Error::<Test>::AlreadyMember
		);
	})
}

#[test]
fn voting_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), BOB));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(BOB),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			crate::Vote::Yes
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed(BOB),
			1,
			crate::Vote::Yes
		));
		assert_eq!(CommunityLoanPool::ongoing_votes(1).unwrap().yes_votes, 2);
	})
}

#[test]
fn vote_rejected_with_no_votes() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), BOB));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(BOB),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			crate::Vote::No
		));
		run_to_block(22);
		assert_noop!(
			CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed(BOB), 1, crate::Vote::No),
			Error::<Test>::InvalidIndex
		);
	})
}

#[test]
fn voting_works_only_for_members() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(BOB),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_noop!(
			CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed(DAVE), 1, crate::Vote::Yes),
			Error::<Test>::InsufficientPermission
		);
	})
}

#[test]
fn vote_evaluated_after_yes_votes() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(BOB),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(CommunityLoanPool::evaluated_loans().len(), 1);
	})
}

#[test]
fn reject_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(ALICE),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_ok!(CommunityLoanPool::reject_proposal(RuntimeOrigin::root(), 1));
		System::assert_last_event(Event::Rejected { proposal_index: 1 }.into());
	})
}

#[test]
fn approve_fails_invalid_index() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			CommunityLoanPool::approve_proposal(RuntimeOrigin::signed(ALICE), 0, 0, 0, 10, ALICE,),
			Error::<Test>::InvalidIndex
		);
	})
}

#[test]
fn milestone_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), ALICE));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed(BOB),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id(BOB)
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(CommunityLoanPool::evaluated_loans().len(), 1);
		assert_ok!(CommunityLoanPool::approve_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			0,
			0,
			10,
			ALICE,
		));
		assert_ok!(CommunityLoanPool::propose_milestone(RuntimeOrigin::signed(BOB), 1));
		assert_ok!(CommunityLoanPool::vote_on_milestone_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			crate::Vote::Yes
		));
		run_to_block(43);
		assert_eq!(CommunityLoanPool::loans(1).unwrap().available_amount, 20);
	})
}
