use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};



#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(XcavateStaking::stake(RuntimeOrigin::signed(ALICE),100));
		System::assert_last_event(Event::Locked{staker: ALICE, amount: 100}.into());
	});
}


 