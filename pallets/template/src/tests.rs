use crate::OracleEvent;
use crate::{mock::*, Error, Event};
use frame_support::print;
use frame_support::{assert_noop, assert_ok, weights::Weight};
#[test]
fn set_oracle_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Call set_oracle as root
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 5));

		// Check that the oracle is set
		assert_eq!(TemplateModule::oracle(), Some(5));

		// Check event was emitted
		System::assert_last_event(Event::OracleUpdated { new_oracle: 5 }.into());
	})
}

#[test]
fn submit_event_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Set an oracle
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 5));

		// Send some data as the oracle
		assert_ok!(TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]));

		// Check that the oracle_event is stored
		assert_eq!(TemplateModule::oracle_events().get(0).unwrap().data, vec![1, 2, 3, 4]);

		// Add another event
		assert_ok!(TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![20, 30]));

		// Check that the oracle_event is stored
		assert_eq!(TemplateModule::oracle_events().get(1).unwrap().data, vec![20, 30]);

		// Check event was emitted
		System::assert_last_event(Event::EventSubmitted { oracle: 5, timestamp: 1 }.into());
	})
}

#[test]
fn submit_event_oracle_not_set() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Send some data as the oracle
		// This should fail because the oracle has not been set
		assert_noop!(
			TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]),
			Error::<Test>::OracleNotSet
		);
	})
}

#[test]
fn submit_event_not_current_oracle() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Set an oracle
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 10));

		// Send some data as a different account than the oracle
		// This should fail
		assert_noop!(
			TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]),
			Error::<Test>::NotCurrentOracle
		);
	})
}

#[test]
fn submit_event_vec_too_big() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Set an oracle
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 5));

		// Send some data that is bigger than 1024 bytes
		// This should fail
		assert_noop!(
			TemplateModule::submit_event(RuntimeOrigin::signed(5), Vec::from([0; 1025])),
			Error::<Test>::VecTooBig
		);
	})
}

#[test]
fn submit_event_vec_oracle_events_overflow() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Set an oracle
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 5));

		// Send 1000 oracle events
		for i in 0..1000 {
			assert_ok!(TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]));
		}

		// Send 1 more oracle event
		// This should fail because the bounded_vec is full
		assert_noop!(
			TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]),
			Error::<Test>::OracleEventsOverflow
		);
	})
}

#[test]
fn on_idle_works() {
	new_test_ext().execute_with(|| {
		// Set an oracle
		assert_ok!(TemplateModule::set_oracle(RuntimeOrigin::root(), 5));

		// Send some data as the oracle
		assert_ok!(TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![1, 2, 3, 4]));

		// Check that the oracle_event is stored
		assert_eq!(TemplateModule::oracle_events().get(0).unwrap().data, vec![1, 2, 3, 4]);

		// Advance the block number
		run_to_block(10);

		// Check that the oracle_event is still stored because it is less than 600 blocks old
		assert_eq!(TemplateModule::oracle_events().get(0).unwrap().data, vec![1, 2, 3, 4]);

		// Add another event
		assert_ok!(TemplateModule::submit_event(RuntimeOrigin::signed(5), vec![10, 20]));

		// Advance the block number
		run_to_block(602);

		// Check that the first oracle_event is removed because it is more than 600 blocks old (one hour)
		// Note that the second oracle_event is still stored
		// Now the second oracle_event is at index 0
		assert_eq!(TemplateModule::oracle_events().get(0).unwrap().data, vec![10, 20]);
	});
}
