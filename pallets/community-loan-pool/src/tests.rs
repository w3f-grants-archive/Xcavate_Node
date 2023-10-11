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
			RuntimeOrigin::signed([0;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
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
				RuntimeOrigin::signed([6;32].into()),
				100,
				get_milestones(10),
				sp_runtime::MultiAddress::Id([1;32].into()),
				13,
				20
			),
			Error::<Test>::InsufficientProposersBalance
		);
	})
}

#[test]
fn add_committee_member_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_eq!(CommunityLoanPool::voting_committee()[0], [0;32].into());
	})
}

#[test]
fn add_committee_member_fails_when_member_is_two_times_added() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_eq!(CommunityLoanPool::voting_committee()[0], [0;32].into());
		assert_noop!(
			CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()),
			Error::<Test>::AlreadyMember
		);
	})
}

#[test]
fn voting_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [1;32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed([0;32].into()),
			1,
			crate::Vote::Yes
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed([1;32].into()),
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
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [1;32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed([0;32].into()),
			1,
			crate::Vote::No
		));
		run_to_block(22);
		assert_noop!(
			CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed([1;32].into()), 1, crate::Vote::No),
			Error::<Test>::InvalidIndex
		);
	})
}

#[test]
fn voting_works_only_for_members() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
		));
		assert_noop!(
			CommunityLoanPool::vote_on_proposal(RuntimeOrigin::signed([2;32].into()), 1, crate::Vote::Yes),
			Error::<Test>::InsufficientPermission
		);
	})
}

#[test]
fn vote_evaluated_after_yes_votes() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed([0;32].into()),
			1,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(CommunityLoanPool::loans(1).unwrap().available_amount, 10);
	})
}

#[test]
fn milestone_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0;32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1;32].into()),
			100,
			get_milestones(10),
			sp_runtime::MultiAddress::Id([1;32].into()),
			13,
			20
		));
		assert_ok!(CommunityLoanPool::vote_on_proposal(
			RuntimeOrigin::signed([0;32].into()),
			1,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_ok!(CommunityLoanPool::propose_milestone(RuntimeOrigin::signed([1;32].into()), 1));
		assert_ok!(CommunityLoanPool::vote_on_milestone_proposal(
			RuntimeOrigin::signed([0;32].into()),
			1,
			crate::Vote::Yes
		));
		run_to_block(43);
		assert_eq!(CommunityLoanPool::loans(1).unwrap().available_amount, 20);
		assert_eq!(CommunityLoanPool::loans(1).unwrap().loan_apy, 1023);
	})
}
