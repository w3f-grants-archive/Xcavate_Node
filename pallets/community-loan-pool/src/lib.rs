#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
/// <https://docs.substrate.io/reference/frame-pallets/>
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, Saturating, StaticLookup, Zero},
	Permill, RuntimeDebug,
};

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced,
		ReservableCurrency, WithdrawReasons},
};


type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkHelper<CollectionId, ItemId> {
	fn to_collection(i: u32) -> CollectionId;
	fn to_nft(i: u32) -> ItemId;
}

#[cfg(feature = "runtime-benchmarks")]
impl<CollectionId: From<u32>, ItemId: From<u32>> BenchmarkHelper<CollectionId, ItemId> for NftHelper
{
	fn to_collection(i: u32) -> CollectionId {
		i.into()
	}
	fn to_nft(i: u32) -> ItemId {
		i.into()
	}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};


    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type ProposalIndex = u32;

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Proposal<AccountId, Balance> {
		proposer: AccountId,
		amount: Balance,
		beneficiary: AccountId,
		bond: Balance,
	}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config{
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Origin from which rejections must come.
		type RejectOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin from which approves must come.
		type ApproveOrigin: EnsureOrigin<Self::RuntimeOrigin>;


		/// Fraction of a proposal's value that should be bonded in order to place the proposal.
		/// An accepted proposal gets these back. A rejected proposal does not.
		#[pallet::constant]
		type ProposalBond: Get<Permill>;

		/// The maximum number of approvals that can wait in the spending queue.
		///
		/// NOTE: This parameter is also used within the Bounties Pallet extension if enabled.
		#[pallet::constant]
		type MaxApprovals: Get<u32>;

		/// Minimum amount of funds that should be placed in a deposit for making a proposal.
		#[pallet::constant]
		type ProposalBondMinimum: Get<BalanceOf<Self>>;
		
		/// Maximum amount of funds that should be placed in a deposit for making a proposal.
		#[pallet::constant]
		type ProposalBondMaximum: Get<Option<BalanceOf<Self>>>;

    }

	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(super) type ProposalCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub(super) type Proposals<T: Config> = StorageMap<
		_, 
		Twox64Concat,
		ProposalIndex,
		Proposal<T::AccountId, BalanceOf<T>>,
		OptionQuery,
		>;

	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub(super) type Approvals<T: Config> = 
		StorageValue<_, BoundedVec<ProposalIndex, T::MaxApprovals>, ValueQuery>;


    #[pallet::error]
    pub enum Error<T> {
		/// Proposer's balance is too low
		InsufficientProposerBalance,
		/// Loan pool's balance is too low
		InsufficientLoanPoolBalance,
		/// No proposal index
		InvalidIndex,
		/// There are too many loan proposals
		TooManyProposals,
		/// The person has no permission to call the function
		InsufficientPermission,
		TooManyApprovals,
    }


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		/// New Proposal
		Proposed {proposal_index: ProposalIndex},
		/// Proposal has been approved
		Approved {proposal_index: ProposalIndex},
		/// Proposal has been rejected
		Rejected {proposal_index: ProposalIndex},
		/// Loan has been created
		Loancreation {proposal_index: ProposalIndex}
    }




    #[pallet::call]
    impl<T: Config> Pallet<T> {
		/// Apply for a loan
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn propose(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			beneficiary: AccountIdLookupOf<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let proposal_index = Self::proposal_count();
			let bond = Self::calculate_bond(amount);

			let proposal = Proposal {
				proposer: origin.clone(),
				amount,
				beneficiary: beneficiary.clone(),
				bond,
			};
			Proposals::<T>::insert(proposal_index, proposal);
			ProposalCount::<T>::put(proposal_index + 1);

			Self::deposit_event(Event::Proposed {proposal_index});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn reject_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
		) -> DispatchResult {
			T::RejectOrigin::ensure_origin(origin)?;
			let proposal = <Proposals<T>>::take(&proposal_index).ok_or(Error::<T>::InvalidIndex).unwrap();
			let value = proposal.bond;
/* 			let imbalance = T::Currency::slash_reserved(&proposal.proposer, value).0;
			T::OnSlash::on_unbalanced(imbalance); */

			Self::deposit_event(Event::<T>::Rejected {
				proposal_index,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
		) -> DispatchResult {
		 	let signer = ensure_signed(origin.clone())?;		
			let proposal = <Proposals<T>>::take(&proposal_index).ok_or(Error::<T>::InvalidIndex).unwrap();
			let user = proposal.beneficiary;
			let beneficiary = <T::Lookup as frame_support::sp_runtime::traits::StaticLookup>::unlookup(user.clone());

			pallet_uniques::Pallet::<T>::create(origin.clone(), collection_id, beneficiary.clone());
			pallet_uniques::Pallet::<T>::mint(origin.clone(),  collection_id, item_id, beneficiary.clone());
			// Call the nft creation, mint it and store it at the contract
			// creates a contract and sends the loan amount to the contract
			Approvals::<T>::try_append(proposal_index)
				.map_err(|_| Error::<T>::TooManyApprovals)?;
			Ok(())
		}
    }

    //** Our helper functions.**//

    impl<T: Config> Pallet<T> {
		fn calculate_bond(value: BalanceOf<T>) -> BalanceOf<T> {
			let mut r = T::ProposalBondMinimum::get().max(T::ProposalBond::get() * value);
			if let Some(m) = T::ProposalBondMaximum::get() {
				r = r.min(m);
			}
			r
		}
    }
}