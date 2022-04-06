use near_sdk::{borsh::{self, BorshDeserialize, BorshSerialize}, json_types::{U128, U64}, PanicOnDefault, Promise, require};
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, BlockHeight};
use serde::{Deserialize, Serialize};
near_sdk::setup_alloc!();
use stygian_atomic_swap_primitives::AtomicSwap as AtomicSwapTrait;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum State {
	Init,
	Claim,
	Revert,
	Commit,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AtomicSwap {
	/// Amount to swap
	amount: U128,
	/// Where the funds are going to
	recipient: AccountId,
	/// The hash used to unlock the vault
	secret_hash: Vec<u8>,
	/// What block the lock is free
	lock_free: BlockHeight,
	/// The current state of the swap
	state: State,
}

type Amount = U128;
type SecretHash = Vec<u8>;
type LockTime = U64;
type Account = AccountId;
type Error = String;
type Response = Promise;

#[near_bindgen]
impl AtomicSwap {

	#[init]
	pub fn new(amount: U128, recipient: AccountId, secret_hash: Vec<u8>, lock_time: u64) -> Self {
		require!(env::attached_deposit() > amount, "You must deposit at least the amount");

		AtomicSwap {
			amount,
			recipient,
			secret_hash,
			state: State::Init,
			lock_free: env::block_height() + lock_time,
		}
	}

	fn claim(&mut self, secret_hash: Vec<u8>) -> Promise {
		self.progress(State::Claim);

		if let Err(err) = self.check_lock() {
			env::log_str(err.as_str());
			return self.revert();
		}

		if env::current_account_id() != self.recipient {
			env::panic_str("Only the recipient can claim the funds unless the lock has surpassed");
		}

		if self.secret_hash == secret_hash {
			self.progress(State::Claim);
			self.commit()
		} else {
			env::panic_str("Secret hash does not match")
		}
	}

	fn check_lock(&self) -> Result<(), Error> {
		if env::block_height() > self.lock_free {
			Err("Lock time has exceeded".to_string())
		} else {
			Ok(())
		}
	}

	fn get_state(&self) -> &State {
		&self.state
	}

	#[private]
	fn revert(&mut self) -> Promise {
		Promise::new(env::signer_account_id())
			.transfer(env::attached_deposit())
			.transfer(self.amount.0)
			.then(self.progress(State::Revert))
	}

	#[private]
	fn commit(&mut self) -> Promise {
		Promise::new(self.recipient.clone())
			.transfer(self.amount.0)
			.then(Promise::new(env::signer_account_id()).transfer(env::attached_deposit()))
			.then(self.progress(State::Commit))
	}

	#[private]
	fn progress(&mut self, state: State) -> Promise {
		self.state = state;
		Promise::new(env::signer_account_id())
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	use near_sdk::MockedBlockchain;
	use near_sdk::{testing_env, VMContext};

	const DEPOSIT: u128 = (10 * 10_u64.pow(18)) as u128;
	const CURRENT_ACCOUNT_ID: &str = "alice";
	const SIGNER_ACCOUNT_ID: &str = "bob";
	const SIGNER_ACCOUNT_PK: [u8; 3] = [0, 1, 2];
	const PREDECESSOR_ACCOUNT_ID: &str = "carol";

	fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
		VMContext {
			current_account_id: CURRENT_ACCOUNT_ID.to_owned(),
			signer_account_id: SIGNER_ACCOUNT_ID.to_owned(),
			signer_account_pk: Vec::from(&SIGNER_ACCOUNT_PK[..]),
			predecessor_account_id: PREDECESSOR_ACCOUNT_ID.to_owned(),
			input,
			block_index: 10,
			block_timestamp: 42,
			account_balance: 0,
			account_locked_balance: 0,
			storage_usage: 0,
			attached_deposit: DEPOSIT,
			prepaid_gas: 10u64.pow(18),
			random_seed: vec![0, 1, 2],
			is_view,
			output_data_receivers: vec![],
			epoch_height: 0,
			view_config: None
		}
	}

	fn get_accounts() -> (AccountId, AccountId) {
		let alice = AccountId::from(CURRENT_ACCOUNT_ID.to_string());
		let bob = AccountId::from("bob_near".to_string());
		(alice, bob)
	}

	#[test]
	fn test_check_lock_happy() {
		let context = get_context(vec![], false);
		testing_env!(context);
		let (alice, bob) = get_accounts();
		let secret_hash = env::sha256("secret_hash".as_bytes());
		let lock_time = 100;

		let mut contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

		assert!(contract.check_lock().is_ok());
	}

	#[test]
	#[should_panic(expected = "Secret hash does not match")]
	fn test_claim_hash_invalid() {
		let context = get_context(vec![], false);
		testing_env!(context);
		let (alice, bob) = get_accounts();
		let secret_hash = env::sha256("secret_hash".as_bytes());
		let lock_time = 100;

		let mut contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

		contract.claim(env::sha256("failed_secret_hash".as_bytes()));
	}

	#[test]
	fn test_revert() {
		let mut context = get_context(vec![], false);
		context.account_balance = 0;
		testing_env!(context);
		let (alice, bob) = get_accounts();
		let secret_hash = env::sha256("secret_hash".as_bytes());
		let lock_time = 100;

		let mut contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

		// Check the deposit exists as part of the contract balance
		assert_eq!(env::account_balance(), DEPOSIT);

		contract.revert();
		assert_eq!(contract.state, State::Revert);

		// Ensures the deposit was removed from the balance
		assert_eq!(env::account_balance(), 0);
	}

	#[test]
	fn test_commit() {
		let context = get_context(vec![], false);
		testing_env!(context);
		let (alice, bob) = get_accounts();
		let secret_hash = env::sha256("secret_hash".as_bytes());
		let lock_time = 100;

		let mut contract =
			AtomicSwap::new(DEPOSIT.into(), bob, secret_hash.clone(), lock_time.into());

		// Check the deposit exists as part of the contract balance
		assert_eq!(env::account_balance(), DEPOSIT);

		contract.claim(secret_hash);

		// Ensures the deposit was removed from the balance
		assert_eq!(env::account_balance(), 0);
	}
}
