//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as XcavateStaking;
use frame_benchmarking::v2::*;
use frame_support::{
	ensure,
	traits::{EnsureOrigin, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;
const SEED: u32 = 0;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn stake() {
		let caller: T::AccountId = whitelisted_caller();
		let value: BalanceOf<T> = 100u32.into();

		#[extrinsic_call]
		stake(RawOrigin::Signed(caller.clone()), value);

		assert_last_event::<T>(Event::Locked { staker: caller.clone(), amount: value }.into());
		assert_eq!(XcavateStaking::<T>::active_stakers()[0], caller);
	}

	#[benchmark]
	fn unstake() {
		let caller: T::AccountId = account("alice", SEED, SEED);
		let value: BalanceOf<T> = 1_000u32.into();
		<T as pallet::Config>::Currency::make_free_balance_be(&caller, 100_000_000u32.into());
		XcavateStaking::<T>::stake(RawOrigin::Signed(caller.clone()).into(), value);
		assert_eq!(XcavateStaking::<T>::active_stakers().len(), 1);
		let unstake_value: BalanceOf<T> = 1u32.into();
		#[extrinsic_call]
		unstake(RawOrigin::Signed(caller.clone()), unstake_value);

		assert_last_event::<T>(Event::Unlocked { staker: caller, amount: unstake_value }.into());
		let staker = &XcavateStaking::<T>::active_stakers()[0];
		let staked_value: BalanceOf<T> = 999u32.into();
		assert_eq!(XcavateStaking::<T>::ledger(staker).unwrap().locked, staked_value);
	}

	impl_benchmark_test_suite!(XcavateStaking, crate::mock::new_test_ext(), crate::mock::Test);
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}
