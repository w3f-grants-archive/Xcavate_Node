
//! Autogenerated weights for `pallet_property_management`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-03-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `LAPTOP-DFFNONK6`, CPU: `11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: 1024

// Executed Command:
// ./target/release/node-template
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_property_management
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// pallets/property-management/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn add_letting_agent() -> Weight;
	fn letting_agent_deposit() -> Weight;
	fn set_letting_agent() -> Weight;
	fn distribute_income() -> Weight;
	fn withdraw_funds() -> Weight;
}

/// Weight functions for `pallet_property_management`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `NftMarketplace::LocationCollections` (r:1 w:0)
	/// Proof: `NftMarketplace::LocationCollections` (`max_values`: None, `max_size`: Some(24), added: 2499, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::LettingInfo` (r:0 w:1)
	/// Proof: `PropertyManagement::LettingInfo` (`max_values`: None, `max_size`: Some(487), added: 2962, mode: `MaxEncodedLen`)
	fn add_letting_agent() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `181`
		//  Estimated: `3489`
		// Minimum execution time: 13_759_000 picoseconds.
		Weight::from_parts(14_495_000, 0)
			.saturating_add(Weight::from_parts(0, 3489))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `PropertyManagement::LettingInfo` (r:1 w:1)
	/// Proof: `PropertyManagement::LettingInfo` (`max_values`: None, `max_size`: Some(487), added: 2962, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::LettingAgentLocations` (r:1 w:1)
	/// Proof: `PropertyManagement::LettingAgentLocations` (`max_values`: None, `max_size`: Some(3222), added: 5697, mode: `MaxEncodedLen`)
	fn letting_agent_deposit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `156`
		//  Estimated: `6687`
		// Minimum execution time: 31_913_000 picoseconds.
		Weight::from_parts(32_924_000, 0)
			.saturating_add(Weight::from_parts(0, 6687))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `NftMarketplace::RegisteredNftDetails` (r:1 w:0)
	/// Proof: `NftMarketplace::RegisteredNftDetails` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::LettingStorage` (r:1 w:1)
	/// Proof: `PropertyManagement::LettingStorage` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::LettingAgentLocations` (r:1 w:0)
	/// Proof: `PropertyManagement::LettingAgentLocations` (`max_values`: None, `max_size`: Some(3222), added: 5697, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::LettingInfo` (r:1 w:1)
	/// Proof: `PropertyManagement::LettingInfo` (`max_values`: None, `max_size`: Some(487), added: 2962, mode: `MaxEncodedLen`)
	fn set_letting_agent() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `621`
		//  Estimated: `6687`
		// Minimum execution time: 32_846_000 picoseconds.
		Weight::from_parts(34_567_000, 0)
			.saturating_add(Weight::from_parts(0, 6687))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `NftMarketplace::PropertyOwner` (r:1 w:0)
	/// Proof: `NftMarketplace::PropertyOwner` (`max_values`: None, `max_size`: Some(3222), added: 5697, mode: `MaxEncodedLen`)
	/// Storage: `NftMarketplace::PropertyOwnerToken` (r:1 w:0)
	/// Proof: `NftMarketplace::PropertyOwnerToken` (`max_values`: None, `max_size`: Some(69), added: 2544, mode: `MaxEncodedLen`)
	/// Storage: `PropertyManagement::StoredFunds` (r:1 w:1)
	/// Proof: `PropertyManagement::StoredFunds` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	fn distribute_income() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `574`
		//  Estimated: `6687`
		// Minimum execution time: 66_360_000 picoseconds.
		Weight::from_parts(68_840_000, 0)
			.saturating_add(Weight::from_parts(0, 6687))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `PropertyManagement::StoredFunds` (r:1 w:1)
	/// Proof: `PropertyManagement::StoredFunds` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn withdraw_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `312`
		//  Estimated: `3593`
		// Minimum execution time: 52_481_000 picoseconds.
		Weight::from_parts(54_633_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
