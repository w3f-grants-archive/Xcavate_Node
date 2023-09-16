//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as XcavateStaking;
use frame_benchmarking::{
	v1::{account, benchmarks, benchmarks_instance_pallet, BenchmarkError},
	whitelisted_caller,
};
use frame_support::{
	ensure,
	traits::{EnsureOrigin, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;

benchmarks! {
	stake {
		let caller: T::AccountId = whitelisted_caller();
		let value: BalanceOf<T> = 100u32.into();
	}: _(RawOrigin::Signed(caller), value)

	unstake {
		let caller: T::AccountId = whitelisted_caller();
		let value: BalanceOf<T> = 100u32.into();
		XcavateStaking::<T>::stake(
			RawOrigin::Signed(caller).into(),
			value,
		)?;
	}: _(RawOrigin::Signed(caller), value)

	impl_benchmark_test_suite!(XcavateStaking, crate::mock::new_test_ext(), crate::mock::Test);
}
