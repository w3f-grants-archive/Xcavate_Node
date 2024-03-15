//! Benchmarking setup for pallet-property-governance
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as PropertyGovernance;
use frame_benchmarking::__private::vec;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::sp_runtime::traits::Bounded;
use pallet_nft_marketplace::Pallet as NftMarketplace;
type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
use pallet_whitelist::Pallet as Whitelist;
use pallet_property_management::Pallet as PropertyManagement;
use frame_support::{traits::Get, assert_ok};

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
	assert_ok!(NftMarketplace::<T>::create_new_region(RawOrigin::Root.into()));
	assert_ok!(NftMarketplace::<T>::create_new_location(RawOrigin::Root.into(), 0));
	assert_ok!(Whitelist::<T>::add_to_whitelist(RawOrigin::Root.into(), caller.clone()));
	assert_ok!(NftMarketplace::<T>::list_object(
		RawOrigin::Signed(caller.clone()).into(),
		0,
		0,
		value.into(),
		vec![0; <T as pallet_nfts::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap(),
	));
	assert_ok!(NftMarketplace::<T>::buy_token(RawOrigin::Signed(caller.clone()).into(), 0, 100));
	let letting_agent: T::AccountId = whitelisted_caller();
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(
		&letting_agent,
		DepositBalanceOf::<T>::max_value(),
	);
	assert_ok!(PropertyManagement::<T>::add_letting_agent(RawOrigin::Root.into(), 0, 0, letting_agent.clone()));
	assert_ok!(PropertyManagement::<T>::letting_agent_deposit(RawOrigin::Signed(letting_agent.clone()).into()));
	assert_ok!(PropertyManagement::<T>::set_letting_agent(RawOrigin::Signed(letting_agent.clone()).into(), 0));	
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn propose() {
		setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		#[extrinsic_call]
		propose(RawOrigin::Signed(caller.clone()), 0);

		assert_eq!(PropertyGovernance::<T>::proposals(1).is_some(), true);
	}

 	#[benchmark]
	fn inquery_against_letting_agent() {
		setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		#[extrinsic_call]
		inquery_against_letting_agent(RawOrigin::Signed(caller.clone()), 0);

		assert_eq!(PropertyGovernance::<T>::inqueries(1).is_some(), true);
	}

	#[benchmark]
	fn vote_on_proposal() {
		setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		assert_ok!{PropertyGovernance::<T>::propose(RawOrigin::Signed(caller.clone()).into(), 0)};
		#[extrinsic_call]
		vote_on_proposal(RawOrigin::Signed(caller.clone()), 1, crate::Vote::Yes);

		assert_eq!(PropertyGovernance::<T>::ongoing_votes(1).unwrap().yes_votes, 100);
		assert_eq!(PropertyGovernance::<T>::proposal_voter(1).len(), 1);
	}

	#[benchmark]
	fn vote_on_letting_agent_inquery() {
		setup_real_estate_object::<T>();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(
			&caller,
			DepositBalanceOf::<T>::max_value(),
		);
		assert_ok!{PropertyGovernance::<T>::inquery_against_letting_agent(RawOrigin::Signed(caller.clone()).into(), 0)};
		#[extrinsic_call]
		vote_on_letting_agent_inquery(RawOrigin::Signed(caller.clone()), 1, crate::Vote::Yes);

		assert_eq!(PropertyGovernance::<T>::ongoing_inquery_votes(1).unwrap().yes_votes, 100);
		assert_eq!(PropertyGovernance::<T>::inquery_voter(1).len(), 1);
	}


	impl_benchmark_test_suite!(PropertyGovernance, crate::mock::new_test_ext(), crate::mock::Test);
}
