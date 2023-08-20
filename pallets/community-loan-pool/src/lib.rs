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
	traits::{
		Currency, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced,
		ReservableCurrency, WithdrawReasons,
	},
	PalletId,
	inherent::Vec,
};

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkHelper<CollectionId, ItemId> {
	fn to_collection(i: u32) -> CollectionId;
	fn to_nft(i: u32) -> ItemId;
}

#[cfg(feature = "runtime-benchmarks")]
impl<CollectionId: From<u32>, ItemId: From<u32>> BenchmarkHelper<CollectionId, ItemId>
	for NftHelper
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

	/// A loan proposal
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

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config /*+ pallet_contracts::Config */ {
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

		/// Minimum amount of funds that should be placed in a deposit for making a proposal.
		#[pallet::constant]
		type ProposalBondMinimum: Get<BalanceOf<Self>>;

		/// Maximum amount of funds that should be placed in a deposit for making a proposal.
		#[pallet::constant]
		type ProposalBondMaximum: Get<Option<BalanceOf<Self>>>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Handler for the unbalanced decrease when slashing for a rejected proposal or bounty.
		type OnSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(super) type ProposalCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub(super) type Proposals<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ProposalIndex,
		Proposal<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Proposer's balance is too low
		InsufficientProposersBalance,
		/// Loan pool's balance is too low
		InsufficientLoanPoolBalance,
		/// No proposal index
		InvalidIndex,
		/// There are too many loan proposals
		InsufficientPermission,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New Proposal
		Proposed { proposal_index: ProposalIndex },
		/// Proposal has been approved
		Approved { proposal_index: ProposalIndex },
		/// Proposal has been rejected
		Rejected { proposal_index: ProposalIndex },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Apply for a loan. A deposit amount is reserved
		/// and slashed if the proposal is rejected. It is returned once the proposal is awarded. 
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
			<T as pallet::Config>::Currency::reserve(&origin, bond)
				.map_err(|_| Error::<T>::InsufficientProposersBalance)?;

			let proposal = Proposal {
				proposer: origin.clone(),
				amount,
				beneficiary: beneficiary.clone(),
				bond,
			};
			Proposals::<T>::insert(proposal_index, proposal);
			ProposalCount::<T>::put(proposal_index + 1);

			Self::deposit_event(Event::Proposed { proposal_index });
			Ok(())
		}

		/// Reject a proposed spend. The original deposit will be slashed.
		///
		/// May only be called from `T::RejectOrigin`.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn reject_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
		) -> DispatchResult {
			T::RejectOrigin::ensure_origin(origin)?;
			let proposal =
				<Proposals<T>>::take(&proposal_index).ok_or(Error::<T>::InvalidIndex)?;
			let value = proposal.bond;
			let imbalance = <T as pallet::Config>::Currency::slash_reserved(&proposal.proposer, value).0;
			T::OnSlash::on_unbalanced(imbalance); 

			Proposals::<T>::remove(proposal_index);

			Self::deposit_event(Event::<T>::Rejected { proposal_index });
			Ok(())
		}

		/// Approve a proposed spend. The original deposit will be released.
		/// It will call the create_loan function in the contract and deposit the proposal amount in the contract.
		///
		/// May only be called from `T::ApproveOrigin`.

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			proposal_index: ProposalIndex,
			collection_id: T::CollectionId,
			collateral_price: BalanceOf<T>,
			item_id: T::ItemId,
			dest: T::AccountId,
			admin: T::AccountId,
			//mut selector: Vec<u8>,
		) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin.clone())?;
			let proposal =
				<Proposals<T>>::take(&proposal_index).ok_or(Error::<T>::InvalidIndex)?;
			let err_amount = <T as pallet::Config>::Currency::unreserve(&proposal.proposer, proposal.bond);
			debug_assert!(err_amount.is_zero());
			let user = proposal.beneficiary;
			let value = proposal.amount;
			let contract =
				<T::Lookup as frame_support::sp_runtime::traits::StaticLookup>::unlookup(
					dest.clone(),
				);
			let admin = <T::Lookup as frame_support::sp_runtime::traits::StaticLookup>::unlookup(
				admin.clone(),
			);

			pallet_uniques::Pallet::<T>::create(origin.clone(), collection_id, admin.clone());
			pallet_uniques::Pallet::<T>::mint(
				origin.clone(),
				collection_id,
				item_id,
				contract.clone(),
			);
			let gas_limit = 10_000_000;
			let value = proposal.amount;
			let mut arg1_enc: Vec<u8> = admin.encode();
			let mut arg2_enc: Vec<u8> = collection_id.clone().encode();
			let mut arg3_enc: Vec<u8> = item_id.clone().encode();
			let mut arg4_enc: Vec<u8> = collateral_price.encode();
			let mut arg5_enc: Vec<u8> = value.clone().encode();
			let mut data = Vec::new();
			//data.append(&mut selector);
			data.append(&mut arg1_enc);
			data.append(&mut arg2_enc);
			data.append(&mut arg3_enc);
			data.append(&mut arg4_enc);
			data.append(&mut arg5_enc);

/* 			pallet_contracts::Pallet::<T>::bare_call(
				signer.clone(),
				dest.clone(),
				value,
				gas_limit,
				storage_deposit_limit,
				data,
				false,
			)
			.result?; */

			// call the contract
			// creates a contract and sends the loan amount to the contract
			Proposals::<T>::remove(proposal_index);
			Self::deposit_event(Event::<T>::Approved { proposal_index });
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
