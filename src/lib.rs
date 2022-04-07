use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    require, PanicOnDefault, Promise,
};
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, BlockHeight};
use serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum State {
    Init,
    Claim,
    Revert,
    Commit,
}

impl State {
    fn emit(&self) {
        env::log_str(format!("{:?}", self).as_str());
    }
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

type Error = String;
#[near_bindgen]
impl AtomicSwap {
    #[init]
    pub fn new(amount: U128, recipient: AccountId, secret_hash: Vec<u8>, lock_time: u64) -> Self {
        require!(
            env::attached_deposit() >= amount.0,
            "You must deposit at least the amount"
        );

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

        if env::signer_account_id() != self.recipient {
            env::panic_str("Only the recipient can claim the funds unless the lock has surpassed");
        }

        if self.secret_hash == secret_hash {
            self.progress(State::Claim).then(self.commit())
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
        Promise::new(env::predecessor_account_id())
            .transfer(env::account_balance())
            .then(self.progress(State::Revert))
    }

    #[private]
    fn commit(&mut self) -> Promise {
        Promise::new(self.recipient.clone())
            .transfer(self.amount.0)
            .then(self.progress(State::Commit))
            .then(
                Promise::new(env::predecessor_account_id())
                    .transfer(env::account_balance() - self.amount.0),
            )
    }

    #[private]
    fn progress(&mut self, state: State) -> Promise {
        self.state = state;
        self.state.emit();
        Promise::new(env::signer_account_id())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    const DEPOSIT: u128 = 10_u128.pow(24);

    fn get_context(signer_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(signer_account_id.clone())
            .predecessor_account_id(accounts(0))
            .attached_deposit(DEPOSIT);
        builder
    }

    fn get_accounts() -> (AccountId, AccountId) {
        let alice: AccountId = accounts(0);
        let bob: AccountId = accounts(1);
        (alice, bob)
    }

    #[test]
    fn test_check_lock_happy() {
        let context = get_context(accounts(2));
        testing_env!(context.build());
        let (_alice, bob) = get_accounts();
        let secret_hash = env::sha256("secret_hash".as_bytes());
        let lock_time: u64 = 100;

        let contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

        assert!(contract.check_lock().is_ok());
    }

    #[test]
    #[should_panic(
        expected = "Only the recipient can claim the funds unless the lock has surpassed"
    )]
    fn test_claim_hash_invalid_recipient() {
        let context = get_context(accounts(3));
        testing_env!(context.build());
        let (_alice, bob) = get_accounts();
        let secret_hash = env::sha256("secret_hash".as_bytes());
        let lock_time: u64 = 100;

        let mut contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

        contract.claim(env::sha256("failed_secret_hash".as_bytes()));
    }

    #[test]
    #[should_panic(expected = "Secret hash does not match")]
    fn test_claim_hash_invalid() {
        let (_alice, bob) = get_accounts();
        let context = get_context(bob.clone());
        testing_env!(context.build());
        let secret_hash = env::sha256("secret_hash".as_bytes());
        let lock_time: u64 = 100;

        let mut contract = AtomicSwap::new(DEPOSIT.into(), bob, secret_hash, lock_time.into());

        contract.claim(env::sha256("failed_secret_hash".as_bytes()));
    }

    #[test]
    fn test_revert() {
        let (alice, bob) = get_accounts();
        let mut context = get_context(alice.clone());
        testing_env!(context.block_index(1).build());
        let secret_hash = env::sha256("secret_hash".as_bytes());
        let lock_time: u64 = 100;

        let mut contract =
            AtomicSwap::new(DEPOSIT.into(), bob, secret_hash.clone(), lock_time.into());

        testing_env!(context.block_index(1_000_000).build());

        assert_eq!(env::attached_deposit(), DEPOSIT);

        contract.claim(secret_hash);

        // Ensures the deposit was removed from the balance
        assert_eq!(env::account_balance(), 0);
    }

    #[test]
    fn test_commit() {
        let (_alice, bob) = get_accounts();
        let context = get_context(bob.clone());
        testing_env!(context.build());
        let secret_hash = env::sha256("secret_hash".as_bytes());
        let lock_time: u64 = 100;

        let mut contract =
            AtomicSwap::new(DEPOSIT.into(), bob, secret_hash.clone(), lock_time.into());

        assert_eq!(env::attached_deposit(), DEPOSIT);

        contract.claim(secret_hash);

        // Ensures the deposit was removed from the balance
        assert_eq!(env::account_balance(), 0);
    }
}
