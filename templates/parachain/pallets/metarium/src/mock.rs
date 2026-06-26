use crate as pallet_metarium;
use frame_support::{derive_impl, parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Metarium: pallet_metarium,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


impl pallet_metarium::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxKuriLength = frame_support::traits::ConstU32<64>;
	type MaxCommitSize = frame_support::traits::ConstU64<18446744073709551615>;
	type MaxIPAddressLength = frame_support::traits::ConstU32<64>;
	type MaxSSHPubKeyLength = frame_support::traits::ConstU32<64>;
	type MaxChannelsPerNode = frame_support::traits::ConstU32<64>;
	type MaxActantsPerChannel = frame_support::traits::ConstU32<64>;
	type MaxListenersPerChannel = frame_support::traits::ConstU32<64>;
	type MaxArikurisToTransfer = frame_support::traits::ConstU32<64>;
	type MaxCustodianMetadataHistoryLength = frame_support::traits::ConstU32<64>;
	type CommitLockTtl = frame_support::traits::ConstU64<10>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
