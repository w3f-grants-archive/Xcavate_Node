//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as CommunityLoanPool;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::sp_runtime::Saturating;
const SEED: u32 = 0;

fn setup_proposal<T: Config>(u: u32) -> (T::AccountId, BalanceOf<T>, BoundedProposedMilestones<T>, AccountIdLookupOf<T>) {
	let caller = account("caller", u, SEED);
	let value: BalanceOf<T> = T::ProposalBondMinimum::get().saturating_mul(100u32.into());
	let _ = <T as pallet::Config>::Currency::make_free_balance_be(&caller, value);
	let milestones = get_max_milestones::<T>();
	let beneficiary = account("beneficiary", u, SEED);
	let beneficiary_lookup = T::Lookup::unlookup(beneficiary);
	(caller, value, milestones, beneficiary_lookup)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn propose() {
		let (caller, value, milestones, beneficiary_lookup) = setup_proposal::<T>(SEED);
		
		#[extrinsic_call]
		propose(RawOrigin::Signed(caller), value, milestones, beneficiary_lookup);

		assert_last_event::<T>(
            Event::Proposed{proposal_index: 1}
            .into(),
        );
	}

	#[benchmark]
	fn reject_proposal() {
		let (caller, value, milestones, beneficiary_lookup) = setup_proposal::<T>(SEED);
		CommunityLoanPool::<T>::propose(
			RawOrigin::Signed(caller).into(),
			value,
			milestones,
			beneficiary_lookup
		);
		let proposal_id = CommunityLoanPool::<T>::proposal_count();
		#[extrinsic_call]
		reject_proposal(RawOrigin::Root, proposal_id);

		assert_last_event::<T>(
            Event::Rejected{proposal_index: proposal_id}
            .into(),
        );
	} 

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
        .map(|_| ProposedMilestone {
            percentage_to_unlock: Percent::from_percent((100 / n) as u8),
        })
        .collect::<Vec<ProposedMilestone>>()
        .try_into()
        .expect("qed")
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}
