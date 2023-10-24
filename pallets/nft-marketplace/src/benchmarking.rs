//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as NftMarketplace;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_system::RawOrigin;
use frame_support::traits::Get;
use frame_benchmarking::__private::vec;

benchmarks! {
	where_clause {
		where
		<T as pallet_nfts::Config>::CollectionId: From<u32>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
		u32: EncodeLike<<T as pallet_nfts::Config>::CollectionId>,
	}

	list_object{
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet::Config>::Currency::make_free_balance_be(&origin, DepositBalanceOf::<T>::max_value());
		<T as pallet::Config>::Currency::make_free_balance_be(&NftMarketplace::<T>::account_id(), 10000u32.into());
	}: _(RawOrigin::Signed(caller), value, vec![0; T::StringLimit::get() as usize].try_into().unwrap())
	verify {
		assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 10);
	}

	  buy_nft{
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet::Config>::Currency::make_free_balance_be(&caller, 100_000_000u32.into());
		NftMarketplace::<T>::list_object(RawOrigin::Signed(caller.clone()).into(), value, vec![0; T::StringLimit::get() as usize].try_into().unwrap());
	}: _(RawOrigin::Signed(caller), 1.into(), 3.into())
	verify {
		assert_eq!(NftMarketplace::<T>::listed_nfts().len(), 9);
	}
	impl_benchmark_test_suite!(NftMarketplace, crate::mock::new_test_ext(), crate::mock::Test);
}
