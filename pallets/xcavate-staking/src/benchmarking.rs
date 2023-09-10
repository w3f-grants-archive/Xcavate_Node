//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as XcavateStaking;
use frame_benchmarking::v1::{benchmarks, account, benchmarks_instance_pallet, BenchmarkError};
use frame_support::{
	ensure,
	traits::{EnsureOrigin, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;
use frame_benchmarking::whitelisted_caller;

benchmarks! {
	stake {
		let caller: T::AccountId = whitelisted_caller();
		let value: BalanceOf<T> = 100u32.into();
	}: _(RawOrigin::Signed(caller), value)

	impl_benchmark_test_suite!(XcavateStaking, crate::mock::new_test_ext(), crate::mock::Test);
}
