use crate::{mock::*, Error, Event};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use frame_support::sp_runtime::Percent;

use pallet_community_loan_pool::{BoundedProposedMilestones, Config, ProposedMilestone};

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

#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 100));
		System::assert_last_event(Event::Locked { staker: [0; 32].into(), amount: 100 }.into());
		let total_stake = XcavateStaking::total_stake();
		assert_eq!(total_stake, 100);
		let stakers = XcavateStaking::active_stakers();
		assert_eq!(stakers.len(), 1);
	});
}

#[test]
fn stake_with_several_people_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 100));
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([1; 32].into()), 400));
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([2; 32].into()), 500));
		let total_stake = XcavateStaking::total_stake();
		assert_eq!(total_stake, 1000);
		let stakers = XcavateStaking::active_stakers();
		assert_eq!(stakers.len(), 3);
	})
}

#[test]
fn person_cant_stake_0_token() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 0),
			Error::<Test>::StakingWithNoValue
		);
	})
}

#[test]
fn unstake_works() {
	new_test_ext().execute_with(|| {
		//System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 100));
		assert_ok!(XcavateStaking::unstake(RuntimeOrigin::signed([0; 32].into()), 100));
		let total_stake = XcavateStaking::total_stake();
		assert_eq!(total_stake, 0);
		let stakers = XcavateStaking::active_stakers();
		assert_eq!(stakers.len(), 0);
	})
}

#[test]
fn unstake_doesnt_work_for_nonstaker() {
	new_test_ext().execute_with(|| {
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 100));
		assert_noop!(
			XcavateStaking::unstake(RuntimeOrigin::signed([1; 32].into()), 100),
			Error::<Test>::NoStaker
		);
	})
}

#[test]
fn claiming_of_rewards_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed([0; 32].into()), 10000000));
		assert_ok!(CommunityLoanPool::add_committee_member(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(CommunityLoanPool::propose(
			RuntimeOrigin::signed([1; 32].into()),
			10003000,
			sp_runtime::MultiAddress::Id([1; 32].into()),
			13,
			20
		));
		assert_ok!(CommunityLoanPool::set_milestones(
			RuntimeOrigin::signed([0; 32].into()),
			1,
			get_milestones(10),
		));
		Timestamp::set_timestamp(0);
		run_to_block(21);
		assert_eq!(CommunityLoanPool::ongoing_loans().len(), 1);
		assert_eq!(CommunityLoanPool::loans(1).unwrap().available_amount, 1000300);
		assert_eq!(CommunityLoanPool::total_loan_amount(), 10003000);
		assert_eq!(XcavateStaking::ledger::<AccountId>([0; 32].into()).unwrap().locked, 10000000);
		Timestamp::set_timestamp(10000);
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		XcavateStaking::on_initialize(System::block_number());
		assert_eq!(XcavateStaking::ledger::<AccountId>([0; 32].into()).unwrap().locked, 10000026);
		System::assert_last_event(Event::RewardsClaimed { amount: 26, apy: 823 }.into());
	})
}
