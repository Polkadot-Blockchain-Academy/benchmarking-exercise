//! Benchmarking setup for pallet-identity
//!
//! This module contains comprehensive benchmarks for the Identity pallet,
//! designed to showcase various benchmarking patterns and complexities:
//!
//! 1. **Linear complexity** - `set_identity` scales with identity data size O(n)
//! 2. **Logarithmic complexity** - `provide_judgement_inline` uses binary search O(log n)
//! 3. **Storage pattern comparison** - Different usage patterns with one unified clear extrinsic:
//!    - `provide_judgement_inline`: Uses BoundedVec with O(log n) binary search
//!    - `provide_judgement_double_map`: Uses DoubleMap with O(1) insertion
//!    - `clear_identity`: Single extrinsic with complexity depending on prior usage
//!      - `clear_identity_inline_usage`: Effectively O(1) cleanup when only inline judgements used
//!      - `clear_identity_double_map_usage`: O(n) cleanup where n = actual double map judgements
//! 4. **Economic operations** - Currency operations (reserve, unreserve)
//! 5. **Vector operations** - Sorted insertion and binary search in bounded collections
//! 6. **Storage operations** - Multiple storage interactions with proper state management
//!
//! ## Learning Objectives
//!
//! - Understanding different complexity patterns (linear vs logarithmic vs constant)
//! - **Storage design tradeoffs** - Comparing BoundedVec vs DoubleMap performance
//! - Using multiple complexity parameters (b for bytes, j for judgements)
//! - Measuring worst-case execution paths for different algorithms
//! - **Real-world performance implications** - Why storage choice matters for cleanup operations
//! - Binary search benchmarking with sorted data structures
//! - Vector operations with bounded collections
//! - Verifying benchmark correctness with comprehensive assertions

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::{Config, IdentityInfo, Judgement, Pallet as Identity};
use frame_benchmarking::v2::*;
use frame_support::{
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_std::vec;

/// Create a reasonable identity info for benchmarking
/// This helper demonstrates how to set up test data for benchmarks
fn create_identity_info<T: Config>(bytes: u32) -> IdentityInfo<T::MaxFieldLength> {
	let data = vec![b'X'; bytes as usize];
	let bounded_data = BoundedVec::try_from(data).expect("BoundedVec input too long.");

	IdentityInfo {
		display: bounded_data.clone(),
		legal: bounded_data.clone(),
		web: bounded_data.clone(),
		email: bounded_data,
	}
}

/// Fund an account with enough balance for benchmarking operations
/// This helper ensures accounts have sufficient funds for deposits
fn fund_account<T: Config>(account: &T::AccountId) {
	let min_balance = T::Currency::minimum_balance();
	let deposit_required = T::BasicDeposit::get() + T::ByteDeposit::get() * 1000u32.into();
	let total = min_balance + deposit_required;
	T::Currency::make_free_balance_be(account, total);
}

#[benchmarks]
mod benchmarks {
	use super::*;

	/// Benchmark: set_identity
	///
	/// Complexity: Linear in the number of bytes of identity information (b)
	/// This benchmark demonstrates:
	/// - Linear complexity with respect to data size
	/// - Economic operations (currency reservation)
	/// - Storage operations (conditional insertion/update)
	/// - Event emission
	#[benchmark]
	fn set_identity(
		// Parameter 'b' represents the number of bytes in the identity info
		// This creates a linear relationship between input size and execution time
		b: Linear<1, { T::MaxFieldLength::get() }>,
	) {
		let caller: T::AccountId = whitelisted_caller();
		fund_account::<T>(&caller);

		let identity_info = create_identity_info::<T>(b);
		let expected_deposit = T::BasicDeposit::get() +
			T::ByteDeposit::get() * u32::from(identity_info.encoded_size()).into();

		#[extrinsic_call]
		set_identity(
			RawOrigin::Signed(caller.clone()),
			identity_info.display.clone(),
			identity_info.legal.clone(),
			identity_info.web.clone(),
			identity_info.email.clone(),
		);

		// Verify the benchmark worked correctly
		let registration = IdentityOf::<T>::get(&caller).unwrap();
		assert_eq!(registration.info, identity_info);
		assert_eq!(registration.deposit, expected_deposit);
		assert_eq!(registration.judgements.len(), 0);
		assert_eq!(T::Currency::reserved_balance(&caller), expected_deposit);
	}

	/// Benchmark: set_identity_update
	///
	/// This benchmark tests the update path when an identity already exists with judgements.
	/// It demonstrates the worst case where we have maximum inline judgements that need to be
	/// filtered for sticky ones. This measures the cost of retaining sticky judgements.
	#[benchmark]
	fn set_identity_update(
		b: Linear<1, { T::MaxFieldLength::get() }>,
		j: Linear<0, { T::MaxJudgements::get() }>, // Number of existing judgements
	) {
		let caller: T::AccountId = whitelisted_caller();
		fund_account::<T>(&caller);

		// Pre-condition: set an initial identity
		let initial_info = create_identity_info::<T>(b / 2);
		let _ = Identity::<T>::set_identity(
			RawOrigin::Signed(caller.clone()).into(),
			initial_info.display,
			initial_info.legal,
			initial_info.web,
			initial_info.email,
		);

		// Add maximum judgements (mix of sticky and non-sticky) for worst case
		for i in 0..j {
			// Alternate between sticky (KnownGood/Erroneous) and non-sticky (Reasonable/LowQuality)
			let judgement_type = if i % 2 == 0 { 2 } else { 1 }; // KnownGood or Reasonable
			let _ = Identity::<T>::provide_judgement_inline(
				RawOrigin::Root.into(),
				i,
				caller.clone(),
				judgement_type,
			);
		}

		let new_identity_info = create_identity_info::<T>(b);

		#[extrinsic_call]
		set_identity(
			RawOrigin::Signed(caller.clone()),
			new_identity_info.display.clone(),
			new_identity_info.legal.clone(),
			new_identity_info.web.clone(),
			new_identity_info.email.clone(),
		);

		// Verify the update worked and sticky judgements were retained
		let registration = IdentityOf::<T>::get(&caller).unwrap();
		assert_eq!(registration.info, new_identity_info);
		// Should have roughly half the judgements (only sticky ones retained)
		assert!(registration.judgements.len() <= j as usize);
	}

	/// Benchmark: provide_judgement_inline
	///
	/// This benchmark tests providing a judgement using inline storage (BoundedVec).
	/// Complexity:
	/// - Linear `O(b + j)` complexity in terms of encoding and decoding (with j being the number of
	///   judgements).
	/// - Logarithmic `O(log j)` complexity in the number of existing judgements for binary search
	///   at insertion.
	/// - Constant `O(1)` complexity in terms of storage reads and writes.
	#[benchmark]
	fn provide_judgement_inline(
		b: Linear<1, { T::MaxFieldLength::get() }>,
		j: Linear<0, { T::MaxJudgements::get() - 1 }>,
	) {
		let target: T::AccountId = account("target", 0, 0);
		fund_account::<T>(&target);

		// Pre-condition: set up identity
		let identity_info = create_identity_info::<T>(b);
		let _ = Identity::<T>::set_identity(
			RawOrigin::Signed(target.clone()).into(),
			identity_info.display,
			identity_info.legal,
			identity_info.web,
			identity_info.email,
		);

		// Add existing judgements to create worst-case binary search scenario
		// We'll add judgements with IDs 1, 3, 5, 7, ... (odd numbers)
		// Then insert with ID 0 to test binary search at the beginning
		for i in 0..j {
			let judgement_id = (i * 2) + 1; // Creates IDs: 1, 3, 5, 7, ...
			IdentityOf::<T>::mutate(&target, |maybe_reg| {
				if let Some(ref mut reg) = maybe_reg {
					let _ = reg.judgements.try_push((judgement_id, Judgement::Reasonable));
				}
			});
		}

		let new_judgement_id = 0u32; // This will be inserted at position 0
		let judgement_type = 2u8; // KnownGood

		#[extrinsic_call]
		provide_judgement_inline(RawOrigin::Root, new_judgement_id, target.clone(), judgement_type);

		// Verify judgement was provided and inserted correctly
		let registration = IdentityOf::<T>::get(&target).unwrap();
		assert_eq!(registration.judgements.len(), (j + 1) as usize);
		assert_eq!(registration.judgements[0], (new_judgement_id, Judgement::KnownGood));
		// Verify ordering is maintained
		for i in 1..registration.judgements.len() {
			assert!(registration.judgements[i - 1].0 < registration.judgements[i].0);
		}
	}

	/// Benchmark: provide_judgement_double_map
	///
	/// This benchmark tests providing a judgement using double map storage
	/// Complexity: Linear `O(b)` complexity, but independent of number of judgements!
	///
	/// NOTE: We ignore the possibility of inline judgements for illustration purposes. This is
	/// meant to showcase an alternative implementation after all. If we actually had both
	/// implementation in parallel - inadvisable - we would need to cover the worst case of `j`
	/// inline judgements as well.
	#[benchmark]
	fn provide_judgement_double_map(
		b: Linear<1, { T::MaxFieldLength::get() }>,
		j: Linear<0, { T::MaxJudgements::get() - 1 }>,
	) {
		let target: T::AccountId = account("target", 0, 0);
		fund_account::<T>(&target);

		// Pre-condition: set up identity
		let identity_info = create_identity_info::<T>(b);
		let _ = Identity::<T>::set_identity(
			RawOrigin::Signed(target.clone()).into(),
			identity_info.display,
			identity_info.legal,
			identity_info.web,
			identity_info.email,
		);

		// Add existing judgements using the proper extrinsic
		for i in 0..j {
			let _ = Identity::<T>::provide_judgement_double_map(
				RawOrigin::Root.into(),
				i,
				target.clone(),
				1, // Reasonable
			);
		}

		let new_judgement_id = j; // This will be a new entry
		let judgement_type = 2u8; // KnownGood

		#[extrinsic_call]
		provide_judgement_double_map(
			RawOrigin::Root,
			new_judgement_id,
			target.clone(),
			judgement_type,
		);

		// Verify judgement was provided
		assert_eq!(
			JudgementsDoubleMap::<T>::get(&target, new_judgement_id),
			Some(Judgement::KnownGood)
		);
		// Verify other judgements still exist
		for i in 0..j {
			assert_eq!(JudgementsDoubleMap::<T>::get(&target, i), Some(Judgement::Reasonable));
		}
	}

	/// Benchmark: clear_identity_inline_usage
	//
	// Implement this benchmark taking into account best practices and the complexity of the code
	// and storage.
	#[benchmark]
	fn clear_identity_inline_usage(
		b: Linear<1, { T::MaxFieldLength::get() }>, // TODO: determine if necessary
		j: Linear<0, { T::MaxJudgements::get() }>,  // TODO: determine if necessary
	) {
		// TODO: implement
		#[block]
		{}
	}

	/// Benchmark: clear_identity_double_map_usage
	//
	// Implement this benchmark taking into account best practices and the complexity of the code
	// and storage.
	#[benchmark]
	fn clear_identity_double_map_usage(
		b: Linear<1, { T::MaxFieldLength::get() }>, // TODO: determine if necessary
		j: Linear<0, { T::MaxJudgements::get() }>,  // TODO: determine if necessary
	) {
		// TODO: implement
		#[block]
		{}
	}

	impl_benchmark_test_suite!(Identity, crate::mock::new_test_ext(), crate::mock::Test);
}
