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
	traits::Zero,
};

use frame_support::{
	pallet_prelude::*,
	traits::{Get, ReservableCurrency},
};

use frame_support::traits::UnixTime;

use sp_std::prelude::*;

pub type Balance = u128;

#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct LedgerAccount {
	/// Balance locked
	#[codec(compact)]
	pub locked: Balance,
	/// Timestamp locked
	pub timestamp: u64,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
	};

	use frame_system::{ensure_signed, pallet_prelude::*};

	const EXAMPLE_ID: LockIdentifier = *b"stkxcavc";

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_community_loan_pool::Config{
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
		type TimeProvider: UnixTime;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn ledger)]
	pub type Ledger<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, LedgerAccount, ValueQuery>;

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
		/// The locked period didn't end yet
		UnlockPeriodNotReached,
		/// No staked amount
		NoStakedAmount,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn stake(
			origin: OriginFor<T>,
			#[pallet::compact] value: Balance,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			let mut ledger = Self::ledger(&staker);

			let available_balance = Self::available_staking_balance(&staker, &ledger);
			let value_to_stake = value.min(available_balance);

			let timestamp = <T as pallet::Config>::TimeProvider::now().as_secs();

			ensure!(value_to_stake > 0, Error::<T>::StakingWithNoValue);

			ledger.locked = ledger.locked.saturating_add(value_to_stake);
			ledger.timestamp = timestamp;

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

			<T as pallet::Config>::Currency::extend_lock(EXAMPLE_ID, &user, value, WithdrawReasons::all());

			Self::deposit_event(Event::ExtendedLock(user, value));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn unstake(
			origin: OriginFor<T>,
			#[pallet::compact] value: Balance,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			ensure!(value > 0, Error::<T>::UnstakingWithNoValue);

			let mut ledger = Self::ledger(&staker);

			let minute_timestamp = <T as pallet::Config>::TimeProvider::now().as_secs();

			ensure!(ledger.timestamp + 120 < minute_timestamp, Error::<T>::UnlockPeriodNotReached);
			ledger.locked = ledger.locked.saturating_sub(value);

			Self::update_ledger(&staker, ledger);

			let total_stake = Self::total_stake();
			TotalStake::<T>::put(total_stake - value);

			Self::deposit_event(Event::Unlocked(staker, value));
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(1_000)]
		pub fn claim_rewards(
			origin: OriginFor<T>
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&staker);
			ensure!(ledger.locked == 0, Error::<T>::NoStakedAmount);
			Ok(())
		}
		
	}

	impl<T: Config> Pallet<T> {
		fn available_staking_balance(staker: &T::AccountId, ledger: &LedgerAccount) -> Balance {
			let free_balance =
			<T as pallet::Config>::Currency::free_balance(staker).saturating_sub(T::MinimumRemainingAmount::get());
			free_balance.saturating_sub(ledger.locked)
		}

		fn update_ledger(staker: &T::AccountId, ledger: LedgerAccount) {
			if ledger.locked.is_zero() {
				Ledger::<T>::remove(&staker);
				<T as pallet::Config>::Currency::remove_lock(EXAMPLE_ID, staker);
			} else {
				<T as pallet::Config>::Currency::set_lock(EXAMPLE_ID, staker, ledger.locked, WithdrawReasons::all());
				Ledger::<T>::insert(staker, ledger);
			}
		}

		fn calculate_current_apy() -> u64 {
			let ongoing_loans = pallet_community_loan_pool::Pallet::<T>::ongoing_loans();
			let mut index = 0;
			let loan_apys = 0;
			for _i in ongoing_loans.clone() {
				let loan_index = ongoing_loans[index];
				let loan = pallet_community_loan_pool::Pallet::<T>::loans(&loan_index);
				loan_apys += loan.loan_apy;
				index += 1;
			}
			let average_loan_apy = loan_apys / ongoing_loans.len();
			average_loan_apy.try_into().unwrap()
		}
	}
}
