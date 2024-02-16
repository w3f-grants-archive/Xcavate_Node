use crate::{mock::*, Error};
use crate::{CollectionId, ItemId};
use frame_support::{assert_noop, assert_ok};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

#[test]
fn create_new_location_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_eq!(NftMarketplace::location_collections(0).unwrap(), 0);
		assert_eq!(NftMarketplace::location_collections(1).unwrap(), 1);
		assert_eq!(NftMarketplace::location_collections(2), None);
	})
}

#[test]
fn list_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 100);
		assert_eq!(NftMarketplace::next_nft_id(0), 1);
		assert_eq!(NftMarketplace::next_nft_id(1), 0);
		assert_eq!(NftMarketplace::next_asset_id(), 1);
		assert_eq!(NftMarketplace::ongoing_object_listing(0).is_some(), true);
		assert_eq!(NftMarketplace::registered_nft_details(0, 0).is_some(), true);
	})
}

#[test]
fn list_object_with_not_existing_location_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::list_object(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				1_000_000,
				bvec![22, 22]
			),
			Error::<Test>::LocationUnknown
		);
	})
}

#[test]
fn buy_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 30));
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 70);
		assert_eq!(NftMarketplace::token_owner::<AccountId, ItemId<Test>>([1; 32].into(), 0), 30);
		assert_eq!(NftMarketplace::token_buyer(0).len(), 1);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_700_000);
		assert_eq!(Assets::balance(0, NftMarketplace::account_id()), 100);
	})
}

#[test]
fn distributes_nfts_and_funds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 70);
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20990000);
		assert_eq!(Balances::free_balance(&NftMarketplace::treasury_account_id()), 9000);
		assert_eq!(Balances::free_balance(&NftMarketplace::community_account_id()), 1000);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_000);
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_eq!(NftMarketplace::registered_nft_details(0, 0).unwrap().spv_created, true);
		assert_eq!(NftMarketplace::listed_token(0), None);
		assert_eq!(NftMarketplace::token_owner::<AccountId, ItemId<Test>>([1; 32].into(), 0), 0);
		assert_eq!(NftMarketplace::token_buyer(0).len(), 0);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 100);
	})
}

#[test]
fn buy_token_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 1, 1),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn buy_token_doesnt_work_2() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_noop!(
			NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 101),
			Error::<Test>::NotEnoughTokenAvailable
		);
	})
}

#[test]
fn listing_and_selling_multiple_objects() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 100));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 2, 20));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 2, 30));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 33));
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 67);
		assert_eq!(NftMarketplace::listed_token(2).unwrap(), 50);
		assert_eq!(NftMarketplace::listed_token(3).unwrap(), 100);
		assert_eq!(NftMarketplace::token_owner::<AccountId, ItemId<Test>>([2; 32].into(), 2), 30);
		assert_eq!(NftMarketplace::token_buyer(2).len(), 2);
		assert_eq!(NftMarketplace::token_owner::<AccountId, ItemId<Test>>([1; 32].into(), 1), 0);
		assert_eq!(NftMarketplace::token_buyer(1).len(), 0);
	});
}

#[test]
fn relist_a_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_eq!(NftMarketplace::registered_nft_details(0, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_eq!(NftMarketplace::token_listings(1).is_some(), true);
		assert_eq!(NftMarketplace::token_listings(1).unwrap().item_id, 0);
		assert_eq!(Assets::balance(0, NftMarketplace::account_id()), 1);
		assert_eq!(Assets::balance(0, &[1; 32].into()), 99);
	})
}

#[test]
fn relist_nfts_not_created_with_marketplace_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Uniques::create(
			RuntimeOrigin::signed([0; 32].into()),
			sp_runtime::MultiAddress::Id([0; 32].into()),
			Default::default()
		));
		assert_ok!(Uniques::mint(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			0,
			sp_runtime::MultiAddress::Id([0; 32].into()),
			None
		));
		assert_noop!(
			NftMarketplace::relist_token(RuntimeOrigin::signed([0; 32].into()), 0, 0, 1000, 1),
			Error::<Test>::LocationUnknown
		);
	})
}

#[test]
fn buy_single_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 100);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20990000);
		assert_eq!(Balances::free_balance(&NftMarketplace::treasury_account_id()), 9000);
		assert_eq!(Balances::free_balance(&NftMarketplace::community_account_id()), 1000);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_000);
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_eq!(NftMarketplace::registered_nft_details(0, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 1));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_eq!(NftMarketplace::token_listings(0).is_some(), false);
		assert_eq!(NftMarketplace::property_owner(0).len(), 2);
		assert_eq!(NftMarketplace::property_owner_token::<u32, AccountId>(0, [1; 32].into()), 99);
		assert_eq!(NftMarketplace::property_owner_token::<u32, AccountId>(0, [3; 32].into()), 1);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_990);
		assert_eq!(Balances::free_balance(&([3; 32].into())), 4_000);
		//assert_eq!(NftMarketplace::seller_listings::<AccountId>([1; 32].into()).len(), 0);
		assert_eq!(Assets::balance(0, &[3; 32].into()), 1);
	})
}

#[test]
fn delist_single_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_ok!(NftMarketplace::delist_token(RuntimeOrigin::signed([1; 32].into()), 1));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
	})
}

#[test]
fn delist_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([4; 32].into()), 1),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([4; 32].into()), 2),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn upgrade_price_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_ok!(NftMarketplace::upgrade_listing(RuntimeOrigin::signed([1; 32].into()), 1, 300));
		assert_eq!(NftMarketplace::token_listings(1).unwrap().price, 300);
	})
}

#[test]
fn upgrade_price_fails_if_not_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::upgrade_listing(RuntimeOrigin::signed([4; 32].into()), 1, 300),
			Error::<Test>::NoPermission
		);
	})
}

#[test]
fn upgrade_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 30000));
		assert_eq!(NftMarketplace::ongoing_object_listing(0).unwrap().current_price, 30000);
	})
}

#[test]
fn upgrade_object_and_distribute_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 100);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 50));
		assert_ok!(NftMarketplace::upgrade_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			2_000_000
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 50));
		assert_eq!(Balances::free_balance(&([0; 32].into())), 21485000);
		assert_eq!(Balances::free_balance(&NftMarketplace::treasury_account_id()), 13500);
		assert_eq!(Balances::free_balance(&NftMarketplace::community_account_id()), 1500);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_500_000);
		assert_eq!(Balances::free_balance(&([2; 32].into())), 150_000);
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);

		assert_eq!(NftMarketplace::registered_nft_details(0, 0).unwrap().spv_created, true);
		assert_eq!(NftMarketplace::listed_token(0), None);
	})
}

#[test]
fn upgrade_single_nft_from_listed_object_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_noop!(
			NftMarketplace::upgrade_listing(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn upgrade_object_for_relisted_nft_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			0,
			1000,
			1
		));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 1, 300),
			Error::<Test>::InvalidIndex
		);
	})
}

#[test]
fn upgrade_unknown_collection_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::InvalidIndex
		);
	})
}

#[test]
fn listing_objects_in_different_regions() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(NftMarketplace::create_new_location(RuntimeOrigin::root()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			2,
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 100));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 2, 100));
		assert_eq!(NftMarketplace::registered_nft_details(1, 0).unwrap().spv_created, true);
		assert_eq!(NftMarketplace::registered_nft_details(2, 0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::relist_token(
			RuntimeOrigin::signed([1; 32].into()),
			1,
			0,
			1000,
			100
		));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([2; 32].into()), 3));
		assert_eq!(Assets::balance(1, &[2; 32].into()), 100);
		assert_eq!(Assets::balance(2, &[2; 32].into()), 100);
	})
}
