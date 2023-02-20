#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn oracle)]
	pub type CurrentOracle<T: Config> = StorageValue<_, T::AccountId>;

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Debug, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct OracleEvent<T: Config> {
		pub data: BoundedVec<u8, ConstU32<1024>>,
		pub oracle: T::AccountId,
		pub timestamp: T::BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn oracle_events)]
	pub type OracleEvents<T> =
		StorageValue<_, BoundedVec<OracleEvent<T>, ConstU32<1000>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The oracle has been updated
		/// [new_oracle]
		/// The new oracle
		OracleUpdated { new_oracle: T::AccountId },
		/// An event has been submitted
		/// [oracle, timestamp]
		/// The oracle that submitted the event
		/// The timestamp of the block in which the event was submitted
		EventSubmitted { oracle: T::AccountId, timestamp: T::BlockNumber },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The oracle has not been set yet
		OracleNotSet,
		/// The caller is not the current oracle
		NotCurrentOracle,
		/// The submitted data is too big
		VecTooBig,
		/// The oracle events storage is full
		OracleEventsOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(current_block: T::BlockNumber, _remaining_weight: Weight) -> Weight {
			// Get all the oracle events in storage
			let mut oracle_events = <OracleEvents<T>>::get();

			// Get the index of the last valid event - an event that is less than 600 blocks old
			let last_valid_event_index = oracle_events
				.iter()
				.position(|event| (current_block - event.timestamp) < T::BlockNumber::from(600u32))
				.unwrap_or(0);

			// If the last valid event is the first event, then there are no events that are less than 600 blocks old
			if last_valid_event_index == 0 {
				return Weight::from_ref_time(0);
			}

			// Remove all the events that are more than 600 blocks old
			oracle_events.drain(0..last_valid_event_index);

			// Update the oracle events storage
			<OracleEvents<T>>::put(oracle_events);

			return Weight::from_ref_time(0);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_oracle(origin: OriginFor<T>, oracle: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			// Update the current oracle in storage
			<CurrentOracle<T>>::put(oracle.clone());

			// Emit an event that the oracle has been updated
			Self::deposit_event(Event::OracleUpdated { new_oracle: oracle });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn submit_event(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get the current oracle from storage
			let oracle = <CurrentOracle<T>>::get().ok_or(Error::<T>::OracleNotSet)?;

			// Ensure that the caller is the current oracle
			ensure!(oracle == who, Error::<T>::NotCurrentOracle);

			// Ensure that the data is not too big
			let data = BoundedVec::<u8, ConstU32<1024>>::try_from(data)
				.map_err(|_| Error::<T>::VecTooBig)?;

			let timestamp = <frame_system::Pallet<T>>::block_number();
			let event = OracleEvent::<T> { data, oracle, timestamp };

			// Append the event to the oracle events storage
			OracleEvents::<T>::try_append(event.clone())
				.map_err(|_| Error::<T>::OracleEventsOverflow)?;

			// Emit an event that the event was submitted
			Self::deposit_event(Event::EventSubmitted {
				oracle: event.oracle,
				timestamp: event.timestamp,
			});

			Ok(())
		}
	}
}
