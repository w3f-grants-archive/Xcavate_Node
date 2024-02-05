use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::CollectionId;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

#[test]
fn list_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 100);
	})
}

#[test]
fn buy_token_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 30));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 70);
		assert_eq!(NftMarketplace::sold_token(0), 30);
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 70);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_700_000);
	})
}

#[test]
fn distributes_nfts_and_funds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
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
		assert_eq!(NftMarketplace::registered_nft_details(0).unwrap().spv_created, true);
		assert_eq!(NftMarketplace::listed_token(0), None);
	})
} 

#[test]
fn buy_token_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 1, 1),
			Error::<Test>::CollectionNotFound
		);
	})
}

#[test]
fn listing_and_selling_multiple_objects() {
	new_test_ext().execute_with(|| {
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
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
		//assert_eq!(NftMarketplace::listed_nfts().len(), 300);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 1, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 200);
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 2, 20));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([3; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([2; 32].into()), 0, 33));
		assert_eq!(NftMarketplace::sold_token(0), 33);
		//assert_eq!(NftMarketplace::listed_nfts().len(), 247);
		assert_eq!(NftMarketplace::listed_token(0).unwrap(), 67);
		assert_eq!(NftMarketplace::listed_token(2).unwrap(), 80);
		assert_eq!(NftMarketplace::listed_token(3).unwrap(), 100);
		assert_eq!(NftMarketplace::sold_token(2), 20);
	});
}

#[test]
fn relist_a_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_eq!(NftMarketplace::registered_nft_details(0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		//assert_eq!(NftMarketplace::listed_nfts()[0], (0, 100));
		assert_eq!(NftMarketplace::token_listings(0).is_some(), true);
		assert_eq!(NftMarketplace::token_listings(0).unwrap().item_id, 0);
		//assert_eq!(NftMarketplace::seller_listings::<AccountId>([1; 32].into()).len(), 1);
	})
}

/* #[test]
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
			NftMarketplace::list_token(RuntimeOrigin::signed([0; 32].into()), 0, 1000, 1),
			Error::<Test>::CollectionNotKnown
		);
	})
} */

#[test]
fn buy_single_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [3; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
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
		assert_eq!(NftMarketplace::registered_nft_details(0).unwrap().spv_created, true);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		assert_ok!(NftMarketplace::buy_relisted_token(RuntimeOrigin::signed([3; 32].into()), 0));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_eq!(NftMarketplace::token_listings(0).is_some(), false);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_000_990);
		assert_eq!(Balances::free_balance(&([3; 32].into())), 4_000);
		//assert_eq!(NftMarketplace::seller_listings::<AccountId>([1; 32].into()).len(), 0);
	})
}

#[test]
fn delist_single_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		assert_ok!(NftMarketplace::delist_token(RuntimeOrigin::signed([1; 32].into()), 0));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
	})
}

#[test]
fn delist_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([4; 32].into()), 0),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			NftMarketplace::delist_token(RuntimeOrigin::signed([4; 32].into()), 1),
			Error::<Test>::TokenNotForSale
		);
	})
}

#[test]
fn upgrade_price_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		assert_ok!(NftMarketplace::upgrade_listing(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			300
		));
		assert_eq!(NftMarketplace::token_listings(0).unwrap().price, 300);
	})
}

#[test]
fn upgrade_price_fails_if_not_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([1; 32].into()), 0, 100));
		//assert_eq!(NftMarketplace::listed_nfts().len(), 0);
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([1; 32].into()), 0, 1000, 1));
		assert_noop!(
			NftMarketplace::upgrade_listing(RuntimeOrigin::signed([4; 32].into()), 0, 300),
			Error::<Test>::NoPermission
		);
	})
}

#[test]
fn upgrade_object_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
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
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
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
		
		assert_eq!(NftMarketplace::registered_nft_details(0).unwrap().spv_created, true);
		assert_eq!(NftMarketplace::listed_token(0), None);
	})
}

#[test]
fn upgrade_single_nft_from_listed_object_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
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
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(NftMarketplace::list_object(
			RuntimeOrigin::signed([0; 32].into()),
			1_000_000,
			bvec![22, 22]
		));
		assert_ok!(NftMarketplace::buy_token(RuntimeOrigin::signed([0; 32].into()), 0, 100));
		assert_ok!(NftMarketplace::list_token(RuntimeOrigin::signed([0; 32].into()), 0, 1000, 1));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::NftAlreadyRelisted
		);
	})
}

#[test]
fn upgrade_unknown_collection_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		create_collection();
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			NftMarketplace::upgrade_object(RuntimeOrigin::signed([0; 32].into()), 0, 300),
			Error::<Test>::InvalidIndex
		);
	})
} 


fn create_collection() {
	let pallet_id: AccountId = NftMarketplace::account_id();
	let collection_id: CollectionId<Test> = 0;
	pallet_nfts::Pallet::<Test>::do_create_collection(
		collection_id.into(),
		pallet_id.clone(),
		pallet_id.clone(),
		CollectionConfig {
			settings: CollectionSettings::from_disabled(CollectionSetting::DepositRequired.into()),
			max_supply: None,
			mint_settings: MintSettings::default(),
		},
		0,
		pallet_nfts::Event::Created {
			creator: pallet_id.clone(),
			owner: pallet_id,
			collection: collection_id.into(),
		},
	);
}
