#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
	traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency, Incrementable},
	PalletId,
};

use frame_support::sp_runtime::traits::{AccountIdConversion, CheckedDiv, CheckedMul};

use enumflags2::BitFlags;

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use codec::EncodeLike;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type BalanceOf1<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type ListedNftIndex = u32;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub struct NftHelper;

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

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftDetails<Balance, CollectionId, ItemId, T: Config> {
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

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet.
		type WeightInfo: WeightInfo;

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

	/// Number of nft collections that have been listed.
	#[pallet::storage]
	#[pallet::getter(fn collection_count)]
	pub(super) type CollectionCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Vector with all currently ongoing listings.
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub(super) type ListedNfts<T: Config> =
		StorageValue<_, BoundedVec<(T::CollectionId, T::ItemId), T::MaxListedNfts>, ValueQuery>;

	/// Mapping from the nft to the nft details.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_nft_details)]
	pub(super) type OngoingNftDetails<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::CollectionId, T::ItemId),
		NftDetails<BalanceOf<T>, T::CollectionId, T::ItemId, T>,
		OptionQuery,
	>;

	/// Mapping of collection to the listed nfts of this collection.
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts_of_collection)]
	pub(super) type ListedNftsOfCollection<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<T::ItemId, T::MaxNftInCollection>,
		ValueQuery,
	>;

	/// Mapping of collection to the already sold nfts of this collection.
	#[pallet::storage]
	#[pallet::getter(fn sold_nfts_collection)]
	pub(super) type SoldNftsCollection<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<T::ItemId, T::MaxNftInCollection>,
		ValueQuery,
	>;

	/// Mapping from real estate developer to his listed nfts.
	#[pallet::storage]
	#[pallet::getter(fn seller_listings)]
	pub(super) type SellerListings<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		BoundedVec<(T::CollectionId, T::ItemId), T::MaxListedNfts>,
		ValueQuery,
	>;

	/// Mapping from real estate developer to his sold nfts.
	#[pallet::storage]
	#[pallet::getter(fn seller_sold_nfts)]
	pub(super) type SellerSoldNfts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		BoundedVec<(T::CollectionId, T::ItemId), T::MaxListedNfts>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace.
		ObjectListed {
			collection_index: T::CollectionId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// A nft has been bought.
		NftBought {
			collection_index: T::CollectionId,
			item_index: T::ItemId,
			buyer: AccountIdOf<T>,
			price: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Max amount of listed Nfts reached.
		TooManyListedNfts,
		/// This index is not taken.
		InvalidIndex,
		/// Too many nfts for this collection.
		TooManyNfts,
		/// The buyer doesn't have enough funds.
		NotEnoughFunds,
		/// The collection has not been found.
		CollectionNotFound,
		/// Not enough nfts available to buy.
		NotEnoughNftsAvailable,
		/// Distribution of the nfts failed.
		DistributionError,
		/// Error by convertion to balance type.
		ConversionError,
		/// Error by dividing a number.
		DivisionError,
		/// Error by multiplying a number.
		MultiplyError,
		/// There is an issue by calling the next collection id.
		UnknownCollection,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u32: From<<T as pallet_nfts::Config>::CollectionId>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
		u32: EncodeLike<<T as pallet_nfts::Config>::CollectionId>,
	{
		/// List a real estate object. A new collection is created and 100 nfts get minted.
		/// This function calls the nfts-pallet to creates a collection, mint nfts and set the Metadata.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `price`: The price of the real estate object that is offered.
		/// - `data`: The Metadata of the collection and the single nfts.
		///
		/// Emits `ObjectListed` event when succesfful
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_object())]
		pub fn list_object(
			origin: OriginFor<T>,
			price: BalanceOf<T>,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult
		where
			u32: From<<T as pallet_nfts::Config>::CollectionId>,
			<T as pallet_nfts::Config>::ItemId: From<u32>,
		{
			let signer = ensure_signed(origin.clone())?;
			if pallet_nfts::NextCollectionId::<T>::get().is_none() {
				pallet_nfts::NextCollectionId::<T>::set(T::CollectionId::initial_value());
			};
			let collection: u32 = pallet_nfts::NextCollectionId::<T>::get()
			.ok_or(Error::<T>::UnknownCollection)?.into();
			let collection_id = pallet_nfts::NextCollectionId::<T>::get()
			.ok_or(Error::<T>::UnknownCollection)?;
			let next_collection_id = collection_id.increment();
			pallet_nfts::NextCollectionId::<T>::set(next_collection_id);

			pallet_nfts::Pallet::<T>::do_create_collection(
				collection_id,
				signer.clone(),
				signer.clone(),
				Self::default_collection_config(),
				T::CollectionDeposit::get(),
				pallet_nfts::Event::Created {
					creator: Self::account_id(),
					owner: Self::account_id(),
					collection: collection_id,
				},
			)?;
			pallet_nfts::Pallet::<T>::set_collection_metadata(
				origin.clone(),
				collection_id,
				data.clone(),
			)?;
			for x in 1..=100 {
				let item_id: T::ItemId = x.into();
				let nft = NftDetails {
					real_estate_developer: signer.clone(),
					owner: Self::account_id(),
					price: price
						.checked_div(&Self::u64_to_balance_option(100)?)
						.ok_or(Error::<T>::DivisionError)?,
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
				pallet_nfts::Pallet::<T>::set_metadata(
					origin.clone(),
					collection_id,
					item_id,
					data.clone(),
				)?;
				OngoingNftDetails::<T>::insert((collection, item_id), nft.clone());
				CollectionCount::<T>::put(collection);
				ListedNftsOfCollection::<T>::try_mutate(collection, |keys| {
					keys.try_push(item_id).map_err(|_| Error::<T>::TooManyNfts)?;
					Ok::<(), DispatchError>(())
				})?;
				ListedNfts::<T>::try_append((collection, item_id))
					.map_err(|_| Error::<T>::TooManyListedNfts)?;
				SellerListings::<T>::try_mutate(signer.clone(), |keys| {
					keys.try_push((collection_id, item_id)).map_err(|_| Error::<T>::TooManyNfts)?;
					Ok::<(), DispatchError>(())
				})?;
			}
			pallet_nfts::Pallet::<T>::set_team(origin.clone(), collection_id, None, None, None)?;
			Self::deposit_event(Event::<T>::ObjectListed {
				collection_index: collection_id,
				price,
				seller: signer,
			});
			Ok(())
		}

		/// Buy listed nfts from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection`: The collection that the investor wants to buy from.
		/// - `amount`: The amount of nfts that the investor wants to buy.
		///
		/// Emits `NftBought` event when succesfful
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_nft())]
		pub fn buy_nft(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			amount: u32,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(Self::collection_exists(collection), Error::<T>::CollectionNotFound);
			ensure!(
				Self::listed_nfts_of_collection(collection).len() >= amount.try_into().unwrap(),
				Error::<T>::NotEnoughNftsAvailable
			);
			for _x in 0..amount as usize {
				let mut listed_items = Self::listed_nfts_of_collection(collection);
				let next_nft = Self::ongoing_nft_details((collection, &listed_items[0]))
					.ok_or(Error::<T>::InvalidIndex)?;

				let mut nft =
					<OngoingNftDetails<T>>::take((next_nft.collection_id, next_nft.item_id))
						.ok_or(Error::<T>::InvalidIndex)?;
				nft.owner = origin.clone();
				nft.sold = true;
				SoldNftsCollection::<T>::try_mutate(collection, |keys| {
					keys.try_push(next_nft.item_id).map_err(|_| Error::<T>::TooManyNfts)?;
					Ok::<(), DispatchError>(())
				})?;
				let price = nft
					.price
					.checked_mul(&Self::u64_to_balance_option(1000000000000)?)
					.ok_or(Error::<T>::MultiplyError)?;
				Self::transfer_funds(&origin, &Self::account_id(), price)?;
				let mut listed_nfts = Self::listed_nfts();
				let index = listed_nfts
					.iter()
					.position(|x| *x == (next_nft.collection_id, next_nft.item_id))
					.unwrap();
				listed_nfts.remove(index);
				ListedNfts::<T>::put(listed_nfts);
				let index = listed_items.iter().position(|x| *x == next_nft.item_id).unwrap();
				listed_items.remove(index);
				ListedNftsOfCollection::<T>::insert(collection, listed_items);
				if Self::sold_nfts_collection(collection).len() == 100 {
					let _ = Self::distribute_nfts(collection);
				}
				let price = nft.price;
				let item = nft.item_id;
				OngoingNftDetails::<T>::insert(
					(next_nft.collection_id, next_nft.item_id),
					nft.clone(),
				);
				SellerListings::<T>::try_mutate(nft.real_estate_developer.clone(), |keys| {
					let index = keys
						.iter()
						.position(|x| *x == (next_nft.collection_id, next_nft.item_id))
						.unwrap();
					keys.remove(index);
					Ok::<(), DispatchError>(())
				})?;
				Self::deposit_event(Event::<T>::NftBought {
					collection_index: collection,
					item_index: item,
					buyer: origin.clone(),
					price,
				});
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_account_truncating()
		}

		/// Sends the nfts to the new owners and the funds to the real estate developer once all 100 Nfts
		/// of a collection are minted.
		fn distribute_nfts(collection_id: T::CollectionId) -> DispatchResult {
			let list = <SoldNftsCollection<T>>::take(collection_id);
			let nft_details = Self::ongoing_nft_details((collection_id, &list[0]))
				.ok_or(Error::<T>::InvalidIndex)?;
			let price = nft_details
				.price
				.checked_mul(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_mul(&Self::u64_to_balance_option(1000000000000)?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::transfer_funds(&Self::account_id(), &nft_details.real_estate_developer, price)?;
			for x in list {
				let nft_details = <OngoingNftDetails<T>>::take((collection_id, x))
					.ok_or(Error::<T>::InvalidIndex)?;
				pallet_nfts::Pallet::<T>::do_transfer(
					collection_id,
					nft_details.item_id,
					nft_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				SellerSoldNfts::<T>::try_mutate(
					nft_details.real_estate_developer.clone(),
					|keys| {
						keys.try_push((collection_id, x)).map_err(|_| Error::<T>::TooManyNfts)?;
						Ok::<(), DispatchError>(())
					},
				)?;
			}
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
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

		/// Set the default item configuration for minting a nft.
		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}

		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}

		/// Checks if the collection exists
		fn collection_exists(collection: T::CollectionId) -> bool {
			let listed_nfts_count = ListedNftsOfCollection::<T>::contains_key(collection);
			let sold_nfts_count = SoldNftsCollection::<T>::contains_key(collection);
			listed_nfts_count || sold_nfts_count
		}

		fn transfer_funds(
			from: &AccountIdOf<T>,
			to: &AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			Ok(<T as pallet::Config>::Currency::transfer(from, to, amount, KeepAlive)
				.map_err(|_| Error::<T>::NotEnoughFunds)?)
		}
	}
}
