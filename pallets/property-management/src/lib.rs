#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
	traits::{
	Currency, ReservableCurrency, 
	ExistenceRequirement::KeepAlive, OnUnbalanced
	},
	PalletId,
};

use frame_support::sp_runtime::{
	traits::{
	AccountIdConversion, CheckedMul, CheckedAdd, CheckedDiv, Zero,
	},
};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Proposal with the proposal Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct LettingAgentInfo<T: Config> {
		pub account: AccountIdOf<T>,
		pub location: u32,
		pub assigned_properties: BoundedVec<u32, T::MaxProperties>,
		pub deposited: bool,
	}	

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config 
		+ pallet_nfts::Config 
		+ pallet_whitelist::Config 
		+ pallet_nft_marketplace::Config 
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The reservable currency type.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>;
		/// The property management's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// Minimum amount that should be left on letting agent account.
		#[pallet::constant]
		type MinimumRemainingAmount: Get<BalanceOf<Self>>;
		/// Origin who can set a new letting agent.
		type AgentOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// The minimum amount of a letting agent that has to be staked.
		type MinStakingAmount: Get<BalanceOf<Self>>;
		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nft_marketplace::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Default
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Item id type from pallet nfts.
		type ItemId: IsType<<Self as pallet_nft_marketplace::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		
		/// Handler for the unbalanced reduction when slashing a letting agent.
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// The maximum amount of properties that can be assigned to a letting agent.
		#[pallet::constant]
		type MaxProperties: Get<u32>;
		/// The maximum amount of letting agents in a location.
		#[pallet::constant]
		type MaxLettingAgents: Get<u32>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	/// Mapping from the real estate object to the letting agent
	#[pallet::storage]
	#[pallet::getter(fn letting_storage)]
	pub type LettingStorage<T> = StorageMap<
		_, 
		Blake2_128Concat,
		u32,
		AccountIdOf<T>,
		OptionQuery,
	>;

	/// Mapping from account to currently stored balance.
	#[pallet::storage]
	#[pallet::getter(fn stored_funds)]
	pub type StoredFunds<T> = StorageMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Mapping from account to letting agent info
	#[pallet::storage]
	#[pallet::getter(fn letting_info)]
	pub type LettingInfo<T: Config> = 
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, LettingAgentInfo<T>, OptionQuery>;

	/// Mapping from letting agent to the properties.
	#[pallet::storage]
	#[pallet::getter(fn letting_agent_locations)]
	pub type LettingAgentLocations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		BoundedVec<AccountIdOf<T>, T::MaxLettingAgents>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new letting agent got set.
		LettingAgentAdded { 
			location: u32,
			who: T::AccountId 
		},
		/// A letting agent deposited the necessary funds.
		Deposited {
			who: T::AccountId,
		},
		/// A letting agent has been added to a property.
		LettingAgentSet {
			asset_id: u32,
			who: T::AccountId,
		},
		/// The rental income has been distributed.
		IncomeDistributed {
			asset_id: u32,
			amount: BalanceOf<T>,
		},
		/// A user withdrew funds.
		WithdrawFunds {
			who: T::AccountId,
			amount: BalanceOf<T>,
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Error by convertion to balance type.
		ConversionError,
		/// Error by dividing a number.
		DivisionError,
		/// Error by multiplying a number.
		MultiplyError,
		ArithmeticOverflow,
		/// One person in the list is not an owner of the property.
		UserNotPropertyOwner,
		/// The call has no funds stored.
		UserHasNoFundsStored,
		/// The pallet has not enough funds.
		NotEnoughFunds,
		/// The letting agent already has too many assigned properties.
		TooManyAssignedProperties,
		/// No letting agent could be selected.
		NoLettingAgentFound,
		/// The location is not registered.
		LocationUnknown,
		/// The location has already the maximum amount of letting agents.
		TooManyLettingAgents,
		/// The user is not a property owner and has no permission to deposit.
		NoPermission,
		/// The letting agent of this property is already set.
		LettingAgentAlreadySet,
		/// The nft could not be found.
		NoNftFound,
		/// The account is not a letting agent of this location.
		AgentNotFound,
		/// The letting already deposited the necessary amount.
		AlreadyDeposited,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Sets the letting agent for an object real estate object.
		/// - Parameter are origin, collection id, nft id and account id.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn add_letting_agent(
			origin: OriginFor<T>, 
			location: u32,
			letting_agent: AccountIdOf<T>,
		) -> DispatchResult {
			T::AgentOrigin::ensure_origin(origin)?;
			ensure!(pallet_nft_marketplace::Pallet::<T>::location_collections(location).is_some(), Error::<T>::LocationUnknown);
			let letting_info = LettingAgentInfo {
				account: letting_agent.clone(),
				location,
				assigned_properties: Default::default(),
				deposited: Default::default(),
			};	
			LettingInfo::<T>::insert(letting_agent.clone(), letting_info);
 			Self::deposit_event(Event::<T>::LettingAgentAdded {
				location,
				who: letting_agent,
			}); 
			Ok(())
		}

		/// Lets the letting agent stake his funds that could be slashed if he acts
		/// malicious
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn letting_agent_deposit(origin: OriginFor<T>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			<T as pallet::Config>::Currency::reserve(&origin, <T as Config>::MinStakingAmount::get())?;
			let mut letting_info = Self::letting_info(origin.clone()).ok_or(Error::<T>::NoPermission)?;
			ensure!(!Self::letting_agent_locations(letting_info.location).contains(&origin), Error::<T>::AlreadyDeposited);
			letting_info.deposited = true;
			LettingAgentLocations::<T>::try_mutate(letting_info.location, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyLettingAgents)?;
				Ok::<(), DispatchError>(())
			})?;
			LettingInfo::<T>::insert(origin.clone(), letting_info);
			Self::deposit_event(Event::<T>::Deposited {
				who: origin,
			});
			Ok(())
		}

		/// Set the letting agent for a property.
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn set_letting_agent(
			origin: OriginFor<T>, 
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
		) -> DispatchResult {
			let _origin = ensure_signed(origin)?;
			let nft_details = pallet_nft_marketplace::Pallet::<T>::registered_nft_details(collection_id.into(), item_id.into()).ok_or(Error::<T>::NoNftFound)?;
			ensure!(Self::letting_storage(nft_details.asset_id).is_none(), Error::<T>::LettingAgentAlreadySet);
			Self::selects_letting_agent(nft_details.location, nft_details.asset_id)?;
			Ok(())
		}

		///	The letting agent can distribute the rental income.
		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn distribute_income(origin: OriginFor<T>, asset_id: u32, amount: BalanceOf<T>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			<T as pallet::Config>::Currency::transfer(
				&origin,
				&Self::account_id(),
				amount.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
					.ok_or(Error::<T>::MultiplyError)?,
				KeepAlive,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(asset_id);
			for owner in owner_list {
				let token_amount = pallet_nft_marketplace::Pallet::<T>::property_owner_token(asset_id, owner.clone());
				let amount_for_owner = Self::u64_to_balance_option(token_amount as u64)?
				.checked_mul(&amount)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
				let mut old_funds = Self::stored_funds(owner.clone());
				old_funds = old_funds.checked_add(&amount_for_owner).ok_or(Error::<T>::ArithmeticOverflow)?;
				StoredFunds::<T>::insert(owner, old_funds);
			};
			Self::deposit_event(Event::<T>::IncomeDistributed {
				asset_id,
				amount,
			});
			Ok(())
		}

		/// A property owner can withdraw the collected funds.
		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn withdraw_funds(origin: OriginFor<T>) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Self::stored_funds(origin.clone()).is_zero(), Error::<T>::UserHasNoFundsStored);
			let user_funds = StoredFunds::<T>::take(origin.clone());
			<T as pallet::Config>::Currency::transfer(
				&Self::account_id(), 
				&origin, 
				user_funds, 
				KeepAlive,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			Self::deposit_event(Event::<T>::WithdrawFunds {
				who: origin,
				amount: user_funds,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}
		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}

		/// Chooses the next free letting agent in a location
		pub fn selects_letting_agent(location: u32, asset_id: u32) -> DispatchResult {
			let letting_agents = Self::letting_agent_locations(location);
			let letting_agent = letting_agents.iter().min_by_key(|letting_agent| {
				Self::letting_info(letting_agent).unwrap().assigned_properties
			}).ok_or(Error::<T>::NoLettingAgentFound)?;
			LettingStorage::<T>::insert(asset_id, letting_agent);
			let mut letting_info = Self::letting_info(letting_agent).ok_or(Error::<T>::AgentNotFound)?;
			letting_info.assigned_properties.try_push(asset_id).map_err(|_| Error::<T>::TooManyAssignedProperties)?;
			LettingInfo::<T>::insert(letting_agent, letting_info);
			Self::deposit_event(Event::<T>::LettingAgentSet {
				asset_id,
				who: letting_agent.clone(),
			}); 
			Ok(())
		}

		pub fn remove_bad_letting_agent(location: u32, agent: AccountIdOf<T>) -> DispatchResult {
			let mut letting_agents = Self::letting_agent_locations(location);
			let index = letting_agents.iter().position(|x| *x == agent).ok_or(Error::<T>::AgentNotFound)?;
			letting_agents.remove(index);
			LettingAgentLocations::<T>::insert(location, letting_agents);
			Ok(())
		}
	}
}