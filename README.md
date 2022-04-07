# Stygian Atomic Swap

<!-- MAGIC COMMENT: DO NOT DELETE! Everything above this line is hidden on NEAR Examples page -->

This smart contract facilitates a simple atomic swap, allowing a user to claim funds with a secret key, or revert if the lock is surpassed.

This contract is designed to be instantiable, not an ephemeral contract living on-chain. Once termination is triggered, we return the funds to the caller.
The intention behind this design are:
- to keep the contract simple 
- allow for additional incentives that could be bespoke for the caller, such as trust fees
- to support cross-chain execution, we want to be able to support a FaaS-like architecture


## Prerequisites

Ensure `near-cli` is installed by running:

```
near --version
```

If needed, install `near-cli`:

```
npm install near-cli -g
```

Ensure `Rust` is installed by running:

```
rustc --version
```

If needed, install `Rust`:

```
curl https://sh.rustup.rs -sSf | sh
```

Install dependencies

```
npm install
```

Install cargo-make for fast deployment:
```
cargo install --force cargo-make
```

## Quick Start

To run this project locally:

1. Prerequisites: Make sure you have Node.js ≥ 12 installed (https://nodejs.org), then use it to install yarn: `npm install --global yarn` (or just `npm i -g yarn`)

## Building this contract

To make the build process compatible with multiple operating systems, the build process exists as a script in `package.json`.
There are a number of special flags used to compile the smart contract into the wasm file.
Run this command to build and place the wasm file in the `res` directory:

```bash
npm run build
```

**Note**: Instead of `npm`, users of [yarn](https://yarnpkg.com) may run:

```bash
yarn build
```

### Important

If you encounter an error similar to:

> note: the `wasm32-unknown-unknown` target may not be installed

Then run:

```bash
rustup target add wasm32-unknown-unknown
```

## Using this contract

### Quick test

If you do not have a NEAR account, please create one with [NEAR Wallet](https://wallet.testnet.near.org).

Make sure you have credentials saved locally for the account you want to deploy the contract to. To perform this run the following `near-cli` command:

```
near login
```

### Quickest deploy

At the root of this repo, we have a `Makefile.toml` that provides some quick commands for deployment.

Important configurations are in the `[env]` section.

To deploy, run:
```bash
cargo make deploy-all
```

This will deploy, check the lock and call get_state.

If you want to change the root parameters, you can do so at the command line, like so:

`cargo make -e LOCK_TIME=1 deploy-all`

You can also manually call individual tasks like so:
```bash
cargo make check_lock
```

So let's go through some user flows, starting with the standard swap, in which the recipient claims the funds with a valid key:

```bash
cargo make deploy-all
cargo make claim
```

Try and claim again, you'll see you can't claim something thats already claimed.

The next one is a swap where the callee tries to claim the funds before the lock has passed.

```bash
cargo make -e RECIPIENT="bob.testnet" deploy-all
cargo make claim
```

You should see a failure since only the recipient can make a claim before the lock has surpassed.

What about if the lock has surpassed, can we revert?
```bash
cargo make -e LOCK_TIME="0" deploy-all
cargo make claim
```
 


### Standard deploy

In this option, the smart contract will get deployed to a specific account created with the NEAR Wallet.

If you do not have a NEAR account, please create one with [NEAR Wallet](https://wallet.testnet.near.org).

Make sure you have credentials saved locally for the account you want to deploy the contract to. To perform this run the following `near-cli` command:

```
near login
```

Deploy the contract:

```bash
near deploy --wasmFile res/stygian_atomic_swap.wasm --accountId YOUR_ACCOUNT_NAME
```

Set a status for your account:

```bash
near call YOUR_ACCOUNT_NAME set_status '{"message": "aloha friend"}' --accountId YOUR_ACCOUNT_NAME
```

Get the status:

```bash
near view YOUR_ACCOUNT_NAME get_status '{"account_id": "YOUR_ACCOUNT_NAME"}'
```

Note that these status messages are stored per account in a `HashMap`. See `src/lib.rs` for the code. We can try the same steps with another account to verify.
**Note**: we're adding `NEW_ACCOUNT_NAME` for the next couple steps.

There are two ways to create a new account:

- the NEAR Wallet (as we did before)
- `near create_account NEW_ACCOUNT_NAME --masterAccount YOUR_ACCOUNT_NAME`

Now call the contract on the first account (where it's deployed):

```bash
near call YOUR_ACCOUNT_NAME set_status '{"message": "bonjour"}' --accountId NEW_ACCOUNT_NAME
```

```bash
near view YOUR_ACCOUNT_NAME get_status '{"account_id": "NEW_ACCOUNT_NAME"}'
```

Returns `bonjour`.

Make sure the original status remains:

```bash
near view YOUR_ACCOUNT_NAME get_status '{"account_id": "YOUR_ACCOUNT_NAME"}'
```

## Testing

To test run:

```bash
cargo test --package stygian-atomic-swap -- --nocapture
```
