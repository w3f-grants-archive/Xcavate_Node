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
 use scale_info::Type;

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

	reject_proposal {
		let (caller, value, beneficiary_lookup) = setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller).into(),
			value,
			beneficiary_lookup
		)?;
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		let reject_origin = T::RejectOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
	}: _<T::RuntimeOrigin>(reject_origin, proposal_id) 

/* 	approve_proposal {
		let (caller, value, beneficiary_lookup) = setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller.clone()).into(),
			value,
			beneficiary_lookup
		)?;
		let caller_key = frame_system::Account::<T>::hashed_key_for(&caller);
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		let collection_id: T::CollectionId = T::Helper::to_collection(2);
		let nft_id: T::ItemId = T::Helper::to_nft(10);
		let loan_apy: LoanApy = 10;
		let contract_account = account("contact", 100, SEED);
		let admin_account = account("admin", 100, SEED);
		let storage_deposit_limit = None;
		let value_funds:BalanceOf1<T> = Default::default();
		let gas_limit: Weight = Weight::from_parts(50, 50);
	}: _(RawOrigin::Signed(caller), proposal_id, collection_id, value, value_funds ,nft_id, loan_apy, contract_account, admin_account ,storage_deposit_limit , gas_limit) */

	impl_benchmark_test_suite!(CommunityLoanPool, crate::mock::new_test_ext(), crate::mock::Test);
}
