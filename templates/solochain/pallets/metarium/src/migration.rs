use super::*;
use frame_support::traits::OnRuntimeUpgrade;
use log;
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	storage_alias,
	traits::{Get, GetStorageVersion, StorageVersion},
	weights::Weight,
	Blake2_128Concat,

	// BoundedVec,
};
const LOG_TARGET: &str = "metarium";

// // only contains V1 storage format (Channels)
// pub mod v1 {
// 	use super::*;

// 	// #[derive(Decode, Encode, Debug)]
// 	// #[derive(
// 	// 	Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
// 	// )]
// 	#[derive(
// 		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
// 	)]
// 	pub struct ChannelV1Info<AccountId, ChannelActantNodes, ChannelListenerNodes, BlockNumber> {
// 		/// block number when the channel was created
// 		pub block_number: BlockNumber,
// 		/// data custodian address
// 		/// can change the data access setting address
// 		pub channel_custodian_node: AccountId,
// 		/// access setting address
// 		/// can assign access to a scribe_node or channel_listener_node
// 		/// can pause or unpause the channel
// 		/// can archive the channel
// 		pub channel_configuration_node: AccountId,
// 		/// channel identifier
// 		pub id: u64,
// 		/// read_only
// 		pub paused: bool,
// 		/// archived
// 		pub archived: bool,
// 		/// actant_nodes list authorized to write to the channel
// 		pub actant_nodes: ChannelActantNodes,
// 		/// listener_nodes list authorized to listen to the channel
// 		pub listener_nodes: ChannelListenerNodes,
// 	}
// 	pub type ChannelV1InfoOf<T> = ChannelV1Info<
// 		<T as frame_system::Config>::AccountId,
// 		ChannelActantNodes<T>,
// 		ChannelListenerNodes<T>,
// 		<T as frame_system::Config>::BlockNumber,
// 	>;

// 	#[storage_alias]
// 	pub(super) type Channels<T: Config> =
// 		StorageMap<Pallet<T>, Blake2_128Concat, u64, ChannelV1InfoOf<T>>;
// }

// // contains checks and transforms storage to V2 format (Channels)
// pub fn migrate_to_v2<T: Config>() -> Weight {
// 	let onchain_version = Pallet::<T>::on_chain_storage_version();
// 	if onchain_version < 2 {
// 		// migrate to v2
// 		// Need to make this more elegant.
// 		let count = v1::Channels::<T>::iter().count();
// 		log::info!(target: LOG_TARGET, " >>> Updating Channels storage. Migrating {} channels...", count);

// 		// We transform the storage values from the old into the new format.
// 		Channels::<T>::translate::<v1::ChannelV1InfoOf<T>, _>(
// 			|channel_id: u64, channel_info: v1::ChannelV1InfoOf<T>| {
// 				// log the channel_id and channel_info
// 				log::info!(
// 					target: LOG_TARGET,
// 					"     Migrating channel_info for channel_id {:?}... channel_info: {:?}",
// 					channel_id,
// 					channel_info
// 				);

// 				// create bounded actant nodes
// 				let bounded_commiter_nodes: BoundedVec<T::AccountId, T::MaxChannelActantNodes> =
// 					channel_info.actant_nodes.clone().into();
// 				// log the bounded_commiter_nodes
// 				log::info!(
// 					target: LOG_TARGET,
// 					"     Migrating bounded_commiter_nodes: {:?}", bounded_commiter_nodes
// 				);

// 				// create bounded listener nodes
// 				let bounded_listener_nodes: BoundedVec<T::AccountId, T::MaxChannelListenerNodes> =
// 					channel_info.listener_nodes.clone().into();
// 				// log the bounded_listener_nodes
// 				log::info!(
// 					target: LOG_TARGET,
// 					"     Migrating bounded_listener_nodes: {:?}", bounded_listener_nodes
// 				);

// 				// Create an empty array for the new CustodianMetadataHistory.
// 				let new_custodian_metadata_history: BoundedVec<
// 					CustodianMetadataEntry<
// 						T::BlockNumber,
// 						T::AccountId,
// 						BoundedVec<u8, T::MaxKURIlength>,
// 					>,
// 					T::MaxCustodianMetadataHistoryLength,
// 				> = Default::default();

// 				Some(ChannelInfo {
// 					block_number: channel_info.block_number,
// 					channel_custodian_node: channel_info.channel_custodian_node.clone(),
// 					channel_configuration_node: channel_info.channel_configuration_node.clone(),
// 					id: channel_id.into(),
// 					paused: channel_info.paused,
// 					archived: channel_info.archived,
// 					actant_nodes: bounded_commiter_nodes.clone(),
// 					listener_nodes: bounded_listener_nodes.clone(),
// 					channel_metadata: Default::default(),
// 					channel_maker_node: channel_info.channel_custodian_node.clone(),
// 					historical_custodian_metadata: new_custodian_metadata_history,
// 					custodian_metadata: Default::default(),
// 					functional_metadata: Default::default(),
// 					visibility: false,
// 				})
// 			},
// 		);

// 		// Update storage version.
// 		StorageVersion::new(2).put::<Pallet<T>>();
// 		// Need to make this more elegant.
// 		let count = Channels::<T>::iter().count();
// 		log::info!(target: LOG_TARGET, " <<< Channels storage updated! Migrated {} channels ✅", count);
// 		// Return the weight consumed by the migration.
// 		T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
// 	} else {
// 		log::info!(target: LOG_TARGET, " >>> Unused migration!");
// 		// We don't do anything here.
// 		Weight::zero()
// 	}
// }

// only contains V2 storage format (ChannelCustodianMetadataCommitThreads)
pub mod v2 {
	use super::*;

	#[derive(
		Debug, Clone, Encode, Decode, Eq, PartialEq, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct ChannelCustodianMetadataCommitThreadV1Info<AccountId, Kuri, Hash> {
		/// Channel ID
		pub channel_id: u64,
		/// Latest commit kuri, can be empty if there is no latest commit
		pub latest_commit_kuri: Option<Kuri>,
		/// Latest commit transaction hash, can be empty if there is no latest commit
		pub latest_commit_transaction_hash: Option<Hash>,
		/// Scribe node that intended to commit to the channel's custodian metadata, can be empty if
		/// there is no latest commit
		pub scribe: Option<AccountId>,
		/// Scribe node that has currently requested to commit to the channel's custodian metadata,
		/// can be empty
		pub locked_by: Option<AccountId>,
	}
	pub type ChannelCustodianMetadataCommitThreadV1InfoOf<T> =
		ChannelCustodianMetadataCommitThreadV1Info<
			<T as frame_system::Config>::AccountId,
			Kuri<T>,
			<T as frame_system::Config>::Hash,
		>;

	#[storage_alias]
	pub(super) type ChannelCustodianMetadataCommitThreads<T: Config> =
		StorageMap<Pallet<T>, Blake2_128Concat, u64, ChannelCustodianMetadataCommitThreadV1InfoOf<T>>;
}

// contains checks and transforms storage to V3 format (Channels)
pub fn migrate_to_v3<T: Config>() -> Weight {
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	if onchain_version < 3 {
		// migrate to v3
		// Need to make this more elegant.
		let count = v2::ChannelCustodianMetadataCommitThreads::<T>::iter().count();
		log::info!(
			target: LOG_TARGET,
			" >>> Updating ChannelCustodianMetadataCommitThreads storage. Migrating {} threads...",
			count
		);

		// We transform the storage values from the old into the new format.
		ChannelCustodianMetadataCommitThreads::<T>::translate::<
			v2::ChannelCustodianMetadataCommitThreadV1InfoOf<T>,
			_,
		>(|channel_id: u64, channel_custodian_metadata_commit_thread_info: v2::ChannelCustodianMetadataCommitThreadV1InfoOf<T>| {
			// log the channel_id and channel_info
			log::info!(
				target: LOG_TARGET,
				"     Migrating channel_info for channel_id {:?}... channel_custodian_metadata_commit_thread_info: {:?}",
				channel_id,
				channel_custodian_metadata_commit_thread_info
			);

			Some(ChannelCustodianMetadataCommitThreadInfo {
				channel_id: channel_custodian_metadata_commit_thread_info.channel_id,
				latest_commit_kuri: channel_custodian_metadata_commit_thread_info.latest_commit_kuri,
				latest_commit_transaction_hash: channel_custodian_metadata_commit_thread_info.latest_commit_transaction_hash,
				scribe: channel_custodian_metadata_commit_thread_info.scribe,
				locked_by: channel_custodian_metadata_commit_thread_info.locked_by,
				latest_commit_block_number: Default::default(),
				latest_commit_size: Default::default(),
			})
		});

		// Update storage version.
		StorageVersion::new(3).put::<Pallet<T>>();
		// Need to make this more elegant.
		let count = ChannelCustodianMetadataCommitThreads::<T>::iter().count();
		log::info!(
			target: LOG_TARGET,
			" <<< ChannelCustodianMetadataCommitThreads storage updated! Migrated {} threads ✅",
			count
		);
		// Return the weight consumed by the migration.
		T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
	} else {
		log::info!(target: LOG_TARGET, " >>> Unused migration!");
		// We don't do anything here.
		Weight::zero()
	}
}

/// v4: backfill the ChannelMembership reverse index from the existing `Channels`, so `books_for_address`
/// is complete for channels created BEFORE this upgrade (every custodian/configurator/maker/actant/listener
/// gets its role bit). Additive + idempotent (the OR-mutate re-applies cleanly).
pub fn migrate_to_v4<T: Config>() -> Weight {
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	if onchain_version < 4 {
		let mut reads: u64 = 0;
		let mut writes: u64 = 0;
		for (channel_id, channel) in Channels::<T>::iter() {
			reads += 1;
			ChannelMembership::<T>::mutate(&channel.custodian, channel_id, |f| *f |= ROLE_CUSTODIAN);
			ChannelMembership::<T>::mutate(&channel.configurator, channel_id, |f| *f |= ROLE_CONFIGURATOR);
			ChannelMembership::<T>::mutate(&channel.maker, channel_id, |f| *f |= ROLE_MAKER);
			writes += 3;
			for a in channel.actants.iter() {
				ChannelMembership::<T>::mutate(a, channel_id, |f| *f |= ROLE_ACTANT);
				writes += 1;
			}
			for l in channel.listeners.iter() {
				ChannelMembership::<T>::mutate(l, channel_id, |f| *f |= ROLE_LISTENER);
				writes += 1;
			}
		}
		StorageVersion::new(4).put::<Pallet<T>>();
		log::info!(
			target: LOG_TARGET,
			" <<< ChannelMembership backfilled ✅ ({} channels, {} role writes)",
			reads, writes
		);
		T::DbWeight::get().reads_writes(reads + 1, writes + 1)
	} else {
		log::info!(target: LOG_TARGET, " >>> migrate_to_v4 unused (already >= v4).");
		Weight::zero()
	}
}
