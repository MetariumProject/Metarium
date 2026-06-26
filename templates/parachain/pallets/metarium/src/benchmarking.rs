//! Benchmarking setup for pallet-metarium
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Metarium;
use frame_benchmarking::{
	__private::{
		vec,
		Vec,
		storage::bounded_vec::BoundedVec,
	},
	v2::*
};
use frame_system::{
	pallet_prelude::BlockNumberFor,
	RawOrigin
};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn force_add_node_to_scribe_set() {
		let scribe: T::AccountId = whitelisted_caller();
		assert_eq!(ScribeSetMap::<T>::get(scribe.clone()), false);
		#[extrinsic_call]
		force_add_node_to_scribe_set(RawOrigin::Root, scribe.clone());
		assert_eq!(ScribeSetMap::<T>::get(scribe), true);
	}

	#[benchmark]
	fn force_remove_node_from_scribe_set() {
		let scribe: T::AccountId = whitelisted_caller();
		assert_eq!(ScribeSetMap::<T>::get(scribe.clone()), false);
		ScribeSetMap::<T>::insert(scribe.clone(), true);
		assert_eq!(ScribeSetMap::<T>::get(scribe.clone()), true);
		#[extrinsic_call]
		force_remove_node_from_scribe_set(RawOrigin::Root, scribe.clone());
		assert_eq!(ScribeSetMap::<T>::get(scribe), false);
	}

	#[benchmark]
	fn node_updated() {
		let scribe: T::AccountId = whitelisted_caller();
		const TEST_SSH_PUB_KEY: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDZ";
		const TEST_IP_ADDRESS: &str = "127.0.0.1";
		let ssh_pub_key: Vec<u8> = TEST_SSH_PUB_KEY.try_into().unwrap();
		let ip_address: Vec<u8> = TEST_IP_ADDRESS.try_into().unwrap();
		let ssh_pub_key_bounded: BoundedVec<u8, T::MaxSSHPubKeyLength> = ssh_pub_key.clone().try_into().unwrap();
		let ip_address_bounded: BoundedVec<u8, T::MaxIPAddressLength> = ip_address.clone().try_into().unwrap();
		let node_info = NodeInfo {
			ssh_pub_key: ssh_pub_key_bounded,
			ip_address: ip_address_bounded,
		};
		ScribeSetMap::<T>::insert(scribe.clone(), true);
		#[extrinsic_call]
		node_updated(RawOrigin::Signed(scribe.clone()), ssh_pub_key, ip_address);

		assert_eq!(NodeInfoMap::<T>::get(scribe), Some(node_info));
	}

	#[benchmark]
	fn force_update_node() {
		let scribe: T::AccountId = whitelisted_caller();
		const TEST_SSH_PUB_KEY: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDZ";
		const TEST_IP_ADDRESS: &str = "127.0.0.1";
		let ssh_pub_key: Vec<u8> = TEST_SSH_PUB_KEY.try_into().unwrap();
		let ip_address: Vec<u8> = TEST_IP_ADDRESS.try_into().unwrap();
		let ssh_pub_key_bounded: BoundedVec<u8, T::MaxSSHPubKeyLength> = ssh_pub_key.clone().try_into().unwrap();
		let ip_address_bounded: BoundedVec<u8, T::MaxIPAddressLength> = ip_address.clone().try_into().unwrap();
		let node_info = NodeInfo {
			ssh_pub_key: ssh_pub_key_bounded,
			ip_address: ip_address_bounded,
		};
		ScribeSetMap::<T>::insert(scribe.clone(), true);
		#[extrinsic_call]
		force_update_node(RawOrigin::Root, scribe.clone(), ssh_pub_key, ip_address);

		assert_eq!(NodeInfoMap::<T>::get(scribe), Some(node_info));
	}

	#[benchmark]
	fn force_add_node_to_custodian_set() {
		let custodian: T::AccountId = whitelisted_caller();
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		assert_eq!(CustodianSetMap::<T>::get(custodian.clone()), false);
		#[extrinsic_call]
		force_add_node_to_custodian_set(RawOrigin::Root, custodian.clone());
		assert_eq!(CustodianSetMap::<T>::get(custodian), true);
	}

	#[benchmark]
	fn force_remove_node_from_custodian_set() {
		let custodian: T::AccountId = whitelisted_caller();
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		assert_eq!(CustodianSetMap::<T>::get(custodian.clone()), true);
		#[extrinsic_call]
		force_remove_node_from_custodian_set(RawOrigin::Root, custodian.clone());
		assert_eq!(CustodianSetMap::<T>::get(custodian), false);
	}

	#[benchmark]
	fn channel_added() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		// let metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = vec![].try_into().unwrap();
		// let custodian_metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = vec![].try_into().unwrap();
		// let functional_metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		assert_eq!(TotalChannels::<T>::get(), None);
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), None);
		#[extrinsic_call]
		channel_added(RawOrigin::Signed(custodian.clone()), configurator.clone());
		assert_eq!(TotalChannels::<T>::get(), Some(1));
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info));
	}

	#[benchmark]
	fn channel_configurator_updated() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let listener: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		assert_eq!(TotalChannels::<T>::get(), None);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(listener.clone(), true);
		#[extrinsic_call]
		channel_configurator_updated(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID, listener.clone());
		let updated_channel_info = ChannelInfo {
			configurator: listener.clone(),
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn force_unarchive_channel() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: true,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		#[extrinsic_call]
		force_unarchive_channel(RawOrigin::Root, TEST_CHANNEL_ID);
		let unarchived_channel_info = ChannelInfo {
			archived: false,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(unarchived_channel_info));
	}

	#[benchmark]
	fn channel_transfer_accepted() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		CustodianSetMap::<T>::insert(configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
		#[extrinsic_call]
		channel_transfer_acceptance_toggled(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), true);
	}

	#[benchmark]
	fn channel_transfer_rejected() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		CustodianSetMap::<T>::insert(configurator.clone(), true);
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), true);
		#[extrinsic_call]
		channel_transfer_acceptance_toggled(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, false);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
	}

	#[benchmark]
	fn channel_transferred_once() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		CustodianSetMap::<T>::insert(configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), true);
		// assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);->This should be false, but it returns true
		#[extrinsic_call]
		channel_transferred(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID, configurator.clone());
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);
		let custodian_metadata_entry_1: CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>
		> = CustodianMetadataEntry {
			start_block_number: block_number_1.clone(),
			end_block_number: block_number_1.clone(),
			custodian: custodian.clone(),
			custodian_metadata: None,
		};
		let updated_historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>>
			= vec![custodian_metadata_entry_1.clone()].try_into().unwrap();
		let updated_historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = updated_historical_custodian_metadata.clone().try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			custodian: configurator.clone(),
			historical_custodian_metadata: updated_historical_custodian_metadata_bounded,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn channel_transferred_twice() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		CustodianSetMap::<T>::insert(configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, configurator.clone(), true);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), true);
		// assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);->This should be false, but it returns true
		// First transfer happens
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, configurator.clone(), false);
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, custodian.clone(), false);
		let custodian_metadata_entry_1: CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>
		> = CustodianMetadataEntry {
			start_block_number: block_number_1.clone(),
			end_block_number: block_number_1.clone(),
			custodian: custodian.clone(),
			custodian_metadata: None,
		};
		let updated_historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>>
			= vec![custodian_metadata_entry_1.clone()].try_into().unwrap();
		let updated_historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = updated_historical_custodian_metadata.clone().try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			custodian: configurator.clone(),
			historical_custodian_metadata: updated_historical_custodian_metadata_bounded,
			..channel_info
		};
		Channels::<T>::insert(TEST_CHANNEL_ID, updated_channel_info.clone());
		ChannelTransferAccepted::<T>::insert(TEST_CHANNEL_ID, custodian.clone(), true);
		#[extrinsic_call]
		channel_transferred(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, custodian.clone());
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, configurator.clone()), false);
		assert_eq!(ChannelTransferAccepted::<T>::get(TEST_CHANNEL_ID, custodian.clone()), false);
		let custodian_metadata_entry_2: CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>
		> = CustodianMetadataEntry {
			start_block_number: block_number_1.clone(),
			end_block_number: block_number_1.clone(),
			custodian: configurator.clone(),
			custodian_metadata: None,
		};
		let updated_historical_custodian_metadata_2: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>>
			= vec![custodian_metadata_entry_1.clone(), custodian_metadata_entry_2.clone()].try_into().unwrap();
		let updated_historical_custodian_metadata_bounded_2: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = updated_historical_custodian_metadata_2.clone().try_into().unwrap();
		let updated_channel_info_2 = ChannelInfo {
			custodian: custodian.clone(),
			historical_custodian_metadata: updated_historical_custodian_metadata_bounded_2,
			..updated_channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info_2));
	}

	#[benchmark]
	fn channel_archived() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		#[extrinsic_call]
		channel_archived(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID);
		let archived_channel_info = ChannelInfo {
			archived: true,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(archived_channel_info));
	}

	#[benchmark]
	fn channel_paused() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		#[extrinsic_call]
		channel_pause_toggled(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, true);
		let paused_channel_info = ChannelInfo {
			paused: true,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(paused_channel_info));
	}

	#[benchmark]
	fn channel_unpaused() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		let paused_channel_info = ChannelInfo {
			paused: true,
			..channel_info
		};
		Channels::<T>::insert(TEST_CHANNEL_ID, paused_channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(paused_channel_info.clone()));
		#[extrinsic_call]
		channel_pause_toggled(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, false);
		let unpaused_channel_info = ChannelInfo {
			paused: false,
			..paused_channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(unpaused_channel_info));
	}

	#[benchmark]
	fn node_added_to_channel_actant_set() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set: Vec<T::AccountId> = vec![];
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = actant_set.clone().try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		#[extrinsic_call]
		node_added_to_channel_actant_set(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, actant.clone());
		let updated_actant_set: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			actants: updated_actant_set,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn node_removed_from_channel_actant_set() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set: Vec<T::AccountId> = vec![actant.clone()];
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = actant_set.clone().try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		let updated_actant_set: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			actants: updated_actant_set,
			..channel_info
		};
		#[extrinsic_call]
		node_removed_from_channel_actant_set(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, actant.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn node_added_to_channel_listener_set() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let listener: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set: Vec<T::AccountId> = vec![];
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = listener_set.clone().try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(listener.clone(), true);
		#[extrinsic_call]
		node_added_to_channel_listener_set(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, listener.clone());
		let updated_listener_set: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![listener.clone()].try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			listeners: updated_listener_set,
			..channel_info
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn node_removed_from_channel_listener_set() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let listener: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set: Vec<T::AccountId> = vec![listener.clone()];
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = listener_set.clone().try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		let updated_listener_set: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let updated_channel_info = ChannelInfo {
			listeners: updated_listener_set,
			..channel_info
		};
		#[extrinsic_call]
		node_removed_from_channel_listener_set(RawOrigin::Signed(configurator.clone()), TEST_CHANNEL_ID, listener.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn channel_maker_updated() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let new_maker: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let updated_channel_info = ChannelInfo {
			maker: new_maker.clone(),
			..channel_info.clone()
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		#[extrinsic_call]
		channel_maker_updated(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID, new_maker.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn channel_metadata_updated() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_KURI: &str = "test_kuri";
		let metadata: Vec<u8> = TEST_KURI.try_into().unwrap();
		let metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = metadata.clone().try_into().unwrap();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let updated_channel_info = ChannelInfo {
			metadata: Some(metadata_bounded.clone()),
			..channel_info.clone()
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		let metadata_as_arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: metadata_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		Arikuris::<T>::insert(TEST_CHANNEL_ID, metadata_bounded.clone(), metadata_as_arikuri.clone());
		#[extrinsic_call]
		channel_metadata_updated(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID, metadata.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));
	}

	#[benchmark]
	fn channel_functional_metadata_updated() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		const TEST_KURI: &str = "test_kuri";
		let metadata: Vec<u8> = TEST_KURI.try_into().unwrap();
		let metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = metadata.clone().try_into().unwrap();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let updated_channel_info = ChannelInfo {
			functional_metadata: Some(metadata_bounded.clone()),
			..channel_info.clone()
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		let metadata_as_arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: metadata_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		Arikuris::<T>::insert(TEST_CHANNEL_ID, metadata_bounded.clone(), metadata_as_arikuri.clone());
		#[extrinsic_call]
		channel_functional_metadata_updated(RawOrigin::Signed(custodian.clone()), TEST_CHANNEL_ID, metadata.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(updated_channel_info));	
	}

	#[benchmark]
	fn channel_custodian_metadata_commit_thread_lock_requested() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: TEST_CHANNEL_ID,
			latest_commit_kuri: None,
			latest_commit_transaction_hash: None,
			scribe: None,
			locked_by: Some(actant.clone()),
			latest_commit_block_number: None,
			latest_commit_size: None,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		#[extrinsic_call]
		channel_custodian_metadata_commit_thread_lock_requested(RawOrigin::Signed(actant.clone()), TEST_CHANNEL_ID);
		assert_eq!(ChannelCustodianMetadataCommitThreads::<T>::get(TEST_CHANNEL_ID), Some(metadata_commit_thread.clone()));
	}

	#[benchmark]
	fn channel_custodian_metadata_commit_thread_lock_release_requested() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: TEST_CHANNEL_ID,
			latest_commit_kuri: None,
			latest_commit_transaction_hash: None,
			scribe: None,
			locked_by: Some(actant.clone()),
			latest_commit_block_number: None,
			latest_commit_size: None,
		};
		let metadata_commit_thread_unlocked: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			locked_by: None,
			..metadata_commit_thread.clone()
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		ChannelCustodianMetadataCommitThreads::<T>::insert(TEST_CHANNEL_ID, metadata_commit_thread.clone());
		#[extrinsic_call]
		channel_custodian_metadata_commit_thread_lock_release_requested(RawOrigin::Signed(actant.clone()), TEST_CHANNEL_ID);
		assert_eq!(ChannelCustodianMetadataCommitThreads::<T>::get(TEST_CHANNEL_ID), Some(metadata_commit_thread_unlocked.clone()));
	}

	#[benchmark]
	fn force_release_channel_custodian_metadata_commit_thread_lock() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: TEST_CHANNEL_ID,
			latest_commit_kuri: None,
			latest_commit_transaction_hash: None,
			scribe: None,
			locked_by: Some(actant.clone()),
			latest_commit_block_number: None,
			latest_commit_size: None,
		};
		let metadata_commit_thread_unlocked: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			locked_by: None,
			..metadata_commit_thread.clone()
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		ChannelCustodianMetadataCommitThreads::<T>::insert(TEST_CHANNEL_ID, metadata_commit_thread.clone());
		#[extrinsic_call]
		force_release_channel_custodian_metadata_commit_thread_lock(RawOrigin::Root, TEST_CHANNEL_ID, actant.clone());
		assert_eq!(ChannelCustodianMetadataCommitThreads::<T>::get(TEST_CHANNEL_ID), Some(metadata_commit_thread_unlocked.clone()));
	}

	#[benchmark]
	fn channel_custodian_metadata_updated_without_thread_lock_release() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		const TEST_KURI: &str = "test_kuri";
		let metadata: Vec<u8> = TEST_KURI.try_into().unwrap();
		let metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = metadata.clone().try_into().unwrap();
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let mut channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let mut metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: TEST_CHANNEL_ID,
			latest_commit_kuri: None,
			latest_commit_transaction_hash: None,
			scribe: None,
			locked_by: Some(actant.clone()),
			latest_commit_block_number: None,
			latest_commit_size: None,
		};
		const TEST_COMMIT_KURI: &str = "test_commit_kuri";
		let commit_kuri: Vec<u8> = TEST_COMMIT_KURI.try_into().unwrap();
		let commit_kuri_bounded: BoundedVec<u8, T::MaxKuriLength> = commit_kuri.clone().try_into().unwrap();
		let commit_transaction_hash: T::Hash = T::Hash::default();
		let commit_block_number: BlockNumberFor<T> = BlockNumberFor::<T>::from(2u32);
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		ChannelCustodianMetadataCommitThreads::<T>::insert(TEST_CHANNEL_ID, metadata_commit_thread.clone());
		let metadata_as_arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: metadata_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		Arikuris::<T>::insert(TEST_CHANNEL_ID, metadata_bounded.clone(), metadata_as_arikuri.clone());
		#[extrinsic_call]
		channel_custodian_metadata_updated(
			RawOrigin::Signed(actant.clone()),
			TEST_CHANNEL_ID,
			previous_custodian_metadata.clone(),
			metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone().try_into().unwrap(),
			commit_block_number.clone(),
			commit_size.clone(),
			false
		);
		channel_info = ChannelInfo {
			custodian_metadata: Some(metadata_bounded.clone()),
			..channel_info.clone()
		};
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
			locked_by: Some(actant.clone()),
			scribe: Some(actant.clone()),
			..metadata_commit_thread.clone()
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<T>::get(TEST_CHANNEL_ID), Some(metadata_commit_thread.clone()));
	}

	#[benchmark]
	fn channel_custodian_metadata_updated_with_thread_lock_release() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		const TEST_KURI: &str = "test_kuri";
		let metadata: Vec<u8> = TEST_KURI.try_into().unwrap();
		let metadata_bounded: BoundedVec<u8, T::MaxKuriLength> = metadata.clone().try_into().unwrap();
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let mut channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		let mut metadata_commit_thread: ChannelCustodianMetadataCommitThreadInfo<
			T::AccountId,
			Kuri<T>,
			T::Hash,
			BlockNumberFor<T>,
		> = ChannelCustodianMetadataCommitThreadInfo {
			channel_id: TEST_CHANNEL_ID,
			latest_commit_kuri: None,
			latest_commit_transaction_hash: None,
			scribe: None,
			locked_by: Some(actant.clone()),
			latest_commit_block_number: None,
			latest_commit_size: None,
		};
		const TEST_COMMIT_KURI: &str = "test_commit_kuri";
		let commit_kuri: Vec<u8> = TEST_COMMIT_KURI.try_into().unwrap();
		let commit_kuri_bounded: BoundedVec<u8, T::MaxKuriLength> = commit_kuri.clone().try_into().unwrap();
		let commit_transaction_hash: T::Hash = T::Hash::default();
		let commit_block_number: BlockNumberFor<T> = BlockNumberFor::<T>::from(2u32);
		let commit_size = 64;
		let previous_custodian_metadata: Vec<u8> = Vec::new();
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		ScribeSetMap::<T>::insert(actant.clone(), true);
		ChannelCustodianMetadataCommitThreads::<T>::insert(TEST_CHANNEL_ID, metadata_commit_thread.clone());
		let metadata_as_arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: metadata_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		Arikuris::<T>::insert(TEST_CHANNEL_ID, metadata_bounded.clone(), metadata_as_arikuri.clone());
		#[extrinsic_call]
		channel_custodian_metadata_updated(
			RawOrigin::Signed(actant.clone()),
			TEST_CHANNEL_ID,
			previous_custodian_metadata.clone(),
			metadata.clone(),
			commit_kuri.clone(),
			commit_transaction_hash.clone().try_into().unwrap(),
			commit_block_number.clone(),
			commit_size.clone(),
			true
		);
		channel_info = ChannelInfo {
			custodian_metadata: Some(metadata_bounded.clone()),
			..channel_info.clone()
		};
		metadata_commit_thread = ChannelCustodianMetadataCommitThreadInfo {
			latest_commit_kuri: Some(commit_kuri_bounded.clone()),
			latest_commit_transaction_hash: Some(commit_transaction_hash.clone()),
			latest_commit_block_number: Some(commit_block_number.clone()),
			latest_commit_size: Some(commit_size.clone()),
			locked_by: None,
			scribe: Some(actant.clone()),
			..metadata_commit_thread.clone()
		};
		assert_eq!(Channels::<T>::get(TEST_CHANNEL_ID), Some(channel_info.clone()));
		assert_eq!(ChannelCustodianMetadataCommitThreads::<T>::get(TEST_CHANNEL_ID), Some(metadata_commit_thread.clone()));
	}

	#[benchmark]
	fn arikuri_added() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		const TEST_KURI: &str = "test_kuri";
		let kuri: Vec<u8> = TEST_KURI.try_into().unwrap();
		let kuri_bounded: BoundedVec<u8, T::MaxKuriLength> = kuri.clone().try_into().unwrap();
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		ScribeSetMap::<T>::insert(actant.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		#[extrinsic_call]
		arikuri_added(RawOrigin::Signed(actant.clone()), kuri.clone(), TEST_CHANNEL_ID);
		let arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: kuri_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		assert_eq!(Arikuris::<T>::get(TEST_CHANNEL_ID, kuri_bounded.clone()), Some(arikuri.clone()));
	}

	#[benchmark]
	fn arikuri_deleted() {
		let custodian: T::AccountId = whitelisted_caller();
		let configurator: T::AccountId = whitelisted_caller();
		let actant: T::AccountId = whitelisted_caller();
		const TEST_CHANNEL_ID: u64 = 1;
		const TEST_KURI: &str = "test_kuri";
		let kuri: Vec<u8> = TEST_KURI.try_into().unwrap();
		let kuri_bounded: BoundedVec<u8, T::MaxKuriLength> = kuri.clone().try_into().unwrap();
		let historical_custodian_metadata: Vec<CustodianMetadataEntry<
			BlockNumberFor<T>,
			T::AccountId,
			BoundedVec<u8, T::MaxKuriLength>>> = vec![].try_into().unwrap();
		let historical_custodian_metadata_bounded: BoundedVec<
			CustodianMetadataEntry<
				BlockNumberFor<T>,
				T::AccountId,
				BoundedVec<u8, T::MaxKuriLength>,
			>,
			T::MaxCustodianMetadataHistoryLength,
		>  = historical_custodian_metadata.clone().try_into().unwrap();
		let actant_set_bounded: BoundedVec<T::AccountId, T::MaxActantsPerChannel> = vec![actant.clone()].try_into().unwrap();
		let listener_set_bounded: BoundedVec<T::AccountId, T::MaxListenersPerChannel> = vec![].try_into().unwrap();
		let block_number_1: BlockNumberFor<T> = BlockNumberFor::<T>::from(1u32);
		let channel_info: ChannelInfo<
			T::AccountId,
			ChannelActants<T>,
			ChannelListeners<T>,
			Kuri<T>,
			BlockNumberFor<T>,
			CustodianMetadataHistory<T>,
		> = ChannelInfo {
			block_number: block_number_1.clone(),
			id: TEST_CHANNEL_ID,
			custodian: custodian.clone(),
			configurator: configurator.clone(),
			maker: custodian.clone(),
			actants: actant_set_bounded.clone(),
			listeners: listener_set_bounded.clone(),
			historical_custodian_metadata: historical_custodian_metadata_bounded,
			metadata: None,
			custodian_metadata: None,
			functional_metadata: None,
			archived: false,
			paused: false,
		};
		ScribeSetMap::<T>::insert(custodian.clone(), true);
		CustodianSetMap::<T>::insert(custodian.clone(), true);
		ScribeSetMap::<T>::insert(configurator.clone(), true);
		ScribeSetMap::<T>::insert(actant.clone(), true);
		Channels::<T>::insert(TEST_CHANNEL_ID, channel_info.clone());
		let mut arikuri: ArikuriInfo<BoundedVec<u8, T::MaxKuriLength>> = ArikuriInfo {
			kuri: kuri_bounded.clone(),
			channel_id: TEST_CHANNEL_ID,
			deleted: false,
		};
		Arikuris::<T>::insert(TEST_CHANNEL_ID, kuri_bounded.clone(), arikuri.clone());
		#[extrinsic_call]
		arikuri_deleted(RawOrigin::Signed(actant.clone()), kuri.clone(), TEST_CHANNEL_ID);
		arikuri = ArikuriInfo {
			deleted: true,
			..arikuri.clone()
		};
		assert_eq!(Arikuris::<T>::get(TEST_CHANNEL_ID, kuri_bounded.clone()), Some(arikuri.clone()));
	}

	impl_benchmark_test_suite!(Metarium, crate::mock::new_test_ext(), crate::mock::Test);
}
