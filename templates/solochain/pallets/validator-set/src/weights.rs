//! Weights for `pallet-validator-set`.
//!
//! These are intentionally simple constant weights. The pallet's extrinsics are
//! sudo-only and infrequent (operations management), so precise benchmarking is
//! not warranted; the values below comfortably cover the single bounded-vec
//! read/write each call performs.

use frame_support::weights::{constants::RocksDbWeight, Weight};

pub trait WeightInfo {
	fn add_validator() -> Weight;
	fn remove_validator() -> Weight;
}

/// Default weights backed by the RocksDB weight constants (1 read + 2 writes).
impl WeightInfo for () {
	fn add_validator() -> Weight {
		Weight::from_parts(20_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}

	fn remove_validator() -> Weight {
		Weight::from_parts(20_000_000, 0)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
}
