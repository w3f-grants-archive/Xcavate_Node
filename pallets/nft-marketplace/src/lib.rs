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

use frame_support::{
	traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency},
	PalletId,
};

use frame_support::sp_runtime::traits::AccountIdConversion;

use enumflags2::BitFlags;

pub use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type BalanceOf1<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[cfg(feature = "runtime-benchmarks")]
pub struct NftHelper;

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkHelper<CollectionId, ItemId> {
	pub fn to_collection(i: u32) -> CollectionId;
	pub fn to_nft(i: u32) -> ItemId;
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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type ListedNftIndex = u32;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NnftDog<Balance, CollectionId, ItemId, T: Config> {
		pub real_estate_developer: AccountIdOf<T>,
		pub owner: AccountIdOf<T>,
		pub price: Balance,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
		pub sold: bool,
	}

	/// AccountId storage
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type.
		type Currency: Currency<AccountIdOf<Self>> + ReservableCurrency<AccountIdOf<Self>>;

		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<Self::CollectionId, Self::ItemId>;

		/// The maximum amount of nfts that can be listed at the same time.
		#[pallet::constant]
		type MaxListedNfts: Get<u32>;

		/// The maximum amount of nfts for a collection.
		type MaxNftInCollection: Get<u32>;
	}

	/// Number of nfts that have been listed.
	#[pallet::storage]
	#[pallet::getter(fn listed_nft_count)]
	pub(super) type ListedNftCount<T> = StorageValue<_, ListedNftIndex, ValueQuery>;

	/// All currently ongoing loans
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub(super) type ListedNfts<T: Config> =
		StorageValue<_, BoundedVec<ListedNftIndex, T::MaxListedNfts>, ValueQuery>;

	/// Milestone proposal that has been made.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_listings)]
	pub(super) type OngoingListings<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ListedNftIndex,
		NnftDog<BalanceOf<T>, T::CollectionId, T::ItemId, T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn listed_collection)]
	pub(super) type ListedCollection<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<NnftDog<BalanceOf<T>, T::CollectionId, T::ItemId, T>, T::MaxNftInCollection>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace
		ObjectListed { collection_index: T::CollectionId, price: BalanceOf<T>, seller: AccountIdOf<T> },
		/// A nft has been bought
		NftBought { collection_index: T::CollectionId, item_index: T::ItemId, buyer: AccountIdOf<T>, price: BalanceOf<T> }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Max amount of listed Nfts reached
		TooManyListedNfts,
		/// This index is not taken
		InvalidIndex,
		/// Too many nfts for this collection
		TooManyNfts,
		/// The buyer doesn't have enough funds
		NotEnoughFunds,
		/// The collection has not been found
		CollectionNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_nfts::Config>::CollectionId: From<u32>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
	{
		/// List a real estate object. A new collection is created and 100 nfts get minted.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn list_object(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult
		where
			<T as pallet_nfts::Config>::CollectionId: From<u32>,
			<T as pallet_nfts::Config>::ItemId: From<u32>,
		{
			let origin = ensure_signed(origin)?;
			let collection_id: T::CollectionId = 1.into();

			let collection = pallet_nfts::Pallet::<T>::do_create_collection(
				collection_id,
				Self::account_id(),
				Self::account_id(),
				Self::default_collection_config(),
				T::CollectionDeposit::get(),
				pallet_nfts::Event::Created {
					creator: Self::account_id(),
					owner: Self::account_id(),
					collection: collection_id,
				},
			)?;
			for x in 1..11 {
				let nft_index = Self::listed_nft_count() + 1;
				let item_id: T::ItemId = x.into();
				let nft = NnftDog {
					real_estate_developer: origin.clone(),
					owner: Self::account_id(),
					price: price / Self::u64_to_balance_option(10).unwrap(),
					collection_id,
					item_id,
					sold: Default::default(),
				};
				pallet_nfts::Pallet::<T>::do_mint(
					collection_id,
					item_id,
					Some(Self::account_id()),
					Self::account_id(),
					Self::default_item_config(),
					|_, _| Ok(()),
				)?;
				OngoingListings::<T>::insert(nft_index, nft);
				ListedNftCount::<T>::put(nft_index);
				ListedNfts::<T>::try_append(nft_index)
					.map_err(|_| Error::<T>::TooManyListedNfts)?;
			}
			Self::deposit_event(Event::<T>::ObjectListed {
				collection_index: collection_id,
				price,
				seller: origin,
			});
			Ok(())
		}

		/// Buying a certain nft.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn buy_nft(origin: OriginFor<T>, listed_nft_index: ListedNftIndex) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let mut nft =
				<OngoingListings<T>>::take(listed_nft_index).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(
				<T as pallet::Config>::Currency::free_balance(&origin) >= nft.price,
				Error::<T>::NotEnoughFunds
			);
			nft.owner = origin.clone();
			nft.sold = true;
			ListedCollection::<T>::try_mutate(nft.collection_id.clone(), |keys| {
				keys.try_push(nft.clone()).map_err(|_| Error::<T>::TooManyNfts)?;
				Ok::<(), DispatchError>(())
			})?;
			<T as pallet::Config>::Currency::transfer(
				&origin,
				&Self::account_id(),
				// For unit tests this line has to be commented out and the line blow has to be uncommented due to the dicmals on polkadot js
				nft.price * Self::u64_to_balance_option(1000000000000).unwrap(),
				//amount,
				KeepAlive,
			)
			.unwrap_or_default();
			let mut ongoing_listings = Self::listed_nfts();
			let index = ongoing_listings.iter().position(|x| *x == listed_nft_index).unwrap();
			ongoing_listings.remove(index);
			ListedNfts::<T>::put(ongoing_listings);
			OngoingListings::<T>::insert(listed_nft_index, nft);
			let nft = Self::ongoing_listings(listed_nft_index).unwrap();
			if Self::listed_collection(nft.collection_id).len() == 10 {
				Self::distribute_nfts(nft.collection_id);
			}
			Self::deposit_event(Event::<T>::NftBought { collection_index: nft.collection_id, item_index: nft.item_id, buyer: origin, price: nft.price });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_account_truncating()
		}

		fn distribute_nfts(collection_id: T::CollectionId) -> DispatchResult {
			let list = <ListedCollection<T>>::take(collection_id);
			<T as pallet::Config>::Currency::transfer(
				&Self::account_id(),
				&list[0].real_estate_developer,
				// For unit tests this line has to be commented out and the line blow has to be uncommented due to the dicmals on polkadot js
				list[0].price
					* Self::u64_to_balance_option(10).unwrap()
					* Self::u64_to_balance_option(1000000000000).unwrap(),
				//amount,
				KeepAlive,
			)
			.unwrap_or_default();
			for x in list {
				pallet_nfts::Pallet::<T>::do_transfer(
					collection_id,
					x.item_id,
					x.owner,
					|_, _| Ok(()),
				)?;
			}
			Ok(())
		}

		fn default_collection_config(
		) -> CollectionConfig<BalanceOf1<T>, BlockNumberFor<T>, T::CollectionId> {
			Self::collection_config_from_disabled_settings(
				CollectionSetting::DepositRequired.into(),
			)
		}

		fn collection_config_from_disabled_settings(
			settings: BitFlags<CollectionSetting>,
		) -> CollectionConfig<BalanceOf1<T>, BlockNumberFor<T>, T::CollectionId> {
			CollectionConfig {
				settings: CollectionSettings::from_disabled(settings),
				max_supply: None,
				mint_settings: MintSettings::default(),
			}
		}

		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}

		pub fn u64_to_balance_option(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}
	}
}
