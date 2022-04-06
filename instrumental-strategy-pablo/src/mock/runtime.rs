use crate as pallet_pablo_strategy;

use frame_support::{
	parameter_types,
	traits::Everything,
};

use sp_runtime::{
	testing::Header,
	traits::IdentityLookup
};
use sp_core::H256;

pub type BlockNumber = u64;
pub type AccountId = u128;
pub type Balance = u128;

pub const ALICE: AccountId = 0;

// -----------------------------------------------------------------------------------------------
//                                             Config                                             
// -----------------------------------------------------------------------------------------------

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// -----------------------------------------------------------------------------------------------
//                                             Balances                                           
// -----------------------------------------------------------------------------------------------

parameter_types! {
	pub const BalanceExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = BalanceExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

// -----------------------------------------------------------------------------------------------
//                                      Instrumental Strategy                                    
// -----------------------------------------------------------------------------------------------

impl pallet_pablo_strategy::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

// -----------------------------------------------------------------------------------------------
//                                        Construct Runtime                                      
// -----------------------------------------------------------------------------------------------

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		PabloStrategy: pallet_pablo_strategy::{Pallet, Call, Storage, Event<T>},
	}
);

// -----------------------------------------------------------------------------------------------
//                                      Externalities Builder                                     
// -----------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ExtBuilder {
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		t.into()
	}
}