use near_sdk::{
	json_types::{U128, U64},
	CryptoHash,
};
use near_sdk_sim::*;
use serde_json::json;
use stygian_atomic_swap::AtomicSwapContract;

lazy_static_include::lazy_static_include_bytes! {
   ATOMIC_SWAP_BYTES => "res/stygian_atomic_swap.wasm",
}

pub fn get_alice_amount() -> u128 {
	to_yocto("10")
}

pub fn get_accounts(root: &UserAccount) -> (UserAccount, UserAccount) {
	let alice = root.create_user("alice".to_string(), get_alice_amount());
	let bob = root.create_user("bob".to_string(), to_yocto("0.01"));

	(alice, bob)
}

pub fn init_contract() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
	let root = init_simulator(None);

	let amount: U128 = to_yocto("1").into();

	let (alice, bob) = get_accounts(&root);
	println!("=====pre-deployment=====");
	println!(
		"[Alice]: {:?}:{:?}",
		alice.account().unwrap().amount,
		alice.account().unwrap().locked
	);
	println!("[Bob]:   {:?}:{:?}", bob.account().unwrap().amount, bob.account().unwrap().locked);

	let secret_hash = CryptoHash::default().to_vec();
	let lock_time = 100_u64;

	let lock_time: U64 = lock_time.into();
	let contract = alice.deploy_and_init(
		&ATOMIC_SWAP_BYTES,
		"atomic-swap".to_string(),
		"new",
		&json!({
			"amount": amount,
			"recipient": bob.account_id(),
			"secret_hash": secret_hash,
			"lock_time": lock_time
		})
		.to_string()
		.into_bytes(),
		to_yocto("5"),
		DEFAULT_GAS.into(),
	);
	println!("=====post-deployment=====");
	println!(
		"[Alice]: {:?}:{:?}",
		alice.account().unwrap().amount,
		alice.account().unwrap().locked
	);
	println!("[Bob]:   {:?}:{:?}", bob.account().unwrap().amount, bob.account().unwrap().locked);
	println!(
		"[Contr]: {:?}:{:?}",
		contract.account().unwrap().amount,
		contract.account().unwrap().locked
	);

	(root, contract, alice, bob)
}
