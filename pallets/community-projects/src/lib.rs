#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use pallet_assets::Instance1;

use frame_support::{
	traits::{Currency, Incrementable, ReservableCurrency, UnixTime},
	BoundedVec, PalletId,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, StaticLookup},
	Saturating,
};

use enumflags2::BitFlags;

use frame_system::RawOrigin;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type BalanceOf<T> = <T as pallet_assets::Config<pallet_assets::Instance1>>::Balance;

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
	pub trait BenchmarkHelper<CollectionId, ItemId, AssetId, T> {
		fn to_collection(i: u32) -> CollectionId;
		fn to_nft(i: u32) -> ItemId;
		fn to_asset(i: u32) -> AssetId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<CollectionId: From<u32>, ItemId: From<u32>, T: Config>
		BenchmarkHelper<CollectionId, ItemId, AssetId<T>, T> for NftHelper
	{
		fn to_collection(i: u32) -> CollectionId {
			i.into()
		}
		fn to_nft(i: u32) -> ItemId {
			i.into()
		}
		fn to_asset(i: u32) -> AssetId<T> {
			i.into()
		}
	}

	/// Details about a project.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct ProjectDetails<Balance, T: Config> {
		pub project_owner: AccountIdOf<T>,
		// The targeted amount of funds that the projects aims to collect.
		pub project_price: Balance,
		pub duration: u32,
		pub milestones: u32,
		pub remaining_milestones: u32,
		// The amount of funds that the project collected.
		pub project_balance: Balance,
		pub launching_timestamp: BlockNumberFor<T>,
		pub strikes: u8,
	}

	/// Details about a nft.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct NftDetails<Balance, CollectionId, ItemId, T: Config> {
		pub project_owner: AccountIdOf<T>,
		pub price: Balance,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	/// Struct Nft donation type.
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

	/// Vote enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Vote {
		Yes,
		No,
	}

	/// Voting stats.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct VoteStats {
		pub yes_votes: u64,
		pub no_votes: u64,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_nfts::Config + pallet_assets::Config<Instance1> + pallet_whitelist::Config
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
		/// The maximum amount of different nft types per project.
		type MaxNftTypes: Get<u32>;
		/// The maximum amount of nfts that can be listed at the same time.
		#[pallet::constant]
		type MaxListedNfts: Get<u32>;
		/// The maximum amount of nfts for a collection.
		type MaxNftInCollection: Get<u32>;
		#[cfg(feature = "runtime-benchmarks")]
		type Helper: crate::BenchmarkHelper<
			<Self as pallet::Config>::CollectionId,
			<Self as pallet::Config>::ItemId,
			<Self as pallet_assets::Config<Instance1>>::AssetId,
			Self,
		>;
		/// lose coupling of pallet timestamp.
		type TimeProvider: UnixTime;
		/// The maximum amount of projects that can run at the same time.
		#[pallet::constant]
		type MaxOngoingProjects: Get<u32>;
		/// The maximum amount of nft holder.
		#[pallet::constant]
		type MaxNftHolder: Get<u32>;

		/// Asset id type from pallet assets.
		type AssetId: IsType<<Self as pallet_assets::Config<Instance1>>::AssetId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy;

		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
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
	}

	pub type AssetId<T> = <T as Config>::AssetId;
	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	/// Vector with all currently ongoing listings.
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub(super) type ListedNfts<T: Config> = StorageValue<
		_,
		BoundedVec<
			(<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId),
			T::MaxListedNfts,
		>,
		ValueQuery,
	>;

	/// Mapping from the nft to the nft details.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_nft_details)]
	pub(super) type OngoingNftDetails<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId, 
		Blake2_128Concat,
		<T as pallet::Config>::ItemId,
		NftDetails<
			BalanceOf<T>,
			<T as pallet::Config>::CollectionId,
			<T as pallet::Config>::ItemId,
			T,
		>,
		OptionQuery,
	>;

	/// Mapping from collection id to the project.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_projects)]
	pub(super) type OngoingProjects<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		ProjectDetails<BalanceOf<T>, T>,
		OptionQuery,
	>;

	/// Mapping of collection to the listed nfts of this collection.
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts_of_collection)]
	pub(super) type ListedNftsOfCollection<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		BoundedVec<<T as pallet::Config>::ItemId, T::MaxNftInCollection>,
		ValueQuery,
	>;

	/// Stores the project keys and round types ending on a given block for milestone period.
	#[pallet::storage]
	pub type MilestonePeriodExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<<T as pallet::Config>::CollectionId, T::MaxOngoingProjects>,
		ValueQuery,
	>;

	/// Stores the project keys and round types ending on a given block for milestone period.
	#[pallet::storage]
	pub type VotingPeriodExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<<T as pallet::Config>::CollectionId, T::MaxOngoingProjects>,
		ValueQuery,
	>;

	/// Mapping of ongoing votes.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_votes)]
	pub(super) type OngoingVotes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		VoteStats,
		OptionQuery,
	>;

	/// Mapping of collection to the users.
	#[pallet::storage]
	#[pallet::getter(fn voted_user)]
	pub(super) type VotedUser<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		BoundedVec<AccountIdOf<T>, T::MaxNftHolder>,
		ValueQuery,
	>;

	/// Mapping of a collection to the nft holder.
	#[pallet::storage]
	#[pallet::getter(fn nft_holder)]
	pub(super) type NftHolder<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		BoundedVec<AccountIdOf<T>, T::MaxNftHolder>,
		ValueQuery,
	>;

	/// Mapping of collection id and account id to the voting power.
	#[pallet::storage]
	#[pallet::getter(fn voting_power)]
	pub(super) type VotingPower<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		<T as pallet::Config>::CollectionId,
		Blake2_128Concat,
		AccountIdOf<T>,
		u64,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new object has been listed on the marketplace.
		ProjectListed {
			collection_index: <T as pallet::Config>::CollectionId,
			seller: AccountIdOf<T>,
		},
		/// A nft has been bought.
		NftBought {
			collection_index: <T as pallet::Config>::CollectionId,
			item_index: <T as pallet::Config>::ItemId,
			buyer: AccountIdOf<T>,
			price: BalanceOf<T>,
		},
		/// Voted on a milestone.
		VotedOnMilestone {
			collection_index: <T as pallet::Config>::CollectionId,
			voter: AccountIdOf<T>,
			vote: Vote,
		},
		/// A project has been sold out.
		ProjectLaunched { collection_index: <T as pallet::Config>::CollectionId },
		/// A Voting period has started.
		VotingPeriodStarted { collection_index: <T as pallet::Config>::CollectionId },
		/// Funds has been sent to the project.
		FundsDestributed {
			collection_index: <T as pallet::Config>::CollectionId,
			owner: AccountIdOf<T>,
			amount: BalanceOf<T>,
		},
		/// A Milestone period has started.
		MilestonePeriodStarted { collection_id: <T as pallet::Config>::CollectionId },
		/// The project has been deleted.
		ProjectDeleted { collection_id: <T as pallet::Config>::CollectionId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Max amount of listed nfts reached.
		TooManyListedNfts,
		/// Too many nfts for this collection.
		TooManyNfts,
		/// The Nft has not been found.
		NftNotFound,
		/// This index is not taken.
		InvalidIndex,
		/// The buyer doesn't have enough funds.
		NotEnoughFunds,
		/// A collection is unknown.
		UnknownCollection,
		/// Error during type conversion.
		ConversionError,
		/// Maximum amount of projects already exist.
		TooManyProjects,
		/// A user has already voted during a voting period.
		AlreadyVoted,
		/// Maximum amount of voters has been reached.
		TooManyVoters,
		/// No permission.
		InsufficientPermission,
		/// No voting period ongoing.
		NoOngoingVotingPeriod,
		/// This account has no voting power.
		NoFundsRemaining,
		/// Metadata is not the same amount as nft types.
		WrongAmountOfMetadata,
		/// The Duration must be at least one.
		DurationMustBeHigherThanZero,
		/// The target price is impossible to reach.
		PriceCannotBeReached,
		/// User has not passed the kyc.
		UserNotWhitelisted,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);
			let ended_milestone = MilestonePeriodExpiring::<T>::take(n);
			ended_milestone.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let _ = Self::start_voting_period(*item);
			});

			let ended_voting = VotingPeriodExpiring::<T>::take(n);
			ended_voting.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let voting_result = <OngoingVotes<T>>::take(item);
				if let Some(voting_result) = voting_result {
					if voting_result.yes_votes > voting_result.no_votes {
						let _ = Self::distribute_funds(*item);
					} else {
						let _ = Self::ckeck_strikes(*item);
					}

					OngoingVotes::<T>::remove(item);
				}
				let project = Self::ongoing_projects(*item);
				if let Some(project) = project {
					if project.remaining_milestones >= 1 {
						let _ = Self::start_milestone_period(*item);
					} else {
						let _ = Self::delete_project(*item);
					}
				}
				VotedUser::<T>::take(*item);
			});

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new project and list the nfts for the project on the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `nft_types`: The different nft types that the project creator wants to offer for the project.
		/// - `metadata`: Different metadatas for the different nfts.
		/// - `duration`: Amount of months that the project will need.
		/// - `price`: Amount of funds that needs to be raised.
		/// - `data`: Metadata of the project collection.
		///
		/// Emits `ProjectListed` event when succesfful
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_project())]
		pub fn list_project(
			origin: OriginFor<T>,
			nft_types: BoundedNftDonationTypes<T>,
			metadata: BoundedVec<
				BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
				<T as Config>::MaxListedNfts,
			>,
			duration: u32,
			price: BalanceOf<T>,
			data: BoundedVec<u8, <T as pallet_nfts::Config>::StringLimit>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;

			ensure!(pallet_whitelist::Pallet::<T>::whitelisted_accounts().contains(&signer), Error::<T>::UserNotWhitelisted);

			ensure!(metadata.len() as u32 == nft_types.iter().fold(0, |sum, nft_type| sum + nft_type.amount), Error::<T>::WrongAmountOfMetadata);
			ensure!(price <= nft_types.iter().fold(Default::default() , |sum, nft_type| sum + nft_type.price), Error::<T>::PriceCannotBeReached);
			ensure!(duration > 0, Error::<T>::DurationMustBeHigherThanZero);
			if pallet_nfts::NextCollectionId::<T>::get().is_none() {
				pallet_nfts::NextCollectionId::<T>::set(
					<T as pallet_nfts::Config>::CollectionId::initial_value(),
				);
			};
			let collection_id =
				pallet_nfts::NextCollectionId::<T>::get().ok_or(Error::<T>::UnknownCollection)?;
			let next_collection_id = collection_id.increment();
			pallet_nfts::NextCollectionId::<T>::set(next_collection_id);
			let collection_id: CollectionId<T> = collection_id.into();

			pallet_nfts::Pallet::<T>::do_create_collection(
				collection_id.into(),
				signer.clone(),
				signer.clone(),
				Self::default_collection_config(),
				T::CollectionDeposit::get(),
				pallet_nfts::Event::Created {
					creator: Self::account_id(),
					owner: Self::account_id(),
					collection: collection_id.into(),
				},
			)?;
			pallet_nfts::Pallet::<T>::set_collection_metadata(
				origin.clone(),
				collection_id.into(),
				data.clone(),
			)?;
			let project = ProjectDetails {
				project_owner: signer.clone(),
				project_price: price,
				duration,
				milestones: duration,
				remaining_milestones: duration,
				project_balance: Default::default(),
				launching_timestamp: Default::default(),
				strikes: Default::default(),
			};
			OngoingProjects::<T>::insert(collection_id, project);
			let nft_metadata = &metadata;
			let mut nft_id_index = 0;
			for nft_type in nft_types {
				for _y in 0..nft_type.amount {
					let item_id: <T as pallet::Config>::ItemId = nft_id_index.into();
					let nft = NftDetails {
						project_owner: signer.clone(),
						price: nft_type.price,
						collection_id,
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
					pallet_nfts::Pallet::<T>::set_metadata(
						origin.clone(),
						collection_id.into(),
						item_id.into(),
						nft_metadata[nft_id_index as usize].clone(),
					)?;
					ListedNfts::<T>::try_append((collection_id, item_id))
						.map_err(|_| Error::<T>::TooManyListedNfts)?;
					OngoingNftDetails::<T>::insert(collection_id, item_id, nft.clone());
					ListedNftsOfCollection::<T>::try_mutate(collection_id, |keys| {
						keys.try_push(item_id).map_err(|_| Error::<T>::TooManyNfts)?;
						Ok::<(), DispatchError>(())
					})?;
					nft_id_index += 1;
				}
			}
			pallet_nfts::Pallet::<T>::set_team(
				origin.clone(),
				collection_id.into(),
				None,
				None,
				None,
			)?;
			Self::deposit_event(Event::<T>::ProjectListed {
				collection_index: collection_id,
				seller: signer,
			});
			Ok(())
		}

		/// Buy listed nft from the marketplace.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection that the investor wants to buy from.
		/// - `item_id`: The item the investor wants to buy.
		///
		/// Emits `NftBought` event when succesfful
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::buy_nft())]
		pub fn buy_nft(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			item_id: <T as pallet::Config>::ItemId,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(
				OngoingNftDetails::<T>::contains_key(collection_id, item_id),
				Error::<T>::NftNotFound
			);
			let mut project =
				OngoingProjects::<T>::take(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let nft = OngoingNftDetails::<T>::take(collection_id, item_id)
				.ok_or(Error::<T>::InvalidIndex)?;
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let asset_id: AssetId<T> = 1.into();
			pallet_assets::Pallet::<T, Instance1>::transfer(
				origin,
				asset_id.into().into(),
				user_lookup,
				nft.price, /* * Self::u64_to_balance_option(1000000000000).unwrap_or_default() */
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			pallet_nfts::Pallet::<T>::do_transfer(
				collection_id.into(),
				item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			project.project_balance += nft.price;
			let mut listed_items = Self::listed_nfts_of_collection(collection_id);
			let index = listed_items
				.iter()
				.position(|x| *x == item_id)
				.ok_or(Error::<T>::InvalidIndex)?;
			listed_items.remove(index);
			ListedNftsOfCollection::<T>::insert(collection_id, listed_items);
			if project.project_balance >= project.project_price {
				OngoingProjects::<T>::insert(collection_id, project);
				Self::launch_project(collection_id)?;
			} else {
				OngoingProjects::<T>::insert(collection_id, project);
			};
			let mut listed_nfts = Self::listed_nfts();
			let index = listed_nfts
				.iter()
				.position(|x| *x == (collection_id, item_id))
				.ok_or(Error::<T>::InvalidIndex)?;
			listed_nfts.remove(index);
			ListedNfts::<T>::put(listed_nfts);
			let nft_holder = Self::nft_holder(collection_id);
			if !nft_holder.contains(&signer.clone()) {
				NftHolder::<T>::try_mutate(collection_id, |keys| {
					keys.try_push(signer.clone()).map_err(|_| Error::<T>::TooManyVoters)?;
					Ok::<(), DispatchError>(())
				})?;
			}
			let mut current_voting_power =
				Self::voting_power(collection_id, signer.clone()).unwrap_or_default();
			current_voting_power +=
				TryInto::<u64>::try_into(nft.price).map_err(|_| Error::<T>::ConversionError)?;
			VotingPower::<T>::insert(collection_id, signer.clone(), current_voting_power);
			Self::deposit_event(Event::<T>::NftBought {
				collection_index: collection_id,
				item_index: item_id,
				buyer: signer.clone(),
				price: nft.price,
			});
			Ok(())
		}

		/// Nft holder vote on milestone during voting period.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection for a project that the user wants to vote for.
		/// - `vote`: Must be either a Yes vote or a No vote.
		///
		/// Emits `VotedOnMilestone` event when succesfful
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::vote_on_milestone())]
		pub fn vote_on_milestone(
			origin: OriginFor<T>,
			collection_id: <T as pallet::Config>::CollectionId,
			vote: Vote,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let mut current_vote =
				OngoingVotes::<T>::take(collection_id).ok_or(Error::<T>::NoOngoingVotingPeriod)?;
			let nft_voter = Self::nft_holder(collection_id);
			ensure!(nft_voter.contains(&origin.clone()), Error::<T>::InsufficientPermission);
			let voted = Self::voted_user(collection_id);
			ensure!(!voted.contains(&origin), Error::<T>::AlreadyVoted);
			let voting_power =
				Self::voting_power(collection_id, origin.clone()).unwrap_or_default();
			if vote == Vote::Yes {
				current_vote.yes_votes += voting_power;
			} else {
				current_vote.no_votes += voting_power;
			};
			VotedUser::<T>::try_mutate(collection_id, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyVoters)?;
				Ok::<(), DispatchError>(())
			})?;
			OngoingVotes::<T>::insert(collection_id, current_vote);
			Self::deposit_event(Event::<T>::VotedOnMilestone {
				collection_index: collection_id,
				voter: origin.clone(),
				vote,
			});
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet.
		pub fn account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_account_truncating()
		}

		/// launch the project and delete all remaining nfts.
		fn launch_project(collection_id: <T as pallet::Config>::CollectionId) -> DispatchResult {
			let remaining_nfts = ListedNftsOfCollection::<T>::take(collection_id);
			for item in remaining_nfts {
				pallet_nfts::Pallet::<T>::do_burn(collection_id.into(), item.into(), |_| Ok(()))?;
				let mut listed_nfts = Self::listed_nfts();
				let index = listed_nfts
					.iter()
					.position(|x| *x == (collection_id, item))
					.ok_or(Error::<T>::InvalidIndex)?;
				listed_nfts.remove(index);
				ListedNfts::<T>::put(listed_nfts);
			}
			let mut project =
				Self::ongoing_projects(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			project.launching_timestamp = current_block_number;
			// The milestone period is so short for testing purpose. Later on it will be about three weeks if the duration is lower than 12.

			let milestone_period =
				if project.duration > 12 { project.duration * 10 / 12 } else { 10 };
			OngoingProjects::<T>::insert(collection_id, project);
			let expiry_block = current_block_number.saturating_add(
				milestone_period.try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			MilestonePeriodExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(collection_id).map_err(|_| Error::<T>::TooManyProjects)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::ProjectLaunched { collection_index: collection_id });
			Ok(())
		}

		/// Starts a voting period and enables nft holders from a collection to vote.
		fn start_voting_period(
			collection_id: <T as pallet::Config>::CollectionId,
		) -> DispatchResult {
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0 };
			OngoingVotes::<T>::insert(collection_id, vote_stats);
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			// The voting period is so short for testing purpose. Later on it will be about 1 week.
			let expiry_block = current_block_number
				.saturating_add(10_u64.try_into().map_err(|_| Error::<T>::ConversionError)?);
			VotingPeriodExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(collection_id).map_err(|_| Error::<T>::TooManyProjects)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::VotingPeriodStarted {
				collection_index: collection_id,
			});
			Ok(())
		}

		/// Starts a milestone period after a voting period has ended.
		fn start_milestone_period(
			collection_id: <T as pallet::Config>::CollectionId,
		) -> DispatchResult {
			let project =
			Self::ongoing_projects(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let milestone_period =
				if project.duration > 12 { project.duration * 10 / 12 } else { 10 };
			let expiry_block = current_block_number.saturating_add(
				milestone_period.try_into().map_err(|_| Error::<T>::ConversionError)?,
			);
			MilestonePeriodExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(collection_id).map_err(|_| Error::<T>::TooManyProjects)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::<T>::MilestonePeriodStarted { collection_id });
			Ok(())
		}

		/// Distributes funds after a successful voting for the project.
		fn distribute_funds(collection_id: <T as pallet::Config>::CollectionId) -> DispatchResult {
			let mut project =
				OngoingProjects::<T>::take(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let user_lookup = <T::Lookup as StaticLookup>::unlookup(project.project_owner.clone());
			let origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			let asset_id: AssetId<T> = 1.into();
			pallet_assets::Pallet::<T, Instance1>::transfer(
				origin,
				asset_id.into().into(),
				user_lookup,
				project.project_balance / project.milestones.into()
					/* * Self::u64_to_balance_option(1000000000000).unwrap_or_default() */,
			)
			.map_err(|_| Error::<T>::NotEnoughFunds)?;
			project.remaining_milestones -= 1;
			project.strikes = Default::default();
			OngoingProjects::<T>::insert(collection_id, project.clone());
			Self::deposit_event(Event::<T>::FundsDestributed {
				collection_index: collection_id,
				owner: project.project_owner,
				amount: project.project_balance / project.milestones.into(),
			});
			Ok(())
		}

		/// Evaluates if the project has 3 or more strikes and calls the delete delete_project_refund if its the case.
		fn ckeck_strikes(collection_id: <T as pallet::Config>::CollectionId) -> DispatchResult {
			let mut project =
				Self::ongoing_projects(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			project.strikes += 1;
			OngoingProjects::<T>::insert(collection_id, project.clone());
			if project.strikes >= 3 {
				Self::delete_project_refund(collection_id)?;
			}
			Ok(())
		}

		/// Deletes the project and refunds the remaining funds to the holders.
		fn delete_project_refund(
			collection_id: <T as pallet::Config>::CollectionId,
		) -> DispatchResult {
			let project =
				OngoingProjects::<T>::take(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			let percentage = project.remaining_milestones * 10000 / project.milestones;
			let nft_holders = NftHolder::<T>::take(collection_id);
			for nft_holder in nft_holders {
				let voting_power = VotingPower::<T>::take(collection_id, nft_holder.clone())
					.ok_or(Error::<T>::NoFundsRemaining)?;
				let remaining_funds = voting_power * percentage as u64 / 10000;
				let origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
				let asset_id: AssetId<T> = 1.into();
				let user_lookup = <T::Lookup as StaticLookup>::unlookup(nft_holder);
				pallet_assets::Pallet::<T, Instance1>::transfer(
					origin,
					asset_id.into().into(),
					user_lookup,
					Self::u64_to_balance_option(remaining_funds)?, /* * Self::u64_to_balance_option(1000000000000)? */
				)
				.map_err(|_| Error::<T>::NotEnoughFunds)?;
			}
			Self::deposit_event(Event::<T>::ProjectDeleted { collection_id });
			Ok(())
		}

		/// Deletes the projects once all milestones has been reached.
		fn delete_project(collection_id: <T as pallet::Config>::CollectionId) -> DispatchResult {
			OngoingProjects::<T>::take(collection_id).ok_or(Error::<T>::InvalidIndex)?;
			Self::deposit_event(Event::<T>::ProjectDeleted { collection_id });
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

		pub fn u64_to_balance_option(input: u64) -> Result<BalanceOf<T>, Error<T>> {
			input.try_into().map_err(|_| Error::<T>::ConversionError)
		}
	}
}
