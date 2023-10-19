//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as NftMarketplace;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
pub mod benchmarks {
	use super::*;

	#[benchmark]
	fn buy_nft() 
	where
		<T as pallet_nfts::Config>::CollectionId: From<u32>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,{
		let value: BalanceOf<T> = 100_000u32.into();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		buy_nft(RawOrigin::Signed(caller), 1);

		assert_eq!(NftMarketplace::<T>::listed_nft_count(), 100);
	}

/* 	#[benchmark]
	fn cause_error() {
		Something::<T>::put(100u32);
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		cause_error(RawOrigin::Signed(caller));

		assert_eq!(Something::<T>::get(), Some(101u32));
	} */

	impl_benchmark_test_suite!(NftMarketplace, crate::mock::new_test_ext(), crate::mock::Test);
}

