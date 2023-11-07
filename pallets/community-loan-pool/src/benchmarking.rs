//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as CommunityLoanPool;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::Saturating;
use frame_system::RawOrigin;
const SEED: u32 = 0;

fn setup_proposal<T: Config>(
	u: u32,
) -> (T::AccountId, BalanceOf<T>, AccountIdLookupOf<T>, u64, u64) {
	let caller = account("caller", u, SEED);
	let value: BalanceOf<T> = T::ProposalBondMinimum::get().saturating_mul(100u32.into());
	let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, value);
	let beneficiary = account("beneficiary", u, SEED);
	let beneficiary_lookup = T::Lookup::unlookup(beneficiary);
	let developer_experience = 13;
	let loan_term = 20;
	(caller, value, beneficiary_lookup, developer_experience, loan_term)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn propose() {
		let (caller, value, beneficiary_lookup, developer_experience, loan_term) =
			setup_proposal::<T>(SEED);

		#[extrinsic_call]
		propose(
			RawOrigin::Signed(caller),
			value,
			beneficiary_lookup,
			developer_experience,
			loan_term,
		);

		assert_last_event::<T>(Event::Proposed { proposal_index: 1 }.into());
	}

	#[benchmark]
	fn add_committee_member() {
		let alice = account("alice", SEED, SEED);
		#[extrinsic_call]
		add_committee_member(RawOrigin::Root, alice);
		assert_eq!(CommunityLoanPool::<T>::voting_committee()[0], account("alice", SEED, SEED));
	}

	#[benchmark]
	fn set_milestones() {
		let alice = account("alice", SEED, SEED);
		CommunityLoanPool::<T>::add_committee_member(RawOrigin::Root.into(), alice);
		let (caller, value, beneficiary_lookup, developer_experience, loan_term) =
			setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller).into(),
			value,
			beneficiary_lookup,
			developer_experience,
			loan_term,
		);
		let alice = account("alice", SEED, SEED);
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		let milestones = get_max_milestones::<T>();
		#[extrinsic_call]
		set_milestones(RawOrigin::Signed(alice), proposal_id, milestones);

		assert_eq!(CommunityLoanPool::<T>::ongoing_votes(proposal_id).unwrap().yes_votes, 1);
	}

	#[benchmark]
	fn vote_on_proposal() {
		let alice = account("alice", SEED, SEED);
		CommunityLoanPool::<T>::add_committee_member(RawOrigin::Root.into(), alice);
		let bob = account("bob", SEED, SEED);
		CommunityLoanPool::<T>::add_committee_member(RawOrigin::Root.into(), bob);
		let (caller, value, beneficiary_lookup, developer_experience, loan_term) =
			setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller).into(),
			value,
			beneficiary_lookup,
			developer_experience,
			loan_term,
		);
		let alice = account("alice", SEED, SEED);
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		let milestones = get_max_milestones::<T>();
		CommunityLoanPool::<T>::set_milestones(
			RawOrigin::Signed(alice).into(),
			proposal_id,
			milestones,
		);
		let bob = account("bob", SEED, SEED);
		#[extrinsic_call]
		vote_on_proposal(RawOrigin::Signed(bob), proposal_id, crate::Vote::Yes);

		assert_eq!(CommunityLoanPool::<T>::ongoing_votes(proposal_id).unwrap().yes_votes, 2);
	}

	/* #[benchmark]
	fn withdraw() {
		let alice = account("alice", SEED, SEED);
		CommunityLoanPool::<T>::add_committee_member(RawOrigin::Root.into(), alice);
		let bob = account("bob", SEED, SEED);
		CommunityLoanPool::<T>::add_committee_member(RawOrigin::Root.into(), bob);
		let (caller, value, beneficiary_lookup, developer_experience, loan_term) =
			setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller.clone()).into(),
			value,
			beneficiary_lookup,
			developer_experience,
			loan_term,
		);
		let alice = account("alice", SEED, SEED);
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		let milestones = get_max_milestones::<T>();
		CommunityLoanPool::<T>::set_milestones(
			RawOrigin::Signed(alice).into(),
			proposal_id,
			milestones,
		);
		let bob = account("bob", SEED, SEED);
		CommunityLoanPool::<T>::vote_on_proposal(
			RawOrigin::Signed(bob).into(),
			proposal_id,
			crate::Vote::Yes,
		);
		run_to_block::<T>(30u32.into());
		assert_eq!(CommunityLoanPool::<T>::ongoing_votes(proposal_id).unwrap().yes_votes, 2);
		assert_eq!(CommunityLoanPool::<T>::ongoing_loans().len(), 1);

		#[extrinsic_call]
		withdraw(RawOrigin::Signed(caller), 1, value);
	} */

	impl_benchmark_test_suite!(CommunityLoanPool, crate::mock::new_test_ext(), crate::mock::Test);
}

fn get_max_milestones<T: Config>() -> BoundedProposedMilestones<T> {
	let max_milestones: u32 = <T as Config>::MaxMilestonesPerProject::get();
	get_milestones::<T>(max_milestones)
}

fn get_milestones<T: Config>(mut n: u32) -> BoundedProposedMilestones<T> {
	let max = <T as Config>::MaxMilestonesPerProject::get();
	if n > max {
		n = max;
	}

	(0..n)
		.map(|_| ProposedMilestone { percentage_to_unlock: Percent::from_percent((100 / n) as u8) })
		.collect::<Vec<ProposedMilestone>>()
		.try_into()
		.expect("qed")
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

/* fn run_to_block<T: Config>(new_block: frame_system::pallet_prelude::BlockNumberFor<T>) {
	frame_system::Pallet::<T>::set_block_number(new_block);
	frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
	CommunityLoanPool::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
} */
