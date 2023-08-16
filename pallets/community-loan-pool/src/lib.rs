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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced,
			ReservableCurrency, WithdrawReasons},
    };
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_io::hashing::blake2_128;
    use sp_runtime::ArithmeticError;

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
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The Currency handler for the kitties pallet.
        type Currency: Currency<Self::AccountId>;

    }

	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(crate) type ProposalCount<T, I = ()> = StorageValue<_, ProposalIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config>, I: 'static = ()> = StorageMap<
		_, 
		Twox64Concat,
		ProposalIndex,
		Proposal<T::AccountId, BalanceOf<T,I>>,
		OptionQuery,
		>;

	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type Approvals<T: Config<I>, I: 'static = ()> = 
		StorageValue<_, BoundedVec<ProposalIndex, T::MaxApprovals>, ValueQuery>;


    #[pallet::error]
    pub enum Error<T> {
		/// Proposer's balance is too low
		InsufficientProposerBalance
		/// Loan pool's balance is too low
		InsufficientLoanPoolBalance
		/// No proposal index
		InvalidIndex
		/// There are too many loan proposals
		TooManyProposals
		/// The person has no permission to call the function
		InsufficientPermission
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
			amount: BalanceOf<T, I>,
			beneficiary: AccountIdLookup<T>,
		) -> DispatchResult {
			let origin = ensure_origin(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let proposal_index = Self::proposal_count();
			let proposal = Proposal {
				proposer: origin.clone(),
				value: amount,
				beneficiary: beneficiary.clone(),
				bond: 2000,
			};
			Proposals::<T, I>::insert(proposal_index, proposal);
			ProposalCount::<T, I>::put(proposal_index + 1);

			Self::deposit_event(Event::T::Proposed {proposal_index});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn reject_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
		) -> DispatchResult {
			let origin = ensure_origin(origin)?;
			let proposal = <Proposals<T, I>>::take(&proposal_id).ok_or(Error::<T, I>::InvalidIndex)?;
			let value = proposal.bond;
			let imbalance = T::Currency::slash_reserved(&proposal.proposer, value).0;
			T::OnSlash::on_unbalanced(imbalance);

			Self::deposit_event(Event::<T, I>::Rejected {
				proposal_index: proposal_id,
				slashed: value,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
		) -> DisptachResult {
			let origin = ensure_origin(origin)?;
			
			ensure!(<Proposals<T,I>>contains_key(proposal_id), Error::<T, I>::InvalidIndex);
			// Call the nft creationm ,mint it and store it at the contract
			// creates a contract and sends the loan amount to the contract
			Approvals::<T, I>::try_append(proposal_id)
				.map_err(|_| Error::<T, I>::TooManyApprovals)?;
			Ok(())
		}
    }

    //** Our helper functions.**//

    impl<T: Config> Pallet<T> {

		// apy calculation
		// 9 %
    
    }
}