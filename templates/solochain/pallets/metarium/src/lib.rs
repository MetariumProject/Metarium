#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod multisig_integration;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod migration;

#[frame_support::pallet]
pub mod pallet {

	use crate::migration::{migrate_to_v3, migrate_to_v4};
	use frame_support::{
		dispatch::DispatchResult, ensure, pallet_prelude::{ValueQuery, *}, BoundedVec,
		weights::Weight,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			// v3 no-ops when already applied; v4 backfills ChannelMembership for pre-upgrade channels.
			migrate_to_v3::<T>().saturating_add(migrate_to_v4::<T>())
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The maximum size of a ChannelListenerNode's IP address
		type MaxIPAddressLength: Get<u32>;
		/// The maximum size of a ChannelListenerNode's SSH Public Key
		type MaxSSHPubKeyLength: Get<u32>;
		/// The maximum number of Channels that can be associated with a Node
		type MaxChannelsPerNode: Get<u32>;
		/// The maximum number of nodes that can commit to Channel
		type MaxActantsPerChannel: Get<u32>;
		/// The maximum number of nodes that can listen to Channel
		type MaxListenersPerChannel: Get<u32>;
		/// The maximum size of a Arikuri's Kuri
		type MaxKuriLength: Get<u32>;
		/// The maximum size of a commit
		type MaxCommitSize: Get<u64>;
		// The maximum number of arikuris that can be transferred
		type MaxArikurisToTransfer: Get<u32>;
		// The maximum length of custodian metadata history
		type MaxCustodianMetadataHistoryLength: Get<u32>;
		/// How many blocks a commit-thread lock may be held before it goes stale and can be taken over.
		type CommitLockTtl: Get<BlockNumberFor<Self>>;
	}

	/// CLASSES ///

	pub type IPAddress<T> = BoundedVec<u8, <T as Config>::MaxIPAddressLength>;
	pub type SSHPubKey<T> = BoundedVec<u8, <T as Config>::MaxSSHPubKeyLength>;
	pub type ChannelsPerNode<T> = BoundedVec<u64, <T as Config>::MaxChannelsPerNode>;
	pub type ChannelActants<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxActantsPerChannel>;
	pub type ChannelListeners<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxListenersPerChannel>;
	pub type Kuri<T> = BoundedVec<u8, <T as Config>::MaxKuriLength>;
	pub type Kuris<T> = BoundedVec<Kuri<T>, <T as Config>::MaxArikurisToTransfer>;

	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct CustodianMetadataEntry<BlockNumber, AccountId, Kuri> {
		/// start block number of the custodian metadata
		pub start_block_number: BlockNumber,
		/// end block number of the custodian metadata (Optional)
		pub end_block_number: BlockNumber,
		/// Custodian metadata (Optional)
		pub custodian_metadata: Option<Kuri>,
		/// Custodian node
		pub custodian: AccountId,
	}

	pub type CustodianMetadataHistory<T> = BoundedVec<
		CustodianMetadataEntry<
			BlockNumberFor<T>,
			<T as frame_system::Config>::AccountId,
			Kuri<T>,
		>,
		<T as Config>::MaxCustodianMetadataHistoryLength,
	>;

	/// STORAGE OBJECTS ///

	/// Scribe info
	/// A Scribe is a substrate account that can self update its own info.
	/// If a Scribe has an IP address, it can behave as a Node, meaning it can listen to a
	/// channel, commit to a channel, and archive a channel. If a Scribe has an IP address and a SSH
	/// public key, it can behave as a public Node, meaning it can communicate with other
	/// Nodes.
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct NodeInfo<SSHPubKey, IPAddress> {
		/// Node SSH public key, it can be empty
		pub ssh_pub_key: SSHPubKey,
		/// Node IP address, it can be empty
		pub ip_address: IPAddress,
	}

	/// ListenerChannels info
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct ListenerChannelsInfo<ChannelListeners> {
		/// List of channels that the listener node is listening to
		pub channel_ids: ChannelListeners,
	}

	// Channel info

	// Channel V2
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct ChannelInfo<
		AccountId,
		ChannelActants,
		ChannelListeners,
		Kuri,
		BlockNumber,
		CustodianMetadataHistory,
	> {
		/// block number when the channel was created
		pub block_number: BlockNumber,
		/// custodian address
		/// can change the data access setting address
		pub custodian: AccountId,
		/// access setting address
		/// can assign access to a node as an actant or a listener
		/// can pause or unpause the channel
		/// can archive the channel
		pub configurator: AccountId,
		/// channel identifier TODO : Convert Channel ID to MultiLocation
		pub id: u64,
		/// read_only
		pub paused: bool,
		/// archived
		pub archived: bool,
		/// actant list authorized to write to the channel
		pub actants: ChannelActants,
		/// listener list authorized to listen to the channel
		pub listeners: ChannelListeners,
		/// main metadata setting address
		/// can set the metadata of the channel
		/// can update the metadata of the channel
		pub maker: AccountId,
		/// channel metadata (optional)
		pub metadata: Option<Kuri>,
		/// historical custodian metadata is an empty list of dictionaries (optional)
		pub historical_custodian_metadata: CustodianMetadataHistory,
		/// custodian metadata
		pub custodian_metadata: Option<Kuri>,
		/// functional metadata
		pub functional_metadata: Option<Kuri>,
	}

	/// ChannelMetadataCommitThread info
	/// A ChannelCustodianMetadataCommitThread is a thread that is associated with
	/// 	a channel,
	/// 	it's latest commit kuri,
	/// 	the commit transaction hash,
	/// 	and the scribe node that intended to commit to the channel's custodian metadata.
	/// A ChannelCustodianMetadataCommitThread is created when a scribe node attempts to commit to a
	/// channel's custodian metadata. A ChannelCustodianMetadataCommitThread also has a locked_by field
	/// that is set to the scribe node that has currently requested to commit to the channel's
	/// custodian metadata.

	/// ChannelMetadataCommitThread V3
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct ChannelCustodianMetadataCommitThreadInfo<AccountId, Kuri, Hash, BlockNumber> {
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
		/// Block number when the latest commit was made, can be empty if there is no latest commit
		pub latest_commit_block_number: Option<BlockNumber>,
		/// Latest commit size, can be empty if there is no latest commit
		pub latest_commit_size: Option<u64>,
	}

	/// Arikuri info
	/// An Arikuri is a Kuri that is associated with a channel.
	/// A Kuri is a unique identifier that is a cryptographic hash of a file.
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct ArikuriInfo<Kuri> {
		/// Arikuri kuri
		pub kuri: Kuri,
		/// Arikuri channel
		pub channel_id: u64,
		/// Arikuri status
		pub deleted: bool,
	}

	/// STORAGE MAPS TO DISPLAY THE CHAIN STATE ///

	pub type NodeInfoOf<T> = NodeInfo<SSHPubKey<T>, IPAddress<T>>;

	pub type ListenerChannelsInfoOf<T> = ListenerChannelsInfo<ChannelsPerNode<T>>;

	pub type ChannelInfoOf<T> = ChannelInfo<
		<T as frame_system::Config>::AccountId,
		ChannelActants<T>,
		ChannelListeners<T>,
		Kuri<T>,
		BlockNumberFor<T>,
		CustodianMetadataHistory<T>,
	>;

	pub type ChannelCustodianMetadataCommitThreadInfoOf<T> = ChannelCustodianMetadataCommitThreadInfo<
		<T as frame_system::Config>::AccountId,
		Kuri<T>,
		<T as frame_system::Config>::Hash,
		// <T as frame_system::Config>::BlockNumber,
		BlockNumberFor<T>,
	>;

	pub type ArikuriInfoOf<T> = ArikuriInfo<Kuri<T>>;

	/// STORAGE FUNCTIONS TO QUERY THE CHAIN STATE ///

	// The pallet's runtime storage items.

	#[pallet::storage]
	#[pallet::getter(fn is_node_in_scribe_set)]
	pub(super) type ScribeSetMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_node_in_custodian_set)]
	pub(super) type CustodianSetMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	/// The delegated scribe-permissions admin: a single key that, alongside Root, may
	/// add or remove scribes. Set or cleared only by Root via `set_scribe_admin`.
	#[pallet::storage]
	#[pallet::getter(fn scribe_admin)]
	pub(super) type ScribeAdmin<T: Config> = StorageValue<_, T::AccountId>;

	/// The delegated custodian-permissions admin: a single key that, alongside Root, may
	/// add or remove custodians. Set or cleared only by Root via `set_custodian_admin`.
	#[pallet::storage]
	#[pallet::getter(fn custodian_admin)]
	pub(super) type CustodianAdmin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn node_info)]
	pub(super) type NodeInfoMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, NodeInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn channels_per_listener)]
	pub(super) type ChannelsPerListenerMap<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<u64, <T as Config>::MaxChannelsPerNode>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_channels)]
	pub type TotalChannels<T> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn channels)]
	pub(super) type Channels<T: Config> = StorageMap<_, Blake2_128Concat, u64, ChannelInfoOf<T>>;

	/// bookUUID → channel id: resolve a portable `mnem://<book>/…` identity to its channel on-chain.
	/// Additive (no `ChannelInfo` change, no storage migration); a bookUUID maps to exactly one channel.
	#[pallet::storage]
	#[pallet::getter(fn channel_for_book)]
	pub(super) type BookUuidToChannel<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxKuriLength>, u64>;

	/// channel id → its bookUUID: the reverse, so a book list resolved from membership can be DISPLAYED
	/// (and the binding is one-to-one). Set once per channel via `channel_book_uuid_set`.
	#[pallet::storage]
	#[pallet::getter(fn book_uuid_of)]
	pub(super) type ChannelBookUuid<T: Config> =
		StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxKuriLength>>;

	/// (account, channel) → a role bitmask — the SCALABLE reverse membership index. `books_for_address`
	/// iterates the (account, *) prefix off-chain (unbounded, paginated), replacing the bounded
	/// `ChannelsPerListenerMap` and adding the previously-missing actant reverse index. ValueQuery: a
	/// `0` flag means not a member (the entry is removed when the last role is dropped).
	#[pallet::storage]
	#[pallet::getter(fn channel_membership)]
	pub(super) type ChannelMembership<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u64,
		u8,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn channel_custodian_metadata_commit_threads)]
	pub(super) type ChannelCustodianMetadataCommitThreads<T: Config> =
		StorageMap<_, Blake2_128Concat, u64, ChannelCustodianMetadataCommitThreadInfoOf<T>>;

	/// Block at which each channel's commit-thread lock was acquired (for stale-lock takeover).
	#[pallet::storage]
	#[pallet::getter(fn commit_lock_acquired_at)]
	pub(super) type CommitLockAcquiredAt<T: Config> =
		StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>>;

	#[pallet::storage]
	#[pallet::getter(fn is_channel_transfer_accepted)]
	pub(super) type ChannelTransferAccepted<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u64,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_arikuris)]
	pub type TotalArikuris<T> = StorageMap<_, Blake2_128Concat, u64, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn arikuris)]
	pub(super) type Arikuris<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, Kuri<T>, ArikuriInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn is_arikuri_transfer_accepted)]
	pub(super) type ArikuriTransferAccepted<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, Kuri<T>, bool, ValueQuery>;
	
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NodeUpdated(T::AccountId),
		NodeAddedToScribeSet(T::AccountId),
		NodeRemovedFromScribeSet(T::AccountId),
		ScribeAdminUpdated(Option<T::AccountId>),
		NodeAddedToCustodianSet(T::AccountId),
		NodeRemovedFromCustodianSet(T::AccountId),
		CustodianAdminUpdated(Option<T::AccountId>),
		ChannelCreated(u64),
		ChannelMakerUpdated(u64, T::AccountId),
		ChannelConfiguratorUpdated(u64, T::AccountId),
		NodeAddedToChannelActantSet(u64, T::AccountId),
		NodeRemovedFromChannelActantSet(u64, T::AccountId),
		NodeAddedToChannelListenerSet(u64, T::AccountId),
		NodeRemovedFromChannelListenerSet(u64, T::AccountId),
		ChannelPaused(u64),
		ChannelUnpaused(u64),
		ChannelArchived(u64),
		ChannelUnarchived(u64),
		ChannelTransferAccepted(u64, T::AccountId),
		ChannelTransferRejected(u64, T::AccountId),
		ChannelTransferred(u64, T::AccountId),
		ChannelMetadataUpdated(u64, BoundedVec<u8, T::MaxKuriLength>),
		ChannelCustodianMetadataUpdated(
			u64,
			BoundedVec<u8, T::MaxKuriLength>,
			BoundedVec<u8, T::MaxKuriLength>,
		),
		ChannelFunctionalMetadataUpdated(u64, BoundedVec<u8, T::MaxKuriLength>),
		ChannelBookUuidSet(u64, BoundedVec<u8, T::MaxKuriLength>),
		ChannelCustodianMetadataCommitThreadLockRequested(u64, T::AccountId),
		ChannelCustodianMetadataCommitThreadLockReleased(u64, T::AccountId),
		ArikuriCreated(u64, BoundedVec<u8, T::MaxKuriLength>),
		ArikuriDeleted(u64, BoundedVec<u8, T::MaxKuriLength>),
		ArikuriUpdatedByRoot(BoundedVec<u8, T::MaxKuriLength>),
		ArikuriTransferAccepted(
			u64,
			u64,
			BoundedVec<Kuri<T>, T::MaxArikurisToTransfer>,
			T::AccountId,
		),
		ArikurisTransferred(u64, u64, BoundedVec<Kuri<T>, T::MaxArikurisToTransfer>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Node has already been removed from the scribe set.
		NodeAlreadyRemovedFromScribeSet,
		/// Node has already been added to the scribe set.
		NodeAlreadyAddedToScribeSet,
		/// Invalid node.
		InvalidNode,
		/// A channel_listener IP address is longer than the allowed limit.
		MaxIPAddressLengthExceeded,
		/// A node SSH Pub Key is longer than the allowed limit.
		MaxSSHPubKeyLengthExceeded,
		/// The length of custodian metadata history is longer than the allowed limit.
		MaxCustodianMetadataHistoryLengthExceeded,
		/// The list of channels a node is associated with is longer than the allowed limit.
		MaxChannelsPerNodeExceeded,
		/// Node has already been removed from the custodian set.
		NodeAlreadyRemovedFromCustodianSet,
		/// Node has already been added to the custodian set.
		NodeAlreadyAddedToCustodianSet,
		/// Maximum allowed channel limit has been reached.
		MaxChannelsExceeded,
		/// Channel not found.
		ChannelNotFound,
		/// The channel already has a bookUUID bound.
		ChannelBookUuidAlreadySet,
		/// The bookUUID is already bound to another channel.
		BookUuidAlreadyBound,
		/// Channel Custodian Metadata Commit Thread not found.
		ChannelCustodianMetadataCommitThreadNotFound,
		/// Channel has already been archived.
		ChannelAlreadyArchived,
		/// Channel has not been archived.
		ChannelNotArchived,
		/// From Channel has already been archived.
		FromChannelAlreadyArchived,
		/// From Channel has already been paused.
		FromChannelAlreadyPaused,
		/// To Channel has already been archived.
		ToChannelAlreadyArchived,
		/// To Channel has already been paused.
		ToChannelAlreadyPaused,
		/// Maximum number of actant nodes per channel has been reached.
		MaxActantsPerChannelExceeded,
		/// Maximum number of listener nodes per channel has been reached.
		MaxListenersPerChannelExceeded,
		/// Node is not in the channel actant set.
		NodeDoesNotExistInChannelActantSet,
		/// Node is already in the channel actant set.
		NodeAlreadyExistsInChannelActantSet,
		/// Node is not in the channel listener set.
		NodeDoesNotExistInChannelListenerSet,
		/// Node is already in the channel listener set.
		NodeAlreadyExistsInChannelListenerSet,
		/// Channel is already paused.
		ChannelAlreadyPaused,
		/// Channel is already unpaused.
		ChannelAlreadyUnpaused,
		/// Channel custodian metadata commit thread is already locked.
		ChannelCustodianMetadataCommitThreadAlreadyLocked,
		/// Channel custodian metadata commit thread is not locked by the node.
		ChannelCustodianMetadataCommitThreadNotLockedByNode,
		/// Channel custodian metadata values do not match.
		ChannelCustodianMetadataMismatch,
		/// A Kuri is longer than the allowed limit.
		MaxKuriLengthExceeded,
		/// A commit size is longer than the allowed limit.
		MaxCommitSizeExceeded,
		/// 404 for a Arikuri.
		ArikuriNotFound,
		/// Arikuri has already been added.
		ArikuriAlreadyAdded,
		/// Arikuri has already been deleted.
		ArikuriAlreadyDeleted,
		/// Arikuri transfer has not been accepted.
		ArikuriTransferNotAccepted,
		/// Maximum number of transferable arikuris' limit has been reached.
		MaxTransferableArikurisLimitExceeded,
		/// 403 for a caller.
		CallForbidden,
	}

	/// ChannelMembership role flags (a bitmask — an account can hold several roles in one channel).
	pub const ROLE_CUSTODIAN: u8 = 0b0000_0001;
	pub const ROLE_CONFIGURATOR: u8 = 0b0000_0010;
	pub const ROLE_MAKER: u8 = 0b0000_0100;
	pub const ROLE_ACTANT: u8 = 0b0000_1000;
	pub const ROLE_LISTENER: u8 = 0b0001_0000;

	impl<T: Config> Pallet<T> {
		/// Record that `account` holds `role` in `channel_id` (the ChannelMembership reverse index).
		fn membership_add(account: &T::AccountId, channel_id: u64, role: u8) {
			ChannelMembership::<T>::mutate(account, channel_id, |flags| *flags |= role);
		}

		/// Clear `role` for `account` in `channel_id`; drop the entry entirely when no roles remain.
		fn membership_remove(account: &T::AccountId, channel_id: u64, role: u8) {
			ChannelMembership::<T>::mutate(account, channel_id, |flags| *flags &= !role);
			if ChannelMembership::<T>::get(account, channel_id) == 0 {
				ChannelMembership::<T>::remove(account, channel_id);
			}
		}
	}

	// Dispatchable functions to interact with the pallet and invoke state changes.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/////// SCRIBE-SET FUNCTIONS ///////

		/// A node is added to the scribe-set by the root.
		#[pallet::call_index(0)]
		#[pallet::weight(
			Weight::from_parts(9_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_add_node_to_scribe_set(
			origin: OriginFor<T>,
			node: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// Root, or the delegated scribe-permissions admin (ScribeAdmin).
			if ensure_root(origin.clone()).is_err() {
				let who = ensure_signed(origin)?;
				ensure!(
					<ScribeAdmin<T>>::get().as_ref() == Some(&who),
					Error::<T>::CallForbidden
				);
			}

			// SANITY CHECKS //

			// Ensure that the node is not already in the scribe_set.
			ensure!(
				!Self::is_node_in_scribe_set(&node),
				Error::<T>::NodeAlreadyAddedToScribeSet
			);

			// UPDATE STORAGE //

			// Update ScribeSetMap storage.
			<ScribeSetMap<T>>::insert(&node, true);

			// EMIT EVENTS //

			// Emit NodeAddedToScribeSet event.
			Self::deposit_event(Event::NodeAddedToScribeSet(node));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// Root sets (or clears, with `None`) the delegated scribe-permissions admin —
		/// a single key that may thereafter add or remove scribes alongside Root.
		#[pallet::call_index(29)]
		#[pallet::weight(
			Weight::from_parts(9_000_000, 0)
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn set_scribe_admin(
			origin: OriginFor<T>,
			admin: Option<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			match &admin {
				Some(a) => <ScribeAdmin<T>>::put(a),
				None => <ScribeAdmin<T>>::kill(),
			}
			Self::deposit_event(Event::ScribeAdminUpdated(admin));
			Ok(())
		}

		/// A node is removed from the scribe-set by the root.
		#[pallet::call_index(1)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(11_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_remove_node_from_scribe_set(
			origin: OriginFor<T>,
			node: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// Root, or the delegated scribe-permissions admin (ScribeAdmin).
			if ensure_root(origin.clone()).is_err() {
				let who = ensure_signed(origin)?;
				ensure!(
					<ScribeAdmin<T>>::get().as_ref() == Some(&who),
					Error::<T>::CallForbidden
				);
			}

			// SANITY CHECKS //

			// Check that the node is in the scribe_set.
			ensure!(
				Self::is_node_in_scribe_set(&node),
				Error::<T>::NodeAlreadyRemovedFromScribeSet
			);

			// UPDATE STORAGE //

			// Update ScribeSetMap storage.
			<ScribeSetMap<T>>::insert(&node, false);

			// EMIT EVENTS //

			// Emit NodeRemovedFromScribeSet event.
			Self::deposit_event(Event::NodeRemovedFromScribeSet(node));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/////// NODE-INFO FUNCTIONS ///////

		/// A node belonging to the scribe-set updates their own information.
		#[pallet::call_index(2)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(11_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn node_updated(
			origin: OriginFor<T>,
			ssh_pub_key: Vec<u8>,
			ip_address: Vec<u8>,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// check that the ssh_pub_key is not longer than the allowed limit.
			let bounded_ssh_pub_key: BoundedVec<u8, T::MaxSSHPubKeyLength> =
				ssh_pub_key.try_into().map_err(|_| Error::<T>::MaxSSHPubKeyLengthExceeded)?;

			// check that the ip_address is not longer than the allowed limit.
			let bounded_ip_address: BoundedVec<u8, T::MaxIPAddressLength> =
				ip_address.try_into().map_err(|_| Error::<T>::MaxIPAddressLengthExceeded)?;

			// UPDATE STORAGE //

			// Update NodeInfoMap storage.
			<NodeInfoMap<T>>::insert(
				&signer,
				NodeInfo { ssh_pub_key: bounded_ssh_pub_key, ip_address: bounded_ip_address },
			);

			// EMIT EVENTS //

			// Emit NodeUpdated event.
			Self::deposit_event(Event::NodeUpdated(signer));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A node belonging to the scribe-set has their information updated by the root.
		#[pallet::call_index(3)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(11_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_update_node(
			origin: OriginFor<T>,
			node: T::AccountId,
			ssh_pub_key: Vec<u8>,
			ip_address: Vec<u8>,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// Check that the signer is a root.
			ensure_root(origin)?;

			// Check that the node is in the scribe_set.
			ensure!(
				Self::is_node_in_scribe_set(&node),
				Error::<T>::NodeAlreadyRemovedFromScribeSet
			);

			// SANITY CHECKS //

			// check that the ssh_pub_key is not longer than the allowed limit.
			let bounded_ssh_pub_key: BoundedVec<u8, T::MaxSSHPubKeyLength> =
				ssh_pub_key.try_into().map_err(|_| Error::<T>::MaxSSHPubKeyLengthExceeded)?;

			// check that the ip_address is not longer than the allowed limit.
			let bounded_ip_address: BoundedVec<u8, T::MaxIPAddressLength> =
				ip_address.try_into().map_err(|_| Error::<T>::MaxIPAddressLengthExceeded)?;

			// UPDATE STORAGE //

			// Create a new node.
			// Update NodeInfoMap storage.
			<NodeInfoMap<T>>::insert(
				&node,
				NodeInfo { ssh_pub_key: bounded_ssh_pub_key, ip_address: bounded_ip_address },
			);

			// EMIT EVENTS //

			// Emit NodeUpdated event.
			Self::deposit_event(Event::NodeUpdated(node));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/////// CHANNEL-CUSTODIAN-SET FUNCTIONS ///////

		/// A node is added to the custodian-set by the root.
		#[pallet::call_index(4)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(12_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_add_node_to_custodian_set(
			origin: OriginFor<T>,
			custodian: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// Root, or the delegated custodian-permissions admin (CustodianAdmin).
			if ensure_root(origin.clone()).is_err() {
				let who = ensure_signed(origin)?;
				ensure!(
					<CustodianAdmin<T>>::get().as_ref() == Some(&who),
					Error::<T>::CallForbidden
				);
			}

			// SANITY CHECKS //

			// Check that the node is in the scribe set.
			ensure!(
				Self::is_node_in_scribe_set(&custodian),
				Error::<T>::NodeAlreadyRemovedFromScribeSet
			);

			// Ensure that the node is not already in the custodian set.
			ensure!(
				!Self::is_node_in_custodian_set(&custodian),
				Error::<T>::NodeAlreadyAddedToCustodianSet
			);

			// UPDATE STORAGE //

			// Update CustodianSetMap storage.
			<CustodianSetMap<T>>::insert(&custodian, true);

			// EMIT EVENTS //

			// Emit NodeAddedToCustodianSet event.
			Self::deposit_event(Event::NodeAddedToCustodianSet(custodian));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// Root sets (or clears, with `None`) the delegated custodian-permissions admin —
		/// a single key that may thereafter add or remove custodians alongside Root.
		#[pallet::call_index(30)]
		#[pallet::weight(
			Weight::from_parts(9_000_000, 0)
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn set_custodian_admin(
			origin: OriginFor<T>,
			admin: Option<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			match &admin {
				Some(a) => <CustodianAdmin<T>>::put(a),
				None => <CustodianAdmin<T>>::kill(),
			}
			Self::deposit_event(Event::CustodianAdminUpdated(admin));
			Ok(())
		}

		/// A node is removed from the custodian-set by the root.
		#[pallet::call_index(5)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(10_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_remove_node_from_custodian_set(
			origin: OriginFor<T>,
			custodian: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// Root, or the delegated custodian-permissions admin (CustodianAdmin).
			if ensure_root(origin.clone()).is_err() {
				let who = ensure_signed(origin)?;
				ensure!(
					<CustodianAdmin<T>>::get().as_ref() == Some(&who),
					Error::<T>::CallForbidden
				);
			}

			// SANITY CHECKS //

			// Ensure that the node is already in the custodian-set.
			ensure!(
				Self::is_node_in_custodian_set(&custodian),
				Error::<T>::NodeAlreadyRemovedFromCustodianSet
			);

			// UPDATE STORAGE //

			// Update CustodianSetMap storage.
			<CustodianSetMap<T>>::insert(&custodian, false);

			// EMIT EVENTS //

			// Emit NodeRemovedFromCustodianSet event.
			Self::deposit_event(Event::NodeRemovedFromCustodianSet(custodian));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/////// CHANNEL FUNCTIONS ///////

		/// A channel is added by a custodian (should also be a scribe).
		#[pallet::call_index(6)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(16_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn channel_added(
			origin: OriginFor<T>,
			configurator: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the signer is added to the custodian-set.
			ensure!(Self::is_node_in_custodian_set(&signer), Error::<T>::CallForbidden);

			// check that the configurator is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&configurator), Error::<T>::InvalidNode);

			// SANITY CHECKS //

			// None.

			// UPDATE STORAGE //

			// Create an empty channel_id.
			let channel_id: u64;

			// Update TotalChannels storage.
			match <TotalChannels<T>>::get() {
				None => {
					// set the channel_id to 1.
					channel_id = 1;
				},
				Some(old) => {
					// increment the channel_id by 1.
					channel_id = old.checked_add(1).ok_or(Error::<T>::MaxChannelsExceeded)?;
				},
			}
			<TotalChannels<T>>::put(channel_id);

			// Create an empty array for the new CustodianMetadataHistory.
			let new_custodian_metadata_history: BoundedVec<
				CustodianMetadataEntry<
					BlockNumberFor<T>,
					T::AccountId,
					BoundedVec<u8, T::MaxKuriLength>,
				>,
				T::MaxCustodianMetadataHistoryLength,
			> = Default::default();

			// Create a new channel. Set all metadata fields to their default values.
			let new_channel = ChannelInfo {
				block_number: <frame_system::Pallet<T>>::block_number(),
				id: channel_id.clone(),
				custodian: signer.clone(),
				configurator: configurator.clone(),
				maker: signer.clone(),
				archived: false,
				paused: false,
				actants: Vec::with_capacity(T::MaxActantsPerChannel::get() as usize)
					.try_into()
					.map_err(|_| Error::<T>::MaxActantsPerChannelExceeded)?,
				listeners: Vec::with_capacity(T::MaxListenersPerChannel::get() as usize)
					.try_into()
					.map_err(|_| Error::<T>::MaxListenersPerChannelExceeded)?,
				metadata: Default::default(),
				custodian_metadata: Default::default(),
				historical_custodian_metadata: new_custodian_metadata_history,
				functional_metadata: Default::default(),
			};

			// Update Channels storage.
			<Channels<T>>::insert(channel_id.clone(), new_channel);

			// Record the creator's roles in the membership reverse index (custodian == maker == signer).
			Self::membership_add(&signer, channel_id, ROLE_CUSTODIAN | ROLE_MAKER);
			Self::membership_add(&configurator, channel_id, ROLE_CONFIGURATOR);

			// EMIT EVENTS //

			// Emit ChannelCreated event.
			Self::deposit_event(Event::ChannelCreated(channel_id));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// Bind a bookUUID to a channel — the portable `mnem://<book>/…` identity → this channel. Additive
		/// + bind-once: a channel gets exactly one bookUUID and a bookUUID maps to exactly one channel, so
		/// the resolution is unambiguous in both directions. Set by the channel's configurator or custodian
		/// (a scribe). Works for channels created before this upgrade, too.
		#[pallet::call_index(31)]
		#[pallet::weight(
			Weight::from_parts(15_000_000, 0)
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn channel_book_uuid_set(
			origin: OriginFor<T>,
			channel_id: u64,
			book_uuid: BoundedVec<u8, T::MaxKuriLength>,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is a scribe.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// the channel must exist, and the signer must be its configurator or custodian.
			let channel = Channels::<T>::get(channel_id).ok_or(Error::<T>::ChannelNotFound)?;
			ensure!(
				channel.configurator == signer || channel.custodian == signer,
				Error::<T>::CallForbidden
			);

			// SANITY CHECKS // bind-once in both directions.
			ensure!(
				!ChannelBookUuid::<T>::contains_key(channel_id),
				Error::<T>::ChannelBookUuidAlreadySet
			);
			ensure!(
				!BookUuidToChannel::<T>::contains_key(&book_uuid),
				Error::<T>::BookUuidAlreadyBound
			);

			// UPDATE STORAGE //
			ChannelBookUuid::<T>::insert(channel_id, book_uuid.clone());
			BookUuidToChannel::<T>::insert(book_uuid.clone(), channel_id);

			// EMIT EVENTS //
			Self::deposit_event(Event::ChannelBookUuidSet(channel_id, book_uuid));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A configurator is updated by the channel's custodian (should also be a scribe).
		#[pallet::call_index(7)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(18_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_configurator_updated(
			origin: OriginFor<T>,
			channel_id: u64,
			new_configurator: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the signer is already in the custodian-set.
			// this check, although redundant, is included for consistency with the other functions.
			// it is also included to catch unknown unknowns.
			ensure!(Self::is_node_in_custodian_set(&signer), Error::<T>::CallForbidden);

			// check that the new_configurator is added to the scribe-set.
			ensure!(
				Self::is_node_in_scribe_set(&new_configurator),
				Error::<T>::InvalidNode
			);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel custodian node.
			ensure!(channel.custodian == signer, Error::<T>::CallForbidden);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.configurator = new_configurator.clone();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelConfiguratorUpdated event.
			Self::deposit_event(Event::ChannelConfiguratorUpdated(
				channel_id,
				new_configurator,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel is unarchived by the root.
		#[pallet::call_index(8)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(11_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3464))
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_unarchive_channel(origin: OriginFor<T>, channel_id: u64) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is the root.
			ensure_root(origin)?;

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the channel is archived.
			ensure!(channel.archived == true, Error::<T>::ChannelNotArchived);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.archived = false;
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelUnarchived event.
			Self::deposit_event(Event::ChannelUnarchived(channel_id));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel transfer acceptance is toggled by a potential custodian (should also be a scribe).
		#[pallet::call_index(9)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(16_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_transfer_acceptance_toggled(
			origin: OriginFor<T>,
			channel_id: u64,
			toggle_value: bool,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the signer is added to the custodian-set.
			ensure!(Self::is_node_in_custodian_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// check that the channel_id does already exist.
			ensure!(Channels::<T>::get(channel_id.clone()) != None, Error::<T>::ChannelNotFound);

			// UPDATE STORAGE //

			// Update ChannelTransferAccepted storage.
			<ChannelTransferAccepted<T>>::insert(channel_id.clone(), signer.clone(), toggle_value);

			// EMIT EVENTS //

			// Emit ChannelTransferAccepted or ChannelTransferRejected event.
			if toggle_value {
				Self::deposit_event(Event::ChannelTransferAccepted(channel_id, signer));
			} else {
				Self::deposit_event(Event::ChannelTransferRejected(channel_id, signer));
			}

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel is transferred by the channel's custodian (should also be a scribe).
		/// Note: The signer need not be in the custodian-set. Such a situation could arise if the custodian is retired from the system.
		/// However, it is important that the signer (the channel's custodian) is still a scribe.
		#[pallet::call_index(10)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(23_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3538))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn channel_transferred(
			origin: OriginFor<T>,
			channel_id: u64,
			new_custodian: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the new_custodian is added to the scribe-set.
			ensure!(
				Self::is_node_in_scribe_set(&new_custodian),
				Error::<T>::CallForbidden
			);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel custodian node.
			ensure!(channel.custodian == signer, Error::<T>::CallForbidden);

			// check that the channel transfer is accepted by the new custodian node.
			ensure!(
				Self::is_channel_transfer_accepted(channel_id.clone(), &new_custodian),
				Error::<T>::CallForbidden
			);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			let bounded_historical_custodian_metadata: BoundedVec<
				CustodianMetadataEntry<
					BlockNumberFor<T>,
					T::AccountId,
					BoundedVec<u8, T::MaxKuriLength>,
				>,
				T::MaxCustodianMetadataHistoryLength,
			> = updated_channel
				.historical_custodian_metadata
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxCustodianMetadataHistoryLengthExceeded)?;

			let mut metadata_start_block_number = updated_channel.block_number.clone();

			// if existing historical_custodian_metadata is empty, set the
			// metadata_start_block_number to the channel's block number. Otherwise, set the
			// metadata_start_block_number to the latest historical_custodian_metadata entry's
			// end_block_number.
			if !updated_channel.historical_custodian_metadata.is_empty() {
				metadata_start_block_number = updated_channel
					.historical_custodian_metadata
					.last()
					.unwrap()
					.end_block_number
					.clone();
			}
			let bounded_updated_historical_custodian_metadata: BoundedVec<
				CustodianMetadataEntry<
					BlockNumberFor<T>,
					T::AccountId,
					BoundedVec<u8, T::MaxKuriLength>,
				>,
				T::MaxCustodianMetadataHistoryLength,
			> = bounded_historical_custodian_metadata
				.into_iter()
				// .chain(Some(latest_historical_custodian_metadata_entry.clone()).into_iter())
				.chain(Some(CustodianMetadataEntry {
					start_block_number: metadata_start_block_number,
					end_block_number: <frame_system::Pallet<T>>::block_number(),
					custodian: updated_channel.custodian.clone(),
					custodian_metadata: updated_channel.custodian_metadata.clone(),
				}))
				.collect::<Vec<_>>()
				.try_into()
				.map_err(|_| Error::<T>::MaxCustodianMetadataHistoryLengthExceeded)?;
			// set the updated historical_custodian_metadata.
			updated_channel.historical_custodian_metadata =
				bounded_updated_historical_custodian_metadata.clone();
			// set the custodian_metadata to the default value.
			updated_channel.custodian_metadata = Default::default();
			// set the new_custodian.
			updated_channel.custodian = new_custodian.clone();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// Update ChannelTransferAccepted storage.
			// remove the channel transfer accepted entry for the new custodian node.
			<ChannelTransferAccepted<T>>::remove(channel_id.clone(), new_custodian.clone());
			// remove the channel transfer accepted entry for the old custodian node.
			<ChannelTransferAccepted<T>>::remove(channel_id.clone(), signer.clone());

			// EMIT EVENTS //

			// Emit ChannelTransferred event.
			Self::deposit_event(Event::ChannelTransferred(channel_id, new_custodian));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel is archived by its custodian.
		#[pallet::call_index(11)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(13_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_archived(origin: OriginFor<T>, channel_id: u64) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel custodian node.
			ensure!(channel.custodian == signer, Error::<T>::CallForbidden);

			// check that the channel is not already archived.
			ensure!(channel.archived.eq(&false), Error::<T>::ChannelAlreadyArchived);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.archived = true;
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelArchived event.
			Self::deposit_event(Event::ChannelArchived(channel_id));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel is paused / unpaused by its configurator.
		#[pallet::call_index(12)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(14_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_pause_toggled(
			origin: OriginFor<T>,
			channel_id: u64,
			toggle_value: bool,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's configurator.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// if the toggle_value is true, check that the channel is not already paused.
			// if the toggle_value is false, check that the channel is not already unpaused.
			if toggle_value {
				ensure!(channel.paused.eq(&false), Error::<T>::ChannelAlreadyPaused);
			} else {
				ensure!(channel.paused.eq(&true), Error::<T>::ChannelAlreadyUnpaused);
			}

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.paused = toggle_value;
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelPaused or ChannelUnpaused event.
			if toggle_value {
				Self::deposit_event(Event::ChannelPaused(channel_id));
			} else {
				Self::deposit_event(Event::ChannelUnpaused(channel_id));
			}

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A node is added to the channel's actant set by the channel's configurator.
		#[pallet::call_index(13)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(15_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn node_added_to_channel_actant_set(
			origin: OriginFor<T>,
			channel_id: u64,
			actant_node: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the actant_node is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&actant_node), Error::<T>::InvalidNode);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's configurator.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// check that the actant_node is not already added to the channel's actant set.
			ensure!(
				channel.actants.contains(&actant_node) == false,
				Error::<T>::NodeAlreadyExistsInChannelActantSet
			);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			let bounded_existing_actants: BoundedVec<
				T::AccountId,
				T::MaxActantsPerChannel,
			> = updated_channel
				.actants
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxActantsPerChannelExceeded)?;
			let bounded_updated_actants: BoundedVec<
				T::AccountId,
				T::MaxActantsPerChannel,
			> = bounded_existing_actants
				.into_iter()
				.chain(Some(actant_node.clone()).into_iter())
				.collect::<Vec<_>>()
				.try_into()
				.map_err(|_| Error::<T>::MaxActantsPerChannelExceeded)?;
			updated_channel.actants = bounded_updated_actants.into();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// Record actant membership in the reverse index.
			Self::membership_add(&actant_node, channel_id, ROLE_ACTANT);

			// EMIT EVENTS //

			// Emit ChannelActantNodeAdded event.
			Self::deposit_event(Event::NodeAddedToChannelActantSet(channel_id, actant_node));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A node is removed from the channel's actant set by the channel's configurator.
		#[pallet::call_index(14)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(15_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn node_removed_from_channel_actant_set(
			origin: OriginFor<T>,
			channel_id: u64,
			actant_node: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's configurator.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// check that the actant is already added to the channel's actant-set.
			ensure!(
				channel.actants.contains(&actant_node) == true,
				Error::<T>::NodeDoesNotExistInChannelActantSet
			);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			let bounded_existing_actants: BoundedVec<
				T::AccountId,
				T::MaxActantsPerChannel,
			> = updated_channel
				.actants
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxActantsPerChannelExceeded)?;
			let bounded_updated_actants: BoundedVec<
				T::AccountId,
				T::MaxActantsPerChannel,
			> = bounded_existing_actants
				.into_iter()
				.filter(|x| x != &actant_node)
				.collect::<Vec<_>>()
				.try_into()
				.map_err(|_| Error::<T>::MaxActantsPerChannelExceeded)?;
			updated_channel.actants = bounded_updated_actants.into();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// Clear actant membership in the reverse index.
			Self::membership_remove(&actant_node, channel_id, ROLE_ACTANT);

			// EMIT EVENTS //

			// Emit NodeRemovedFromChannelActantSet event.
			Self::deposit_event(Event::NodeRemovedFromChannelActantSet(channel_id, actant_node));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A node is added to the channel's listener-set by the channel's configurator.
		#[pallet::call_index(15)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(19_000_000, 0)
				.saturating_add(Weight::from_parts(0, 527805))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn node_added_to_channel_listener_set(
			origin: OriginFor<T>,
			channel_id: u64,
			listener: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the listener is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&listener), Error::<T>::InvalidNode);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel configurator.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// check that the listener is not already added to the channel's listener-set.
			ensure!(
				channel.listeners.contains(&listener) == false,
				Error::<T>::NodeAlreadyExistsInChannelListenerSet
			);

			// UPDATE STORAGE //

			// Verify Channels storage.
			let mut updated_channel = channel.clone();
			let bounded_existing_listeners: BoundedVec<
				T::AccountId,
				T::MaxListenersPerChannel,
			> = updated_channel
				.listeners
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxListenersPerChannelExceeded)?;
			let bounded_updated_listeners: BoundedVec<T::AccountId, T::MaxListenersPerChannel> =
				bounded_existing_listeners
					.into_iter()
					.chain(Some(listener.clone()).into_iter())
					.collect::<Vec<_>>()
					.try_into()
					.map_err(|_| Error::<T>::MaxListenersPerChannelExceeded)?;
			updated_channel.listeners = bounded_updated_listeners.into();

			// Verify ChannelsPerListenerMap storage.
			let channels_per_listener_map: BoundedVec<u64, T::MaxChannelsPerNode> =
				ChannelsPerListenerMap::<T>::get(listener.clone());
			// add the channel_id to the channels_per_listener_map.
			let updated_channels_per_listener_map: BoundedVec<u64, T::MaxChannelsPerNode> =
				channels_per_listener_map
					.clone()
					.into_iter()
					.chain(Some(channel_id.clone()).into_iter())
					.collect::<Vec<_>>()
					.try_into()
					.map_err(|_| Error::<T>::MaxChannelsPerNodeExceeded)?;

			// Update Channels storage.
			<Channels<T>>::insert(channel_id.clone(), updated_channel);
			// update ChannelsPerListenerMap storage.
			ChannelsPerListenerMap::<T>::insert(listener.clone(), updated_channels_per_listener_map);

			// Record listener membership in the (scalable) reverse index.
			Self::membership_add(&listener, channel_id, ROLE_LISTENER);

			// EMIT EVENTS //

			// emit NodeAddedToChannelListenerSet event.
			Self::deposit_event(Event::NodeAddedToChannelListenerSet(channel_id, listener));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A node is removed from the channel's listener-set by the channel's configurator.
		#[pallet::call_index(16)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(17_000_000, 0)
				.saturating_add(Weight::from_parts(0, 527805))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn node_removed_from_channel_listener_set(
			origin: OriginFor<T>,
			channel_id: u64,
			listener: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// Get the channel.
			let channel = <Channels<T>>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is a configurator for the channel.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// check that the listener is already added to the channel's listener-set.
			ensure!(
				channel.listeners.contains(&listener) == true,
				Error::<T>::NodeDoesNotExistInChannelListenerSet
			);

			// UPDATE STORAGE //

			// Verify Channels storage.
			let mut updated_channel = channel.clone();
			let bounded_existing_listeners: BoundedVec<
				T::AccountId,
				T::MaxListenersPerChannel,
			> = updated_channel
				.listeners
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxListenersPerChannelExceeded)?;
			let bounded_updated_listeners: BoundedVec<T::AccountId, T::MaxListenersPerChannel> =
				bounded_existing_listeners
					.into_iter()
					.filter(|x| x != &listener)
					.collect::<Vec<_>>()
					.try_into()
					.map_err(|_| Error::<T>::MaxListenersPerChannelExceeded)?;
			updated_channel.listeners = bounded_updated_listeners.into();

			// Verify ChannelsPerListenerMap storage.
			let channels_per_listener_map: BoundedVec<u64, T::MaxChannelsPerNode> =
				ChannelsPerListenerMap::<T>::get(listener.clone());
			// remove the channel_id from the channels_per_listener_map.
			let updated_channels_per_listener_map: BoundedVec<u64, T::MaxChannelsPerNode> =
				channels_per_listener_map
					.clone()
					.into_iter()
					.filter(|x| x != &channel_id)
					.collect::<Vec<_>>()
					.try_into()
					.map_err(|_| Error::<T>::MaxChannelsPerNodeExceeded)?;

			// Update Channels storage.
			<Channels<T>>::insert(channel_id.clone(), updated_channel);
			// update ChannelsPerListenerMap storage.
			ChannelsPerListenerMap::<T>::insert(listener.clone(), updated_channels_per_listener_map);

			// Clear listener membership in the reverse index.
			Self::membership_remove(&listener, channel_id, ROLE_LISTENER);

			// EMIT EVENTS //

			// emit NodeRemovedFromChannelListenerSet event.
			Self::deposit_event(Event::NodeRemovedFromChannelListenerSet(channel_id, listener));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's maker is updated by the channel's maker.
		#[pallet::call_index(17)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(15_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3514))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_maker_updated(
			origin: OriginFor<T>,
			channel_id: u64,
			new_maker: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the new_maker is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&new_maker), Error::<T>::InvalidNode);

			// SANITY CHECKS //

			// Get the channel.
			let channel = <Channels<T>>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's maker.
			ensure!(channel.maker == signer, Error::<T>::CallForbidden);

			// UPDATE STORAGE //

			// Verify Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.maker = new_maker.clone();

			// Update Channels storage.
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// emit ChannelMakerUpdated event.
			Self::deposit_event(Event::ChannelMakerUpdated(channel_id, new_maker));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's metadata is updated by the channel's maker.
		#[pallet::call_index(18)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(17_000_000, 0)
				.saturating_add(Weight::from_parts(0, 4030))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_metadata_updated(
			origin: OriginFor<T>,
			channel_id: u64,
			kuri: Vec<u8>,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the kuri is not longer than the allowed limit.
			let bounded_kuri: BoundedVec<u8, T::MaxKuriLength> =
				kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel maker node.
			ensure!(channel.maker == signer, Error::<T>::CallForbidden);

			// check that the arikuri points to this channel.
			ensure!(
				Arikuris::<T>::contains_key(channel_id.clone(), bounded_kuri.clone()),
				Error::<T>::ArikuriNotFound
			);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.metadata = bounded_kuri.clone().into();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelMetadataUpdated event.
			Self::deposit_event(Event::ChannelMetadataUpdated(channel_id, bounded_kuri.into()));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's functional metadata is updated by the channel's custodian or configurator.
		#[pallet::call_index(19)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(17_000_000, 0)
				.saturating_add(Weight::from_parts(0, 4030))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_functional_metadata_updated(
			origin: OriginFor<T>,
			channel_id: u64,
			kuri: Vec<u8>,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the kuri is not longer than the allowed limit.
			let bounded_kuri: BoundedVec<u8, T::MaxKuriLength> =
				kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel custodian node or is the configurator for
			// the channel.
			ensure!(
				channel.custodian == signer || channel.configurator == signer,
				Error::<T>::CallForbidden
			);

			// check that the arikuri points to this channel.
			ensure!(
				Arikuris::<T>::contains_key(channel_id.clone(), bounded_kuri.clone()),
				Error::<T>::ArikuriNotFound
			);

			// UPDATE STORAGE //

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.functional_metadata = bounded_kuri.clone().into();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelFunctionalMetadataUpdated event.
			Self::deposit_event(Event::ChannelFunctionalMetadataUpdated(channel_id, bounded_kuri.into()));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's actant requests to lock the channel's custodian metadata commit thread
		/// to its address.
		#[pallet::call_index(20)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(16_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3869))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_custodian_metadata_commit_thread_lock_requested(
			origin: OriginFor<T>,
			channel_id: u64,
		) -> DispatchResult {
			// INPUT VALIDATION //

			let signer = ensure_signed(origin)?;
			
			// check that the signer is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the signer is a committe_node for the channel.
			ensure!(channel.actants.contains(&signer) == true, Error::<T>::CallForbidden);
			// get the channel_custodian_metadata_commit_thread if it exists.
			let channel_custodian_metadata_commit_thread =
				<ChannelCustodianMetadataCommitThreads<T>>::get(channel_id.clone());

			let now = <frame_system::Pallet<T>>::block_number();
			let mut updated_channel_custodian_metadata_commit_thread;
			// check if the channel_custodian_metadata_commit_thread exists.
			if channel_custodian_metadata_commit_thread != None {
				updated_channel_custodian_metadata_commit_thread =
					channel_custodian_metadata_commit_thread.unwrap();
				// If locked, the lock may be taken over only once it is stale (held longer
				// than CommitLockTtl) — so a crashed holder can't freeze the book forever.
				if updated_channel_custodian_metadata_commit_thread.locked_by != Default::default() {
					let stale = match <CommitLockAcquiredAt<T>>::get(channel_id.clone()) {
						Some(a) => now > a + T::CommitLockTtl::get(),
						None => true,
					};
					ensure!(stale, Error::<T>::ChannelCustodianMetadataCommitThreadAlreadyLocked);
				}
				updated_channel_custodian_metadata_commit_thread.locked_by = Some(signer.clone());
			} else {
				updated_channel_custodian_metadata_commit_thread =
					ChannelCustodianMetadataCommitThreadInfo {
						channel_id: channel_id.clone(),
						latest_commit_kuri: Default::default(),
						latest_commit_transaction_hash: Default::default(),
						scribe: Default::default(),
						locked_by: Some(signer.clone()),
						latest_commit_block_number: Default::default(),
						latest_commit_size: Default::default(),
					};
			}
			// Record when the lock was (re)acquired, for stale-takeover.
			<CommitLockAcquiredAt<T>>::insert(channel_id.clone(), now);

			// UPDATE STORAGE //

			// Update ChannelCustodianMetadataCommitThreads storage.
			<ChannelCustodianMetadataCommitThreads<T>>::insert(
				channel_id.clone(),
				updated_channel_custodian_metadata_commit_thread,
			);

			// EMIT EVENTS //

			// Emit ChannelCustodianMetadataCommitThreadLockRequested event.
			Self::deposit_event(Event::ChannelCustodianMetadataCommitThreadLockRequested(
				channel_id, signer,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's actant requests to release the lock on the channel's custodian metadata
		/// commit thread from its address.
		#[pallet::call_index(21)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(17_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3869))
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn channel_custodian_metadata_commit_thread_lock_release_requested(
			origin: OriginFor<T>,
			channel_id: u64,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the signer is an actant for the channel.
			ensure!(channel.actants.contains(&signer) == true, Error::<T>::CallForbidden);
			// get the channel_custodian_metadata_commit_thread if it exists.
			let channel_custodian_metadata_commit_thread =
				<ChannelCustodianMetadataCommitThreads<T>>::get(channel_id.clone())
					.ok_or(Error::<T>::ChannelCustodianMetadataCommitThreadNotFound)?;

			// ensure that the signer is the locked_by address.
			ensure!(
				channel_custodian_metadata_commit_thread.locked_by == Some(signer.clone()),
				Error::<T>::ChannelCustodianMetadataCommitThreadNotLockedByNode
			);

			// UPDATE STORAGE //

			// Update ChannelCustodianMetadataCommitThreads storage.
			let mut updated_channel_custodian_metadata_commit_thread =
				channel_custodian_metadata_commit_thread.clone();
			updated_channel_custodian_metadata_commit_thread.locked_by = Default::default();
			<ChannelCustodianMetadataCommitThreads<T>>::insert(
				channel_id.clone(),
				updated_channel_custodian_metadata_commit_thread,
			);

			// EMIT EVENTS //

			// Emit ChannelCustodianMetadataCommitThreadLockReleased event.
			Self::deposit_event(Event::ChannelCustodianMetadataCommitThreadLockReleased(
				channel_id, signer,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// The root releases the lock on a channel's custodian metadata commit thread.
		#[pallet::call_index(22)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(14_000_000, 0)
				.saturating_add(Weight::from_parts(0, 3869))
				.saturating_add(T::DbWeight::get().reads(2))
				.saturating_add(T::DbWeight::get().writes(1))
		)]
		pub fn force_release_channel_custodian_metadata_commit_thread_lock(
			origin: OriginFor<T>,
			channel_id: u64,
			node: T::AccountId,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is root.
			ensure_root(origin)?;

			// SANITY CHECKS //

			// get the channel.
			let _channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// get the channel_custodian_metadata_commit_thread if it exists.
			let channel_custodian_metadata_commit_thread =
				<ChannelCustodianMetadataCommitThreads<T>>::get(channel_id.clone())
					.ok_or(Error::<T>::ChannelCustodianMetadataCommitThreadNotFound)?;

			// ensure that the node is the locked_by address.
			ensure!(
				channel_custodian_metadata_commit_thread.locked_by == Some(node.clone()),
				Error::<T>::ChannelCustodianMetadataCommitThreadNotLockedByNode
			);

			// UPDATE STORAGE //

			// Update ChannelCustodianMetadataCommitThreads storage.
			let mut updated_channel_custodian_metadata_commit_thread =
				channel_custodian_metadata_commit_thread.clone();
			updated_channel_custodian_metadata_commit_thread.locked_by = Default::default();
			<ChannelCustodianMetadataCommitThreads<T>>::insert(
				channel_id.clone(),
				updated_channel_custodian_metadata_commit_thread,
			);

			// EMIT EVENTS //

			// Emit ChannelCustodianMetadataCommitThreadLockReleased event.
			Self::deposit_event(Event::ChannelCustodianMetadataCommitThreadLockReleased(
				channel_id, node,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// A channel's custodian metadata is updated by the channel's custodian or actant.
		#[pallet::call_index(23)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(22_000_000, 0)
				.saturating_add(Weight::from_parts(0, 4030))
				.saturating_add(T::DbWeight::get().reads(4))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn channel_custodian_metadata_updated(
			origin: OriginFor<T>,
			channel_id: u64,
			from_kuri: Vec<u8>,
			to_kuri: Vec<u8>,
			commit_kuri: Vec<u8>,
			commit_transaction_hash: T::Hash,
			commit_block_number: BlockNumberFor<T>,
			commit_size: u64,
			release_lock: bool,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that from_kuri is not longer than the allowed limit.
			let bounded_from_kuri: BoundedVec<u8, T::MaxKuriLength> =
				from_kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// check that the to_kuri is not longer than the allowed limit.
			let bounded_to_kuri: BoundedVec<u8, T::MaxKuriLength> =
				to_kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// check that the commit_kuri is not longer than the allowed limit.
			let bounded_commit_kuri: BoundedVec<u8, T::MaxKuriLength> =
				commit_kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// check that the commit_size is not longer than the allowed limit.
			ensure!(commit_size <= T::MaxCommitSize::get(), Error::<T>::MaxCommitSizeExceeded);

			// SANITY CHECKS //

			// get the channel.
			let channel = Channels::<T>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's custodian or actant.
			ensure!(
				channel.custodian == signer ||
					channel.actants.contains(&signer) == true,
				Error::<T>::CallForbidden
			);
			// if channel custodian metadata is set, ensure that the bounded_from_kuri is the same as
			// the channel's custodian_metadata.
			if channel.custodian_metadata != Default::default() {
				ensure!(
					channel.custodian_metadata == bounded_from_kuri.clone().into(),
					Error::<T>::ChannelCustodianMetadataMismatch
				);
			}

			// check that the arikuri points to this channel.
			ensure!(
				Arikuris::<T>::contains_key(channel_id.clone(), bounded_to_kuri.clone()),
				Error::<T>::ArikuriNotFound
			);

			// get the channel_custodian_metadata_commit_thread.
			let channel_custodian_metadata_commit_thread =
				<ChannelCustodianMetadataCommitThreads<T>>::get(channel_id.clone())
					.ok_or(Error::<T>::ChannelCustodianMetadataCommitThreadNotFound)?;

			// ensure that the channel_custodian_metadata_commit_thread is locked by the signer.
			ensure!(
				channel_custodian_metadata_commit_thread.locked_by == Some(signer.clone()),
				Error::<T>::ChannelCustodianMetadataCommitThreadNotLockedByNode
			);

			// UPDATE STORAGE //

			// Update ChannelCustomMetadataCommitThreads storage.
			let mut updated_channel_custodian_metadata_commit_thread =
				channel_custodian_metadata_commit_thread.clone();
			updated_channel_custodian_metadata_commit_thread.latest_commit_kuri = bounded_commit_kuri
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;
			updated_channel_custodian_metadata_commit_thread.latest_commit_transaction_hash =
				Some(commit_transaction_hash.clone());
			if release_lock {
				updated_channel_custodian_metadata_commit_thread.locked_by = Default::default();
			}
			updated_channel_custodian_metadata_commit_thread.scribe = Some(signer.clone());
			updated_channel_custodian_metadata_commit_thread.latest_commit_block_number =
				Some(commit_block_number.clone());
			updated_channel_custodian_metadata_commit_thread.latest_commit_size = Some(commit_size);

			<ChannelCustodianMetadataCommitThreads<T>>::insert(
				channel_id.clone(),
				updated_channel_custodian_metadata_commit_thread,
			);

			// Update Channels storage.
			let mut updated_channel = channel.clone();
			updated_channel.custodian_metadata = bounded_to_kuri.clone().into();
			<Channels<T>>::insert(channel_id.clone(), updated_channel);

			// EMIT EVENTS //

			// Emit ChannelCustodianMetadataUpdated event.
			Self::deposit_event(Event::ChannelCustodianMetadataUpdated(
				channel_id,
				bounded_from_kuri.into(),
				bounded_to_kuri.into(),
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		////// ARIKuri FUNCTIONS //////

		/// A new arikuri is added by a channel's actant.
		#[pallet::call_index(24)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(
			Weight::from_parts(18_000_000, 0)
				.saturating_add(Weight::from_parts(0, 4030))
				.saturating_add(T::DbWeight::get().reads(4))
				.saturating_add(T::DbWeight::get().writes(2))
		)]
		pub fn arikuri_added(origin: OriginFor<T>, kuri: Vec<u8>, channel_id: u64) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the code is not longer than the allowed limit.
			let bounded_code: BoundedVec<u8, T::MaxKuriLength> =
				kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// SANITY CHECKS //

			// check that the arikuri does not already exist.
			ensure!(
				Arikuris::<T>::get(channel_id.clone(), bounded_code.clone()) == None,
				Error::<T>::ArikuriAlreadyAdded
			);

			// get the channel.
			let channel = <Channels<T>>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the signer is the channel's actant.
			ensure!(
				channel.actants.contains(&signer) == true,
				Error::<T>::NodeDoesNotExistInChannelActantSet
			);
			// Check that the channel is not archived.
			ensure!(channel.archived.eq(&false), Error::<T>::ChannelAlreadyArchived);
			// check that the channel is not paused.
			ensure!(channel.paused.eq(&false), Error::<T>::ChannelAlreadyPaused);

			// UPDATE STORAGE //

			// create a new arikuri.
			let new_arikuri = ArikuriInfo {
				kuri: bounded_code.clone(),
				channel_id: channel_id.clone(),
				deleted: false,
			};

			// update Arikuris storage.
			Arikuris::<T>::insert(channel_id.clone(), bounded_code.clone(), new_arikuri);

			// update TotalArikuris storage.
			TotalArikuris::<T>::mutate(channel_id.clone(), |total_arikuris| *total_arikuris += 1);

			// EMIT EVENTS //

			// emit ArikuriCreated event.
			Self::deposit_event(Event::ArikuriCreated(channel_id, bounded_code));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// An arikuri is deleted by a channel's configurator.
		#[pallet::call_index(25)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn arikuri_deleted(
			origin: OriginFor<T>,
			kuri: Vec<u8>,
			channel_id: u64,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is added to the scribe-set.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the code is not longer than the allowed limit.
			let bounded_arikuri: BoundedVec<u8, T::MaxKuriLength> =
				kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// SANITY CHECKS //

			// get the channel.
			let channel = <Channels<T>>::get(channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;

			// check that the signer is the channel's configurator.
			ensure!(channel.configurator == signer, Error::<T>::CallForbidden);

			// Check that the channel is not archived.
			ensure!(channel.archived.eq(&false), Error::<T>::ChannelAlreadyArchived);

			// check that the channel is not paused.
			ensure!(channel.paused.eq(&false), Error::<T>::ChannelAlreadyPaused);

			// get the arikuri.
			let arikuri = Arikuris::<T>::get(channel_id.clone(), bounded_arikuri.clone())
				.ok_or(Error::<T>::ArikuriNotFound)?;
			// check that the arikuri is not already deleted.
			ensure!(arikuri.deleted.eq(&false), Error::<T>::ArikuriAlreadyDeleted);

			// UPDATE STORAGE //

			// mark the arikuri as deleted.
			let mut deleted_arikuri = arikuri;
			deleted_arikuri.deleted = true;

			// update Arikuris storage.
			Arikuris::<T>::insert(channel_id.clone(), bounded_arikuri.clone(), deleted_arikuri);

			// EMIT EVENTS //

			// emit ArikuriDeleted event.
			Self::deposit_event(Event::ArikuriDeleted(channel_id, bounded_arikuri));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// Arikuri transfers are accepted by a recipient channel's custodian.
		#[pallet::call_index(26)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn arikuri_transfers_accepted(
			origin: OriginFor<T>,
			kuris: Kuris<T>,
			from_channel_id: u64,
			to_channel_id: u64,
		) -> DispatchResult {
			// INPUT VALIDATION //

			let signer = ensure_signed(origin)?;

			// check that the signer is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the kuris are not longer than the allowed limit.
			let bounded_kuris: BoundedVec<
				BoundedVec<u8, T::MaxKuriLength>,
				T::MaxArikurisToTransfer,
			> = kuris.try_into().map_err(|_| Error::<T>::MaxTransferableArikurisLimitExceeded)?;

			// SANITY CHECKS //

			// get the from channel.
			let from_channel =
				Channels::<T>::get(from_channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the from_channel is not archived.
			ensure!(from_channel.archived.eq(&false), Error::<T>::FromChannelAlreadyArchived);
			// check that the from_channel is not paused.
			ensure!(from_channel.paused.eq(&false), Error::<T>::FromChannelAlreadyPaused);

			// get the to channel.
			let to_channel =
				Channels::<T>::get(to_channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the to_channel is not archived.
			ensure!(to_channel.archived.eq(&false), Error::<T>::ToChannelAlreadyArchived);
			// check that the to_channel is not paused.
			ensure!(to_channel.paused.eq(&false), Error::<T>::ToChannelAlreadyPaused);
			// check that signer is a configurator for the to channel.
			ensure!(to_channel.configurator == signer, Error::<T>::CallForbidden);

			// loop through the kuris.
			for kuri in bounded_kuris.clone() {
				// check that the kuri is not longer than the allowed limit.
				let bounded_kuri: BoundedVec<u8, T::MaxKuriLength> =
					kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

				// get the arikuri.
				let arikuri = Arikuris::<T>::get(from_channel_id.clone(), bounded_kuri.clone())
					.ok_or(Error::<T>::ArikuriNotFound)?;
				// check that the arikuri is not already deleted.
				ensure!(arikuri.deleted.eq(&false), Error::<T>::ArikuriAlreadyDeleted);

				// UPDATE STORAGE //

				// Update ArikuriTransferAccepted storage.
				ArikuriTransferAccepted::<T>::insert(
					to_channel_id.clone(),
					bounded_kuri.clone(),
					true,
				);
			}

			// EMIT EVENTS //

			// emit ArikuriTransferAccepted event.
			Self::deposit_event(Event::ArikuriTransferAccepted(
				from_channel_id,
				to_channel_id,
				bounded_kuris,
				signer,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// Arikuris are transferred from one channel to another by the configurator of the from
		/// channel.
		#[pallet::call_index(27)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn arikuris_transferred(
			origin: OriginFor<T>,
			kuris: Kuris<T>,
			from_channel_id: u64,
			to_channel_id: u64,
		) -> DispatchResult {
			// INPUT VALIDATION //

			let signer = ensure_signed(origin)?;

			// check that the signer is added to the scribe-set.
			ensure!(Self::is_node_in_scribe_set(&signer), Error::<T>::CallForbidden);

			// check that the kuri list is valid.
			let bounded_kuris: BoundedVec<
				BoundedVec<u8, T::MaxKuriLength>,
				T::MaxArikurisToTransfer,
			> = kuris.try_into().map_err(|_| Error::<T>::MaxTransferableArikurisLimitExceeded)?;

			// SANITY CHECKS //

			// get the from channel.
			let from_channel =
				Channels::<T>::get(from_channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the from_channel is not archived.
			ensure!(from_channel.archived.eq(&false), Error::<T>::FromChannelAlreadyArchived);
			// check that the from_channel is not paused.
			ensure!(from_channel.paused.eq(&false), Error::<T>::FromChannelAlreadyPaused);
			// ensure that the signer is a configurator for the from channel.
			ensure!(from_channel.configurator == signer, Error::<T>::CallForbidden);

			// get the to channel.
			let to_channel =
				Channels::<T>::get(to_channel_id.clone()).ok_or(Error::<T>::ChannelNotFound)?;
			// check that the to_channel is not archived.
			ensure!(to_channel.archived.eq(&false), Error::<T>::ToChannelAlreadyArchived);
			// check that the to_channel is not paused.
			ensure!(to_channel.paused.eq(&false), Error::<T>::ToChannelAlreadyPaused);

			// loop through the kuris.
			for kuri in bounded_kuris.clone() {
				// check that the kuri is not longer than the allowed limit.
				let bounded_kuri: BoundedVec<u8, T::MaxKuriLength> =
					kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

				// get the arikuri.
				let arikuri = Arikuris::<T>::get(from_channel_id.clone(), bounded_kuri.clone())
					.ok_or(Error::<T>::ArikuriNotFound)?;
				// check that the arikuri is not already deleted.
				ensure!(arikuri.deleted.eq(&false), Error::<T>::ArikuriAlreadyDeleted);

				// check that the arikuri transfer has been accepted.
				ensure!(
					ArikuriTransferAccepted::<T>::get(to_channel_id.clone(), bounded_kuri.clone())
						.eq(&true),
					Error::<T>::ArikuriTransferNotAccepted
				);

				// UPDATE STORAGE //

				// Update Arikuris storage.
				let transferred_arikuri = ArikuriInfo {
					kuri: bounded_kuri.clone(),
					channel_id: to_channel_id.clone(),
					deleted: false,
				};

				// create a new arikuri for to_channel_id.
				Arikuris::<T>::insert(
					to_channel_id.clone(),
					bounded_kuri.clone(),
					transferred_arikuri,
				);
				// delete the arikuri for from_channel_id.
				Arikuris::<T>::remove(from_channel_id.clone(), bounded_kuri.clone());
			}

			// UPDATE STORAGE (contd) //
			// increment TotalArikuris storage for to_channel_id by length of bounded_kuris.
			TotalArikuris::<T>::mutate(to_channel_id.clone(), |total_arikuris| {
				*total_arikuris += bounded_kuris.len() as u64
			});
			// decrement TotalArikuris storage for from_channel_id by length of bounded_kuris.
			TotalArikuris::<T>::mutate(from_channel_id.clone(), |total_arikuris| {
				*total_arikuris -= bounded_kuris.len() as u64
			});

			// EMIT EVENTS //

			// emit ArikurisTransferred event.
			Self::deposit_event(Event::ArikurisTransferred(
				from_channel_id,
				to_channel_id,
				bounded_kuris,
			));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}

		/// An arikuri is updated by the root.
		#[pallet::call_index(28)]
		// #[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn force_update_arikuri(
			origin: OriginFor<T>,
			kuri: Vec<u8>,
			channel_id: u64,
			deleted: bool,
		) -> DispatchResult {
			// INPUT VALIDATION //

			// check that the signer is the root.
			ensure_root(origin)?;

			// check that the kuri is not longer than the allowed limit.
			let bounded_kuri: BoundedVec<u8, T::MaxKuriLength> =
				kuri.try_into().map_err(|_| Error::<T>::MaxKuriLengthExceeded)?;

			// SANITY CHECKS //

			// check that the arikuri exists.
			ensure!(
				Arikuris::<T>::contains_key(channel_id.clone(), bounded_kuri.clone()),
				Error::<T>::ArikuriNotFound
			);

			// UPDATE STORAGE //

			// Update Arikuris storage.
			let updated_arikuri =
				ArikuriInfo { kuri: bounded_kuri.clone(), channel_id: channel_id.clone(), deleted };

			Arikuris::<T>::insert(channel_id.clone(), bounded_kuri.clone(), updated_arikuri);

			// EMIT EVENTS //

			// emit ArikuriUpdatedByRoot event.
			Self::deposit_event(Event::ArikuriUpdatedByRoot(bounded_kuri));

			// RETURN SUCCESSFUL DISPATCHRESULT //
			Ok(())
		}
	}
}
