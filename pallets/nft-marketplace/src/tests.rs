use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn list_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::list_object(RuntimeOrigin::signed([0; 32].into()), 1_000_000));
		assert_eq!(NftMarketplace::listed_nfts().len(), 10);
	})
}