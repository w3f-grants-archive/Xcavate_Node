use crate::{mock::*, Error, Event};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
};

use crate::Config;
use crate::{BalanceOf, BoundedNftDonationTypes, NftDonationTypes};

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
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			5,
			900,
			bvec![22, 22]
		));
		assert_eq!(CommunityProjects::listed_nfts().len(), 6);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
			5,
			900,
			bvec![22, 22]
		));
		assert_ok!(CommunityProjects::buy_nft(RuntimeOrigin::signed([1; 32].into()), 0, 1));
		assert_eq!(CommunityProjects::listed_nfts().len(), 5);
		assert_eq!(Balances::free_balance(&([1; 32].into())), 14_800);
		assert_eq!(Balances::free_balance(&CommunityProjects::account_id()), 20_000_200);
	});
}

#[test]
fn launch_project_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
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
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
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
	});
}

#[test]
fn distributing_funds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
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
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20_000_098);
		assert_eq!(Balances::free_balance(&CommunityProjects::account_id()), 20_000_200);
	});
}

#[test]
fn delete_project_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(CommunityProjects::list_project(
			RuntimeOrigin::signed([0; 32].into()),
			get_project_nfts(3),
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
		assert_eq!(Balances::free_balance(&([0; 32].into())), 20_000_298);
		assert_eq!(Balances::free_balance(&CommunityProjects::account_id()), 20_000_000);
		assert_eq!(CommunityProjects::ongoing_projects(0), None);
	})
}
