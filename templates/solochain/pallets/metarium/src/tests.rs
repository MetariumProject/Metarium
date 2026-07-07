use sp_core::H256;

use crate::{
	mock::*,
	Event, Error,
	ChannelActants, ChannelListeners, Kuri, CustodianMetadataHistory,
	CustodianMetadataEntry,
	ScribeSetMap, CustodianSetMap,
	NodeInfoMap, NodeInfo,
	TotalChannels, Channels, ChannelInfo,
	BookUuidToChannel, ChannelBookUuid,
	ChannelMembership, ROLE_CUSTODIAN, ROLE_CONFIGURATOR, ROLE_MAKER, ROLE_ACTANT, ROLE_LISTENER,
	ChannelTransferAccepted,
	ChannelCustodianMetadataCommitThreadInfo, ChannelCustodianMetadataCommitThreads,
	ArikuriInfo, Arikuris, TotalArikuris, ArikuriTransferAccepted,
};
use {
	frame_support::{
    	assert_noop, assert_ok,
		BoundedVec,
    	error::BadOrigin,
		pallet_prelude::ConstU32,
	},
	std::str,
};



/////// HELPER CONSTANTS ///////
const TEST_SSH_PUB_KEY: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDZ";
const TEST_IP_ADDRESS: &str = "127.0.0.1";
const SCRIBE_1: u64 = 1;//Also the CUSTODIAN
const CONFIGURATOR: u64 = 2;
const LISTENER: u64 = 3;
const ACTANT: u64 = 4;
const NON_SCRIBE: u64 = 5;
const SCRIBE_2: u64 = 6;
const CONFIGURATOR_2: u64 = 7;
const ACTANT_2: u64 = 8;
const SCRIBE_ADMIN: u64 = 9;
const CUSTODIAN_ADMIN: u64 = 10;
// const TEST_KURI_1: &str = "blake3://7d8626e9ad0546117483ac3270c06b422c6c0e4eb91df04fa4ead962fa971af6";
const TEST_KURI_1: &str = "test_kuri_1";
const TEST_KURI_2: &str = "test_kuri_2";
const TEST_KURI_3: &str = "test_kuri_3";
const TEST_KURI_4: &str = "test_kuri_4";
const TEST_KURI_5: &str = "test_kuri_5";
const TEST_KURI_6: &str = "test_kuri_6";
const TEST_KURI_7: &str = "test_kuri_7";
const TEST_KURI_8: &str = "test_kuri_8";
const TEST_TRANSACTION_HASH_1: &str = "0xd8186830b1545327994835ac7c3ac15522bebf7b2a57c130dc082766284ec210";

/////// HELPER FUNCTIONS ///////
fn metarium_events() -> Vec<Event<Test>> {
	let result = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Metarium(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>();

	System::reset_events();

	result
}


/////// SCRIBE-SET FUNCTIONS ///////

// force_add_node_to_scribe_set

#[test]
fn force_add_node_to_scribe_set_succeeds_when_called_by_root() {
	new_test_ext().execute_with(|| {
        // Assert that SCRIBE_1 is not a scribe.
        assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), false);
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that the expected event is emitted.
        // assert!(metarium_events().contains(&Event::<Test>::NodeAddedToScribeSet(SCRIBE_1)));
        // Assert that SCRIBE_1 is now a scribe.
        assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
	});
}

#[test]
fn force_add_node_to_scribe_set_fails_when_called_for_aleady_added_scribe_by_root() {
	new_test_ext().execute_with(|| {
        // Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when non root account tries to add SCRIBE_1 again to the scribe set.
		assert_noop!(
			Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1),
			Error::<Test>::NodeAlreadyAddedToScribeSet
		);
	});
}

#[test]
fn force_add_node_to_scribe_set_fails_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when NON_SCRIBE tries to add itself to the scribe set.
		assert_noop!(
			Metarium::force_add_node_to_scribe_set(RuntimeOrigin::signed(NON_SCRIBE), NON_SCRIBE),
			Error::<Test>::CallForbidden
		);
	});
}

// force_remove_node_from_scribe_set

#[test]
fn force_remove_node_from_scribe_set_succeeds_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeRemovedFromScribeSet(SCRIBE_1)));
		// Assert that SCRIBE_1 is not a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), false);
	});
}

#[test]
fn force_remove_node_from_scribe_set_fails_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to remove itself from the scribe set.
		assert_noop!(
			Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::signed(SCRIBE_1), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn force_remove_node_from_scribe_set_fails_for_non_scribe_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when root account tries to remove a NON_SCRIBE.
		assert_noop!(
			Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), NON_SCRIBE),
			Error::<Test>::NodeAlreadyRemovedFromScribeSet
		);
	});
}


/////// NODE-INFO FUNCTIONS ///////

// node_updated

#[test]
fn node_updated_succeeds_when_called_by_scribe_as_signed_origin() {
	new_test_ext().execute_with(|| {
		let ssh_pub_key: Vec<u8> = TEST_SSH_PUB_KEY.to_string().into();
		let ip_address: Vec<u8> = TEST_IP_ADDRESS.to_string().into();
		let ssh_pub_key_bounded: BoundedVec<u8, ConstU32<64>> = ssh_pub_key.clone().try_into().unwrap();
		let ip_address_bounded: BoundedVec<u8, ConstU32<64>> = ip_address.clone().try_into().unwrap();
		let node_info = NodeInfo {
			ssh_pub_key: ssh_pub_key_bounded,
			ip_address: ip_address_bounded,
		};
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Scribe SCRIBE_1 updates its node info.
		assert_ok!(Metarium::node_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			ssh_pub_key,
			ip_address
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeUpdated(SCRIBE_1));
		// Assert that the node info is updated.
		assert_eq!(NodeInfoMap::<Test>::get(SCRIBE_1), Some(node_info));
	});
}

#[test]
fn node_updated_fails_when_called_for_scribe_by_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update its node info as unsigned origin.
		assert_noop!(
			Metarium::node_updated(
				RuntimeOrigin::none(),
				TEST_SSH_PUB_KEY.to_string().into(),
				TEST_IP_ADDRESS.to_string().into()
			),
			BadOrigin
		);
	});
}

#[test]
fn node_updated_fails_when_called_by_non_scribe() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when non scribe account tries to update its node info.
		assert_noop!(
			Metarium::node_updated(
				RuntimeOrigin::signed(NON_SCRIBE),
				TEST_SSH_PUB_KEY.to_string().into(),
				TEST_IP_ADDRESS.to_string().into()
			),
			Error::<Test>::CallForbidden
		);
	});
}

// force_update_node

#[test]
fn force_update_node_succeeds_when_called_for_scribe_by_root() {
	new_test_ext().execute_with(|| {
		let ssh_pub_key: Vec<u8> = TEST_SSH_PUB_KEY.to_string().into();
		let ip_address: Vec<u8> = TEST_IP_ADDRESS.to_string().into();
		let ssh_pub_key_bounded: BoundedVec<u8, ConstU32<64>> = ssh_pub_key.clone().try_into().unwrap();
		let ip_address_bounded: BoundedVec<u8, ConstU32<64>> = ip_address.clone().try_into().unwrap();
		let node_info = NodeInfo {
			ssh_pub_key: ssh_pub_key_bounded,
			ip_address: ip_address_bounded,
		};
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root updates the node info of SCRIBE_1.
		assert_ok!(Metarium::force_update_node(
			RuntimeOrigin::root(),
			SCRIBE_1,
			ssh_pub_key,
			ip_address
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeUpdated(SCRIBE_1));
		// Assert that the node info is updated.
		assert_eq!(NodeInfoMap::<Test>::get(SCRIBE_1), Some(node_info));
	});
}

#[test]
fn force_update_node_fails_when_called_for_non_scribe_by_root() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when root account tries to update the node info of SCRIBE_1 when it is not a scribe.
		assert_noop!(
			Metarium::force_update_node(
				RuntimeOrigin::root(),
				NON_SCRIBE,
				TEST_SSH_PUB_KEY.to_string().into(),
				TEST_IP_ADDRESS.to_string().into()
			),
			Error::<Test>::NodeAlreadyRemovedFromScribeSet
		);
	});
}

#[test]
fn force_update_node_fails_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update it's node info as root.
		assert_noop!(
			Metarium::force_update_node(
				RuntimeOrigin::signed(SCRIBE_1),
				2,
				TEST_SSH_PUB_KEY.to_string().into(),
				TEST_IP_ADDRESS.to_string().into()
			),
			BadOrigin
		);
	});
}


/////// CHANNEL-CUSTODIAN-SET FUNCTIONS ///////

// force_add_node_to_custodian_set

#[test]
fn force_add_node_to_custodian_set_succeeds_for_scribe_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Assert that SCRIBE_1 is not a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), false);
		// Assert that SCRIBE_1 is not a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), false);
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeAddedToCustodianSet(SCRIBE_1)));
		// Assert that SCRIBE_1 is now a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), true);
	});
}

#[test]
fn force_add_node_to_custodian_set_fails_for_scribe_and_aleady_added_custodian_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when root account tries to add SCRIBE_1 again to the custodian set.
		assert_noop!(
			Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1),
			Error::<Test>::NodeAlreadyAddedToCustodianSet
		);
	});
}

#[test]
fn force_add_node_to_custodian_set_fails_for_non_scribe_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Assert that NON_SCRIBE is not a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(NON_SCRIBE), false);
		// Ensure the expected error is thrown when root account tries to add NON_SCRIBE to the custodian set.
		assert_noop!(
			Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), NON_SCRIBE),
			Error::<Test>::NodeAlreadyRemovedFromScribeSet
		);
	});
}

#[test]
fn force_add_node_to_custodian_set_fails_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to add itself to the custodian set.
		assert_noop!(
			Metarium::force_add_node_to_custodian_set(RuntimeOrigin::signed(SCRIBE_1), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
	});
}

// force_remove_node_from_custodian_set

#[test]
fn force_remove_node_from_custodian_set_succeeds_for_scribe_and_custodian_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), true);
		// Root removes SCRIBE_1 from the custodian set.
		assert_ok!(Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeRemovedFromCustodianSet(SCRIBE_1)));
		// Assert that SCRIBE_1 is still a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// Assert that SCRIBE_1 is not a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), false);
	});
}

#[test]
fn force_remove_node_from_custodian_set_succeeds_for_removed_scribe_and_custodian_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is now a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), true);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that SCRIBE_1 is not a scribe.
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), false);
		// Root removes SCRIBE_1 from the custodian set.
		assert_ok!(Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeRemovedFromCustodianSet(SCRIBE_1)));
		// Assert that SCRIBE_1 is not a custodian.
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), false);
	});
}

#[test]
fn force_remove_node_from_custodian_set_fails_for_scribe_and_non_custodian_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when root account tries to remove SCRIBE_1 from the custodian set when it is not a custodian.
		assert_noop!(
			Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::root(), SCRIBE_1),
			Error::<Test>::NodeAlreadyRemovedFromCustodianSet
		);
	});
}

#[test]
fn force_remove_node_from_custodian_set_fails_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to remove itself from the custodian set.
		assert_noop!(
			Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::signed(1), 2),
			Error::<Test>::CallForbidden
		);
	});
}


/////// CHANNEL FUNCTIONS ///////

// channel_added

#[test]
fn channel_added_succeeds_when_called_by_valid_custodian_as_signed_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Assert that the total channels is 0.
		assert_eq!(TotalChannels::<Test>::get(), None);
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCreated(new_channel_id.clone())));
		// Assert that the total channels is updated.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
	});
}

// channel_book_uuid_set (C1: the bookUUID ↔ channel binding)

fn setup_channel_1() {
	// SCRIBE_1 = custodian + scribe; CONFIGURATOR = the configurator; channel id 1.
	assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
	assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
	assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
	assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
}

#[test]
fn channel_book_uuid_set_binds_both_directions() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		let book: Kuri<Test> = b"book-uuid-1".to_vec().try_into().unwrap();
		assert_ok!(Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 1, book.clone()));
		assert_eq!(BookUuidToChannel::<Test>::get(book.clone()), Some(1));   // book → channel
		assert_eq!(ChannelBookUuid::<Test>::get(1), Some(book));             // channel → book
	});
}

#[test]
fn channel_book_uuid_set_can_be_done_by_the_configurator() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		let book: Kuri<Test> = b"book-cfg".to_vec().try_into().unwrap();
		assert_ok!(Metarium::channel_book_uuid_set(RuntimeOrigin::signed(CONFIGURATOR), 1, book.clone()));
		assert_eq!(BookUuidToChannel::<Test>::get(book), Some(1));
	});
}

#[test]
fn channel_book_uuid_set_is_bind_once_per_channel() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		let book1: Kuri<Test> = b"book-1".to_vec().try_into().unwrap();
		let book2: Kuri<Test> = b"book-2".to_vec().try_into().unwrap();
		assert_ok!(Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 1, book1));
		assert_noop!(
			Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 1, book2),
			Error::<Test>::ChannelBookUuidAlreadySet
		);
	});
}

#[test]
fn channel_book_uuid_set_rejects_a_bookuuid_bound_elsewhere() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));  // channel 2
		let book: Kuri<Test> = b"shared".to_vec().try_into().unwrap();
		assert_ok!(Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 1, book.clone()));
		assert_noop!(
			Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 2, book),
			Error::<Test>::BookUuidAlreadyBound
		);
	});
}

#[test]
fn channel_book_uuid_set_forbidden_for_a_non_owner_scribe() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_2));
		let book: Kuri<Test> = b"book-x".to_vec().try_into().unwrap();
		// SCRIBE_2 is a scribe but neither configurator nor custodian of channel 1
		assert_noop!(
			Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_2), 1, book),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_book_uuid_set_fails_for_an_unknown_channel() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		let book: Kuri<Test> = b"book-none".to_vec().try_into().unwrap();
		assert_noop!(
			Metarium::channel_book_uuid_set(RuntimeOrigin::signed(SCRIBE_1), 99, book),
			Error::<Test>::ChannelNotFound
		);
	});
}

// ChannelMembership (C2: the scalable reverse index — books_for_address)

#[test]
fn channel_added_records_creator_membership() {
	new_test_ext().execute_with(|| {
		setup_channel_1();   // channel 1: SCRIBE_1 = custodian + maker, CONFIGURATOR = configurator
		assert_eq!(ChannelMembership::<Test>::get(SCRIBE_1, 1), ROLE_CUSTODIAN | ROLE_MAKER);
		assert_eq!(ChannelMembership::<Test>::get(CONFIGURATOR, 1), ROLE_CONFIGURATOR);
	});
}

#[test]
fn actant_membership_is_recorded_and_cleared() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		assert_ok!(Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_eq!(ChannelMembership::<Test>::get(ACTANT, 1) & ROLE_ACTANT, ROLE_ACTANT);
		assert_ok!(Metarium::node_removed_from_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_eq!(ChannelMembership::<Test>::get(ACTANT, 1), 0);   // last role gone → entry dropped
	});
}

#[test]
fn an_account_can_hold_actant_and_listener_roles() {
	new_test_ext().execute_with(|| {
		setup_channel_1();
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		assert_ok!(Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_ok!(Metarium::node_added_to_channel_listener_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_eq!(ChannelMembership::<Test>::get(ACTANT, 1), ROLE_ACTANT | ROLE_LISTENER);
		// dropping one role keeps the other
		assert_ok!(Metarium::node_removed_from_channel_listener_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_eq!(ChannelMembership::<Test>::get(ACTANT, 1), ROLE_ACTANT);
	});
}

#[test]
fn membership_reverse_index_lists_an_accounts_channels() {
	new_test_ext().execute_with(|| {
		setup_channel_1();   // channel 1, SCRIBE_1 custodian
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));  // channel 2
		// books_for_address = iterate the (account, *) prefix — the scalable lookup
		let mut channels: std::vec::Vec<u64> =
			ChannelMembership::<Test>::iter_prefix(SCRIBE_1).map(|(c, _flags)| c).collect();
		channels.sort();
		assert_eq!(channels, std::vec![1, 2]);
	});
}

#[test]
fn migrate_to_v4_backfills_membership_from_existing_channels() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::StorageVersion;
		setup_channel_1();   // channel 1 exists (+ forward-path membership)
		// simulate the PRE-UPGRADE state: the channel exists but its membership entries were never written
		ChannelMembership::<Test>::remove(SCRIBE_1, 1);
		ChannelMembership::<Test>::remove(CONFIGURATOR, 1);
		StorageVersion::new(3).put::<Metarium>();   // on-chain version < 4 so the migration runs
		assert_eq!(ChannelMembership::<Test>::get(SCRIBE_1, 1), 0);

		crate::migration::migrate_to_v4::<Test>();

		assert_eq!(ChannelMembership::<Test>::get(SCRIBE_1, 1), ROLE_CUSTODIAN | ROLE_MAKER);
		assert_eq!(ChannelMembership::<Test>::get(CONFIGURATOR, 1), ROLE_CONFIGURATOR);
	});
}

#[test]
fn channel_added_fails_when_called_by_valid_custodian_as_signed_origin_with_invalid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to add a channel with a non scribe configurator.
		assert_noop!(
			Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), NON_SCRIBE),
			Error::<Test>::InvalidNode
		);
	});
}

#[test]
fn channel_added_fails_when_called_by_non_custodian_as_signed_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when SCRIBE_1 tries to add a channel without being a custodian.
		assert_noop!(
			Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_added_fails_when_called_by_non_scribe_as_signed_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when NON_SCRIBE tries to add a channel.
		assert_noop!(
			Metarium::channel_added(RuntimeOrigin::signed(NON_SCRIBE), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_added_fails_when_called_by_valid_custodian_as_unsigned_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when SCRIBE_1 tries to add a channel as unsigned origin.
		assert_noop!(
			Metarium::channel_added(RuntimeOrigin::none(), CONFIGURATOR),
			BadOrigin
		);
	});
}

// channel_configurator_updated

#[test]
fn channel_configurator_updated_succeeds_when_called_by_valid_custodian_as_signed_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), LISTENER));
		// Scribe SCRIBE_1 updates the configurator of the channel.
		assert_ok!(Metarium::channel_configurator_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelConfiguratorUpdated(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let updated_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: LISTENER,
			maker: SCRIBE_1,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
	});
}

#[test]
fn channel_configurator_updated_fails_when_called_by_valid_custodian_as_signed_origin_with_invalid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the configurator of the channel with a non scribe.
		assert_noop!(
			Metarium::channel_configurator_updated(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), NON_SCRIBE),
			Error::<Test>::InvalidNode
		);
	});
}

#[test]
fn channel_configurator_updated_fails_when_called_by_invalid_custodian_as_signed_origin_with_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Root removes SCRIBE_1 from the custodian set.
		assert_ok!(Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the configurator of the channel to itself without being a custodian.
		assert_noop!(
			Metarium::channel_configurator_updated(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the configurator of the channel to itself without being a scribe.
		assert_noop!(
			Metarium::channel_configurator_updated(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when CONFIGURATOR tries to update the configurator of the channel to a non scribe.
		assert_noop!(
			Metarium::channel_configurator_updated(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), NON_SCRIBE),
			Error::<Test>::CallForbidden
		);
	});
}

// force_unarchive_channel

#[test]
fn force_unarchive_channel_succeeds_for_archived_channel_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelArchived(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let archived_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: true,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(archived_channel_info));
		// Root unarchives the channel.
		assert_ok!(Metarium::force_unarchive_channel(RuntimeOrigin::root(), new_channel_id.clone()));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelUnarchived(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let unarchived_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(unarchived_channel_info));
	});
}

#[test]
fn force_unarchive_channel_fails_for_non_archived_channel_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Ensure the expected error is thrown when root account tries to unarchive the channel which is not archived.
		assert_noop!(
			Metarium::force_unarchive_channel(RuntimeOrigin::root(), new_channel_id.clone()),
			Error::<Test>::ChannelNotArchived
		);
	});
}

#[test]
fn force_unarchive_channel_fails_for_archived_channel_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Ensure the expected error is thrown when SCRIBE_1 tries to unarchive the channel.
		assert_noop!(
			Metarium::force_unarchive_channel(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()),
			BadOrigin
		);
	});
}

// channel_transfer_acceptance_toggled

#[test]
fn channel_transfer_acceptance_toggled_succeeds_for_valid_channel_id_when_called_by_valid_custodian_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Assert channel transfer acceptance by CONFIGURATOR is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), false);
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferAccepted(new_channel_id.clone(), CONFIGURATOR)));
		// Assert channel transfer acceptance by CONFIGURATOR is true.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), true);
		// CONFIGURATOR rejects the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			false
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferRejected(new_channel_id.clone(), CONFIGURATOR)));
		// Assert channel transfer acceptance by CONFIGURATOR is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), false);
	});
}

#[test]
fn channel_transfer_acceptance_toggled_fails_for_invalid_channel_id_when_called_by_valid_custodian_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let invalid_channel_id: u64 = 2;
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to toggle the transfer acceptance of an invalid channel.
		assert_noop!(
			Metarium::channel_transfer_acceptance_toggled(RuntimeOrigin::signed(CONFIGURATOR), invalid_channel_id.clone(), true),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_transfer_acceptance_toggled_fails_for_valid_channel_id_when_called_by_invalid_custodian_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when CONFIGURATOR tries to toggle the transfer acceptance of a channel without being a custodian.
		assert_noop!(
			Metarium::channel_transfer_acceptance_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true),
			Error::<Test>::CallForbidden
		);
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to toggle the transfer acceptance of a channel without being a scribe.
		assert_noop!(
			Metarium::channel_transfer_acceptance_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_transfer_acceptance_toggled_fails_for_valid_channel_id_when_called_by_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when CONFIGURATOR tries to toggle the transfer acceptance of a channel as unsigned origin.
		assert_noop!(
			Metarium::channel_transfer_acceptance_toggled(RuntimeOrigin::none(), new_channel_id.clone(), true),
			BadOrigin
		);
	});
}

// channel_transferred

#[test]
fn channel_transferred_succeeds_for_valid_channel_id_when_called_by_valid_custodian_as_signed_origin_with_valid_new_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>> = vec![];
		let historical_custodian_metadata_bounded = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set: Vec<u64> = vec![];
		let actant_set_bounded: BoundedVec<u64, ConstU32<64>> = actant_set.clone().try_into().unwrap();
		let listener_set: Vec<u64> = vec![];
		let listener_set_bounded: BoundedVec<u64, ConstU32<64>> = listener_set.clone().try_into().unwrap();
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferAccepted(new_channel_id.clone(), CONFIGURATOR)));
		// Assert channel transfer acceptance by CONFIGURATOR is true.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), true);
		// SCRIBE_1 transfers the channel to CONFIGURATOR.
		assert_ok!(Metarium::channel_transferred(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			CONFIGURATOR
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferred(new_channel_id.clone(), CONFIGURATOR)));
		// Assert that the channel info is updated.
		let updated_historical_custodian_metadata: Vec<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>> = vec![
			CustodianMetadataEntry {
				start_block_number: 0,
				end_block_number: 0,
				custodian: SCRIBE_1,
				..Default::default()
			}
		];
		let transferred_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: CONFIGURATOR,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: updated_historical_custodian_metadata.clone().try_into().unwrap(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(transferred_channel_info));
		// Assert channel transfer acceptance by CONFIGURATOR is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), false);
		// Assert channel transfer acceptance by SCRIBE_1 is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), SCRIBE_1), false);
		// SCRIBE_1 accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferAccepted(new_channel_id.clone(), SCRIBE_1)));
		// CONFIGURATOR transfers the channel to SCRIBE_1.
		assert_ok!(Metarium::channel_transferred(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			SCRIBE_1
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferred(new_channel_id.clone(), SCRIBE_1)));
		// Assert that the channel info is updated.
		let updated_historical_custodian_metadata_2: Vec<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>> = vec![
			CustodianMetadataEntry {
				start_block_number: 0,
				end_block_number: 0,
				custodian: SCRIBE_1,
				..Default::default()
			},
			CustodianMetadataEntry {
				start_block_number: 0,
				end_block_number: 0,
				custodian: CONFIGURATOR,
				..Default::default()
			}
		];
		let transferred_channel_info_2 = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: updated_historical_custodian_metadata_2.clone().try_into().unwrap(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(transferred_channel_info_2));
		// Assert channel transfer acceptance by CONFIGURATOR is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), false);
		// Assert channel transfer acceptance by SCRIBE_1 is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), SCRIBE_1), false);
	});
}

#[test]
fn channel_transferred_fails_for_invalid_channel_id_when_called_by_valid_custodian_as_signed_origin_with_valid_new_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer an invalid channel.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(SCRIBE_1), invalid_channel_id.clone(), CONFIGURATOR),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_transferred_fails_for_valid_channel_id_when_called_by_valid_custodian_as_signed_origin_with_invalid_new_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer the channel to a non scribe.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(SCRIBE_1), new_channel_id, NON_SCRIBE),
			Error::<Test>::CallForbidden
		);
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// CONFIGURATOR rejects the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			false
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferRejected(new_channel_id.clone(), CONFIGURATOR)));
		// Assert channel transfer acceptance by CONFIGURATOR is false.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), false);
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer the channel to CONFIGURATOR after rejection.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelTransferAccepted(new_channel_id.clone(), CONFIGURATOR)));
		// Assert channel transfer acceptance by CONFIGURATOR is true.
		assert_eq!(ChannelTransferAccepted::<Test>::get(new_channel_id.clone(), CONFIGURATOR), true);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer the channel to CONFIGURATOR after removal.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_transferred_fails_for_valid_channel_id_when_called_by_unsigned_origin_with_valid_new_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer the channel as unsigned origin.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::none(), new_channel_id.clone(), CONFIGURATOR),
			BadOrigin
		);
	});
}

#[test]
fn channel_transferred_fails_for_valid_channel_id_when_called_by_invalid_custodian_as_signed_origin_with_valid_new_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Root adds CONFIGURATOR to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), CONFIGURATOR));
		// CONFIGURATOR accepts the transfer of the channel.
		assert_ok!(Metarium::channel_transfer_acceptance_toggled(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			true
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to transfer the channel to itself.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Ensure the expected error is thrown when SCRIBE_1 tries to transfer the channel without being a scribe.
		assert_noop!(
			Metarium::channel_transferred(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), CONFIGURATOR),
			Error::<Test>::CallForbidden
		);
	});
}

// channel_archived

#[test]
fn channel_archived_succeeds_for_unarchived_and_valid_channel_id_when_called_by_valid_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelArchived(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let archived_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			archived: true,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(archived_channel_info));
	});
}

#[test]
fn channel_archived_fails_for_archived_and_valid_channel_id_when_called_by_valid_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Ensure the expected error is thrown when SCRIBE_1 tries to archive the channel again.
		assert_noop!(
			Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()),
			Error::<Test>::ChannelAlreadyArchived
		);
	});
}

#[test]
fn channel_archived_fails_for_unarchived_and_valid_channel_id_when_called_by_invalid_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when CONFIGURATOR tries to archive the channel without being a custodian.
		assert_noop!(
			Metarium::channel_archived(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone()),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_archived_fails_for_invalid_channel_id_when_called_by_valid_custodian() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when SCRIBE_1 tries to archive an invalid channel.
		assert_noop!(
			Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), invalid_channel_id.clone()),
			Error::<Test>::ChannelNotFound
		);
	});
}

// channel_pause_toggled

#[test]
fn channel_pause_toggled_succeeds_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			paused: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// CONFIGURATOR pauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelPaused(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let paused_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			paused: true,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(paused_channel_info));
		// CONFIGURATOR unpauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), false));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelUnpaused(new_channel_id.clone())));
		// Assert that the channel info is updated.
		let unpaused_channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			paused: false,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(unpaused_channel_info));
	});
}

#[test]
fn channel_pause_true_fails_for_paused_and_valid_channel_id_when_called_by_valid_configurator_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// CONFIGURATOR pauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true));
		// Ensure the expected error is thrown when CONFIGURATOR tries to pause the channel again.
		assert_noop!(
			Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true),
			Error::<Test>::ChannelAlreadyPaused
		);
	});
}

#[test]
fn channel_pause_false_fails_for_unpaused_and_valid_channel_id_when_called_by_valid_configurator_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to unpause the channel without being paused.
		assert_noop!(
			Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), false),
			Error::<Test>::ChannelAlreadyUnpaused
		);
	});
}

#[test]
fn channel_pause_toggled_fails_for_valid_channel_id_when_called_by_invalid_configurator_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_2 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Ensure the expected error is thrown when SCRIBE_2 tries to pause the channel without being a configurator.
		assert_noop!(
			Metarium::channel_pause_toggled(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone(), true),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_pause_toggled_fails_for_invalid_channel_id_when_called_by_valid_configurator_as_signed_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when CONFIGURATOR tries to pause an invalid channel.
		assert_noop!(
			Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), invalid_channel_id.clone(), true),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_pause_toggled_fails_for_valid_channel_id_when_called_by_valid_configurator_as_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to pause the channel as unsigned origin.
		assert_noop!(
			Metarium::channel_pause_toggled(RuntimeOrigin::none(), new_channel_id.clone(), true),
			BadOrigin
		);
	});
}

// node_added_to_channel_actant_set

#[test]
fn node_added_to_channel_actant_set_succeeds_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		let new_channel_id: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>,
			ConstU32<64>,
		> = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set: Vec<u64> = vec![];
		let actant_set_bounded: BoundedVec<u64, ConstU32<64>> = actant_set.clone().try_into().unwrap();
		let listener_set: Vec<u64> = vec![];
		let listener_set_bounded: BoundedVec<u64, ConstU32<64>> = listener_set.clone().try_into().unwrap();
		// Assert that the total channels is 1.
		assert_eq!(TotalChannels::<Test>::get(), Some(1));
		// Assert that the channel info is updated.
		let new_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded.clone(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(new_channel_info));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeAddedToChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let updated_actant_set: Vec<u64> = vec![ACTANT];
		let updated_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_actant_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add ACTANT to the channel actant set again.
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), ACTANT),
			Error::<Test>::NodeAlreadyExistsInChannelActantSet
		);
	});
}

#[test]
fn node_added_to_channel_actant_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_invalid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add NON_SCRIBE to the channel actant set.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), NON_SCRIBE),
			Error::<Test>::InvalidNode
		);
	});
}

#[test]
fn node_added_to_channel_actant_set_fails_for_invalid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add ACTANT to an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), invalid_channel_id.clone(), ACTANT),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn node_added_to_channel_actant_set_fails_for_valid_channel_id_when_called_by_invalid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		// Ensure the expected error is thrown when ACTANT tries to add itself to the channel actant set without being a configurator.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(ACTANT), new_channel_id.clone(), ACTANT),
			Error::<Test>::CallForbidden
		);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add ACTANT to the channel actant set after removal.
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), ACTANT),
			Error::<Test>::CallForbidden
		);
	});

}

#[test]
fn node_added_to_channel_actant_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_unsigned_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), CONFIGURATOR));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), ACTANT));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add ACTANT to the channel actant set as unsigned origin.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::node_added_to_channel_actant_set(RuntimeOrigin::none(), new_channel_id.clone(), ACTANT),
			BadOrigin
		);
	});
}

// node_removed_from_channel_actant_set

#[test]
fn node_removed_from_channel_actant_set_succeeds_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeAddedToChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let updated_actant_set: Vec<u64> = vec![ACTANT];
		let updated_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_actant_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info.clone()));
		// CONFIGURATOR removes ACTANT from the channel actant set.
		assert_ok!(Metarium::node_removed_from_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeRemovedFromChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let empty_actant_set: Vec<u64> = vec![];
		let empty_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = empty_actant_set.clone().try_into().unwrap();
		let empty_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: empty_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(empty_channel_info.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove ACTANT from the channel actant set again.
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::NodeDoesNotExistInChannelActantSet
		);
		// CONFIGURATOR adds ACTANT to the channel actant set again.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the channel info is updated.
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
		// Root removes ACTANT from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR removes ACTANT from the channel actant set.
		assert_ok!(Metarium::node_removed_from_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the channel info is updated.
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(empty_channel_info));
	});
}

#[test]
fn node_removed_from_channel_actant_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_invalid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove ACTANT from the channel actant set.
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::NodeDoesNotExistInChannelActantSet
		);
	});
}

#[test]
fn node_removed_from_channel_actant_set_fails_for_invalid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove ACTANT from an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				invalid_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn node_removed_from_channel_actant_set_fails_for_valid_channel_id_when_called_by_invalid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to remove itself from the channel actant set without being a configurator.
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::CallForbidden
		);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove ACTANT from the channel actant set after removal.
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn node_removed_from_channel_actant_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_unsigned_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove ACTANT from the channel actant set as unsigned origin.
		assert_noop!(
			Metarium::node_removed_from_channel_actant_set(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
	});
}

// node_added_to_channel_listener_set

#[test]
fn node_added_to_channel_listener_set_succeeds_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeAddedToChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let updated_listener_set: Vec<u64> = vec![LISTENER];
		let updated_listener_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_listener_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: updated_listener_set_bounded.clone(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add LISTENER to the channel listener set again.
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::NodeAlreadyExistsInChannelListenerSet
		);
	});
}

#[test]
fn node_added_to_channel_listener_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_invalid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add NON_SCRIBE to the channel listener set.
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				NON_SCRIBE
			),
			Error::<Test>::InvalidNode
		);
	});
}

#[test]
fn node_added_to_channel_listener_set_fails_for_invalid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add LISTENER to an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				invalid_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn node_added_to_channel_listener_set_fails_for_valid_channel_id_when_called_by_invalid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to add itself to the channel listener set without being a configurator.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::CallForbidden
		);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add LISTENER to the channel listener set after removal.
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn node_added_to_channel_listener_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_unsigned_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to add LISTENER to the channel listener set as unsigned origin.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::node_added_to_channel_listener_set(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				LISTENER
			),
			BadOrigin
		);
	});
}

// node_removed_from_channel_listener_set

#[test]
fn node_removed_from_channel_listener_set_succeeds_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel listener set.
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeAddedToChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let updated_listener_set: Vec<u64> = vec![LISTENER];
		let updated_listener_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_listener_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: updated_listener_set_bounded.clone(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info.clone()));
		// CONFIGURATOR removes LISTENER from the channel listener set.
		assert_ok!(Metarium::node_removed_from_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::NodeRemovedFromChannelActantSet(new_channel_id.clone(), ACTANT)));
		// Assert that the channel info is updated.
		let empty_listener_set: Vec<u64> = vec![];
		let empty_listener_set_bounded: BoundedVec<u64, ConstU32<64>> = empty_listener_set.clone().try_into().unwrap();
		let empty_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: empty_listener_set_bounded.clone(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(empty_channel_info.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove LISTENER from the channel listener set again.
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::NodeDoesNotExistInChannelListenerSet
		);
		// CONFIGURATOR adds LISTENER to the channel listener set again.
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the channel info is updated.
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
		// Root removes LISTENER from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR removes LISTENER from the channel listener set.
		assert_ok!(Metarium::node_removed_from_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the channel info is updated.
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(empty_channel_info));
	});
}

#[test]
fn node_removed_from_channel_listener_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_invalid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove LISTENER from the channel listener set.
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::NodeDoesNotExistInChannelListenerSet
		);
	});
}

#[test]
fn node_removed_from_channel_listener_set_fails_for_invalid_channel_id_when_called_by_valid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel listener set.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove LISTENER from an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				invalid_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn node_removed_from_channel_listener_set_fails_for_valid_channel_id_when_called_by_invalid_configurator_as_signed_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel listener set.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));	
		// Ensure the expected error is thrown when LISTENER tries to remove itself from the channel listener set without being a configurator.
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::CallForbidden
		);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove LISTENER from the channel listener set after removal.
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn node_removed_from_channel_listener_set_fails_for_valid_channel_id_when_called_by_valid_configurator_as_unsigned_origin_with_valid_node() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel listener set.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::node_added_to_channel_listener_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to remove LISTENER from the channel listener set as unsigned origin.
		assert_noop!(
			Metarium::node_removed_from_channel_listener_set(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				LISTENER
			),
			BadOrigin
		);
	});
}

// channel_maker_updated

#[test]
fn channel_maker_updated_succeeds_for_valid_channel_id_when_called_by_valid_maker_as_signed_origin_with_valid_new_maker() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Scribe SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info.clone()));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// SCRIBE_1 updates the channel maker to LISTENER.
		assert_ok!(Metarium::channel_maker_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			LISTENER
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelMakerUpdated(new_channel_id.clone(), LISTENER)));
		// Assert that the channel info is updated.
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: LISTENER,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
	});
}

#[test]
fn channel_maker_updated_fails_for_valid_channel_id_when_called_by_valid_maker_as_signed_origin_with_invalid_new_maker() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// SCRIBE_1 adds a channel with SCRIBE_1 as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel maker to NON_SCRIBE.
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				NON_SCRIBE
			),
			Error::<Test>::InvalidNode
		);
	});
}

#[test]
fn channel_maker_updated_fails_for_invalid_channel_id_when_called_by_valid_maker_as_signed_origin_with_valid_new_maker() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// SCRIBE_1 adds a channel with SCRIBE_1 as the configurator.
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the maker of an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				invalid_channel_id.clone(),
				SCRIBE_1
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_maker_updated_fails_for_valid_channel_id_when_called_by_invalid_maker_as_signed_origin_with_valid_new_maker() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to set itself as the maker of the channel.
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				CONFIGURATOR
			),
			Error::<Test>::CallForbidden
		);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update itself as maker of the channel after removal.
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				SCRIBE_1
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the maker of the channel after removal.
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				CONFIGURATOR
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_maker_updated_fails_for_valid_channel_id_when_called_by_valid_maker_as_unsigned_origin_with_valid_new_maker() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel maker as unsigned origin.
		assert_noop!(
			Metarium::channel_maker_updated(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				CONFIGURATOR
			),
			BadOrigin
		);
	});
}

// channel_metadata_updated

#[test]
fn channel_metadata_updated_succeeds_for_valid_channel_id_when_called_by_valid_maker_as_signed_origin_with_valid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info.clone()));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		let channel_metadata_bounded: BoundedVec<u8, ConstU32<64>> = channel_metadata.clone().try_into().unwrap();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_metadata.clone(),
			new_channel_id.clone()
		));
		// SCRIBE_1 updates the channel metadata.
		assert_ok!(Metarium::channel_metadata_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			channel_metadata.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelMetadataUpdated(new_channel_id.clone(), channel_metadata.clone())));
		// Assert that the channel info is updated.
		let updated_actant_set: Vec<u64> = vec![ACTANT];
		let updated_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_actant_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			metadata: Some(channel_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
	});
}

#[test]
fn channel_metadata_updated_fails_for_valid_channel_id_when_called_by_valid_maker_as_signed_origin_with_invalid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let mut invalid_channel_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_channel_metadata.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel metadata with empty metadata.
		invalid_channel_metadata.clear();
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_channel_metadata.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		invalid_channel_metadata = "a".repeat(129).into();
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_channel_metadata.clone()
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
	});
}

#[test]
fn channel_metadata_updated_fails_for_invalid_channel_id_when_called_by_valid_maker_as_signed_origin_with_valid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the metadata of an invalid channel.
		let invalid_channel_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				invalid_channel_id.clone(),
				invalid_channel_metadata.clone()
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_metadata_updated_fails_for_valid_channel_id_when_called_by_invalid_maker_as_signed_origin_with_valid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to update the channel metadata.
		let channel_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				channel_metadata.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel metadata after removal.
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				channel_metadata.clone()
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_metadata_updated_fails_for_valid_channel_id_when_called_by_valid_maker_as_unsigned_origin_with_valid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// SCRIBE_1 adds a channel with SCRIBE_1 as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel metadata as unsigned origin.
		let channel_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				channel_metadata.clone()
			),
			BadOrigin
		);
	});
}

// channel_functional_metadata_updated

#[test]
fn channel_functional_metadata_updated_succeeds_for_valid_channel_id_when_called_by_valid_custodian_or_configurator_as_signed_origin_with_valid_functional_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: BoundedVec::<u64, ConstU32<64>>::default(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info.clone()));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		let channel_functional_metadata_bounded: BoundedVec<u8, ConstU32<64>> = channel_functional_metadata.clone().try_into().unwrap();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_functional_metadata.clone(),
			new_channel_id.clone()
		));
		// SCRIBE_1 updates the channel functional metadata.
		assert_ok!(Metarium::channel_functional_metadata_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			channel_functional_metadata.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelMetadataUpdated(new_channel_id.clone(), channel_metadata.clone())));
		// Assert that the channel info is updated.
		let updated_actant_set: Vec<u64> = vec![ACTANT];
		let updated_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_actant_set.clone().try_into().unwrap();
		let updated_channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			functional_metadata: Some(channel_functional_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info));
		// ACTANT adds another metadata as a kuri to the channel.
		let channel_functional_metadata_2: Vec<u8> = TEST_KURI_2.to_string().into();
		let channel_functional_metadata_2_bounded: BoundedVec<u8, ConstU32<64>> = channel_functional_metadata_2.clone().try_into().unwrap();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_functional_metadata_2.clone(),
			new_channel_id.clone()
		));
		// CONFIGURATOR updates the channel functional metadata.
		assert_ok!(Metarium::channel_functional_metadata_updated(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			channel_functional_metadata_2.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelMetadataUpdated(new_channel_id.clone(), channel_metadata.clone())));
		// Assert that the channel info is updated.
		let updated_channel_info_2: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			functional_metadata: Some(channel_functional_metadata_2_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(updated_channel_info_2));
	});
}

#[test]
fn channel_functional_metadata_updated_fails_for_valid_channel_id_when_called_by_valid_custodian_or_configurator_as_signed_origin_with_invalid_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let mut invalid_functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_functional_metadata.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata with empty metadata.
		invalid_functional_metadata.clear();
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_functional_metadata.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		invalid_functional_metadata = "a".repeat(129).into();
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				invalid_functional_metadata.clone()
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
	});
}

#[test]
fn channel_functional_metadata_updated_fails_for_invalid_channel_id_when_called_by_valid_custodian_or_configurator_as_signed_origin_with_valid_functional_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata of an invalid channel.
		let invalid_functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_functional_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				invalid_channel_id.clone(),
				invalid_functional_metadata.clone()
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_functional_metadata_updated_fails_for_valid_channel_id_when_called_by_invalid_custodian_or_configurator_as_signed_origin_with_valid_functional_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to update the functional metadata.
		let functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_functional_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				functional_metadata.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Root removes CONFIGURATOR from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to update the functional metadata after removal.
		assert_noop!(
			Metarium::channel_functional_metadata_updated(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				functional_metadata.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Root removes SCRIBE_1 from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata after removal.
		assert_noop!(
			Metarium::channel_functional_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				functional_metadata.clone()
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_functional_metadata_updated_fails_for_valid_channel_id_when_called_by_valid_custodian_or_configurator_as_unsigned_origin_with_valid_functional_metadata() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the functional metadata as unsigned origin.
		let functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::channel_functional_metadata_updated(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				functional_metadata.clone()
			),
			BadOrigin
		);
	});
}

// channel_custodian_metadata_commit_thread_lock_requested

#[test]
fn channel_custodian_metadata_commit_thread_lock_requested_succeeds_for_valid_channel_id_succeeds_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), None);
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataCommitThreadLockRequested(new_channel_id.clone(), ACTANT)));
		// Assert that the channel custodian metadata commit thread lock is set.
		let metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock for the same channel again.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadAlreadyLocked
		);
		// CONFIGURATOR adds itself to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to request a thread lock for the channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadAlreadyLocked
		);	
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_requested_fails_for_invalid_channel_id_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock for an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(ACTANT),
				invalid_channel_id.clone()
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_requested_fails_for_valid_channel_id_when_called_by_invalid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when LISTENER tries to request a thread lock for a channel.
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to request a thread lock for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when CONFIGURATOR tries to request a thread lock for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when Root tries to request a thread lock.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::root(),
				new_channel_id.clone()
			),
			BadOrigin
		);
		// Root removes ACTANT from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock after removal.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_requested_fails_for_valid_channel_id_when_called_by_valid_actant_as_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock for a channel as unsigned origin.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::none(),
				new_channel_id.clone()
			),
			BadOrigin
		);
	});
}

// channel_custodian_metadata_commit_thread_lock_release_requested

#[test]
fn channel_custodian_metadata_commit_thread_lock_release_requested_for_valid_channel_id_with_valid_locked_thread_succeeds_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), None);
		let metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// ACTANT requests a thread lock release for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataCommitThreadLockReleased(new_channel_id.clone(), ACTANT)));
		// Assert that the channel custodian metadata commit thread lock is released.
		let metadata_commit_thread_released: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread_released));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for the same channel again.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_release_requested_fails_for_valid_channel_id_with_invalid_locked_thread_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), None);
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for the channel's custodian metadata without a thread ever having been locked.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotFound
		);
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		let mut metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to request a thread lock release for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// ACTANT requests a thread lock release for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for the same channel again.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// LISTENER requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(LISTENER),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(LISTENER),
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for a channel locked by LISTENER.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// LISTENER requests a thread lock release for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
			RuntimeOrigin::signed(LISTENER),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
	});	
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_release_requested_fails_for_invalid_channel_id_with_valid_locked_thread_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				invalid_channel_id.clone()
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_release_requested_fails_for_valid_channel_id_with_valid_locked_thread_when_called_by_invalid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when NON_SCRIBE tries to request a thread lock release for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(NON_SCRIBE),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to request a thread lock release for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// CONFIGURATOR adds LISTENER to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to request a thread lock release for a channel.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// Root removes ACTANT from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for a channel after removal.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn channel_custodian_metadata_commit_thread_lock_release_requested_fails_for_valid_channel_id_with_valid_locked_thread_when_called_by_valid_actant_as_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to request a thread lock release for a channel as unsigned origin.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
				RuntimeOrigin::none(),
				new_channel_id.clone()
			),
			BadOrigin
		);
	});
}

// force_release_channel_custodian_metadata_commit_thread_lock

#[test]
fn force_release_channel_custodian_metadata_commit_thread_lock_for_valid_channel_id_with_valid_locked_thread_succeeds_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), None);
		let mut metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Root force releases the channel custodian metadata commit thread lock.
		assert_ok!(Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
			RuntimeOrigin::root(),
			new_channel_id.clone(),
			ACTANT
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataCommitThreadLockReleased(new_channel_id.clone(), ACTANT)));
		// Assert that the channel custodian metadata commit thread lock is released.
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// ACTANT requests a thread lock for the channel custodian metadata again.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// CONFIGURATOR removes ACTANT from the channel actant set.
		assert_ok!(Metarium::node_removed_from_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// Root force releases the channel custodian metadata commit thread lock.
		assert_ok!(Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
			RuntimeOrigin::root(),
			new_channel_id.clone(),
			ACTANT
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// CONFIGURATOR adds ACTANT back to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata again.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Root removes ACTANT from the scribe set.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Root force releases the channel custodian metadata commit thread lock.
		assert_ok!(Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
			RuntimeOrigin::root(),
			new_channel_id.clone(),
			ACTANT
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			..Default::default()
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
	});
}

#[test]
fn force_release_channel_custodian_metadata_commit_thread_lock_fails_for_invalid_channel_id_with_valid_locked_thread_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when Root tries to force release the channel custodian metadata commit thread lock for an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::root(),
				invalid_channel_id.clone(),
				ACTANT
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn force_release_channel_custodian_metadata_commit_thread_lock_fails_for_valid_channel_id_with_invalid_locked_thread_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when Root tries to force release the channel custodian metadata commit thread lock for a channel locked by a different actant.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::root(),
				new_channel_id.clone(),
				NON_SCRIBE
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::root(),
				new_channel_id.clone(),
				SCRIBE_1
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::root(),
				new_channel_id.clone(),
				CONFIGURATOR
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// CONFIGURATOR adds LISTENER to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when Root tries to force release the channel custodian metadata commit thread lock for a channel locked by a different actant.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::root(),
				new_channel_id.clone(),
				LISTENER
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
	});
}

#[test]
fn force_release_channel_custodian_metadata_commit_thread_lock_fails_for_valid_channel_id_with_valid_locked_thread_when_called_by_non_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to force release the channel custodian metadata commit thread lock.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when CONFIGURATOR tries to force release the channel custodian metadata commit thread lock.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to force release the channel custodian metadata commit thread lock.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when NON_SCRIBE tries to force release the channel custodian metadata commit thread lock.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::signed(NON_SCRIBE),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when unsigned origin tries to force release the channel custodian metadata commit thread lock.
		assert_noop!(
			Metarium::force_release_channel_custodian_metadata_commit_thread_lock(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				ACTANT
			),
			BadOrigin
		);
	});
}

// channel_custodian_metadata_updated

#[test]
fn channel_custodian_metadata_updated_succeeds_for_valid_channel_id_with_valid_metadata_and_valid_params_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		let mut updated_actant_set: Vec<u64> = vec![ACTANT];
		let mut updated_actant_set_bounded: BoundedVec<u64, ConstU32<64>> = updated_actant_set.clone().try_into().unwrap();
		let mut channel_info: ChannelInfo<u64, ChannelActants::<Test>, ChannelListeners::<Test>, Kuri::<Test>, u64, CustodianMetadataHistory::<Test>> = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			..Default::default()
		};
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info.clone()));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_custodian_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		let channel_custodian_metadata_bounded: BoundedVec<u8, ConstU32<64>> = channel_custodian_metadata.clone().try_into().unwrap();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_custodian_metadata.clone(),
			new_channel_id.clone()
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), None);
		let mut metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<u64, Kuri::<Test>, <Test as frame_system::Config>::Hash, u64> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			..Default::default()
		};
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// ACTANT updates the channel custodian metadata without releasing the lock
		let commit_kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_kuri_bounded: BoundedVec<u8, ConstU32<64>> = commit_kuri.clone().try_into().unwrap();
		let transaction_hash: H256 = H256::from_slice(&[0; 32]);
		let commit_transaction_hash: <Test as frame_system::Config>::Hash = transaction_hash.try_into().unwrap();
		let commit_block_number = 2;
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		assert_ok!(Metarium::channel_custodian_metadata_updated(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone(),
			previous_custodian_metadata.clone(),
			channel_custodian_metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone(),
			commit_block_number.clone(),
			commit_size.clone(),
			false
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataUpdated(new_channel_id.clone(), channel_custodian_metadata.clone())));
		// Assert that the channel custodian metadata is updated.
		channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			custodian_metadata: Some(channel_custodian_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(ACTANT),
			scribe: Some(ACTANT),
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// ACTANT updates the channel custodian metadata to the same value with releasing the lock
		assert_ok!(Metarium::channel_custodian_metadata_updated(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone(),
			channel_custodian_metadata.clone(),
			channel_custodian_metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone(),
			commit_block_number.clone(),
			commit_size.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataUpdated(new_channel_id.clone(), channel_custodian_metadata.clone())));
		// Assert that the channel custodian metadata is updated.
		channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			custodian_metadata: Some(channel_custodian_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			scribe: Some(ACTANT),
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// Ensure the expected error is thrown when SCRIBE_1 tries to request a thread lock for the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// CONFIGURATOR adds SCRIBE_1 to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			SCRIBE_1
		));
		// SCRIBE_1 requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone()
		));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(SCRIBE_1),
			scribe: Some(ACTANT),
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// SCRIBE_1 updates the channel custodian metadata to the same value without releasing the lock
		assert_ok!(Metarium::channel_custodian_metadata_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			channel_custodian_metadata.clone(),
			channel_custodian_metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone(),
			commit_block_number.clone(),
			commit_size.clone(),
			false
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataUpdated(new_channel_id.clone(), channel_custodian_metadata.clone())));
		// Assert that the channel custodian metadata is updated.
		updated_actant_set.push(SCRIBE_1);
		updated_actant_set_bounded = updated_actant_set.clone().try_into().unwrap();
		channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			custodian_metadata: Some(channel_custodian_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: Some(SCRIBE_1),
			scribe: Some(SCRIBE_1),
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
		// SCRIBE_1 updates the channel custodian metadata to the same value with releasing the lock
		assert_ok!(Metarium::channel_custodian_metadata_updated(
			RuntimeOrigin::signed(SCRIBE_1),
			new_channel_id.clone(),
			channel_custodian_metadata.clone(),
			channel_custodian_metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone(),
			commit_block_number.clone(),
			commit_size.clone(),
			true
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ChannelCustodianMetadataUpdated(new_channel_id.clone(), channel_custodian_metadata.clone())));
		// Assert that the channel custodian metadata is updated.
		channel_info = ChannelInfo {
			block_number: 0,
			id: new_channel_id.clone(),
			custodian: SCRIBE_1,
			configurator: CONFIGURATOR,
			maker: SCRIBE_1,
			actants: updated_actant_set_bounded.clone(),
			listeners: BoundedVec::<u64, ConstU32<64>>::default(),
			historical_custodian_metadata: BoundedVec::<CustodianMetadataEntry<u64, u64, BoundedVec<u8, ConstU32<64>>>, ConstU32<64>>::default(),
			custodian_metadata: Some(channel_custodian_metadata_bounded.clone()),
			..Default::default()
		};
		assert_eq!(Channels::<Test>::get(new_channel_id), Some(channel_info));
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: new_channel_id.clone(),
			locked_by: None,
			scribe: Some(SCRIBE_1),
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
		};
		assert_eq!(ChannelCustodianMetadataCommitThreads::<Test>::get(new_channel_id), Some(metadata_commit_thread));
	});
}

#[test]
fn channel_custodian_metadata_updated_fails_for_invalid_channel_id_with_valid_metadata_and_valid_params_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_custodian_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_custodian_metadata.clone(),
			new_channel_id.clone()
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata for an invalid channel.
		let invalid_channel_id: u64 = 2;
		let commit_kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_transaction_hash: H256 = H256::from_slice(&[0; 32]);
		let commit_block_number = 2;
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				invalid_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelNotFound
		);
	});
}

#[test]
fn channel_custodian_metadata_updated_fails_for_valid_channel_id_with_invalid_metadata_and_valid_params_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let mut invalid_functional_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_transaction_hash: H256 = H256::from_slice(&[0; 32]);
		let commit_block_number = 2;
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				invalid_functional_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ArikuriNotFound
		);
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata with invalid metadata.
		invalid_functional_metadata.clear();
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				invalid_functional_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ArikuriNotFound
		);
		invalid_functional_metadata = "a".repeat(129).into();
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata with invalid metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				invalid_functional_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
	});
}

#[test]
fn channel_custodian_metadata_updated_fails_for_valid_channel_id_with_valid_metadata_and_invalid_params_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_custodian_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_custodian_metadata.clone(),
			new_channel_id.clone()
		));
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		let mut commit_kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_transaction_hash: H256 = H256::from_slice(&[0; 32]);
		let commit_block_number = 2;
		let commit_size = 64;
		let mut previous_custodian_metadata: Vec<u8> = Vec::new();
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata for an invalid previous custodian metadata.
		previous_custodian_metadata = "a".repeat(129).into();
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
		previous_custodian_metadata = Vec::new();
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata for an invalid commit_kuri.
		commit_kuri = "a".repeat(129).into();
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
		commit_kuri = TEST_KURI_1.to_string().into();
		// ACTANT updates the channel custodian metadata without releasing the lock
		assert_ok!(Metarium::channel_custodian_metadata_updated(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone(),
			previous_custodian_metadata.clone(),
			channel_custodian_metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone().try_into().unwrap(),
			commit_block_number.clone(),
			commit_size.clone(),
			false
		));
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata for a mismatching previous custodian metadata.
		previous_custodian_metadata = "b".repeat(64).into();
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataMismatch
		);
	});
}

#[test]
fn channel_custodian_metadata_updated_fails_for_valid_channel_id_with_valid_metadata_and_valid_params_when_called_by_invalid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds metadata as a kuri to the channel.
		let channel_custodian_metadata: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			channel_custodian_metadata.clone(),
			new_channel_id.clone()
		));
		let commit_kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let commit_transaction_hash: H256 = H256::from_slice(&[0; 32]);
		let commit_block_number = 2;
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata without a thread lock ever.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotFound
		);
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// ACTANT requests a thread lock release for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_release_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to update the channel custodian metadata without a thread lock.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(ACTANT),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// ACTANT requests a thread lock for the channel custodian metadata.
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(
			RuntimeOrigin::signed(ACTANT),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when NON_SCRIBE tries to update the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(NON_SCRIBE),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel custodian metadata without being added to the channel actant set.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// CONFIGURATOR adds SCRIBE_1 to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			SCRIBE_1
		));
		// Ensure the expected error is thrown when SCRIBE_1 tries to update the channel custodian metadata without thread lock despite being added to the channel actant set.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(SCRIBE_1),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// Ensure the expected error is thrown when CONFIGURATOR tries to update the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(CONFIGURATOR),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::CallForbidden
		);
		// Root adds LISTENER to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to update the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::CallForbidden
		);
		// CONFIGURATOR adds LISTENER to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			LISTENER
		));
		// Ensure the expected error is thrown when LISTENER tries to update the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::signed(LISTENER),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			Error::<Test>::ChannelCustodianMetadataCommitThreadNotLockedByNode
		);
		// Ensure the expected error is thrown when unsigned origin tries to update the channel custodian metadata.
		assert_noop!(
			Metarium::channel_custodian_metadata_updated(
				RuntimeOrigin::none(),
				new_channel_id.clone(),
				previous_custodian_metadata.clone(),
				channel_custodian_metadata.clone(),
				commit_kuri.clone(),
				commit_transaction_hash.clone().try_into().unwrap(),
				commit_block_number.clone(),
				commit_size.clone(),
				false
			),
			BadOrigin
		);
	});
}

// arikuri_added

#[test]
fn arikuri_added_succeeds_for_valid_channel_id_and_valid_kuri_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds a kuri to the channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let kuri_bounded: BoundedVec<u8, ConstU32<64>> = kuri.clone().try_into().unwrap();
		assert_eq!(TotalArikuris::<Test>::get(new_channel_id.clone()), 0);
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), None);
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ArikuriAdded(new_channel_id.clone(), kuri.clone())));
		// Assert that the kuri is added to the channel.
		let new_arikuri: ArikuriInfo<BoundedVec<u8, ConstU32<64>>> = ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: false,
		};
		assert_eq!(TotalArikuris::<Test>::get(new_channel_id.clone()), 1);
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(new_arikuri));
	});
}

#[test]
fn arikuri_added_fails_for_invalid_channel_id_and_valid_kuri_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds a kuri to the channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when ACTANT tries to add a kuri to an invalid channel.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				invalid_channel_id.clone()
			),
			Error::<Test>::ChannelNotFound
		);
		// CONFIGURATOR pauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true));
		// Ensure the expected error is thrown when ACTANT tries to add a kuri to a paused channel.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelAlreadyPaused
		);
		// CONFIGURATOR unpauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), false));
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Ensure the expected error is thrown when ACTANT tries to add a kuri to an archived channel.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelAlreadyArchived
		);
	});
}

#[test]
fn arikuri_added_fails_for_valid_channel_id_and_invalid_kuri_when_called_by_valid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds a kuri to the channel.
		let mut kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when ACTANT tries to add the same kuri to the channel again.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ArikuriAlreadyAdded
		);
		// Ensure the expected error is thrown when ACTANT tries to add an invalid kuri to the channel.
		kuri = "a".repeat(129).into();
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
	});
}

#[test]
fn arikuri_added_fails_for_valid_channel_id_and_valid_kuri_when_called_by_invalid_actant() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Ensure the expected error is thrown when NON_SCRIBE tries to add a kuri to the channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(NON_SCRIBE),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// Ensure the expected error is thrown when ACTANT tries to add a kuri to the channel without being added to the channel actant set.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::NodeDoesNotExistInChannelActantSet
		);
		// Ensure the expected error is thrown when an unsigned origin tries to add a kuri to the channel.
		assert_noop!(
			Metarium::arikuri_added(
				RuntimeOrigin::none(),
				kuri.clone(),
				new_channel_id.clone()
			),
			BadOrigin
		);
	});
}

// arikuri_deleted

#[test]
fn arikuri_deleted_succeeds_for_valid_channel_id_and_valid_kuri_when_called_by_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds a kuri to the channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let kuri_bounded: BoundedVec<u8, ConstU32<64>> = kuri.clone().try_into().unwrap();
		// CONFIGURATOR deletes the kuri from the channel.
		assert_eq!(TotalArikuris::<Test>::get(new_channel_id.clone()), 1);
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: false,
		}));
		assert_ok!(Metarium::arikuri_deleted(
			RuntimeOrigin::signed(CONFIGURATOR),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Assert that the expected event is emitted.
		// assert!(metarium_events().contains(&Event::<Test>::ArikuriDeleted(new_channel_id.clone(), kuri.clone())));
		// Assert that the kuri is deleted from the channel.
		assert_eq!(TotalArikuris::<Test>::get(new_channel_id.clone()), 1);
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: true,
		}));
	});
}

#[test]
fn arikuri_deleted_fails_for_invalid_channel_id_and_valid_kuri_when_called_by_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		// ACTANT adds a kuri to the channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let invalid_channel_id: u64 = 2;
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete a kuri from an invalid channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				invalid_channel_id.clone()
			),
			Error::<Test>::ChannelNotFound
		);
		// CONFIGURATOR pauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), true));
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete a kuri from a paused channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelAlreadyPaused
		);
		// CONFIGURATOR unpauses the channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), new_channel_id.clone(), false));
		// Scribe SCRIBE_1 archives the channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), new_channel_id.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete a kuri from an archived channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ChannelAlreadyArchived
		);
	});
}

#[test]
fn arikuri_deleted_fails_for_valid_channel_id_and_invalid_kuri_when_called_by_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete an invalid kuri from the channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let invalid_kuri: Vec<u8> = "a".repeat(129).into();
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete an invalid kuri from the channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				invalid_kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
		// SCRIBE_1 adds a second channel with CONFIGURATOR as the configurator.
		let new_channel_id_2: u64 = 2;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id_2.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete a kuri from the wrong channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				new_channel_id_2.clone()
			),
			Error::<Test>::ArikuriNotFound
		);
		// CONFIGURATOR deletes the kuri from the first channel.
		assert_ok!(Metarium::arikuri_deleted(
			RuntimeOrigin::signed(CONFIGURATOR),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when CONFIGURATOR tries to delete the same kuri from the first channel again.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(CONFIGURATOR),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::ArikuriAlreadyDeleted
		);
	});
}

#[test]
fn arikuri_deleted_fails_for_valid_channel_id_and_valid_kuri_when_called_by_invalid_caller() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when NON_SCRIBE tries to delete a kuri from the channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(NON_SCRIBE),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when SCRIBE_1 as the custodian tries to delete a kuri from the channel without being the configurator.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(SCRIBE_1),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when ACTANT tries to delete a kuri from the channel without being the configurator.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone()
			),
			Error::<Test>::CallForbidden
		);
		// Ensure the expected error is thrown when an unsigned origin tries to delete a kuri from the channel.
		assert_noop!(
			Metarium::arikuri_deleted(
				RuntimeOrigin::none(),
				kuri.clone(),
				new_channel_id.clone()
			),
			BadOrigin
		);
	});
}


// arikuri_transfers_accepted

#[test]
fn arikuri_transfers_accepted_succeeds_for_valid_from_and_to_channel_ids_and_valid_kuris_when_called_by_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_2
		));
		// Root adds SCRIBE_2 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_2
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Root adds CONFIGURATOR_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR_2
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let from_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// SCRIBE_2 adds a channel with CONFIGURATOR_2 as the configurator.
		let to_channel_id: u64 = 2;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_2),
			CONFIGURATOR_2
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set for the from channel.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			from_channel_id.clone(),
			ACTANT
		));
		// Root adds ACTANT_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT_2
		));
		// CONFIGURATOR_2 adds ACTANT_2 to the channel actant set for the to channel.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR_2),
			to_channel_id.clone(),
			ACTANT_2
		));
		// ACTANT adds a kuri to the from channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			from_channel_id.clone()
		));
		// ACTANT adds a kuri_2 to the from channel.
		let kuri_2: Vec<u8> = TEST_KURI_2.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri_2.clone(),
			from_channel_id.clone()
		));
		// ACTANT adds a kuri_3 to the from channel.
		let kuri_3: Vec<u8> = TEST_KURI_3.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri_3.clone(),
			from_channel_id.clone()
		));
		// ACTANT adds a kuri_4 to the from channel.
		let kuri_4: Vec<u8> = TEST_KURI_4.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri_4.clone(),
			from_channel_id.clone()
		));
		// ACTANT_2 adds a kuri_5 to the to channel.
		let kuri_5: Vec<u8> = TEST_KURI_5.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT_2),
			kuri_5.clone(),
			to_channel_id.clone()
		));
		// ACTANT_2 adds a kuri_6 to the to channel.
		let kuri_6: Vec<u8> = TEST_KURI_6.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT_2),
			kuri_6.clone(),
			to_channel_id.clone()
		));
		// ACTANT_2 adds a kuri_7 to the to channel.
		let kuri_7: Vec<u8> = TEST_KURI_7.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT_2),
			kuri_7.clone(),
			to_channel_id.clone()
		));
		// ACTANT_2 adds a kuri_8 to the to channel.
		let kuri_8: Vec<u8> = TEST_KURI_8.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT_2),
			kuri_8.clone(),
			to_channel_id.clone()
		));
		// Ensure the kuri and channel states are as expected.
		assert_eq!(TotalArikuris::<Test>::get(from_channel_id.clone()), 4);
		assert_eq!(TotalArikuris::<Test>::get(to_channel_id.clone()), 4);
		let kuri_bounded: BoundedVec<u8, ConstU32<64>> = kuri.clone().try_into().unwrap();
		let kuri_2_bounded: BoundedVec<u8, ConstU32<64>> = kuri_2.clone().try_into().unwrap();
		let kuri_3_bounded: BoundedVec<u8, ConstU32<64>> = kuri_3.clone().try_into().unwrap();
		let kuri_4_bounded: BoundedVec<u8, ConstU32<64>> = kuri_4.clone().try_into().unwrap();
		assert_eq!(Arikuris::<Test>::get(from_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri.clone().try_into().unwrap(),
			channel_id: from_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(from_channel_id.clone(), kuri_2_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_2.clone().try_into().unwrap(),
			channel_id: from_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(from_channel_id.clone(), kuri_3_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_3.clone().try_into().unwrap(),
			channel_id: from_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(from_channel_id.clone(), kuri_4_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_4.clone().try_into().unwrap(),
			channel_id: from_channel_id.clone(),
			deleted: false,
		}));
		let kuri_5_bounded: BoundedVec<u8, ConstU32<64>> = kuri_5.clone().try_into().unwrap();
		let kuri_6_bounded: BoundedVec<u8, ConstU32<64>> = kuri_6.clone().try_into().unwrap();
		let kuri_7_bounded: BoundedVec<u8, ConstU32<64>> = kuri_7.clone().try_into().unwrap();
		let kuri_8_bounded: BoundedVec<u8, ConstU32<64>> = kuri_8.clone().try_into().unwrap();
		assert_eq!(Arikuris::<Test>::get(to_channel_id.clone(), kuri_5_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_5.clone().try_into().unwrap(),
			channel_id: to_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(to_channel_id.clone(), kuri_6_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_6.clone().try_into().unwrap(),
			channel_id: to_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(to_channel_id.clone(), kuri_7_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_7.clone().try_into().unwrap(),
			channel_id: to_channel_id.clone(),
			deleted: false,
		}));
		assert_eq!(Arikuris::<Test>::get(to_channel_id.clone(), kuri_8_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_8.clone().try_into().unwrap(),
			channel_id: to_channel_id.clone(),
			deleted: false,
		}));
		// Ensure the transfer of kuri, kuri_2, kuri_3, and kuri_4 from the from channel to the to channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_2_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_3_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_4_bounded.clone()), false);
		// Ensure the transfer of kuri, kuri_2, kuri_3, and kuri_4 from the to channel to the from channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_2_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_3_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_4_bounded.clone()), false);
		// Ensure the transfer of kuri_5, kuri_6, kuri_7, and kuri_8 from the from channel to the to channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_5_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_6_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_7_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_8_bounded.clone()), false);
		// Ensure the transfer of kuri_5, kuri_6, kuri_7, and kuri_8 from the to channel to the from channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_5_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_6_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_7_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_8_bounded.clone()), false);
		// CONFIGURATOR_2 accepts the transfer of kuri, kuri_2, kuri_3, and kuri_4 from the from channel to the to channel.
		let bounded_from_kuris: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<64>> = vec![kuri_bounded.clone(), kuri_2_bounded.clone(), kuri_3_bounded.clone(), kuri_4_bounded.clone()].try_into().unwrap();
		assert_ok!(Metarium::arikuri_transfers_accepted(
			RuntimeOrigin::signed(CONFIGURATOR_2),
			bounded_from_kuris.clone(),
			from_channel_id.clone(),
			to_channel_id.clone()
		));
		// Ensure the transfer of kuri, kuri_2, kuri_3, and kuri_4 from the from channel to the to channel is accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_bounded.clone()), true);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_2_bounded.clone()), true);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_3_bounded.clone()), true);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_4_bounded.clone()), true);
		// Ensure the transfer of kuri, kuri_2, kuri_3, and kuri_4 from the to channel to the from channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_2_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_3_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_4_bounded.clone()), false);
		// Ensure the transfer of kuri_5, kuri_6, kuri_7, and kuri_8 from the from channel to the to channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_5_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_6_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_7_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(to_channel_id.clone(), kuri_8_bounded.clone()), false);
		// Ensure the transfer of kuri_5, kuri_6, kuri_7, and kuri_8 from the to channel to the from channel is not yet accepted.
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_5_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_6_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_7_bounded.clone()), false);
		assert_eq!(ArikuriTransferAccepted::<Test>::get(from_channel_id.clone(), kuri_8_bounded.clone()), false);
	});
}

#[test]
fn arikuri_transfers_accepted_fails_for_invalid_channel_ids_and_valid_kuris_when_called_by_valid_configurator() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_2
		));
		// Root adds SCRIBE_2 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_2
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// Root adds CONFIGURATOR_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR_2
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let from_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// SCRIBE_2 adds a channel with CONFIGURATOR_2 as the configurator.
		let to_channel_id: u64 = 2;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_2),
			CONFIGURATOR_2
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set for the from channel.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			from_channel_id.clone(),
			ACTANT
		));
		// Root adds ACTANT_2 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT_2
		));
		// CONFIGURATOR_2 adds ACTANT_2 to the channel actant set for the to channel.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR_2),
			to_channel_id.clone(),
			ACTANT_2
		));
		// ACTANT adds a kuri to the from channel.
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			from_channel_id.clone()
		));
		// ACTANT_2 adds a kuri to the to channel.
		let kuri_2: Vec<u8> = TEST_KURI_2.to_string().into();
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT_2),
			kuri_2.clone(),
			to_channel_id.clone()
		));
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from an invalid from channel to the to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				3,
				to_channel_id.clone()
			),
			Error::<Test>::ChannelNotFound
		);
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the from channel to an invalid to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				from_channel_id.clone(),
				4
			),
			Error::<Test>::ChannelNotFound
		);
		// SCRIBE_1 archives the from channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_1), from_channel_id.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the archived from channel to the to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				from_channel_id.clone(),
				to_channel_id.clone()
			),
			Error::<Test>::FromChannelAlreadyArchived
		);
		// Root unarchives the channel.
		assert_ok!(Metarium::force_unarchive_channel(RuntimeOrigin::root(), from_channel_id.clone()));
		// CONFIGURATOR pauses the from channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), from_channel_id.clone(), true));
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the paused from channel to the to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				from_channel_id.clone(),
				to_channel_id.clone()
			),
			Error::<Test>::FromChannelAlreadyPaused
		);
		// CONFIGURATOR unpauses the from channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR), from_channel_id.clone(), false));
		// SCRIBE_2 archives the to channel.
		assert_ok!(Metarium::channel_archived(RuntimeOrigin::signed(SCRIBE_2), to_channel_id.clone()));
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the from channel to the archived to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				from_channel_id.clone(),
				to_channel_id.clone()
			),
			Error::<Test>::ToChannelAlreadyArchived
		);
		// Root unarchives the channel.
		assert_ok!(Metarium::force_unarchive_channel(RuntimeOrigin::root(), to_channel_id.clone()));
		// CONFIGURATOR_2 pauses the to channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR_2), to_channel_id.clone(), true));
		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the from channel to the paused to channel.
		assert_noop!(
			Metarium::arikuri_transfers_accepted(
				RuntimeOrigin::signed(CONFIGURATOR_2),
				vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
				from_channel_id.clone(),
				to_channel_id.clone()
			),
			Error::<Test>::ToChannelAlreadyPaused
		);
		// CONFIGURATOR_2 unpauses the to channel.
		assert_ok!(Metarium::channel_pause_toggled(RuntimeOrigin::signed(CONFIGURATOR_2), to_channel_id.clone(), false));
		// // Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri from the from channel to the to channel.
		// assert_noop!(
		// 	Metarium::arikuri_transfers_accepted(
		// 		RuntimeOrigin::signed(CONFIGURATOR_2),
		// 		vec![kuri.clone().try_into().unwrap()].try_into().unwrap(),
		// 		from_channel_id.clone(),
		// 		to_channel_id.clone()
		// 	),
		// 	Error::<Test>::CallForbidden
		// );
		// // Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri_2 from the from channel to the to channel.
		// assert_noop!(
		// 	Metarium::arikuri_transfers_accepted(
		// 		RuntimeOrigin::signed(CONFIGURATOR_2),
		// 		vec![kuri_2.clone().try_into().unwrap()].try_into().unwrap(),
		// 		from_channel_id.clone(),
		// 		to_channel_id.clone()
		// 	),
		// 	Error::<Test>::CallForbidden
		// );
	});
}

// #[test]
// fn arikuri_transfers_accepted_fails_for_valid_from_and_to_channel_ids_and_invalid_kuris_when_called_by_valid_configurator() {
// 	new_test_ext().execute_with(|| {
// 		// Root adds SCRIBE_1 to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			SCRIBE_1
// 		));
// 		// Root adds SCRIBE_1 to the custodian set.
// 		assert_ok!(Metarium::force_add_node_to_custodian_set(
// 			RuntimeOrigin::root(),
// 			SCRIBE_1
// 		));
// 		// Root adds SCRIBE_2 to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			SCRIBE_2
// 		));
// 		// Root adds SCRIBE_2 to the custodian set.
// 		assert_ok!(Metarium::force_add_node_to_custodian_set(
// 			RuntimeOrigin::root(),
// 			SCRIBE_2
// 		));
// 		// Root adds CONFIGURATOR to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			CONFIGURATOR
// 		));
// 		// Root adds CONFIGURATOR_2 to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			CONFIGURATOR_2
// 		));
// 		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
// 		let from_channel_id: u64 = 1;
// 		assert_ok!(Metarium::channel_added(
// 			RuntimeOrigin::signed(SCRIBE_1),
// 			CONFIGURATOR
// 		));
// 		// SCRIBE_2 adds a channel with CONFIGURATOR_2 as the configurator.
// 		let to_channel_id: u64 = 2;
// 		assert_ok!(Metarium::channel_added(
// 			RuntimeOrigin::signed(SCRIBE_2),
// 			CONFIGURATOR_2
// 		));
// 		// Root adds ACTANT to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			ACTANT
// 		));
// 		// CONFIGURATOR adds ACTANT to the channel actant set for the from channel.
// 		assert_ok!(Metarium::node_added_to_channel_actant_set(
// 			RuntimeOrigin::signed(CONFIGURATOR),
// 			from_channel_id.clone(),
// 			ACTANT
// 		));
// 		// Root adds ACTANT_2 to the scribe set.
// 		assert_ok!(Metarium::force_add_node_to_scribe_set(
// 			RuntimeOrigin::root(),
// 			ACTANT_2
// 		));
// 		// CONFIGURATOR_2 adds ACTANT_2 to the channel actant set for the to channel.
// 		assert_ok!(Metarium::node_added_to_channel_actant_set(
// 			RuntimeOrigin::signed(CONFIGURATOR_2),
// 			to_channel_id.clone(),
// 			ACTANT_2
// 		));
// 		// ACTANT adds a kuri to the from channel.
// 		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
// 		assert_ok!(Metarium::arikuri_added(
// 			RuntimeOrigin::signed(ACTANT),
// 			kuri.clone(),
// 			from_channel_id.clone()
// 		));
// 		// ACTANT_2 adds a kuri to the to channel.
// 		let kuri_2: Vec<u8> = TEST_KURI_2.to_string().into();
// 		assert_ok!(Metarium::arikuri_added(
// 			RuntimeOrigin::signed(ACTANT_2),
// 			kuri_2.clone(),
// 			to_channel_id.clone()
// 		));
// 		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri and kuri_2 from the to channel to the from channel.
// 		assert_noop!(
// 			Metarium::arikuri_transfers_accepted(
// 				RuntimeOrigin::signed(CONFIGURATOR_2),
// 				vec![kuri.clone().try_into().unwrap(), kuri_2.clone().try_into().unwrap()].try_into().unwrap(),
// 				from_channel_id.clone(),
// 				to_channel_id.clone()
// 			),
// 			Error::<Test>::ArikuriNotFound
// 		);
// 		// // Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of an invalid kuri from the from channel to the to channel.
// 		// let invalid_kuri: Vec<u8> = "a".repeat(128).into();
// 		// let bounded_invalid_kuri: BoundedVec<u8, ConstU32<64>> = invalid_kuri.clone().try_into().unwrap();
// 		// assert_noop!(
// 		// 	Metarium::arikuri_transfers_accepted(
// 		// 		RuntimeOrigin::signed(CONFIGURATOR_2),
// 		// 		vec![bounded_invalid_kuri.clone()].try_into().unwrap(),
// 		// 		from_channel_id.clone(),
// 		// 		to_channel_id.clone()
// 		// 	),
// 		// 	Error::<Test>::MaxKuriLengthExceeded
// 		// );
// 		// ACTANT adds a 65 more kuris to the to channel.
// 		for i in 3..68 {
// 			let kuri_i: Vec<u8> = format!("test_kuri_{}", i).into();
// 			assert_ok!(Metarium::arikuri_added(
// 				RuntimeOrigin::signed(ACTANT),
// 				kuri_i.clone(),
// 				from_channel_id.clone()
// 			));
// 		}
// 		// Ensure the expected error is thrown when CONFIGURATOR_2 tries to accept the transfer of kuri_3, ..., kuri_67 from the from channel to the to channel.
// 		let mut kuris_3_to_67: Vec<BoundedVec<u8, ConstU32<64>>> = vec![];
// 		for i in 3..68 {
// 			let kuri_i: Vec<u8> = format!("test_kuri_{}", i).into();
// 			let bounded_kuri_i: BoundedVec<u8, ConstU32<64>> = kuri_i.clone().try_into().unwrap();
// 			kuris_3_to_67.push(bounded_kuri_i.clone());
// 		}
// 		let kuris_3_to_67_bounded: BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<64>> = kuris_3_to_67.clone().try_into().unwrap();
// 		assert_noop!(
// 			Metarium::arikuri_transfers_accepted(
// 				RuntimeOrigin::signed(CONFIGURATOR_2),
// 				kuris_3_to_67_bounded.clone(),
// 				from_channel_id.clone(),
// 				to_channel_id.clone()
// 			),
// 			Error::<Test>::MaxTransferableArikurisLimitExceeded
// 		);
// 	});
// }

// force_update_arikuri

#[test]
fn force_update_arikuri_succeeds_for_valid_channel_id_and_valid_kuri_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let kuri_bounded: BoundedVec<u8, ConstU32<64>> = kuri.clone().try_into().unwrap();
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: false,
		}));
		// Root updates the kuri in the channel.
		assert_ok!(Metarium::force_update_arikuri(
			RuntimeOrigin::root(),
			kuri_bounded.to_vec(),
			new_channel_id.clone(),
			true
		));
		// Ensure the kuri is updated in the channel.
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: true,
		}));
		// Root updates the kuri in the channel.
		assert_ok!(Metarium::force_update_arikuri(
			RuntimeOrigin::root(),
			kuri_bounded.to_vec(),
			new_channel_id.clone(),
			false
		));
		// Ensure the kuri is updated in the channel.
		assert_eq!(Arikuris::<Test>::get(new_channel_id.clone(), kuri_bounded.clone()), Some(ArikuriInfo{
			kuri: kuri_bounded.clone(),
			channel_id: new_channel_id.clone(),
			deleted: false,
		}));
	});
}

#[test]
fn force_update_arikuri_fails_for_invalid_channel_id_and_valid_kuri_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let kuri_bounded: BoundedVec<u8, ConstU32<64>> = kuri.clone().try_into().unwrap();
		// Ensure the expected error is thrown when root tries to update a kuri in an invalid channel.
		let invalid_channel_id: u64 = 2;
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::root(),
				kuri_bounded.to_vec(),
				invalid_channel_id.clone(),
				true
			),
			Error::<Test>::ArikuriNotFound
		);
	});
}

#[test]
fn force_update_arikuri_fails_for_valid_channel_id_and_invalid_kuri_when_called_by_root() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when root tries to update an invalid kuri in the channel.
		let invalid_kuri: Vec<u8> = "a".repeat(129).into();
		let new_channel_id: u64 = 1;
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::root(),
				invalid_kuri.clone(),
				new_channel_id.clone(),
				true
			),
			Error::<Test>::MaxKuriLengthExceeded
		);
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		let kuri_2: Vec<u8> = TEST_KURI_2.to_string().into();
		// Ensure the expected error is thrown when root tries to update an invalid kuri in the channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::root(),
				kuri_2.clone(),
				new_channel_id.clone(),
				true
			),
			Error::<Test>::ArikuriNotFound
		);
		// SCRIBE_1 adds a second channel with CONFIGURATOR as the configurator.
		let new_channel_id_2: u64 = 2;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id_2.clone(),
			ACTANT
		));
		// Ensure the expected error is thrown when root tries to update a kuri in the wrong channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::root(),
				kuri.clone(),
				new_channel_id_2.clone(),
				true
			),
			Error::<Test>::ArikuriNotFound
		);
	});
}

#[test]
fn force_update_arikuri_fails_for_valid_channel_id_and_valid_kuri_when_called_by_invalid_caller() {
	new_test_ext().execute_with(|| {
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		let new_channel_id: u64 = 1;
		// Ensure the expected error is thrown when NON_SCRIBE tries to update a kuri in the channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(NON_SCRIBE),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when SCRIBE_1 as the custodian tries to update a kuri in the channel without being the configurator.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(SCRIBE_1),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when ACTANT tries to update a kuri in the channel without being the configurator.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when an unsigned origin tries to update a kuri in the channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::none(),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Root adds SCRIBE_1 to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds SCRIBE_1 to the custodian set.
		assert_ok!(Metarium::force_add_node_to_custodian_set(
			RuntimeOrigin::root(),
			SCRIBE_1
		));
		// Root adds CONFIGURATOR to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			CONFIGURATOR
		));
		// SCRIBE_1 adds a channel with CONFIGURATOR as the configurator.
		let new_channel_id: u64 = 1;
		assert_ok!(Metarium::channel_added(
			RuntimeOrigin::signed(SCRIBE_1),
			CONFIGURATOR
		));
		// Root adds ACTANT to the scribe set.
		assert_ok!(Metarium::force_add_node_to_scribe_set(
			RuntimeOrigin::root(),
			ACTANT
		));
		// CONFIGURATOR adds ACTANT to the channel actant set.
		assert_ok!(Metarium::node_added_to_channel_actant_set(
			RuntimeOrigin::signed(CONFIGURATOR),
			new_channel_id.clone(),
			ACTANT
		));
		let kuri: Vec<u8> = TEST_KURI_1.to_string().into();
		// ACTANT adds a kuri to the channel.
		assert_ok!(Metarium::arikuri_added(
			RuntimeOrigin::signed(ACTANT),
			kuri.clone(),
			new_channel_id.clone()
		));
		// Ensure the expected error is thrown when NON_SCRIBE tries to update a kuri in the channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(NON_SCRIBE),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when SCRIBE_1 as the custodian tries to update a kuri in the channel without being the configurator.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(SCRIBE_1),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when ACTANT tries to update a kuri in the channel without being the configurator.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::signed(ACTANT),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
		// Ensure the expected error is thrown when an unsigned origin tries to update a kuri in the channel.
		assert_noop!(
			Metarium::force_update_arikuri(
				RuntimeOrigin::none(),
				kuri.clone(),
				new_channel_id.clone(),
				true
			),
			BadOrigin
		);
	});
}

/////// SLICE 1: delegated scribe-admin origin ///////

#[test]
fn root_sets_scribe_admin_and_admin_can_manage_scribes() {
	new_test_ext().execute_with(|| {
		assert_ok!(Metarium::set_scribe_admin(RuntimeOrigin::root(), Some(SCRIBE_ADMIN)));
		assert_eq!(Metarium::scribe_admin(), Some(SCRIBE_ADMIN));
		// The admin (not root) can add a scribe.
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::signed(SCRIBE_ADMIN), SCRIBE_1));
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
		// ...and remove it.
		assert_ok!(Metarium::force_remove_node_from_scribe_set(RuntimeOrigin::signed(SCRIBE_ADMIN), SCRIBE_1));
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), false);
		// Root can clear the admin.
		assert_ok!(Metarium::set_scribe_admin(RuntimeOrigin::root(), None));
		assert_eq!(Metarium::scribe_admin(), None);
	});
}

#[test]
fn root_can_still_manage_scribes() {
	new_test_ext().execute_with(|| {
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_eq!(ScribeSetMap::<Test>::get(SCRIBE_1), true);
	});
}

#[test]
fn non_admin_non_root_cannot_manage_scribes() {
	new_test_ext().execute_with(|| {
		// No admin set: a signed non-root is forbidden.
		assert_noop!(
			Metarium::force_add_node_to_scribe_set(RuntimeOrigin::signed(NON_SCRIBE), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
		// With an admin set, a *different* signer is still forbidden.
		assert_ok!(Metarium::set_scribe_admin(RuntimeOrigin::root(), Some(SCRIBE_ADMIN)));
		assert_noop!(
			Metarium::force_add_node_to_scribe_set(RuntimeOrigin::signed(NON_SCRIBE), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn only_root_sets_scribe_admin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Metarium::set_scribe_admin(RuntimeOrigin::signed(SCRIBE_1), Some(SCRIBE_ADMIN)),
			BadOrigin
		);
	});
}


/////// SLICE 2: delegated custodian-admin origin ///////

#[test]
fn root_sets_custodian_admin_and_admin_can_manage_custodians() {
	new_test_ext().execute_with(|| {
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_ok!(Metarium::set_custodian_admin(RuntimeOrigin::root(), Some(CUSTODIAN_ADMIN)));
		assert_eq!(Metarium::custodian_admin(), Some(CUSTODIAN_ADMIN));
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::signed(CUSTODIAN_ADMIN), SCRIBE_1));
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), true);
		assert_ok!(Metarium::force_remove_node_from_custodian_set(RuntimeOrigin::signed(CUSTODIAN_ADMIN), SCRIBE_1));
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), false);
	});
}

#[test]
fn root_can_still_manage_custodians() {
	new_test_ext().execute_with(|| {
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_eq!(CustodianSetMap::<Test>::get(SCRIBE_1), true);
	});
}

#[test]
fn non_admin_non_root_cannot_manage_custodians() {
	new_test_ext().execute_with(|| {
		assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_noop!(
			Metarium::force_add_node_to_custodian_set(RuntimeOrigin::signed(NON_SCRIBE), SCRIBE_1),
			Error::<Test>::CallForbidden
		);
	});
}

#[test]
fn only_root_sets_custodian_admin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Metarium::set_custodian_admin(RuntimeOrigin::signed(SCRIBE_1), Some(CUSTODIAN_ADMIN)),
			BadOrigin
		);
	});
}


/////// SLICE 3: commit-lock expiry (stale takeover) ///////

#[test]
fn stale_commit_lock_can_be_taken_over_after_ttl() {
	new_test_ext().execute_with(|| {
		for s in [SCRIBE_1, CONFIGURATOR, ACTANT, ACTANT_2] {
			assert_ok!(Metarium::force_add_node_to_scribe_set(RuntimeOrigin::root(), s));
		}
		assert_ok!(Metarium::force_add_node_to_custodian_set(RuntimeOrigin::root(), SCRIBE_1));
		assert_ok!(Metarium::channel_added(RuntimeOrigin::signed(SCRIBE_1), CONFIGURATOR));
		assert_ok!(Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT));
		assert_ok!(Metarium::node_added_to_channel_actant_set(RuntimeOrigin::signed(CONFIGURATOR), 1, ACTANT_2));
		// ACTANT acquires the lock at block 1.
		System::set_block_number(1);
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(RuntimeOrigin::signed(ACTANT), 1));
		// Within TTL (10), ACTANT_2 cannot take over.
		System::set_block_number(5);
		assert_noop!(
			Metarium::channel_custodian_metadata_commit_thread_lock_requested(RuntimeOrigin::signed(ACTANT_2), 1),
			Error::<Test>::ChannelCustodianMetadataCommitThreadAlreadyLocked
		);
		// Past TTL, the stale lock can be taken over.
		System::set_block_number(20);
		assert_ok!(Metarium::channel_custodian_metadata_commit_thread_lock_requested(RuntimeOrigin::signed(ACTANT_2), 1));
		assert_eq!(
			ChannelCustodianMetadataCommitThreads::<Test>::get(1).unwrap().locked_by,
			Some(ACTANT_2)
		);
	});
}
