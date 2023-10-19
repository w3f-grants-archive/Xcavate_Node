//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as NftMarketplace;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_system::RawOrigin;

benchmarks! {
	where_clause {
		where
		<T as pallet_nfts::Config>::CollectionId: From<u32>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
	}

	list_object{
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), value)
	verify {
		assert_eq!(NftMarketplace::<T>::listed_nft_count(), 10);
	}

  	buy_nft{
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		<T as pallet::Config>::Currency::make_free_balance_be(&caller, 100_000_000u32.into());
		NftMarketplace::<T>::list_object(RawOrigin::Signed(caller.clone()).into(), value);
	}: _(RawOrigin::Signed(caller), 1)
	verify {
		assert_eq!(NftMarketplace::<T>::listed_nft_count(), 10);
	}  
	impl_benchmark_test_suite!(NftMarketplace, crate::mock::new_test_ext(), crate::mock::Test);
}

