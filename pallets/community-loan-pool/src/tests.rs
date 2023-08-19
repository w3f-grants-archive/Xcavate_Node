use crate::{mock::*, Error, Event};
use frame_support::{assert_ok, BoundedVec, assert_noop};
use sp_core::ConstU32;

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityLoanPool::propose(RuntimeOrigin::signed(ALICE, 100, BOB)));
	})
}