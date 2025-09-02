//! # Simplified Identity Pallet
//!
//! A simplified version of the Identity pallet designed for benchmarking exercises.
//!
//! This pallet provides basic identity management functionality:
//! - Set identity information with configurable fields
//! - Clear identity information
//! - Provide judgements (from a configurable origin)
//! - Force operations (admin functions)
//!
//! ## Overview
//!
//! This pallet allows users to set identity information that can be verified.
//! Users pay deposits for storing identity information, and verifiers can provide judgements
//! about the validity of identities.
//!
//! ### Key Features
//! - **Identity Information**: Users can set display name, legal name, web, email etc.
//! - **Judgement System**: Configurable origin can verify identity information
//! - **Deposits**: Economic mechanism to prevent spam and ensure data quality
//! - **Judgements**: Verification opinions on identity validity
//!
//! ## Benchmarking Focus
//!
//! This enhanced version demonstrates multiple complexity patterns for comprehensive benchmarking
//! education:
//!
//! ### Linear Complexity Patterns:
//! - **`set_identity`**: Scales with identity data size O(b) where b = total bytes
//! - **`clear_identity`**: Scales with number of judgements O(j) during cleanup
//!
//! ### Logarithmic Complexity Patterns:
//! - **`provide_judgement`**: Binary search and sorted insertion O(log j)
//!
//! ### Key Learning Features:
//! - Multiple complexity parameters (b = bytes, j = judgements)
//! - **Configurable bounds**: MaxJudgements parameter controls vector size limits
//! - Binary search operations in sorted bounded vectors
//! - Economic operations with deposit calculations
//! - Conditional logic (sticky vs non-sticky judgements)
//! - Vector operations with maintained ordering
//! - Storage cleanup with linear time complexity

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Get, ReservableCurrency},
	BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::vec;

pub use weights::WeightInfo;

/// Identity information that can be set by users
#[derive(
	Encode,
	Decode,
	Default,
	CloneNoBound,
	PartialEqNoBound,
	Eq,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(MaxFieldLength))]
pub struct IdentityInfo<MaxFieldLength: Get<u32>> {
	/// A reasonable display name for the controller of the account.
	pub display: BoundedVec<u8, MaxFieldLength>,
	/// The full legal name in the local jurisdiction of the entity.
	pub legal: BoundedVec<u8, MaxFieldLength>,
	/// A representative website field.
	pub web: BoundedVec<u8, MaxFieldLength>,
	/// An email address.
	pub email: BoundedVec<u8, MaxFieldLength>,
}

impl<MaxFieldLength: Get<u32>> IdentityInfo<MaxFieldLength> {
	/// Get the encoded size of this identity info
	pub fn encoded_size(&self) -> u32 {
		self.encode().len() as u32
	}
}

/// Judgement provided by verifiers
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Judgement {
	/// The default value; no opinion is held.
	Unknown,
	/// The target is known and the identity is reasonable.
	Reasonable,
	/// The target is known and the identity is good.
	KnownGood,
	/// The target is known and the identity is erroneous.
	Erroneous,
	/// An erroneous identity may be corrected.
	LowQuality,
}

impl Judgement {
	/// Returns true if this judgement is "sticky" (cannot be removed except by complete
	/// removal of the identity or by the verifier).
	pub fn is_sticky(&self) -> bool {
		matches!(self, Judgement::KnownGood | Judgement::Erroneous)
	}
}

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type JudgementId = u32;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	/// Information concerning the identity of the controller of an account.
	#[derive(
		Encode,
		Decode,
		CloneNoBound,
		PartialEqNoBound,
		Eq,
		RuntimeDebugNoBound,
		MaxEncodedLen,
		TypeInfo,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct Registration<T: Config> {
		/// Information about the identity.
		pub info: IdentityInfo<T::MaxFieldLength>,
		/// Judgements on this identity. Stored as (judgement_id, judgement) pairs, ordered by ID.
		pub judgements: BoundedVec<(u32, Judgement), T::MaxJudgements>,
		/// Count of judgements stored in the double map (for educational comparison).
		pub judgements_count_double_map: u32,
		/// Amount reserved for the identity information.
		pub deposit: BalanceOf<T>,
	}

	impl<T: Config> Registration<T> {
		/// Calculate the total deposit for this registration
		pub fn total_deposit(&self) -> BalanceOf<T>
		where
			BalanceOf<T>: Zero + Saturating + Copy,
		{
			self.deposit
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The amount held on deposit for a registered identity.
		#[pallet::constant]
		type BasicDeposit: Get<BalanceOf<Self>>;

		/// The amount held on deposit per encoded byte for a registered identity.
		#[pallet::constant]
		type ByteDeposit: Get<BalanceOf<Self>>;

		/// Maximum number of judgements allowed for a single identity.
		#[pallet::constant]
		type MaxJudgements: Get<u32>;

		/// The origin which may provide judgements on identities. Root can always do this.
		type JudgementOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Maximum length for identity field data.
		#[pallet::constant]
		type MaxFieldLength: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Information that is pertinent to identify the entity behind an account.
	#[pallet::storage]
	pub type IdentityOf<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Registration<T>, OptionQuery>;

	/// Alternative judgement storage using a double map for educational purposes.
	/// This demonstrates different storage patterns and their performance implications.
	/// Key1: AccountId (identity holder), Key2: JudgementId, Value: Judgement
	#[pallet::storage]
	pub type JudgementsDoubleMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		JudgementId,
		Judgement,
		OptionQuery,
	>;

	/// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name was set or reset (which will remove judgement).
		IdentitySet { who: T::AccountId },
		/// A name was cleared, and the given balance returned.
		IdentityCleared { who: T::AccountId, deposit: BalanceOf<T> },
		/// A judgement was given.
		JudgementGiven { target: T::AccountId },
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account isn't found.
		NotFound,
		/// No identity found.
		NoIdentity,
		/// Sticky judgement.
		StickyJudgement,
		/// Judgement given.
		JudgementGiven,
		/// Invalid judgement.
		InvalidJudgement,
		/// The target is invalid.
		InvalidTarget,
		/// Too many judgements for this identity.
		TooManyJudgements,
	}

	/// Dispatchable functions allow users to interact with the pallet and invoke state changes.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set an account's identity information and reserve the appropriate deposit.
		///
		/// If the account already has identity information, the deposit is taken as part payment
		/// for the new deposit.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `display`: The display name.
		/// - `legal`: The legal name.
		/// - `web`: The web address.
		/// - `email`: The email address.
		///
		/// Emits `IdentitySet` if successful.
		#[pallet::weight(T::WeightInfo::set_identity_update(
			T::MaxFieldLength::get(), // worst case: fields at max length
			T::MaxJudgements::get() // worst case: maximum judgements
		))]
		pub fn set_identity(
			origin: OriginFor<T>,
			display: BoundedVec<u8, T::MaxFieldLength>,
			legal: BoundedVec<u8, T::MaxFieldLength>,
			web: BoundedVec<u8, T::MaxFieldLength>,
			email: BoundedVec<u8, T::MaxFieldLength>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let info = IdentityInfo { display, legal, web, email };

			// Calculate the length of the longest field for weight calculation
			let max_field_length = info
				.display
				.len()
				.max(info.legal.len())
				.max(info.web.len())
				.max(info.email.len());

			let id = match IdentityOf::<T>::take(&sender) {
				Some(mut id) => {
					// Calculate expected weight based on previous state
					let expected_judgements_count = id.judgements.len() as u32;

					// Make sure we account for long existing fields in storage
					let max_field_length = max_field_length
						.max(id.info.display.len())
						.max(id.info.legal.len())
						.max(id.info.web.len())
						.max(id.info.email.len());
					// Only keep sticky judgements when setting new identity
					id.judgements.retain(|(_id, judgement)| judgement.is_sticky());
					id.info = info;
					// Note: We preserve judgements_count_double_map to maintain consistency
					// with double map storage (double map judgements are independent of inline)

					// Calculate actual weight used
					let actual_weight = T::WeightInfo::set_identity_update(
						max_field_length as u32,
						expected_judgements_count,
					);

					(id, actual_weight)
				},
				None => {
					let reg = Registration {
						info,
						judgements: BoundedVec::default(),
						judgements_count_double_map: 0,
						deposit: Zero::zero(),
					};

					// Calculate actual weight for new identity
					let actual_weight = T::WeightInfo::set_identity(max_field_length as u32);

					(reg, actual_weight)
				},
			};
			let (mut registration, actual_weight) = id;

			let new_deposit = Self::calculate_identity_deposit(&registration.info);
			let old_deposit = registration.deposit;
			Self::rejig_deposit(&sender, old_deposit, new_deposit)?;

			registration.deposit = new_deposit;
			IdentityOf::<T>::insert(&sender, registration);
			Self::deposit_event(Event::IdentitySet { who: sender });

			// Return actual weight consumed
			Ok(Some(actual_weight).into())
		}

		/// Provide a judgement for an account's identity using inline storage (BoundedVec).
		/// This demonstrates the efficient storage pattern where judgements are stored
		/// inline within the Registration struct as a BoundedVec.
		///
		/// The dispatch origin for this call must be `T::JudgementOrigin`.
		///
		/// - `judgement_id`: a unique identifier for this judgement provider.
		/// - `target`: the account whose identity the judgement is upon. This must be an account
		///   with a registered identity.
		/// - `judgement_type`: the type of judgement (0=Unknown, 1=Reasonable, 2=KnownGood,
		///   3=Erroneous, 4=LowQuality).
		///
		/// Emits `JudgementGiven` if successful.
		#[pallet::weight(T::WeightInfo::provide_judgement_inline(T::MaxJudgements::get()))]
		pub fn provide_judgement_inline(
			origin: OriginFor<T>,
			judgement_id: JudgementId,
			target: T::AccountId,
			judgement_type: u8,
		) -> DispatchResult {
			T::JudgementOrigin::ensure_origin(origin)?;

			// Convert u8 to Judgement
			let judgement = Self::u8_to_judgement(judgement_type)?;

			// Add judgement only to the inline BoundedVec storage
			Self::add_judgement_inline(&target, judgement_id, judgement)?;

			Self::deposit_event(Event::JudgementGiven { target });

			Ok(())
		}

		/// Provide a judgement for an account's identity using double map storage.
		/// This demonstrates the double map storage pattern where judgements are stored
		/// in a separate DoubleMap, which can be less efficient for cleanup operations but slightly
		/// faster for `set_identity` in case of many judgements.
		///
		/// Note: This version assumes judgement deposit is not necessary and will create storage
		/// bloat if `T::JudgementOrigin` is not properly managed.
		///
		/// The dispatch origin for this call must be `T::JudgementOrigin`.
		///
		/// - `judgement_id`: a unique identifier for this judgement provider.
		/// - `target`: the account whose identity the judgement is upon. This must be an account
		///   with a registered identity.
		/// - `judgement_type`: the type of judgement (0=Unknown, 1=Reasonable, 2=KnownGood,
		///   3=Erroneous, 4=LowQuality).
		///
		/// Emits `JudgementGiven` if successful.
		#[pallet::weight(T::WeightInfo::provide_judgement_double_map())]
		pub fn provide_judgement_double_map(
			origin: OriginFor<T>,
			judgement_id: JudgementId,
			target: T::AccountId,
			judgement_type: u8,
		) -> DispatchResult {
			T::JudgementOrigin::ensure_origin(origin)?;

			// Convert u8 to Judgement
			let judgement = Self::u8_to_judgement(judgement_type)?;

			// Check that target has an identity and validate sticky judgements
			let _is_new_judgement =
				IdentityOf::<T>::try_mutate(&target, |maybe_reg| -> Result<bool, DispatchError> {
					let reg = maybe_reg.as_mut().ok_or(Error::<T>::InvalidTarget)?;

					// Check for existing judgement in double map
					if let Some(existing_judgement) =
						JudgementsDoubleMap::<T>::get(&target, judgement_id)
					{
						if existing_judgement.is_sticky() {
							return Err(Error::<T>::StickyJudgement.into());
						}
						// Existing judgement being replaced
						Ok(false)
					} else {
						// New judgement being added - increment counter
						ensure!(
							reg.judgements_count_double_map < T::MaxJudgements::get(),
							Error::<T>::TooManyJudgements
						);
						reg.judgements_count_double_map =
							reg.judgements_count_double_map.saturating_add(1);
						Ok(true)
					}
				})?;

			// Add judgement to the double map storage
			JudgementsDoubleMap::<T>::insert(&target, judgement_id, judgement);

			Self::deposit_event(Event::JudgementGiven { target });

			Ok(())
		}

		/// Clear an account's identity info and return all deposits.
		/// This extrinsic handles both storage patterns - the complexity depends on usage:
		/// - O(1) if only inline judgements were used (via provide_judgement_inline)
		/// - O(n) if double map judgements were used, where n = actual number of double map
		///   judgements
		///
		/// Payment: All reserved balances on the account are returned.
		///
		/// The dispatch origin for this call must be _Signed_ and the sender must have a registered
		/// identity.
		///
		/// Emits `IdentityCleared` if successful.
		#[pallet::weight(T::WeightInfo::clear_identity_double_map_usage(T::MaxJudgements::get()))]
		pub fn clear_identity(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let id = IdentityOf::<T>::take(&sender).ok_or(Error::<T>::NoIdentity)?;
			let deposit = id.total_deposit();

			// Always cleanup double map judgements (this is O(n) where n = actual judgements)
			// This operation uses drain_prefix and will be fast if no double map judgements exist
			let cleared = Self::clear_judgements_double_map(&sender);
			debug_assert_eq!(cleared, id.judgements_count_double_map);

			// The inline judgements are automatically dropped with the Registration struct (O(1))

			let err_amount = T::Currency::unreserve(&sender, deposit);
			debug_assert!(err_amount.is_zero());

			Self::deposit_event(Event::IdentityCleared { who: sender, deposit });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the identity information for an account.
		pub fn identity_of(who: &T::AccountId) -> Option<Registration<T>> {
			IdentityOf::<T>::get(who)
		}

		/// Convert u8 to Judgement enum
		fn u8_to_judgement(judgement_type: u8) -> Result<Judgement, DispatchError> {
			match judgement_type {
				0 => Ok(Judgement::Unknown),
				1 => Ok(Judgement::Reasonable),
				2 => Ok(Judgement::KnownGood),
				3 => Ok(Judgement::Erroneous),
				4 => Ok(Judgement::LowQuality),
				_ => Err(Error::<T>::InvalidJudgement.into()),
			}
		}

		/// Calculate the deposit required for an identity.
		fn calculate_identity_deposit(info: &IdentityInfo<T::MaxFieldLength>) -> BalanceOf<T> {
			let bytes = info.encoded_size();
			let byte_deposit = T::ByteDeposit::get().saturating_mul(BalanceOf::<T>::from(bytes));
			T::BasicDeposit::get().saturating_add(byte_deposit)
		}

		/// Helper function to clear all judgements from the double map for an account.
		/// This demonstrates efficient cleanup using clear_prefix - O(j) where j is actual
		/// judgements.
		fn clear_judgements_double_map(who: &T::AccountId) -> u32 {
			let removed = JudgementsDoubleMap::<T>::drain_prefix(who);
			removed.count() as u32
		}

		/// Helper function to add a judgement to inline storage only (BoundedVec).
		/// This demonstrates the efficient inline storage pattern.
		fn add_judgement_inline(
			who: &T::AccountId,
			judgement_id: JudgementId,
			judgement: Judgement,
		) -> Result<(), DispatchError> {
			IdentityOf::<T>::try_mutate(who, |maybe_reg| -> Result<(), DispatchError> {
				let reg = maybe_reg.as_mut().ok_or(Error::<T>::InvalidTarget)?;

				// Use binary search for the BoundedVec (efficient)
				let item = (judgement_id, judgement);
				match reg.judgements.binary_search_by_key(&judgement_id, |x| x.0) {
					Ok(position) => {
						// Judgement exists, check if it's sticky
						if reg.judgements[position].1.is_sticky() {
							return Err(Error::<T>::StickyJudgement.into())
						}
						// Replace the existing judgement
						reg.judgements[position] = item;
					},
					Err(position) => {
						// Insert new judgement at the correct position to maintain ordering
						reg.judgements
							.try_insert(position, item)
							.map_err(|_| Error::<T>::TooManyJudgements)?;
					},
				}
				Ok(())
			})
		}

		/// Take the `current` deposit that `who` is holding, and update it to a `new` one.
		fn rejig_deposit(
			who: &T::AccountId,
			current: BalanceOf<T>,
			new: BalanceOf<T>,
		) -> DispatchResult {
			if new > current {
				T::Currency::reserve(who, new - current)?;
			} else if new < current {
				let err_amount = T::Currency::unreserve(who, current - new);
				debug_assert!(err_amount.is_zero());
			}
			Ok(())
		}
	}
}
