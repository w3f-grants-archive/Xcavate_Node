//! Benchmarking setup for pallet-nft-marketplace
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as NftMarketplace;
use frame_benchmarking::__private::vec;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::traits::Bounded;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_whitelist::Pallet as Whitelist;
type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
use frame_support::assert_ok;

fn setup_object_listing<T: Config>() -> (T::AccountId, BalanceOf<T>) {
	let value: BalanceOf<T> = 100_000u32.into();
	let caller: T::AccountId = whitelisted_caller();
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(
		&caller,
		DepositBalanceOf::<T>::max_value(),
	);
	assert_ok!(NftMarketplace::<T>::create_new_region(RawOrigin::Root.into()));
	assert_ok!(NftMarketplace::<T>::create_new_location(RawOrigin::Root.into(), 0));
	(caller, value)
}

#[benchmarks]
mod benchmarks {
	use super::*;
	#[benchmark]
	fn list_object() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		#[extrinsic_call]
		list_object(
			RawOrigin::Signed(caller),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		);
		assert_eq!(
			NftMarketplace::<T>::registered_nft_details::<
				<T as pallet::Config>::CollectionId,
				<T as pallet::Config>::ItemId,
			>(0.into(), 0.into())
			.unwrap()
			.asset_id,
			0
		);
	}

	#[benchmark]
	fn buy_token() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		#[extrinsic_call]
		buy_token(RawOrigin::Signed(caller), 0, 100);

		assert_eq!(
			NftMarketplace::<T>::registered_nft_details::<
				<T as pallet::Config>::CollectionId,
				<T as pallet::Config>::ItemId,
			>(0.into(), 0.into())
			.unwrap()
			.spv_created,
			true
		);
	}

	#[benchmark]
	fn relist_token() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));
		let listing_value: BalanceOf<T> = 2_000u32.into();
		#[extrinsic_call]
		relist_token(RawOrigin::Signed(caller), 0, 0, 0.into(), listing_value, 80);
		//assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 1);
	}

	#[benchmark]
	fn buy_relisted_token() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));
		let listing_value: BalanceOf<T> = 2_000u32.into();
		assert_ok!(NftMarketplace::<T>::relist_token(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			0.into(),
			listing_value,
			80,
		));
		let nft_buyer: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&nft_buyer,
			DepositBalanceOf::<T>::max_value(),
		);
		#[extrinsic_call]
		buy_relisted_token(RawOrigin::Signed(nft_buyer), 1);
		//assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 0);
	}

	#[benchmark]
	fn upgrade_listing() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));
		let listing_value: BalanceOf<T> = 2_000u32.into();
		assert_ok!(NftMarketplace::<T>::relist_token(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			0.into(),
			listing_value,
			80,
		));
		let new_price: BalanceOf<T> = 5_000u32.into();
		#[extrinsic_call]
		upgrade_listing(RawOrigin::Signed(caller), 1, new_price);
		/* 		assert_eq!(
			NftMarketplace::<T>::ongoing_nft_details::<
				<T as pallet::Config>::CollectionId,
				<T as pallet::Config>::ItemId,
			>(0.into(), 22.into())
			.unwrap()
			.price,
			new_price
		); */
	}

	#[benchmark]
	fn upgrade_object() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		let new_price: BalanceOf<T> = 300_000u32.into();
		#[extrinsic_call]
		upgrade_object(RawOrigin::Signed(caller), 0, new_price);
		assert_eq!(NftMarketplace::<T>::ongoing_object_listing(0).unwrap().current_price,new_price); 
	}

	#[benchmark]
	fn delist_token() {
		let (caller, value) = setup_object_listing::<T>();
		assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
		assert_ok!(NftMarketplace::<T>::list_object(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			value,
			vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
				.try_into()
				.unwrap(),
		));
		assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));
		let listing_value: BalanceOf<T> = 2_000u32.into();
		assert_ok!(NftMarketplace::<T>::relist_token(
			RawOrigin::Signed(caller.clone()).into(),
			0,
			0,
			0.into(),
			listing_value,
			80,
		));
		#[extrinsic_call]
		delist_token(RawOrigin::Signed(caller), 1);
		//assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 0);
	}

	#[benchmark]
	fn create_new_location() {
		assert_ok!(NftMarketplace::<T>::create_new_region(RawOrigin::Root.into()));
		#[extrinsic_call]
		create_new_location(RawOrigin::Root, 0);
	}

	#[benchmark]
	fn create_new_region() {
		#[extrinsic_call]
		create_new_region(RawOrigin::Root);
	}

	impl_benchmark_test_suite!(NftMarketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
