//! Slice 4 integration test: genesis `sudo.key` set to a k-of-n multisig account,
//! and a scribe grant executed through `as_multi` → `sudo.sudo`.
//!
//! Composes `pallet_balances + pallet_sudo + pallet_multisig + pallet_metarium` in a
//! dedicated mock so the multisig-governs-sudo flow is exercised deterministically,
//! without needing the full node/devnet build. This is the in-process counterpart to
//! the live devnet check (Slice 5).

use crate as pallet_metarium;
use codec::Encode;
use frame_support::{
	assert_ok, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64, Everything},
	weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;
type Balance = u64;

const ALICE: AccountId = 1;
const BOB: AccountId = 2;
const CHARLIE: AccountId = 3;
const NEW_SCRIBE: AccountId = 99;
const THRESHOLD: u16 = 2;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		Sudo: pallet_sudo,
		Multisig: pallet_multisig,
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
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	// Balances live in frame_system account data so the multisig deposit can be reserved.
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

impl pallet_sudo::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

parameter_types! {
	pub const DepositBase: Balance = 1;
	pub const DepositFactor: Balance = 1;
}

impl pallet_multisig::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = ();
	type BlockNumberProvider = System;
}

impl pallet_metarium::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxKuriLength = ConstU32<64>;
	type MaxCommitSize = ConstU64<18446744073709551615>;
	type MaxIPAddressLength = ConstU32<64>;
	type MaxSSHPubKeyLength = ConstU32<64>;
	type MaxChannelsPerNode = ConstU32<64>;
	type MaxActantsPerChannel = ConstU32<64>;
	type MaxListenersPerChannel = ConstU32<64>;
	type MaxArikurisToTransfer = ConstU32<64>;
	type MaxCustodianMetadataHistoryLength = ConstU32<64>;
	type CommitLockTtl = ConstU64<10>;
}

/// The 2-of-3 multisig account over the sorted signatory set. This is a pure derivation
/// (`blake2_256` of `(b"modlpy/utilisuba", signatories, threshold)`) — the same value the
/// pallet computes at dispatch time — so it can be set as the genesis sudo key offline.
fn multisig_account() -> AccountId {
	pallet_multisig::Pallet::<Test>::multi_account_id(&[ALICE, BOB, CHARLIE], THRESHOLD)
}

fn new_test_ext() -> sp_io::TestExternalities {
	let multi = multisig_account();
	let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 1_000_000),
			(BOB, 1_000_000),
			(CHARLIE, 1_000_000),
			(multi, 1_000_000),
		],
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();
	// Genesis sudo.key IS the multisig account: chain governance is k-of-n, not one key.
	pallet_sudo::GenesisConfig::<Test> { key: Some(multi) }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn sudo_key_is_the_multisig_account_at_genesis() {
	new_test_ext().execute_with(|| {
		assert_eq!(pallet_sudo::Key::<Test>::get(), Some(multisig_account()));
	});
}

#[test]
fn two_of_three_multisig_grants_a_scribe_via_sudo() {
	new_test_ext().execute_with(|| {
		let multi = multisig_account();
		assert_eq!(pallet_sudo::Key::<Test>::get(), Some(multi));
		assert!(!Metarium::is_node_in_scribe_set(NEW_SCRIBE));

		// The call the multisig must authorise: a Root-only scribe grant, wrapped in sudo.
		let inner = RuntimeCall::Metarium(pallet_metarium::Call::force_add_node_to_scribe_set {
			node: NEW_SCRIBE,
		});
		let sudo_call = RuntimeCall::Sudo(pallet_sudo::Call::sudo { call: Box::new(inner) });
		let call_hash = blake2_256(&sudo_call.encode());
		let max_weight = Weight::from_parts(1_000_000_000_000, 10_000_000);

		// Step 1 — Alice opens the operation (1 of 2). other_signatories sorted, self excluded.
		assert_ok!(Multisig::as_multi(
			RuntimeOrigin::signed(ALICE),
			THRESHOLD,
			vec![BOB, CHARLIE],
			None,
			Box::new(sudo_call.clone()),
			max_weight,
		));
		// One approval is not enough — still not a scribe.
		assert!(!Metarium::is_node_in_scribe_set(NEW_SCRIBE));

		// Recover the recorded timepoint from storage (robust vs. guessing height/index).
		let when = pallet_multisig::Multisigs::<Test>::get(multi, call_hash)
			.expect("multisig operation was opened")
			.when;

		// Step 2 — Bob approves (2 of 2) → threshold met → the wrapped sudo call dispatches.
		assert_ok!(Multisig::as_multi(
			RuntimeOrigin::signed(BOB),
			THRESHOLD,
			vec![ALICE, CHARLIE],
			Some(when),
			Box::new(sudo_call),
			max_weight,
		));

		// The k-of-n grant took effect: the node is now a scribe.
		assert!(Metarium::is_node_in_scribe_set(NEW_SCRIBE));
		// And the operation entry is cleared once executed.
		assert!(pallet_multisig::Multisigs::<Test>::get(multi, call_hash).is_none());
	});
}
