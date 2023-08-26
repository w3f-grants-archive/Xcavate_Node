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
    traits::{AccountIdConversion, Zero},
    Perbill,
};

use frame_support::{
	pallet_prelude::*,
	traits::{
		Get, ReservableCurrency,		
	},
};



#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, 
		traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons}
	};

	use frame_system::pallet_prelude::*;
	use frame_system::ensure_signed;

	const EXAMPLE_ID: LockIdentifier = *b"stkxcavc";

	type Balance = u128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The lockable currency type.
		type Currency: Currency<Self::AccountId, Balance = Balance>
		+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
		+ ReservableCurrency<Self::AccountId>;
		/// Minimum amount that should be left on staker account after staking.
        /// Serves as a safeguard to prevent users from locking their entire free balance.
        #[pallet::constant]
        type MinimumRemainingAmount: Get<Balance>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Balance, ValueQuery>;

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	pub(super) type TotalStake<T> = StorageValue<_, Balance, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {	
		/// Balance was locked successfully.
		Locked(<T as frame_system::Config>::AccountId, Balance),
		/// Lock was extended successfully.
		ExtendedLock(<T as frame_system::Config>::AccountId, Balance),
		/// Balance was unlocked successfully.
		Unlocked(<T as frame_system::Config>::AccountId, Balance),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Can not stake with zero value.
		StakingWithNoValue,
        /// Unstaking a contract with zero value
        UnstakingWithNoValue,
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn stake(
			origin: OriginFor<T>,
			#[pallet::compact] value: Balance
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			let mut ledger = Self::ledger(&staker);

			let available_balance = Self::available_staking_balance(&staker);
			let value_to_stake = value.min(available_balance);

			ensure!(
				value_to_stake > 0,
				Error::<T>::StakingWithNoValue
			);

			ledger = ledger.saturating_add(value_to_stake);

			Self::update_ledger(&staker, ledger);

			let total_stake = Self::total_stake();
			TotalStake::<T>::put(total_stake + value_to_stake);

			Self::deposit_event(Event::Locked(staker, value));
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
		pub fn extend_lock(
			origin: OriginFor<T>,
			#[pallet::compact] value: Balance,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			T::Currency::extend_lock(
				EXAMPLE_ID,
				&user,
				value,
				WithdrawReasons::all(),
			);

			Self::deposit_event(Event::ExtendedLock(user, value));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn unstake(
			origin: OriginFor<T>,
			#[pallet::compact] value: Balance
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			ensure!(value > 0, Error::<T>::UnstakingWithNoValue);

			let mut ledger = Self::ledger(&staker);
			ledger = ledger.saturating_sub(value);

			Self::update_ledger(&staker, ledger);

			let total_stake = Self::total_stake();
			TotalStake::<T>::put(total_stake - value);

			Self::deposit_event(Event::Unlocked(staker, value));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn available_staking_balance(staker: &T::AccountId) -> Balance {
			let free_balance = T::Currency::free_balance(staker).saturating_sub(T::MinimumRemainingAmount::get());
			free_balance.saturating_sub(Self::ledger(staker))
		}

		fn update_ledger(staker: &T::AccountId, ledger: Balance) {
			if ledger == 0 {
				Ledger::<T>::remove(&staker);
				T::Currency::remove_lock(EXAMPLE_ID, staker);
			} else {
				T::Currency::set_lock(EXAMPLE_ID, staker, ledger, WithdrawReasons::all());
				Ledger::<T>::insert(staker, ledger);
			}
		}
	}
}

