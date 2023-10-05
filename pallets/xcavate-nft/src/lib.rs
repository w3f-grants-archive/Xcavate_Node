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

pub type CollectionId = u32,

pub type ItemId = i32,

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct CollectionDetails<AccountId> {
		pub(super) owner: AccountId,
		pub(super) items: u32,
		pub(super) item_metadatas: u32,
		pub(super) item_configs: u32,
		pub(super) attributes: u32,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	/// Details of a collection.
	#[pallet::storage]
	pub type Collection<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		CollectionDetails<T::AccountId>,
	>;

	/// The items in existence and their ownership details.
	#[pallet::storage]
	pub type Item<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CollectionId,
		Blake2_128Concat,
		T::ItemId,
		T::AccountId,
		OptionQuery,
	>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn unique_asset)]
	pub(super) type UniqueAsset<T: Config> =
		StorageMap<_, Blake2_128Concat, UniqueAssetId, UniqueAssetDetails<T, T::MaxLength>>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	/// The holdings of a specific account for a specific asset.
	pub(super) type Account<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		UniqueAssetId,
		Blake2_128Concat,
		T::AccountId,
		u128,
		ValueQuery,
	>;

	/// Stores the `CollectionId` that is going to be used for the next collection.
	/// This gets incremented whenever a new collection is created.
	#[pallet::storage]
	pub type NextCollectionId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::CollectionId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New unique asset created
		CollectionCreated {
			creator: T::AccountId,
			collection_id: CollectionId,
		},

		Issued {
			collection: CollectionId,
			item: ItemId,
			owner: T::AccountId,
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The asset ID is unknown
		UnknownAssetId,
		/// The signing account does not own any amount of this asset
		NotOwned,
		/// Supply must be positive
		NoSupply,
		CollectionIdInUse,
		UnknownCollection,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_collection(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			let collection = NextCollectionId::<T, I>::get()
			.or(T::CollectionId::initial_value())
			.ok_or(Error::<T, I>::UnknownCollection)?;

			ensure!(!Collection::<T, I>::contains_key(collection), Error::<T, I>::CollectionIdInUse);
			Collection::<T, I>::insert()
			NextCollectionId::<T>::set(id.saturating_add(1));
			Self::deposit_event(Event::<T>::CollectionCreated{creator: origin, collection_id: collection});
			Ok(())
		}

		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn mint(
			origin: OriginFor<T>,
			collection: CollectionId,
			item: ItemId,
			owner: T::AccountId,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			Collection::<T>::try_mutate(&collection |maybe_collection_details| -> DispatchResult{
				let collection_details = maybe_collection_details.as_mut().ok_or(Error::<T>::UnknownCollection)?;

				collection_details.items.saturating_inc();

				Account::<T>::insert((&owner, &collection, &item), ());
				Item::<T>::insert(&collection, &item, owner.clone());
				Ok(())
			})?;

			Self::deposit_event(Event::<T>::Issued {
				collection,
				item,
				owner,
			});
			Ok(())
		}
	}
}