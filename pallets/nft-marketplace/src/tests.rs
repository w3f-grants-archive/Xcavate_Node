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
		assert_eq!(NftMarketplace::listed_nfts().len(), 100);
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
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([0; 32].into()), 0, 30));
		assert_eq!(Balances::free_balance(&(NftMarketplace::account_id())), 20_300_000);
		assert_eq!(NftMarketplace::listed_nfts().len(), 70);
		assert_eq!(NftMarketplace::sold_nfts_collection(1).len(), 30);
	})
}

#[test]
fn buy_nft_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(
			NftMarketplace::buy_nft(RuntimeOrigin::signed([0; 32].into()), 1, 1),
			Error::<Test>::CollectionNotFound
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
		assert_eq!(NftMarketplace::listed_nfts().len(), 100);
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20999998);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_000);
		assert_eq!(NftMarketplace::listed_nfts().len(), 0);
	})
}

#[test]
fn listing_and_selling_multiple_objects() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([2; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_nfts().len(), 300);
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 2, 100));
		assert_eq!(NftMarketplace::listed_nfts().len(), 200);
		assert_eq!(NftMarketplace::ongoing_nft_details((2, 8)), None);
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([1; 32].into()), 3, 20));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_nft(RuntimeOrigin::signed([2; 32].into()), 1, 33));
		assert_eq!(NftMarketplace::ongoing_nft_details((1, 3)).unwrap().sold, true);
		assert_eq!(NftMarketplace::ongoing_nft_details((1, 34)).unwrap().sold, false);
		assert_eq!(NftMarketplace::listed_nfts().len(), 247);
		assert_eq!(NftMarketplace::listed_nfts_of_collection(1).len(), 67);
		assert_eq!(NftMarketplace::listed_nfts_of_collection(2).len(), 0);
		assert_eq!(NftMarketplace::listed_nfts_of_collection(3).len(), 80);
		assert_eq!(NftMarketplace::listed_nfts_of_collection(4).len(), 100);
		assert_eq!(NftMarketplace::ongoing_nft_details((3, 2)).unwrap().sold, true);
	});
}
