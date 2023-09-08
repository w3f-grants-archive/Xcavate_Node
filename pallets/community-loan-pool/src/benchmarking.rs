//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as CommunityLoanPool;
use frame_benchmarking::v1::{benchmarks, account, benchmarks_instance_pallet, BenchmarkError};
use frame_support::{
	ensure,
	traits::{EnsureOrigin, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;
use frame_support::sp_runtime::Saturating;

const SEED: u32 = 0;

fn setup_proposal<T: Config>(
	u: u32,
) -> (T::AccountId, BalanceOf<T>, AccountIdLookupOf<T>) {
	let caller = account("caller", u, SEED);
	let value: BalanceOf<T> = T::ProposalBondMinimum::get().saturating_mul(100u32.into());
	let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, value);
	let beneficiary = account("beneficiary", u, SEED);
	let beneficiary_lookup = T::Lookup::unlookup(beneficiary);
	(caller, value, beneficiary_lookup)
}

fn setup_pot_account<T: Config>() {
	let pot_account = CommunityLoanPool::<T>::account_id();
	let value = <T as pallet::Config>::Currency::minimum_balance().saturating_mul(1_000_000_000u32.into());
	let _ = <T as pallet::Config>::Currency::make_free_balance_be(&pot_account, value);
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	propose {
		let (caller, value, beneficiary_lookup) = setup_proposal::<T>(SEED);
		let caller_key = frame_system::Account::<T>::hashed_key_for(&caller);
		frame_benchmarking::benchmarking::add_to_whitelist(caller_key.into());
	}: _(RawOrigin::Signed(caller), value, beneficiary_lookup)

	impl_benchmark_test_suite!(CommunityLoanPool, crate::mock::new_test_ext(), crate::mock::Test);
}
