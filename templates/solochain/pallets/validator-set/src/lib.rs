#![cfg_attr(not(feature = "std"), no_std)]

//! # Validator Set pallet
//!
//! A deliberately small pallet that owns the active validator set for the
//! metarium solochain and exposes it to `pallet-session` as a
//! [`SessionManager`](pallet_session::SessionManager).
//!
//! Validators are added/removed by a configurable origin (sudo / root on
//! metarium today). When the set changes, the change is staged and applied at
//! the next session rotation, at which point `pallet-aura` (block production)
//! and `pallet-grandpa` (finality) pick up the new authorities via the session
//! key plumbing. This is what makes "spin up a new validator on demand"
//! possible without a chain restart or coordinated fork:
//!
//!   1. operator boots a new node and rotates session keys (`author_rotateKeys`)
//!   2. the validator account submits `session.setKeys`
//!   3. sudo calls `validatorSet.addValidator(account)`
//!   4. at the next session the node begins authoring & finalizing.

pub use pallet::*;

pub use weights::WeightInfo;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin allowed to add and remove validators (root / sudo on metarium).
		type AddRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The minimum number of validators the set may shrink to. The set can
		/// never be reduced below this, so finality and authoring cannot stall.
		#[pallet::constant]
		type MinValidators: Get<u32>;

		/// The maximum number of validators the set may grow to.
		#[pallet::constant]
		type MaxValidators: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The current active validator set.
	#[pallet::storage]
	#[pallet::getter(fn validators)]
	pub type Validators<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxValidators>, ValueQuery>;

	/// Set to `true` whenever the validator set changes, so the next
	/// `new_session` hands the updated set to `pallet-session`.
	#[pallet::storage]
	pub type Flag<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A validator was added and will be active next session.
		ValidatorAdded { account: T::AccountId },
		/// A validator was removed and will be inactive next session.
		ValidatorRemoved { account: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The account is already in the validator set.
		AlreadyValidator,
		/// The account is not in the validator set.
		NotValidator,
		/// Adding the account would exceed `MaxValidators`.
		TooManyValidators,
		/// Removing the account would drop the set below `MinValidators`.
		TooFewValidators,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_validators: Vec<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { initial_validators: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				(self.initial_validators.len() as u32) >= T::MinValidators::get(),
				"validator-set: fewer initial validators than MinValidators"
			);
			let bounded: BoundedVec<T::AccountId, T::MaxValidators> = self
				.initial_validators
				.clone()
				.try_into()
				.expect("validator-set: more initial validators than MaxValidators");
			Validators::<T>::put(bounded);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a validator to the set. Takes effect at the next session.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_validator())]
		pub fn add_validator(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;
			Validators::<T>::try_mutate(|vals| -> DispatchResult {
				ensure!(!vals.contains(&who), Error::<T>::AlreadyValidator);
				vals.try_push(who.clone()).map_err(|_| Error::<T>::TooManyValidators)?;
				Ok(())
			})?;
			Flag::<T>::put(true);
			Self::deposit_event(Event::ValidatorAdded { account: who });
			Ok(())
		}

		/// Remove a validator from the set. Takes effect at the next session.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::remove_validator())]
		pub fn remove_validator(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;
			Validators::<T>::try_mutate(|vals| -> DispatchResult {
				ensure!(
					(vals.len() as u32) > T::MinValidators::get(),
					Error::<T>::TooFewValidators
				);
				let pos =
					vals.iter().position(|v| v == &who).ok_or(Error::<T>::NotValidator)?;
				vals.remove(pos);
				Ok(())
			})?;
			Flag::<T>::put(true);
			Self::deposit_event(Event::ValidatorRemoved { account: who });
			Ok(())
		}
	}
}

/// Hand the active validator set to `pallet-session`. The set is only returned
/// when it has changed (or at genesis), which avoids needless session churn.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: u32) -> Option<sp_std::vec::Vec<T::AccountId>> {
		if Flag::<T>::get() || new_index == 0 {
			Flag::<T>::put(false);
			Some(Validators::<T>::get().into_inner())
		} else {
			None
		}
	}

	fn end_session(_end_index: u32) {}

	fn start_session(_start_index: u32) {}
}
