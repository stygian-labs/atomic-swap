mod utils;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use crate::utils::*;
	use near_sdk::CryptoHash;
	use near_sdk_sim::*;
	use serde_json::json;

	#[test]
	fn simulate_lock_fail_when_lock_surpassed() {
		let (root, contract, _alice, _bob) = crate::utils::init_contract();

		{
			let mut root_runtime = root.borrow_runtime_mut();
			assert!(root_runtime.produce_blocks(120).is_ok(), "Couldn't produce blocks");
		}

		let locked: HashMap<String, String> = root
			.call(
				contract.account_id(),
				"check_lock",
				&json!({}).to_string().into_bytes(),
				DEFAULT_GAS,
				0,
			)
			.unwrap_json();

		assert_eq!(
			locked.get(&"Err".to_string()).unwrap(),
			&"Lock time has been exceeded".to_string()
		);
	}

	#[test]
	fn simulate_lock_pass_within_lock() {
		let (root, contract, _alice, _bob) = crate::utils::init_contract();

		let locked: HashMap<String, ()> = root
			.call(
				contract.account_id(),
				"check_lock",
				&json!({}).to_string().into_bytes(),
				DEFAULT_GAS,
				0,
			)
			.unwrap_json();
		assert_eq!(locked.get(&"Ok".to_string()).unwrap(), &());
	}

	#[test]
	fn simulate_claim() {
		let (_root, contract, alice, bob) = crate::utils::init_contract();

		alice
			.call(
				contract.account_id(),
				"claim",
				&json!({ "secret_hash": CryptoHash::default() }).to_string().into_bytes(),
				DEFAULT_GAS,
				0,
			)
			.promise_results();

		assert!(bob.account().unwrap().amount > to_yocto("1").into());
	}

	#[test]
	fn simulate_revert() {
		let (root, contract, alice, bob) = crate::utils::init_contract();

		{
			let mut root_runtime = root.borrow_runtime_mut();
			assert!(root_runtime.produce_blocks(120).is_ok(), "Couldn't produce blocks");
		}
		// Check that alice had less than 5
		assert!(alice.account().unwrap().amount < to_yocto("5").into());

		alice
			.call(
				contract.account_id(),
				"claim",
				&json!({ "secret_hash": CryptoHash::default() }).to_string().into_bytes(),
				DEFAULT_GAS,
				0,
			)
			.promise_results();

		assert!(bob.account().unwrap().amount < to_yocto("1").into());
		// Check that alice has more than 5 now
		assert!(alice.account().unwrap().amount > to_yocto("5").into());
	}

	#[test]
	fn simulate_commit() {}
}
