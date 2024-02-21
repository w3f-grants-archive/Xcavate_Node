#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

use frame_support::{
	sp_runtime::Saturating,
	traits::{
	Currency, ReservableCurrency, OnUnbalanced
	},
};

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type ProposalIndex = u32;
	pub type InqueryIndex = u32;

	/// Proposal with the proposal Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<BlockNumber, T: Config> {
		pub proposer: AccountIdOf<T>,
		pub asset_id: u32,
		pub created_at: BlockNumber,
	}	

	/// Inquery with the inquery Details.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Inquery<BlockNumber, T: Config> {
		pub proposer: AccountIdOf<T>,
		pub asset_id: u32,
		pub created_at: BlockNumber,
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

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft_marketplace::Config + pallet_property_management::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The reservable currency type.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>;
		/// The amount of time given to vote for a proposal.
		type VotingTime: Get<BlockNumberFor<Self>>;

		/// The maximum amount of votes per block.
		type MaxVotesForBlock: Get<u32>;
		/// Handler for the unbalanced reduction when slashing a letting agent.
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;
		/// The minimum amount of a letting agent that will be slashed.
		type MinSlashingAmount: Get<BalanceOf<Self>>;
		/// The maximum amount of users who can vote on a ongoing voting.
		type MaxVoter: Get<u32>;
	}

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(super) type ProposalCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Number of inqueries that have been made.
	#[pallet::storage]
	#[pallet::getter(fn inquery_count)]
	pub(super) type InqueryCount<T> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub(super) type Proposals<T> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalIndex,
		Proposal<BlockNumberFor<T>, T>,
		OptionQuery,
	>;

	/// Inqueries that have been made.
	#[pallet::storage]
	#[pallet::getter(fn inqueries)]
	pub(super) type Inqueries<T> = StorageMap<
		_,
		Blake2_128Concat,
		InqueryIndex,
		Inquery<BlockNumberFor<T>, T>,
		OptionQuery,
	>;

	/// Mapping of ongoing votes.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_votes)]
	pub(super) type OngoingVotes<T> = 
		StorageMap<_, Blake2_128Concat, ProposalIndex, VoteStats, OptionQuery>;
	
	/// Mapping from proposal to vector of users who voted.
	#[pallet::storage]
	#[pallet::getter(fn proposal_voter)]
	pub(super) type ProposalVoter<T: Config> = 
		StorageMap<_, Blake2_128Concat, ProposalIndex, BoundedVec<AccountIdOf<T>, T::MaxVoter>, ValueQuery>;

	/// Mapping from inquery to vector of users who voted.
	#[pallet::storage]
	#[pallet::getter(fn inquery_voter)]
	pub(super) type InqueryVoter<T: Config> = 
		StorageMap<_, Blake2_128Concat, InqueryIndex, BoundedVec<AccountIdOf<T>, T::MaxVoter>, ValueQuery>;

	/// Mapping of ongoing votes about inqueries.
	#[pallet::storage]
	#[pallet::getter(fn ongoing_inquery_votes)]
	pub(super) type OngoingInqueryVotes<T> = 
		StorageMap<_, Blake2_128Concat, InqueryIndex, VoteStats, OptionQuery>;

	#[pallet::storage]
	pub type RoundsExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<ProposalIndex, T::MaxVotesForBlock>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type InqueryRoundsExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<InqueryIndex, T::MaxVotesForBlock>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New proposal has been created.
		Proposed {proposal_id: ProposalIndex, asset_id: u32, proposer: AccountIdOf<T>},
		/// A new inquery has been made.
		Inquery {inquery_id: InqueryIndex, asset_id: u32, proposer: AccountIdOf<T>},
		/// Voted on proposal.
		VotedOnProposal { proposal_id: ProposalIndex, voter: AccountIdOf<T>, vote: Vote},
		/// Voted on inquery.
		VotedOnInquery { inquery_id: InqueryIndex, voter: AccountIdOf<T>, vote: Vote},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The user is not a property owner and has no permission to propose.
		NoPermission,
		/// There are already too many proposals in the ending block.
		TooManyProposals,
		/// The proposal is not ongoing.
		NotOngoing,
		/// Too many user voted already.
		TooManyVotes,
		/// The user already voted.
		AlreadyVoted,
		/// Letting Agent not found.
		LettingAgentNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);

			let ended_votings = RoundsExpiring::<T>::take(n);
			ended_votings.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let _ = <OngoingVotes<T>>::take(item);
			});

			let ended_inquery_votings = InqueryRoundsExpiring::<T>::take(n);
			ended_inquery_votings.iter().for_each(|item| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let voting_results = <OngoingVotes<T>>::take(item);
				if let Some(voting_result) = voting_results {
					if voting_result.yes_votes > voting_result.no_votes {
						let _ = Self::change_letting_agent(*item);
					}
				}
			});	
			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn propose(origin: OriginFor<T>, asset_id: u32) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let onwer_list = pallet_nft_marketplace::Pallet::<T>::property_owner(asset_id);
			ensure!(onwer_list.contains(&origin), Error::<T>::NoPermission);
			let proposal_id = Self::proposal_count().saturating_add(1);
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let expiry_block =
				current_block_number.saturating_add(<T as Config>::VotingTime::get());
			let proposal = Proposal {
				proposer: origin.clone(),
				asset_id,
				created_at: current_block_number,
			};
			RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(proposal_id).map_err(|_| Error::<T>::TooManyProposals)?;
				Ok::<(), DispatchError>(())
			})?;
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0};

			Proposals::<T>::insert(proposal_id, proposal);
			OngoingVotes::<T>::insert(proposal_id, vote_stats);
			Self::deposit_event(Event::Proposed {
				proposal_id, 
				asset_id, 
				proposer: origin,
			});
			Ok(())
		}

		/// Create proposal against a letting_agent
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn inquery_agains_letting_agent(origin: OriginFor<T>, asset_id: u32) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			let inquery_id = Self::inquery_count().saturating_add(1);
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let expiry_block =
				current_block_number.saturating_add(<T as Config>::VotingTime::get());
			let inquery = Inquery {
				proposer: origin.clone(),
				asset_id,
				created_at: current_block_number,
			};
			RoundsExpiring::<T>::try_mutate(expiry_block, |keys| {
				keys.try_push(inquery_id).map_err(|_| Error::<T>::TooManyProposals)?;
				Ok::<(), DispatchError>(())
			})?;
			let vote_stats = VoteStats { yes_votes: 0, no_votes: 0};

			Inqueries::<T>::insert(inquery_id, inquery);
			OngoingInqueryVotes::<T>::insert(inquery_id, vote_stats);
			Self::deposit_event(Event::Inquery {
				inquery_id, 
				asset_id, 
				proposer: origin,
			});
			Ok(())
		}

		/// Voting on a proposal
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn vote_on_proposal(origin: OriginFor<T>, proposal_id: ProposalIndex, vote: Vote) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let proposal = Self::proposals(proposal_id).ok_or(Error::<T>::NotOngoing)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(proposal.asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			ensure!(!Self::proposal_voter(proposal_id).contains(&origin), Error::<T>::AlreadyVoted);
			let mut current_vote = Self::ongoing_votes(proposal_id).ok_or(Error::<T>::NotOngoing)?;
			if vote == Vote::Yes {
				current_vote.yes_votes += 1;
			} else {
				current_vote.no_votes += 1;
			};
			OngoingVotes::<T>::insert(proposal_id, current_vote);
			ProposalVoter::<T>::try_mutate(proposal_id, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyVotes)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::VotedOnProposal{
				proposal_id, 
				voter: origin, 
				vote,
			});
			Ok(())	
		}

		/// Voting against a letting agent
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn vote_on_letting_agent_inquery(origin: OriginFor<T>, inquery_id: InqueryIndex, vote: Vote) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let inquery = Self::inqueries(inquery_id).ok_or(Error::<T>::NotOngoing)?;
			let owner_list = pallet_nft_marketplace::Pallet::<T>::property_owner(inquery.asset_id);
			ensure!(owner_list.contains(&origin), Error::<T>::NoPermission);
			ensure!(!Self::inquery_voter(inquery_id).contains(&origin), Error::<T>::AlreadyVoted);
			let mut current_vote = Self::ongoing_inquery_votes(inquery_id).ok_or(Error::<T>::NotOngoing)?;
			if vote == Vote::Yes {
				current_vote.yes_votes += 1;
			} else {
				current_vote.no_votes += 1;
			};
			OngoingInqueryVotes::<T>::insert(inquery_id, current_vote);
			InqueryVoter::<T>::try_mutate(inquery_id, |keys| {
				keys.try_push(origin.clone()).map_err(|_| Error::<T>::TooManyVotes)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::VotedOnInquery{
				inquery_id,
				voter: origin,
				vote,
			});
			Ok(())	
		}
	}

	impl<T: Config> Pallet<T> {
		fn change_letting_agent(inquery_id: InqueryIndex) -> DispatchResult {
			let inquery = Self::inqueries(inquery_id).ok_or(Error::<T>::NotOngoing)?;
			let letting_agent = pallet_property_management::Pallet::<T>::letting_storage(inquery.asset_id).unwrap();
			let amount = <T as Config>::MinSlashingAmount::get();
			<T as pallet::Config>::Slash::on_unbalanced(<T as pallet::Config>::Currency::slash_reserved(&letting_agent, amount).0);
			let agent_info = pallet_property_management::Pallet::<T>::letting_info(letting_agent.clone()).ok_or(Error::<T>::LettingAgentNotFound)?;
			pallet_property_management::Pallet::<T>::remove_bad_letting_agent(agent_info.location, letting_agent);
			Ok(())
		}
	}
}