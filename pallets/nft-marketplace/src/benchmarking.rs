//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as NftMarketplace;
use frame_benchmarking::__private::vec;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::traits::Bounded;
use frame_support::traits::Get;
use frame_system::RawOrigin;
type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[benchmarks]
mod benchmarks {
	use super::*;
	#[benchmark]
	fn list_object() {
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(&NftMarketplace::<T>::account_id(), DepositBalanceOf::<T>::max_value());

		#[extrinsic_call]
		list_object(RawOrigin::Signed(caller), value, vec![0; T::StringLimit::get() as usize].try_into().unwrap());

		assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 100);
	}

	#[benchmark]
	fn buy_nft() {
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());
		NftMarketplace::<T>::list_object(RawOrigin::Signed(caller.clone()).into(), value, vec![0; T::StringLimit::get() as usize].try_into().unwrap());
		#[extrinsic_call]
		buy_nft(RawOrigin::Signed(caller), 0.into(), 100);

		assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 0);
	}
	impl_benchmark_test_suite!(NftMarketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
