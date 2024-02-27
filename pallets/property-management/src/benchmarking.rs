//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as PropertyManagement;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::sp_runtime::traits::Bounded;
use pallet_nft_marketplace::Pallet as NftMarketplace;
 type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;  
use pallet_whitelist::Pallet as Whitelist;
use frame_support::traits::Get;

type BalanceOf1<T> = <<T as pallet_nft_marketplace::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance; 

fn setup_real_estate_object<T: Config>() {
	let value: BalanceOf1<T> = 100_000u32.into();
	let caller: T::AccountId = whitelisted_caller();
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(
		&caller,
		DepositBalanceOf::<T>::max_value(),
	);
	NftMarketplace::<T>::create_new_location(RawOrigin::Root.into());
	Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone());
	NftMarketplace::<T>::list_object(
		RawOrigin::Signed(caller.clone()).into(),
		0,
		value.into(),
		vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap(),
	);
	NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100);
} 

#[benchmarks]
mod benchmarks {
	use super::*;

 	#[benchmark]
	fn add_letting_agent() {
		NftMarketplace::<T>::create_new_location(RawOrigin::Root.into());
		let letting_agent: T::AccountId = whitelisted_caller();
/* 		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		); */
		#[extrinsic_call]
		add_letting_agent(RawOrigin::Root, 0, letting_agent.clone());

		assert_eq!(PropertyManagement::<T>::letting_info(letting_agent).is_some(), true);
	} 

/* 	#[benchmark]
	fn letting_agent_deposit() {
		NftMarketplace::<T>::create_new_location(RawOrigin::Root.into());
		let letting_agent: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		);
		PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, letting_agent.clone());
		#[extrinsic_call]
		letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()));

		assert_eq!(PropertyManagement::<T>::letting_info(letting_agent).unwrap().deposited, true);
		assert_eq!(PropertyManagement::<T>::letting_agent_locations(0).len(), 1);
	}

	#[benchmark]
	fn set_letting_agent() {
		setup_real_estate_object::<T>();
		let letting_agent: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		);
		PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, letting_agent.clone());
		PropertyManagement::<T>::letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()).into());
		#[extrinsic_call]
		set_letting_agent(RawOrigin::Signed(letting_agent.clone()), 0.into(), 0.into());

		assert_eq!(PropertyManagement::<T>::letting_storage(0).is_some(), true);
	}

	#[benchmark]
	fn distribute_income() {
		setup_real_estate_object::<T>();
		let letting_agent: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		);
		PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, letting_agent.clone());
		PropertyManagement::<T>::letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()).into());
		PropertyManagement::<T>::set_letting_agent(RawOrigin::Signed(letting_agent.clone()).into(), 0.into(), 0.into());
		let amount: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		distribute_income(RawOrigin::Signed(letting_agent), 0, amount);

		assert_eq!(PropertyManagement::<T>::stored_funds(caller), amount);
	}

	#[benchmark]
	fn withdraw_funds() {
		setup_real_estate_object::<T>();
		let letting_agent: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&letting_agent,
			DepositBalanceOf::<T>::max_value(),
		);
		PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, letting_agent.clone());
		PropertyManagement::<T>::letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()).into());
		PropertyManagement::<T>::set_letting_agent(RawOrigin::Signed(letting_agent.clone()).into(), 0.into(), 0.into());
		let amount: BalanceOf<T> = 100_000u32.into();
		PropertyManagement::<T>::distribute_income(RawOrigin::Signed(letting_agent).into(), 0, amount);
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		withdraw_funds(RawOrigin::Signed(caller.clone()));

		assert_eq!(PropertyManagement::<T>::stored_funds(caller), 0u32.into());
	} */


	impl_benchmark_test_suite!(PropertyManagement, crate::mock::new_test_ext(), crate::mock::Test);
}
