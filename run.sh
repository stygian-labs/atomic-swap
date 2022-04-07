
echo -e "\e[32mDeploying with lock of 1000 and self as recipient, proving commit path\e[0m"
near dev-deploy -f \
  target/wasm32-unknown-unknown/release/stygian_atomic_swap.wasm \
  new \
  '{"amount":"2", "recipient":"awesomeibex.testnet", "secret_hash":[1,1,1,1], "lock_time":1000}' \
  30000000000000 \
  1 \
  --helperUrl https://near-contract-helper.onrender.com

echo -e "\e[32mEnter temp contract id [ENTER]:\e[0m"
read -r CONTRACT

echo -e "\e[32mChecking lock\e[0m"
near view \
  "$CONTRACT" \
  check_lock

echo -e "\e[32mChecking state\e[0m"
near view \
  "$CONTRACT" \
  get_state

echo -e "\e[32mClaiming funds\e[0m"
near call \
  "$CONTRACT" \
  claim \
  '{"secret_hash":[1,1,1,1]}' \
  --accountId awesomeibex.testnet

echo -e "\e[32mDeploying with lock of 1 and self as recipient, proving revert path\e[0m"
near dev-deploy -f \
  target/wasm32-unknown-unknown/release/stygian_atomic_swap.wasm \
  new \
  '{"amount":"2", "recipient":"awesomeibex.testnet", "secret_hash":[1,1,1,1], "lock_time":1}' \
  30000000000000 \
  1 \
  --helperUrl https://near-contract-helper.onrender.com

echo -e "\e[32mEnter temp contract id [ENTER]:\e[0m"
read -r CONTRACT

echo -e "\e[32mChecking lock\e[0m"
near view \
  "$CONTRACT" \
  check_lock

echo -e "\e[32mChecking state\e[0m"
near view \
  "$CONTRACT" \
  get_state

echo -e "\e[32mClaiming funds\e[0m"
near call \
  "$CONTRACT" \
  claim \
  '{"secret_hash":[1,1,1,1]}' \
  --accountId awesomeibex.testnet

echo -e "\e[32mDeploying with lock of 1000 and self as recipient, proving unclaimable path\e[0m"
near dev-deploy -f \
  target/wasm32-unknown-unknown/release/stygian_atomic_swap.wasm \
  new \
  '{"amount":"2", "recipient":"bob.testnet", "secret_hash":[1,1,1,1], "lock_time":1000}' \
  30000000000000 \
  1 \
  --helperUrl https://near-contract-helper.onrender.com

echo -e "\e[32mEnter temp contract id [ENTER]:\e[0m"
read -r CONTRACT

echo -e "\e[32mChecking lock\e[0m"
near view \
  "$CONTRACT" \
  check_lock

echo -e "\e[32mChecking state\e[0m"
near view \
  "$CONTRACT" \
  get_state

echo -e "\e[32mClaiming funds\e[0m"
near call \
  "$CONTRACT" \
  claim \
  '{"secret_hash":[1,1,1,1]}' \
  --accountId awesomeibex.testnet

# prove cant claim