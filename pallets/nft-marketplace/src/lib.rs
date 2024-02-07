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
	AccountIdConversion, CheckedDiv, CheckedMul, StaticLookup, CheckedAdd
};

use enumflags2::BitFlags;

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type BalanceOf1<T> = <T as pallet_nft_fractionalization::Config>::AssetBalance;

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

	/// Infos regardint an listed nft of a real estate object on the marketplace.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftDetails {
		pub spv_created: bool,
		pub asset_id: u32,
	}

	/// Infos regarding the listing of an real estate object.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftListingDetails<Balance, ItemId, T: Config> {
		pub real_estate_developer: AccountIdOf<T>,
		pub owner: AccountIdOf<T>,
		pub current_price: Balance,
		pub collected_funds: Balance,
		pub asset_id: u32,
		pub item_id: ItemId,
	}

	/// Infos regarding the listing of a token.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct TokenListingDetails<Balance, ItemId, T: Config> {
		pub seller: AccountIdOf<T>,
		pub price: Balance,
		pub asset_id: u32,
		pub item_id: ItemId,
		pub amount: u8,
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

		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Default
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Item id type from pallet nfts.
		type ItemId: IsType<<Self as pallet_nfts::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Collection id type from pallet nft fractionalization.
		type FractionalizeCollectionId: IsType<<Self as pallet_nft_fractionalization::Config>::NftCollectionId>
			+ Parameter
			+ From<CollectionId<Self>>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Item id type from pallet nft fractionalization.
		type FractionalizeItemId: IsType<<Self as pallet_nft_fractionalization::Config>::NftId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;

		/// Asset id type from pallet nft fractionalization.
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

	pub(super) type NftListingDetailsType<T> = NftListingDetails<
		BalanceOf<T>,
		<T as pallet::Config>::ItemId,
		T,
	>;

	pub(super) type ListingDetailsType<T> = TokenListingDetails<
		BalanceOf<T>,
		<T as pallet::Config>::ItemId,
		T,
	>;

	/// Id for the next nft.
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub(super) type NextNftId<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	/// Id of the possible next asset that would be used for
	/// Nft fractionalization.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	/// The Id for the next token listing.
	#[pallet::storage]
	#[pallet::getter(fn next_listing_id)]
	pub(super) type NextListingId<T: Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;

	/// Storage of the collection id that will be used for all the nfts.
	#[pallet::storage]
	#[pallet::getter(fn pallet_collection_id)]
	pub(super) type PalletCollectionId<T: Config> = StorageValue<
		_,
		<T as pallet::Config>::CollectionId,
		ValueQuery,
	>;

	/// Mapping from the Nft to the Nft details.
	#[pallet::storage]
	#[pallet::getter(fn registered_nft_details)]
	pub(super) type RegisteredNftDetails<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		NftDetails,
		OptionQuery,
	>;

	/// Mapping from the nft to the ongoing nft listing details.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_object_listing)]
	pub(super) type OngoingObjectListing<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		NftListingDetailsType<T>,
		OptionQuery,
	>;

	/// Mapping of the nft to the amount of listed token.
	#[pallet::storage]
	#[pallet::getter(fn listed_token)]
	pub(super) type ListedToken<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		u8,
		OptionQuery,
	>;

	/// Mapping of a nft to the buyer of the sold token.
	#[pallet::storage]
	#[pallet::getter(fn token_buyer)]
	pub(super) type TokenBuyer<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		BoundedVec<AccountIdOf<T>, T::MaxListedNfts>,
		ValueQuery,
	>;

	/// Double mapping of the account id of the token owner
	/// and the nft to the amount of token.
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

	/// Mapping of the listing id to the listing details of a token listing.
	#[pallet::storage]
	#[pallet::getter(fn token_listings)]
	pub(super) type TokenListings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		ListingDetailsType<T>,
		OptionQuery,
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
			let _ = pallet_nfts::Pallet::<T>::do_create_collection(
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
			item_index: <T as pallet::Config>::ItemId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// A token has been bought.
		TokenBought {
			item_index: <T as pallet::Config>::ItemId,
			buyer: AccountIdOf<T>,
			price: BalanceOf<T>,
		},
		/// Token from listed object have been bought.
		TokenBoughtObject {
			item_index: <T as pallet::Config>::ItemId,
			buyer: AccountIdOf<T>,
			amount: u8,
			price: BalanceOf<T>,
		},
		/// Token have been listed.
		TokenListed {
			item_index: <T as pallet::Config>::ItemId,
			price: BalanceOf<T>,
			seller: AccountIdOf<T>,
		},
		/// The price of the token listing has been updated.
		ListingUpdated {
			listing_index: u32,
			new_price: BalanceOf<T>,
		},
		/// The nft has been delisted.
		ListingDelisted {
			listing_index: u32,
		},
		/// The price of the listed object has been updated.
		ObjectUpdated {
			item_index: <T as pallet::Config>::ItemId,
			new_price: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// This index is not taken.
		InvalidIndex,
		/// The buyer doesn't have enough funds.
		NotEnoughFunds,
		/// The collection has not been found.
		CollectionNotFound,
		/// Not enough token available to buy.
		NotEnoughTokenAvailable,
		/// Error by convertion to balance type.
		ConversionError,
		/// Error by dividing a number.
		DivisionError,
		/// Error by multiplying a number.
		MultiplyError,
		/// No sufficient permission.
		NoPermission,
		/// The SPV has already been created.
		SpvAlreadyCrated,
		/// User has not passed the kyc.
		UserNotWhitelisted,
		ArithmeticUnderflow,
		ArithmeticOverflow,
		/// The token is not for sale.
		TokenNotForSale,
		/// The nft has not been registered on the marketplace.
		NftNotFound,
		/// There are already too many token buyer.
		TooManyTokenBuyer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// List a real estate object. A new nft gets minted.
		/// This function calls the nfts-pallet to mint a new nft and sets the Metadata.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `price`: The price of the real estate object that is offered.
		/// - `data`: The Metadata of the nft.
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
				asset_number = asset_number.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
				asset_id = asset_number.into();
			}; 
			let asset_id: AssetId<T> = asset_number.into();
			let item_id: ItemId<T> = next_item_id.into();
			let nft = NftListingDetails {
				real_estate_developer: signer.clone(),
				owner: Self::account_id(),
				current_price: price,
				collected_funds: Default::default(),
				asset_id: asset_number,
				item_id,
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
			let registered_nft_details = NftDetails { spv_created: Default::default(), asset_id: asset_number};
			RegisteredNftDetails::<T>::insert(item_id, registered_nft_details);
			OngoingObjectListing::<T>::insert(item_id, nft.clone());
			ListedToken::<T>::insert(item_id, 100);
			
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let nft_balance: BalanceOf1<T> = 100_u32.into();
			let fractionalize_collection_id: FractionalizeCollectionId<T> = collection_id.try_into().map_err(|_| Error::<T>::ConversionError)?;
			let fractionalize_item_id: FractionalizeItemId<T> = next_item_id.into();
			pallet_nft_fractionalization::Pallet::<T>::fractionalize(
				pallet_origin.clone(),
				fractionalize_collection_id.into(),
				fractionalize_item_id.into(),
				asset_id.into(),
				user_lookup,
				nft_balance,
			)?;
			next_item_id = next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			asset_number = asset_number.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextNftId::<T>::put(next_item_id);
			NextAssetId::<T>::put(asset_number);

			Self::deposit_event(Event::<T>::ObjectListed {
				item_index: item_id,
				price,
				seller: signer,
			});
			Ok(())
		}

		/// List token on the marketplace.
		/// The nft must be registered on the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `item_id`: The item id of the nft.
		/// - `price`: The price of token that are offered.
		/// - `amount`: The amount of token of the real estate object that should be listed.
		///
		/// Emits `TokenListed` event when succesfful
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_nft())]
		pub fn list_token(
			origin: OriginFor<T>,
			item_id: <T as pallet::Config>::ItemId,
			price: BalanceOf<T>,
			amount: u8,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;

			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let nft_details = Self::registered_nft_details(item_id).ok_or(Error::<T>::NftNotFound)?;
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let asset_id: AssetId2<T> = nft_details.asset_id.into();
			let token_amount = amount.into();
			pallet_assets::Pallet::<T, Instance1>::transfer(
				origin,
				asset_id.into().into(),
				pallet_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			let mut listing_id = Self::next_listing_id();
			let token_listing = TokenListingDetails {
				seller: signer.clone(),
				price,
				asset_id: nft_details.asset_id,
				item_id,
				amount,
			};
			TokenListings::<T>::insert(listing_id, token_listing);
			listing_id = listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextListingId::<T>::put(listing_id);

			Self::deposit_event(Event::<T>::TokenListed {
				item_index: item_id,
				price,
				seller: signer,
			});
			Ok(())
		} 

		/// Buy listed token from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `item`: The real estate nft that the investor wants to buy token from.
		/// - `amount`: The amount of token that the investor wants to buy.
		///
		/// Emits `TokenBoughtObject` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_token())]
		pub fn buy_token(
			origin: OriginFor<T>,
			item: <T as pallet::Config>::ItemId,
			amount: u8,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(origin.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let mut listed_token = ListedToken::<T>::take(item).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(
				listed_token >= amount,
				Error::<T>::NotEnoughTokenAvailable
			);
			let mut nft_details =
				Self::ongoing_object_listing(item).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(!Self::registered_nft_details(item).ok_or(Error::<T>::InvalidIndex)?.spv_created, Error::<T>::SpvAlreadyCrated);

			let price = nft_details
				.current_price
				.checked_mul(&Self::u64_to_balance_option(amount as u64)?)
				.ok_or(Error::<T>::MultiplyError)?
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
			let transfer_price = price
				.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::transfer_funds(&origin, &Self::account_id(), transfer_price)?;
			listed_token = listed_token
				.checked_sub(amount)
				.ok_or(Error::<T>::ArithmeticUnderflow)?;
			TokenBuyer::<T>::try_mutate(item, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyTokenBuyer)?;
				Ok::<(), DispatchError>(())
			})?;
			let mut token_of_owner = TokenOwner::<T>::take(origin.clone(), item);
			token_of_owner = token_of_owner.checked_add(amount).ok_or(Error::<T>::ArithmeticOverflow)?;
			nft_details.collected_funds = nft_details.collected_funds
				.checked_add(&price).ok_or(Error::<T>::ArithmeticOverflow)?;
			OngoingObjectListing::<T>::insert(item, nft_details);
			TokenOwner::<T>::insert(origin.clone(), item, token_of_owner);
			if listed_token == 0 {
				Self::distribute_nfts(item)?;
			} else {
				ListedToken::<T>::insert(item, listed_token);
			}
			Self::deposit_event(Event::<T>::TokenBoughtObject {
				item_index: item,
				buyer: origin.clone(),
				amount,
				price,
			});
			Ok(())
		}

		/// Buy token from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the investor wants to buy from.
		///
		/// Emits `TokenBought` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_single_nft())]
		pub fn buy_relisted_token(
			origin: OriginFor<T>,
			listing_id: u32,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(origin.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let listing_details =
				TokenListings::<T>::take(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			let price = listing_details
				.price
				.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::calculate_fees(price, origin.clone(), listing_details.seller)?;
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(origin.clone());
			let asset_id: AssetId2<T> = listing_details.asset_id.into();
			let token_amount = listing_details.amount.into();
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_assets::Pallet::<T, Instance1>::transfer(
				pallet_origin,
				asset_id.into().into(),
				user_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			Self::deposit_event(Event::<T>::TokenBought {
				item_index: listing_details.item_id,
				buyer: origin.clone(),
				price: listing_details.price,
			});
			Ok(())
		}

		/// Upgrade the price from a listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the seller wants to update.
		/// - `new_price`: The new price of the nft.
		///
		/// Emits `ListingUpdated` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::upgrade_listing())]
		pub fn upgrade_listing(
			origin: OriginFor<T>,
			listing_id: u32,
			new_price: BalanceOf<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let mut listing_details =
				Self::token_listings(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.seller == signer, Error::<T>::NoPermission);
			listing_details.price = new_price;
			TokenListings::<T>::insert(listing_id, listing_details);
			Self::deposit_event(Event::<T>::ListingUpdated {
				listing_index: listing_id,
				new_price,
			});
			Ok(())
		}

		/// Upgrade the price from a listed object.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `item_id`: The real estate nft that the investor wants to buy token from.
		/// - `new_price`: The new price of the object.
		///
		/// Emits `ObjectUpdated` event when succesfful.
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
			ensure!(!Self::registered_nft_details(item_id).ok_or(Error::<T>::InvalidIndex)?.spv_created, Error::<T>::SpvAlreadyCrated);
			let mut nft_details =
				Self::ongoing_object_listing(item_id).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(ListedToken::<T>::contains_key(item_id), Error::<T>::TokenNotForSale);
			nft_details.current_price = new_price;
			OngoingObjectListing::<T>::insert(item_id, nft_details);
			Self::deposit_event(Event::<T>::ObjectUpdated {
				item_index: item_id,
				new_price,
			});
			Ok(())
		}

 		/// Delist the choosen listing from the marketplace.
		/// Works only for relisted token.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing that the seller wants to delist.
		///
		/// Emits `ListingDelisted` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_token())]
		pub fn delist_token(
			origin: OriginFor<T>,
			listing_id: u32,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(
				pallet_whitelist::Pallet::<T>::whitelisted_accounts(signer.clone()),
				Error::<T>::UserNotWhitelisted
			);
			let listing_details =
				TokenListings::<T>::take(listing_id).ok_or(Error::<T>::TokenNotForSale)?;
			ensure!(listing_details.seller == signer, Error::<T>::NoPermission);
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(signer.clone());
			let asset_id: AssetId2<T> = listing_details.asset_id.into();
			let token_amount = listing_details.amount.into();
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_assets::Pallet::<T, Instance1>::transfer(
				pallet_origin,
				asset_id.into().into(),
				user_lookup,
				token_amount,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			Self::deposit_event(Event::<T>::ListingDelisted {
				listing_index: listing_id,
			});
			Ok(())
		}  
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

		/// Sends the token to the new owners and the funds to the real estate developer once all 100 token
		/// of a collection are sold.
		fn distribute_nfts(item: <T as pallet::Config>::ItemId) -> DispatchResult {
			let list = <TokenBuyer<T>>::take(item);

			let nft_details = OngoingObjectListing::<T>::take(item)
				.ok_or(Error::<T>::InvalidIndex)?;
			let price = nft_details.collected_funds
				.checked_mul(&Self::u64_to_balance_option(1/* 000000000000 */)?)
				.ok_or(Error::<T>::MultiplyError)?;
			Self::calculate_fees(price, Self::account_id(), nft_details.real_estate_developer)?;
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
			let mut registered_nft_details = Self::registered_nft_details(item).ok_or(Error::<T>::InvalidIndex)?;
			registered_nft_details.spv_created = true;
			RegisteredNftDetails::<T>::insert(item, registered_nft_details); 
			Ok(())
		}

		fn calculate_fees(price: BalanceOf<T>, sender: AccountIdOf<T>, receiver: AccountIdOf<T>) -> DispatchResult {
			let fees = price
				.checked_div(&Self::u64_to_balance_option(100)?)
				.ok_or(Error::<T>::DivisionError)?;
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
			Self::transfer_funds(&sender, &treasury_id, treasury_fees)?;
			Self::transfer_funds(&sender, &community_projects_id, community_fees)?;
			Self::transfer_funds(
				&sender,
				&receiver,
				seller_part,
			)?;
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
/* 		fn default_collection_config() -> CollectionConfig<
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
		} */

		/// Set the default item configuration for minting a nft.
		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}

		/// Converts a u64 to a balance.
		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
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
