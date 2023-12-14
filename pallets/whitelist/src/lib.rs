#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
/* pub mod weights;
pub use weights::*; */

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		//type WeightInfo: WeightInfo;
		type WhitelistOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Max users allowed in the whitelist.
		type MaxUsersInWhitelist: Get<u32>;
	}

	/// Mapping of an account to a bool.
	#[pallet::storage]
	#[pallet::getter(fn whitelisted_accounts)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type WhitelistedAccounts<T: Config> = StorageValue<_,BoundedVec<AccountIdOf<T>, T::MaxUsersInWhitelist>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new user has been successfully whitelisted.
		NewUserWhitelisted { user: T::AccountId },
		/// A new user has been successfully whitelisted.
		UserRemoved { user: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The user is already registered in the whitelist.
		AccountAlreadyWhitelisted,
		/// The user has not been registered in the whitelist.
		UserNotInWhitelist,
		/// Too many users are already in the whitelist.
		TooManyUsers,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn add_to_whitelist(origin: OriginFor<T>, user: AccountIdOf<T>) -> DispatchResult {

			T::WhitelistOrigin::ensure_origin(origin)?;
			let current_whitelist = Self::whitelisted_accounts();
			ensure!(!current_whitelist.contains(&user), Error::<T>::AccountAlreadyWhitelisted);
			WhitelistedAccounts::<T>::try_append(user.clone()).map_err(|_| Error::<T>::TooManyUsers)?;
			Self::deposit_event(Event::<T>::NewUserWhitelisted { user });
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn remove_from_whitelist(origin: OriginFor<T>, user: AccountIdOf<T>) -> DispatchResult {
			T::WhitelistOrigin::ensure_origin(origin)?;
			let mut current_whitelist = Self::whitelisted_accounts();
			ensure!(current_whitelist.contains(&user), Error::<T>::UserNotInWhitelist);
			let index = current_whitelist
				.iter()
				.position(|x| *x == user)
				.ok_or(Error::<T>::UserNotInWhitelist)?;
			current_whitelist.remove(index);
			WhitelistedAccounts::<T>::put(current_whitelist);
			Self::deposit_event(Event::<T>::UserRemoved { user });
			Ok(())
		}
	}
}
