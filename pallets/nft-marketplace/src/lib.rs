#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
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

use codec::EncodeLike;

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

		/// Type representing the weight of this pallet
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

	/// Number of nfts that have been listed.

	#[pallet::storage]
	#[pallet::getter(fn collection_count)]
	pub(super) type CollectionCount<T> = StorageValue<_, u32, ValueQuery>;

	/// Vector with all currently ongoing listings
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub(super) type ListedNfts<T: Config> =
		StorageValue<_, BoundedVec<(T::CollectionId, T::ItemId), T::MaxListedNfts>, ValueQuery>;

	/// Mapping of the listed nfts to details
	#[pallet::storage]
	#[pallet::getter(fn ongoing_listings)]
	pub(super) type OngoingListings<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::CollectionId, T::ItemId),
		NnftDog<BalanceOf<T>, T::CollectionId, T::ItemId, T>,
		OptionQuery,
	>;

	/// Mapping of collection to the already sold nfts of this collection
	#[pallet::storage]
	#[pallet::getter(fn listed_collection)]
	pub(super) type ListedCollection<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CollectionId,
		BoundedVec<NnftDog<BalanceOf<T>, T::CollectionId, T::ItemId, T>, T::MaxNftInCollection>,
		ValueQuery,
	>;

	/// Mapping from owner to nfts
	#[pallet::storage]
	#[pallet::getter(fn seller_listings)]
	pub(super) type SellerListings<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		BoundedVec<NnftDog<BalanceOf<T>, T::CollectionId, T::ItemId, T>, T::MaxListedNfts>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace
		ObjectListed {
			collection_index: T::CollectionId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// A nft has been bought
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
		/// Not enough nfts available to buy
		NotEnoughNftsAvailable,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_nfts::Config>::CollectionId: From<u32>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
		u32: EncodeLike<<T as pallet_nfts::Config>::CollectionId>,
	{
		/// List a real estate object. A new collection is created and 100 nfts get minted.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_object())]
		pub fn list_object(
			origin: OriginFor<T>,
			price: BalanceOf<T>,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult
		where
			<T as pallet_nfts::Config>::CollectionId: From<u32>,
			<T as pallet_nfts::Config>::ItemId: From<u32>,
		{
			let signer = ensure_signed(origin.clone())?;
			let collection = Self::collection_count() + 1;
			let collection_id: T::CollectionId = collection.into();

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
			for x in 1..11 {
				let item_id: T::ItemId = x.into();
				let nft = NnftDog {
					real_estate_developer: signer.clone(),
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
				pallet_nfts::Pallet::<T>::set_metadata(
					origin.clone(),
					collection_id,
					item_id,
					data.clone(),
				)?;
				OngoingListings::<T>::insert((collection, item_id), nft.clone());
				CollectionCount::<T>::put(collection);
				ListedNfts::<T>::try_append((collection, item_id))
					.map_err(|_| Error::<T>::TooManyListedNfts)?;
				SellerListings::<T>::try_mutate(signer.clone(), |keys| {
					keys.try_push(nft).map_err(|_| Error::<T>::TooManyNfts)?;
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

		/// Buying a certain nft.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_nft())]
		pub fn buy_nft(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			amount: u32,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(Self::listed_nfts().len() >= amount.try_into().unwrap(), Error::<T>::NotEnoughNftsAvailable);
			for x in 0..amount as usize {
				let mut ongoing_listings = Self::listed_nfts();
				let next_nft = ongoing_listings[0];
				let mut nft =
					<OngoingListings<T>>::take(next_nft).ok_or(Error::<T>::InvalidIndex)?;
				ensure!(
					<T as pallet::Config>::Currency::free_balance(&origin) >= nft.price,
					Error::<T>::NotEnoughFunds
				);
				let old_nft_data = nft.clone();
				nft.owner = origin.clone();
				nft.sold = true;
				ListedCollection::<T>::try_mutate(collection, |keys| {
					keys.try_push(nft.clone()).map_err(|_| Error::<T>::TooManyNfts)?;
					Ok::<(), DispatchError>(())
				})?;
				<T as pallet::Config>::Currency::transfer(
					&origin,
					&Self::account_id(),
					// For unit tests this line has to be commented out and the line blow has to be uncommented due to the dicmals on polkadot js
					nft.price * Self::u64_to_balance_option(1000000000000).unwrap_or_default(),
					//amount,
					KeepAlive,
				)
				.unwrap_or_default();
				let index = ongoing_listings.iter().position(|x| *x == next_nft).unwrap();
				ongoing_listings.remove(index);
				ListedNfts::<T>::put(ongoing_listings);
				if Self::listed_collection(collection).len() == 10 {
					Self::distribute_nfts(collection);
				}
				let price = nft.price.clone();
				let item = nft.item_id.clone();
				SellerListings::<T>::try_mutate(nft.real_estate_developer.clone(), |keys| {
					let index = keys.iter().position(|x| *x == old_nft_data).unwrap();
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
