use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE), 100));
		System::assert_last_event(Event::Locked { staker: ALICE, amount: 100 }.into());
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
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE), 100));
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(BOB), 400));
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(CHARLIE), 500));
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
		assert_noop!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE), 0), Error::<Test>::StakingWithNoValue);
	})
}

#[test]
fn unstake_works() {
	new_test_ext().execute_with(|| {
		//System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE), 100));
		assert_ok!(XcavateStaking::unstake(RuntimeOrigin::signed(ALICE), 100));
		let total_stake = XcavateStaking::total_stake();
		assert_eq!(total_stake, 0);
		let stakers = XcavateStaking::active_stakers();
		assert_eq!(stakers.len(), 0);
	})
}

#[test]
fn unstake_doesnt_work_for_nonstaker() {
	new_test_ext().execute_with(|| {
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE), 100));
		assert_noop!(XcavateStaking::unstake(RuntimeOrigin::signed(BOB), 100), Error::<Test>::NoStaker);
	})
}
