// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! A mock runtime for xcm benchmarking.

use crate as pallet_xcm_benchmarks;
use crate::{mock_shared::*, *};
use frame_support::{parameter_types, traits::Contains, weights::Weight};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use xcm::opaque::v0::{prelude::XcmResult, Junction, MultiAsset, MultiLocation, Response, Xcm};
use xcm_executor::AssetId;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the pallet, we construct a mock runtime.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		XcmPalletBenchmarks: pallet_xcm_benchmarks::{Pallet},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::AllowAll;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 7;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const AssetDeposit: u64 = 100 * ExistentialDeposit::get();
	pub const ApprovalDeposit: u64 = 1 * ExistentialDeposit::get();
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 10 * ExistentialDeposit::get();
	pub const MetadataDepositPerByte: u64 = 1 * ExistentialDeposit::get();
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

pub struct CheckAsset;
impl Contains<u32> for CheckAsset {
	fn contains(_: &u32) -> bool {
		true
	}
}

pub struct MatchAnyFungibles;
impl xcm_executor::traits::MatchesFungibles<u32, u64> for MatchAnyFungibles {
	fn matches_fungibles(m: &MultiAsset) -> Result<(u32, u64), xcm_executor::traits::Error> {
		//                                                     ^^ TODO: this error is too out of scope.
		use sp_runtime::traits::SaturatedConversion;
		match m {
			// TODO:
			MultiAsset::ConcreteFungible { amount, .. } => Ok(
				(
					0,
					(*amount).saturated_into::<u64>(),
				)
			),
			_ => Err(xcm_executor::traits::Error::AssetNotFound),
		}
	}
}

parameter_types! {
	pub const CheckedAccount: u64 = 100;
}

pub type AssetTransactor = xcm_builder::FungiblesAdapter<
	Assets,
	MatchAnyFungibles,
	AccountIdConverter,
	u64,
	CheckAsset,
	CheckedAccount,
>;

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = Call;
	type XcmSender = DevNull;
	type AssetTransactor = AssetTransactor;
	type OriginConverter = ();
	type IsReserve = ();
	type IsTeleporter = (); // no one can teleport.
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type Barrier = YesItShould<Test>;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
	type Trader = xcm_builder::FixedRateOfConcreteFungible<WeightPrice, ()>;
	type ResponseHandler = DevNull;
}

impl pallet_xcm_benchmarks::Config for Test {
	type XcmConfig = XcmConfig;
	type FungibleTransactAsset = Balances;
	type FungiblesTransactAsset = Assets;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = GenesisConfig { ..Default::default() }.build_storage().unwrap();
	sp_tracing::try_init_simple();
	t.into()
}
