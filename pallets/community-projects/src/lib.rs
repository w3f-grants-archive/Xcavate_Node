#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::EncodeLike;

use frame_support::{
	traits::{
		Currency, ExistenceRequirement::KeepAlive, Incrementable, ReservableCurrency, UnixTime,
	},
	BoundedVec, PalletId,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_support::sp_runtime::{
	Saturating,
	traits::{AccountIdConversion, CheckedDiv, CheckedMul},
};

use enumflags2::BitFlags;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type BalanceOf1<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub type BoundedNftDonationTypes<T> =
	BoundedVec<NftDonationTypes<BalanceOf<T>>, <T as Config>::MaxNftTypes>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

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
	pub struct ProjectDetails<Balance, T: Config> {
		pub project_owner: AccountIdOf<T>,
		pub project_price: Balance,
		pub duration: u32,
		pub project_balance: Balance,
		pub launching_timestamp: BlockNumberFor<T>,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftDetails<Balance, CollectionId, ItemId, T: Config> {
		pub project_owner: AccountIdOf<T>,
		pub price: Balance,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct NftDonationTypes<Balance> {
		pub price: Balance,
		pub amount: u32,
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
		/// The maximum amount of different nft types per project.
		type MaxNftTypes: Get<u32>;
		/// The maximum amount of nfts that can be listed at the same time.
		#[pallet::constant]
		type MaxListedNfts: Get<u32>;

		/// The maximum amount of nfts for a collection.
		type MaxNftInCollection: Get<u32>;
		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<Self::CollectionId, Self::ItemId>;
		/// lose coupling of pallet timestamp.
		type TimeProvider: UnixTime;
		/// The maximum amount of projects that can run at the same time.
		#[pallet::constant]
		type MaxOngoingProjects: Get<u32>;
	}

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

	/// Mapping from collection id to the project
	#[pallet::storage]
	#[pallet::getter(fn ongoing_projects)]
	pub(super) type OngoingProjects<T: Config> =
		StorageMap<_, Twox64Concat, T::CollectionId, ProjectDetails<BalanceOf<T>, T>, OptionQuery>;

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

	/// Stores the project keys and round types ending on a given block for milestone period.
	#[pallet::storage]
	pub type MilestonePeriodExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<T::CollectionId, T::MaxOngoingProjects>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace.
		ProjectListed { collection_index: T::CollectionId, seller: AccountIdOf<T> },
		/// A nft has been bought.
		NftBought {
			collection_index: T::CollectionId,
			item_index: T::ItemId,
			buyer: AccountIdOf<T>,
			price: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Max amount of listed Nfts reached.
		TooManyListedNfts,
		/// Too many nfts for this collection.
		TooManyNfts,
		/// The Nft has not been found.
		NftNotFound,
		/// This index is not taken.
		InvalidIndex,
		/// The buyer doesn't have enough funds.
		NotEnoughFunds,
		UnknownCollection,
		ConversionError,
		TooManyProjects,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
	{
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);
			let ended_milestone = MilestonePeriodExpiring::<T>::take(n);
			ended_milestone.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				start_voting_period(item);
			})

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u32: From<<T as pallet_nfts::Config>::CollectionId>,
		<T as pallet_nfts::Config>::ItemId: From<u32>,
		u32: EncodeLike<<T as pallet_nfts::Config>::CollectionId>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn list_project(
			origin: OriginFor<T>,
			nft_types: BoundedNftDonationTypes<T>,
			duration: u32,
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
				.ok_or(Error::<T>::UnknownCollection)?
				.into();
			let collection_id =
				pallet_nfts::NextCollectionId::<T>::get().ok_or(Error::<T>::UnknownCollection)?;
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
			let project = ProjectDetails {
				project_owner: signer.clone(),
				project_price: price,
				duration,
				project_balance: Default::default(),
				launching_timestamp: Default::default(),
			};
			OngoingProjects::<T>::insert(collection_id, project);
			let mut nft_id_index = 0;
			for nft_type in nft_types {
				for y in 0..nft_type.amount {
					let item_id: T::ItemId = nft_id_index.into();
					let nft = NftDetails {
						project_owner: signer.clone(),
						price: nft_type.price,
						collection_id,
						item_id,
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
					ListedNfts::<T>::try_append((collection_id, item_id))
						.map_err(|_| Error::<T>::TooManyListedNfts)?;
					OngoingNftDetails::<T>::insert((collection, item_id), nft.clone());
					ListedNftsOfCollection::<T>::try_mutate(collection_id, |keys| {
						keys.try_push(item_id).map_err(|_| Error::<T>::TooManyNfts)?;
						Ok::<(), DispatchError>(())
					})?;
					nft_id_index += 1;
				}
			}
			pallet_nfts::Pallet::<T>::set_team(origin.clone(), collection_id, None, None, None)?;

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn buy_nft(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
		) -> DispatchResult {
			/// Buy x nfts
			let origin = ensure_signed(origin)?;

			ensure!(
				OngoingNftDetails::<T>::contains_key((collection_id, item_id)),
				Error::<T>::NftNotFound
			);
			let mut project =
				OngoingProjects::<T>::take(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let nft = OngoingNftDetails::<T>::take((collection_id, item_id))
				.ok_or(Error::<T>::InvalidIndex)?;
			<T as pallet::Config>::Currency::transfer(
				&origin.clone(),
				&nft.project_owner,
				nft.price * Self::u64_to_balance_option(1000000000000).unwrap_or_default(),
				KeepAlive,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			pallet_nfts::Pallet::<T>::do_transfer(
				collection_id,
				item_id,
				origin.clone(),
				|_, _| Ok(()),
			)?;
			project.project_balance += nft.price;
			if project.project_balance >= project.project_price {
				Self::launch_project(collection_id);
			} else {
				let mut listed_nfts = Self::listed_nfts();
				let index =
					listed_nfts.iter().position(|x| *x == (collection_id, item_id)).unwrap();
				listed_nfts.remove(index);
				ListedNfts::<T>::put(listed_nfts);
				let mut listed_items = Self::listed_nfts_of_collection(collection_id);
				let index = listed_items.iter().position(|x| *x == item_id).unwrap();
				listed_items.remove(index);
				ListedNftsOfCollection::<T>::insert(collection_id, listed_items);
			};
			OngoingProjects::<T>::insert(collection_id, project);
			Self::deposit_event(Event::<T>::NftBought {
				collection_index: collection_id,
				item_index: item_id,
				buyer: origin.clone(),
				price: nft.price,
			});
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_account_truncating()
		}

		/// launch the project and delete all remaining nfts
		fn launch_project(collection_id: T::CollectionId) -> DispatchResult {
			let remaining_nfts = ListedNftsOfCollection::<T>::take(collection_id);
			for item in remaining_nfts {
				pallet_nfts::Pallet::<T>::do_burn(collection_id, item, |_| Ok(()));
				let mut listed_nfts = Self::listed_nfts();
				let index = listed_nfts.iter().position(|x| *x == (collection_id, item)).unwrap();
				listed_nfts.remove(index);
			}
			let mut project = Self::ongoing_projects(collection_id).unwrap();
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			project.launching_timestamp = current_block_number;
			OngoingProjects::<T>::insert(collection_id, project);
			let expiry_block = current_block_number.saturating_add(10_u64.try_into().ok().unwrap());
			MilestonePeriodExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(collection_id).map_err(|_| Error::<T>::TooManyProjects)?;
				Ok::<(), DispatchError>(())
			})?;
			Ok(())
		}

		fn start_voting_period(collection_id: T::CollectionId) -> DispatchResult {
			
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

		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}
	}
}
