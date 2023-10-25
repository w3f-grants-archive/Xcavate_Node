use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn list_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_nfts().len(), 10);
	})
}

#[test]
fn buy_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([0; 32].into()), 1, 3));
		assert_eq!(Balances::free_balance(&(NftMarketplace::account_id())), 20_300_000);
		assert_eq!(NftMarketplace::listed_collection(1).len(), 3);
		assert_eq!(NftMarketplace::listed_nfts().len(), 7);
	})
}

#[test]
fn buy_nft_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			NftMarketplace::buy_nft(RuntimeOrigin::signed([0; 32].into()), 1, 1),
			Error::<Test>::NotEnoughNftsAvailable
		);
	})
}

#[test]
fn distributes_nfts_and_funds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_nfts().len(), 10);
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 1, 10));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20999998);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_000);
		assert_eq!(NftMarketplace::listed_nfts().len(), 0);
	})
}

#[test]
fn listing_and_selling_multiple_objects() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([2; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_nfts().len(), 30);
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 1, 10));
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 4, 2));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([2; 32].into()), 2, 3));
		assert_eq!(NftMarketplace::listed_nfts().len(), 25);
	});
}
