use std::sync::{Mutex};
use types::{Action, ActionError};
use bincode;

struct ActionRunner {
	replay: bool,
	lock: Mutex
}

impl ActionRunner {
	pub fn new(replay: bool) -> ActionRunner {
		ActionRunner {
			replay,
			lock: Mutex::new(0)
		}
	}

	pub fn run(&self, action: &Action) -> Result<(), ActionError> {
		// Lock, so we cannot run actions concurrently when called from different threads
		let mut _guard = self.lock.lock().unwrap();

		let name = action.get_name();
		let encoded: Vec<u8> = bincode::encode(&action, bincode::SizeLimit::Infinite).unwrap();
		// TODO: save to log

		// Run the action
		let result = action.run(self.replay);

		// TODO: update log: finished with result (result)

		// Return the result
		result
	}
}