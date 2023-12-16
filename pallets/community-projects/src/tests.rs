use crate::{mock::*, Error, Event};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};

use crate::Config;
use crate::{BalanceOf, BoundedNftDonationTypes, NftDonationTypes};
use sp_core::{bounded::BoundedVec, Pair};

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn get_project_nfts(mut n: u32) -> BoundedNftDonationTypes<Test> {
	let max = <Test as Config>::MaxNftTypes::get();
	if n > max {
		n = max
	}
	(1..=n)
		.map(|x| NftDonationTypes::<BalanceOf<Test>> { price: (100 * x).into(), amount: x })
		.collect::<Vec<NftDonationTypes<BalanceOf<Test>>>>()
		.try_into()
		.expect("bound is ensured; qed")
}

fn get_nft_metadata(
	mut n: u32,
) -> BoundedVec<
	BoundedVec<u8, <Test as pallet_nfts::Config>::StringLimit>,
	<Test as Config>::MaxListedNfts,
> {
	let max = <Test as Config>::MaxListedNfts::get();
	if n > max {
		n = max
	}
	(1..=n)
		.map(|_| bvec![22, 22])
		.collect::<Vec<BoundedVec<u8, <Test as pallet_nfts::Config>::StringLimit>>>()
		.try_into()
		.expect("bound is ensured; qed")
}

fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 0 {
			CommunityProjects::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		CommunityProjects::on_initialize(System::block_number());
	}
}

#[test]
fn list_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			400,
			bvec![22, 22]
		));
		assert_eq!(CommunityProjects::listed_nfts().len(), 6);
	});
}

#[test]
fn list_fails_with_not_enough_metadata() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			CommunityProjects::list_project(
				RuntimeOrigin::signed([0; 32].into()),
				get_project_nfts(3),
				get_nft_metadata(4),
				5,
				400,
				bvec![22, 22]
			),
			Error::<Test>::WrongAmountOfMetadata
		);
	});
}

#[test]
fn list_fails_with_price_too_high() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			CommunityProjects::list_project(
				RuntimeOrigin::signed([0; 32].into()),
				get_project_nfts(3),
				get_nft_metadata(3),
				5,
				1000,
				bvec![22, 22]
			),
			Error::<Test>::PriceCannotBeReached
		);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			400,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_eq!(CommunityProjects::listed_nfts().len(), 5);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 1300);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 200);
	});
}

#[test]
fn buy_fails_nft_not_available() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			400,
			bvec![22, 22]
		));
		assert_noop!(
			CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 2, 1),
			Error::<Test>::NftNotFound
		);
	});
}

#[test]
fn buy_fails_nft_not_enough_assets() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [4; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			400,
			bvec![22, 22]
		));
		assert_noop!(
			CommunityProjects::buy_nft(RuntimeOrigin::signed([4; 32].into()), 0, 1),
			Error::<Test>::NotEnoughFunds
		);
		assert_eq!(CommunityProjects::listed_nfts().len(), 6);
	});
}

#[test]
fn launch_project_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 0));
		assert_eq!(CommunityProjects::listed_nfts().len(), 0);
	});
}

#[test]
fn voting_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			5,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 0));
		assert_eq!(CommunityProjects::listed_nfts().len(), 0);
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		assert_eq!(CommunityProjects::ongoing_votes(0).unwrap().yes_votes, 100);
	});
}

#[test]
fn rejecting_vote_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(4),
			get_nft_metadata(4),
			4,
			900,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 5));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 4));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 3));
		assert_eq!(Assets::balance(1, &[1; 32].into()), 900);
		assert_eq!(Assets::balance(1, &[2; 32].into()), 149_700);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 900);
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::Yes
		));
		assert_eq!(CommunityProjects::ongoing_votes(0).unwrap().no_votes, 0);
		run_to_block(31);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::No
		));
		run_to_block(51);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::No
		));
		run_to_block(71);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::No
		));
		run_to_block(81);
		assert_eq!(CommunityProjects::ongoing_projects(0), None);
		assert_eq!(Assets::balance(1, &[1; 32].into()), 1_350);
		assert_eq!(Assets::balance(1, &[2; 32].into()), 149_925);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 0);
	})
}

#[test]
fn voting_fails_with_no_permission() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 5));
		run_to_block(11);
		assert_noop!(
			CommunityProjects::vote_on_milestone(
				RuntimeOrigin::signed([2; 32].into()),
				0,
				crate::Vote::Yes
			),
			Error::<Test>::InsufficientPermission
		);
	})
}

#[test]
fn voting_fails_with_double_voting() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 5));
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::Yes
		));
		assert_noop!(
			CommunityProjects::vote_on_milestone(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				crate::Vote::Yes
			),
			Error::<Test>::AlreadyVoted
		);
	})
}

#[test]
fn voting_fails_with_no_ongoing_voting() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 5));
		assert_noop!(
			CommunityProjects::vote_on_milestone(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				crate::Vote::Yes
			),
			Error::<Test>::NoOngoingVotingPeriod
		);
	})
}

#[test]
fn set_strikes_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 5));
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::No
		));
		run_to_block(31);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::No
		));
		run_to_block(41);
		assert_eq!(CommunityProjects::ongoing_projects(0).unwrap().strikes, 2);
		run_to_block(51);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([1; 32].into()),
			0,
			crate::Vote::Yes
		));
		run_to_block(61);
		assert_eq!(CommunityProjects::ongoing_projects(0).unwrap().strikes, 0);
	})
}

#[test]
fn distributing_funds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 0));
		assert_eq!(CommunityProjects::listed_nfts().len(), 0);
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_100);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 200);
	});
}

#[test]
fn distributing_funds_for_2_rounds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			3,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 0));
		assert_eq!(CommunityProjects::listed_nfts().len(), 0);
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_100);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 200);
		run_to_block(32);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		run_to_block(42);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_200);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 100);
	});
}

#[test]
fn delete_project_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [1; 32].into()));
		assert_ok!(Whitelist::add_to_whitelist(RuntimeOrigin::root(), [2; 32].into()));
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			get_nft_metadata(3),
			1,
			300,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([2; 32].into()), 0, 0));
		run_to_block(11);
		assert_ok!(CommunityProjects::vote_on_milestone(
			RuntimeOrigin::signed([2; 32].into()),
			0,
			crate::Vote::Yes
		));
		run_to_block(22);
		assert_eq!(Assets::balance(1, &[0; 32].into()), 20_000_300);
		assert_eq!(Assets::balance(1, &CommunityProjects::account_id()), 0);
		assert_eq!(CommunityProjects::ongoing_projects(0), None);
	})
}
