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

use pallet_assets::Instance1;

use frame_support::{
	traits::{Currency, ExistenceRequirement::KeepAlive, Incrementable, ReservableCurrency},
	PalletId,
};

use frame_support::sp_runtime::traits::{
	AccountIdConversion, CheckedDiv, CheckedMul, StaticLookup,
};

use enumflags2::BitFlags;

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type BalanceOf1<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

type BalanceOf2<T> = <T as pallet_nft_fractionalization::Config>::AssetBalance;

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
	pub struct NftDetails<Balance, ItemId, T: Config> {
		pub real_estate_developer: AccountIdOf<T>,
		pub owner: AccountIdOf<T>,
		pub price: Balance,
		pub asset_id: u32,
		pub item_id: ItemId,
		pub spv_created: bool,
	}

	/// AccountId storage
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config:
		frame_system::Config 
		+ pallet_nfts::Config 
		+ pallet_whitelist::Config 
		+ pallet_assets::Config<Instance1>
		+ pallet_nft_fractionalization::Config
	{
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
		type Helper: crate::BenchmarkHelper<
			<Self as pallet_nfts::Config>::CollectionId,
			<Self as pallet_nfts::Config>::ItemId,
		>;

		/// The maximum amount of nfts that can be listed at the same time.
		#[pallet::constant]
		type MaxListedNfts: Get<u32>;

		/// The maximum amount of nfts for a collection.
		type MaxNftInCollection: Get<u32>;

		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Default
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		type ItemId: IsType<<Self as pallet_nfts::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		type FractionalizeCollectionId: IsType<<Self as pallet_nft_fractionalization::Config>::NftCollectionId>
			+ Parameter
			+ From<CollectionId<Self>>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		type FractionalizeItemId: IsType<<Self as pallet_nft_fractionalization::Config>::NftId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		type AssetId: IsType<<Self as pallet_nft_fractionalization::Config>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// Asset id type from pallet assets.
		type AssetId2: IsType<<Self as pallet_assets::Config<Instance1>>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// The Trasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryId: Get<PalletId>;

		/// The CommunityProjects's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type CommunityProjectsId: Get<PalletId>;
	}

	pub type AssetId<T> = <T as Config>::AssetId;
	pub type AssetId2<T> = <T as Config>::AssetId2;
	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;
	pub type FractionalizeCollectionId<T> = <T as Config>::FractionalizeCollectionId;
	pub type FractionalizeItemId<T> = <T as Config>::FractionalizeItemId;

	pub(super) type NftDetailsType<T> = NftDetails<
		BalanceOf<T>,
		<T as pallet::Config>::ItemId,
		T,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub(super) type NextNftId<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pallet_collection_id)]
	pub(super) type PalletCollectionId<T: Config> = StorageValue<
		_,
		<T as pallet::Config>::CollectionId,
		ValueQuery,
	>;

	/// Mapping from the nft to the nft details.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_nft_details)]
	pub(super) type OngoingNftDetails<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		NftDetailsType<T>,
		OptionQuery,
	>;

	/// Mapping of collection to the listed nfts of this collection.
	#[pallet::storage]
	#[pallet::getter(fn listed_token)]
	pub(super) type ListedToken<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		u8,
		OptionQuery,
	>;

	/// Mapping of collection to the already sold nfts of this collection.
	#[pallet::storage]
	#[pallet::getter(fn sold_token)]
	pub(super) type SoldToken<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		u8,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_buyer)]
	pub(super) type TokenBuyer<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		BoundedVec<AccountIdOf<T>, T::MaxListedNfts>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn token_owner)]
	pub(super) type TokenOwner<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		u8,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		#[serde(skip)]
		_config: sp_std::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if pallet_nfts::NextCollectionId::<T>::get().is_none() {
				pallet_nfts::NextCollectionId::<T>::set(
					<T as pallet_nfts::Config>::CollectionId::initial_value(),
				);
			};
			let collection_id =
				pallet_nfts::NextCollectionId::<T>::get().unwrap();
			let next_collection_id = collection_id.increment();
			pallet_nfts::NextCollectionId::<T>::set(next_collection_id);
			let collection_id: CollectionId<T> = 1.into();
			PalletCollectionId::<T>::put(collection_id);
			let pallet_id: AccountIdOf<T> = <T as pallet::Config>::PalletId::get().into_account_truncating();
			pallet_nfts::Pallet::<T>::do_create_collection(
				collection_id.into(),
				pallet_id.clone(),
				pallet_id.clone(),
				CollectionConfig {
					settings: CollectionSettings::from_disabled(CollectionSetting::DepositRequired.into()),
					max_supply: None,
					mint_settings: MintSettings::default(),
				},
				T::CollectionDeposit::get(),
				pallet_nfts::Event::Created {
					creator: pallet_id.clone(),
					owner: pallet_id,
					collection: collection_id.into(),
				},
			);
/* 			pallet_nfts::Pallet::<T>::set_team(
				origin.clone(),
				collection_id.into(),
				None,
				None,
				None,
			); */
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace.
		ObjectListed {
			collection_index: <T as pallet::Config>::CollectionId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// A nft has been bought.
		NftBought {
			collection_index: <T as pallet::Config>::CollectionId,
			item_index: <T as pallet::Config>::ItemId,
			buyer: AccountIdOf<T>,
			price: BalanceOf<T>,
		},
		/// A sigle nft has been listed.
		NftListed {
			collection_index: <T as pallet::Config>::CollectionId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// The price of the nft has been updated.
		NftUpdated {
			collection_index: <T as pallet::Config>::CollectionId,
			item_index: <T as pallet::Config>::ItemId,
			new_price: BalanceOf<T>,
		},
		/// The nft has been delisted.
		NftDelisted {
			collection_index: <T as pallet::Config>::CollectionId,
			item_index: <T as pallet::Config>::ItemId,
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
		/// The nft has not been found.
		ItemNotFound,
		/// No sufficient permission.
		NoPermission,
		/// The Nft is not listed on the marketplace.
		NftNotListed,
		/// The nft is relisted and can't be bought with this function.
		NftIsRelisted,
		/// The nft can't be bought with this function.
		NftAlreadyRelisted,
		/// Nft can only be bought through presale.
		SpvNotCreated,
		/// The Nft is not listed on the marketplace.
		NftNotForSale,
		/// Collection is not known to this pallet.
		CollectionNotKnown,
		/// User has not passed the kyc.
		UserNotWhitelisted,
		ArithmeticUnderflow,
		ArithmeticOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			data: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;

			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let collection_id: CollectionId<T> = Self::pallet_collection_id();

			let mut next_item_id = Self::next_nft_id();
			let mut asset_number: u32 = Self::next_asset_id();
			let mut asset_id: AssetId2<T> = asset_number.into();
 			while pallet_assets::Pallet::<T, Instance1>::maybe_total_supply(asset_id.into()).is_some() {
				asset_number = asset_number.checked_add(1).ok_or(Error::<T>::DivisionError)?;
				asset_id = asset_number.into();
			}; 
			let asset_id: AssetId<T> = asset_number.into();
			let item_id: ItemId<T> = next_item_id.into();
			let nft = NftDetails {
				real_estate_developer: signer.clone(),
				owner: Self::account_id(),
				price,
				asset_id: asset_number,
				item_id,
				spv_created: Default::default(),
			};
			pallet_nfts::Pallet::<T>::do_mint(
				collection_id.into(),
				item_id.into(),
				Some(Self::account_id()),
				Self::account_id(),
				Self::default_item_config(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::set_metadata(
				pallet_origin.clone(),
				collection_id.into(),
				item_id.into(),
				data.clone(),
			)?;
			OngoingNftDetails::<T>::insert(item_id, nft.clone());
			ListedToken::<T>::insert(item_id, 100);
			
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let nft_balance: BalanceOf2<T> = 100_u32.into();
			let fractionalize_collection_id: FractionalizeCollectionId<T> = collection_id.try_into().map_err(|_| Error::<T>::ConversionError)?;
			let fractionalize_item_id: FractionalizeItemId<T> = next_item_id.try_into().map_err(|_| Error::<T>::ConversionError)?;
			pallet_nft_fractionalization::Pallet::<T>::fractionalize(
				pallet_origin.clone(),
				fractionalize_collection_id.into(),
				fractionalize_item_id.into(),
				asset_id.into(),
				user_lookup,
				nft_balance,
			)?;
			next_item_id = next_item_id.checked_add(1).ok_or(Error::<T>::DivisionError)?;
			asset_number = asset_number.checked_add(1).ok_or(Error::<T>::DivisionError)?;
			NextNftId::<T>::put(next_item_id);
			NextAssetId::<T>::put(asset_number);

			Self::deposit_event(Event::<T>::ObjectListed {
				collection_index: collection_id,
				price,
				seller: signer,
			});
			Ok(())
		}

		/* /// List a single nft on the marketplace.
		/// The nft collection must be registered on the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection of the nft.
		/// - `item_id`: The item id of the nft.
		/// - `price`: The price of the nft that is offered.
		///
		/// Emits `NftListed` event when succesfful
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_nft())]
		pub fn list_nft(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;

			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);

			ensure!(
				Self::listed_collection_details(collection_id).is_some(),
				Error::<T>::CollectionNotKnown
			);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let nft = NftDetails {
				real_estate_developer: signer.clone(),
				owner: signer.clone(),
				price,
				collection_id,
				item_id,
				sold: Default::default(),
			};
			OngoingNftDetails::<T>::insert(collection_id, item_id, nft.clone());
			ListedNftsOfCollection::<T>::try_mutate(collection_id, |keys| {
				keys.try_push(item_id).map_err(|_| Error::<T>::TooManyNfts)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::NftListed {
				collection_index: collection_id,
				price,
				seller: signer,
			});
			Ok(())
		} */

		/// Buy listed nfts from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection`: The collection that the investor wants to buy from.
		/// - `amount`: The amount of nfts that the investor wants to buy.
		///
		/// Emits `NftBought` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_nft())]
		pub fn buy_nft(
			origin: OriginFor<T>,
			item: <T as pallet::Config>::ItemId,
			amount: u8,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(origin.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(Self::collection_exists(item), Error::<T>::CollectionNotFound);
			let mut listed_token = ListedToken::<T>::take(item).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(
				listed_token >= amount,
				Error::<T>::NotEnoughNftsAvailable
			);
			let nft_details =
				Self::ongoing_nft_details(item).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(!nft_details.spv_created, Error::<T>::NftAlreadyRelisted);

			let price = nft_details
				.price
				.checked_mul(&Self::u64_to_balance_option(amount as u64)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?
				.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::transfer_funds(&origin, &Self::account_id(), price)?;
			listed_token = listed_token
				.checked_sub(amount)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			TokenBuyer::<T>::try_mutate(item, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::NotEnoughNftsAvailable)?;
				Ok::<(), DispatchError>(())
			})?;
			let mut token_of_owner = TokenOwner::<T>::take(origin.clone(), item);
			token_of_owner = token_of_owner.checked_add(amount).ok_or(Error::<T>::ArithmeticOverflow)?;
			TokenOwner::<T>::insert(origin.clone(), item, token_of_owner);
			let mut sold_token = Self::sold_token(item);
			SoldToken::<T>::insert(item, amount);
			if listed_token == 0 {
				Self::distribute_nfts(item)?;
			} else {
				ListedToken::<T>::insert(item, listed_token);
			}
			Ok(())
		}

		/* /// Buy single nfts from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection that the investor wants to buy from.
		/// - `item_id`: The Item from the collection that the investor wants to buy from.
		///
		/// Emits `NftBought` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_single_nft())]
		pub fn buy_single_nft(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(origin.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(Self::collection_exists(collection_id), Error::<T>::CollectionNotFound);
			let nft_details =
				Self::nft_details(item_id).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(collection_details.spv_created, Error::<T>::SpvNotCreated);
			let nft = <OngoingNftDetails<T>>::take(collection_id, item_id)
				.ok_or(Error::<T>::NftNotForSale)?;
			let price = nft
				.price
				.checked_mul(&Self::u64_to_balance_option(1 /* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			let fees = price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::MultiplyError)?;
			let treasury_id = Self::treasury_account_id();
			let treasury_fees = fees
				.checked_mul(&Self::u64_to_balance_option(90)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			let community_projects_id = Self::community_account_id();
			let community_fees = fees
				.checked_div(&Self::u64_to_balance_option(10)?)
				.ok_or(Error::<T>::DivisionError)?;
			let seller_part = price
				.checked_mul(&Self::u64_to_balance_option(99)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			Self::transfer_funds(&origin, &treasury_id, treasury_fees)?;
			Self::transfer_funds(&origin, &community_projects_id, community_fees)?;
			Self::transfer_funds(&origin, &nft.owner, seller_part)?;
			pallet_nfts::Pallet::<T>::do_transfer(
				collection_id.into(),
				item_id.into(),
				origin.clone(),
				|_, _| Ok(()),
			)?;
			let mut listed_items = Self::listed_nfts_of_collection(collection_id);
			let index = listed_items
				.iter()
				.position(|x| *x == nft.item_id)
				.ok_or(Error::<T>::ItemNotFound)?;
			listed_items.remove(index);
			ListedNftsOfCollection::<T>::insert(collection_id, listed_items);
			Self::deposit_event(Event::<T>::NftBought {
				collection_index: collection_id,
				item_index: item_id,
				buyer: origin.clone(),
				price: nft.price,
			});
			Ok(())
		}

		/// Upgrade the price from a listed nft.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection that the seller wants to upgrade.
		/// - `item_id`: The Item from the collection that the seller wants to update.
		/// - `new_price`: The new price of the nft.
		///
		/// Emits `NftUpdated` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upgrade_listing())]
		pub fn upgrade_listing(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
			new_price: BalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(
				Self::ongoing_nft_details(collection_id, item_id).is_some(),
				Error::<T>::NftNotListed
			);
			let mut nft = Self::ongoing_nft_details(collection_id, item_id)
				.ok_or(Error::<T>::InvalidIndex)?;
			ensure!(nft.owner == signer, Error::<T>::NoPermission);
			nft.price = new_price;
			OngoingNftDetails::<T>::insert(collection_id, item_id, nft);
			Self::deposit_event(Event::<T>::NftUpdated {
				collection_index: collection_id,
				item_index: item_id,
				new_price,
			});
			Ok(())
		}

		/// Upgrade the price from a listed object.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection that the investor wants to buy from.
		/// - `new_price`: The new price of the nft.
		///
		/// Emits `NftUpdated` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upgrade_object())]
		pub fn upgrade_object(
			origin: OriginFor<T>,
			item_id: <T as pallet::Config>::ItemId,
			new_price: BalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(Self::collection_exists(collection_id), Error::<T>::CollectionNotFound);
			let nft_details =
				Self::nft_details(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(!collection_details.spv_created, Error::<T>::NftAlreadyRelisted);
			let list = Self::listed_nfts_of_collection(collection_id);
			let new_item_price = new_price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			for item in list {
				let mut nft = Self::ongoing_nft_details(collection_id, item)
					.ok_or(Error::<T>::InvalidIndex)?;
				ensure!(nft.real_estate_developer == signer, Error::<T>::NoPermission);
				nft.price = new_item_price;
				OngoingNftDetails::<T>::insert(collection_id, item, nft);
			}
			Ok(())
		}

		/// Delist the choosen nft from the marketplace.
		/// Works only for relisted nfts.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection that the investor wants to buy from.
		/// - `item_id`: The Item from the collection that the investor wants to buy from.
		///
		/// Emits `NftDelisted` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_nft())]
		pub fn delist_nft(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			ensure!(
				Self::ongoing_nft_details(collection_id, item_id).is_some(),
				Error::<T>::NftNotListed
			);
			let nft = <OngoingNftDetails<T>>::take(collection_id, item_id)
				.ok_or(Error::<T>::InvalidIndex)?;
			ensure!(nft.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				collection_id.into(),
				item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let mut listed_items = Self::listed_nfts_of_collection(collection_id);
			let index = listed_items
				.iter()
				.position(|x| *x == nft.item_id)
				.ok_or(Error::<T>::ItemNotFound)?;
			listed_items.remove(index);
			ListedNftsOfCollection::<T>::insert(collection_id, listed_items);
			Self::deposit_event(Event::<T>::NftDelisted {
				collection_index: collection_id,
				item_index: item_id,
			});
			Ok(())
		} */
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		pub fn treasury_account_id() -> AccountIdOf<T> {
			T::TreasuryId::get().into_account_truncating()
		}

		pub fn community_account_id() -> AccountIdOf<T> {
			T::CommunityProjectsId::get().into_account_truncating()
		}

		/// Sends the nfts to the new owners and the funds to the real estate developer once all 100 Nfts
		/// of a collection are sold.
		fn distribute_nfts(item: <T as pallet::Config>::ItemId) -> DispatchResult {
			let list = <TokenBuyer<T>>::take(item);

			let mut nft_details = Self::ongoing_nft_details(item)
				.ok_or(Error::<T>::InvalidIndex)?;
			let price = nft_details.price
				.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			let fees = price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::MultiplyError)?;
			let treasury_id = Self::treasury_account_id();
			let treasury_fees = fees
				.checked_mul(&Self::u64_to_balance_option(90)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			let community_projects_id = Self::community_account_id();
			let community_fees = fees
				.checked_div(&Self::u64_to_balance_option(10)?)
				.ok_or(Error::<T>::DivisionError)?;
			let seller_part = price
				.checked_mul(&Self::u64_to_balance_option(99)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			Self::transfer_funds(&Self::account_id(), &treasury_id, treasury_fees)?;
			Self::transfer_funds(&Self::account_id(), &community_projects_id, community_fees)?;
			Self::transfer_funds(
				&Self::account_id(),
				&nft_details.real_estate_developer,
				seller_part,
			)?;
			let origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			for owner in list {
				let user_lookup = <T::Lookup as StaticLookup>::unlookup(owner.clone());
				let token: u64 = TokenOwner::<T>::take(owner.clone(), item) as u64;
				let token_amount = token
					.try_into().map_err(|_| Error::<T>::ConversionError)?;
				let asset_id: AssetId2<T> = nft_details.asset_id.into();
				pallet_assets::Pallet::<T, Instance1>::transfer(
					origin.clone(),
					asset_id.into().into(),
					user_lookup,
					token_amount,
				)
				.map_err(|_| Error::<T>::NotEnoughFunds)?;
			}
			SoldToken::<T>::take(item);
			nft_details.spv_created = true;
			OngoingNftDetails::<T>::insert(item, nft_details); 
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
		fn default_collection_config() -> CollectionConfig<
			BalanceOf1<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			Self::collection_config_from_disabled_settings(
				CollectionSetting::DepositRequired.into(),
			)
		}

		fn collection_config_from_disabled_settings(
			settings: BitFlags<CollectionSetting>,
		) -> CollectionConfig<
			BalanceOf1<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
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
		fn collection_exists(item: <T as pallet::Config>::ItemId) -> bool {
			let listed_nfts_count = ListedToken::<T>::contains_key(item);
			listed_nfts_count
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
