# deterministic-address-soroban
Playground in order to experiment deterministic contract addresses in Soroban, the smart contract platform in the Stellar Blockchain

In Soroban, when an smart contract is being deployed by another smart contract (usually called Deployer or Factory), the address of the new smart contract can be determined by the address of the deployer contract plus "salt".  A "salt" in this context is usually is a random or unique value that is combined with the deployer's contract address to create a deterministic, yet distinct, identifier for the smart contract to be deployed.

So.... how can we play with this deterministic contract addresses?

All the code of this Playground is currently supporting Soroban Preview 11:

In this repo you'll find
- 2 dummy contracts to be deployed `deployer/contract` and `deployer/contract_b`
- 1 factory contract that will deploy these dummy contracts `deployer/deployer`
- 1 test script that will be used for our research `deployer/deployer/test.rs`


```
bash quickstart.sh standalone
bash run.sh
cd deployer/contract
make build
cd ../contract_b
make build
cd ../deployer
make build
make test
```

**Note:** All of the tests refered in this README are inside `deployer/deployer/test.rs`

# Questions to answer

## 1. Same WASM, different salt ✅

Can a deployer deploy 2 contracts with same WASM and different salt?

Answer: **✅ Yes!**

Check `test_deploy_from_contract_twice_same_wasm_different_salt`

## 2. Same WASM, same salt ❌
Can a deployer deploys 2 contracts with same WASM and same salt?

Answer: **❌ No!**

Check `test_deploy_from_contract_twice_same_wasm_same_salt_should_panic`

## 3. Different WASM, same salt ❌

Can a deployer deploys 2 different contracts (different WASM) with same salt?

Answer: **❌ No!**

Check `test_deploy_from_contract_different_wasm_same_salt_should_panic`

This proves that in fact **the contract addresses does not depend on the WASM, but on the combination of address & salt**


## 4. Two deployers, same WASM, same salt ✅
Can one deployer deploys a (wasm/salt) and another deployer deploys the same (wasm/salt)?

Answer: **✅ Yes!**

Check `test_deploy_from_two_contract_deployers_same_wasm_same_salt`

They have indeed different contract addreess, because, again, the contract address depends on the combination of address&salt.!!

## 5. Calculate a deterministic address
Can we calculate the address of a contract only knowing the address of the factory that will (or that already did) deploy the contract and the salt used (or to be used)?

Answer: **✅ Yes!**

Check `test_calculate_address`

After `soroban-sdk = { version = "20.0.0-rc2" }`, we can use the `deployed_address` method for a `DeployerWithAddress` struct. For more information please check the documentation here: https://docs.rs/soroban-sdk/20.0.0-rc2/soroban_sdk/deploy/struct.DeployerWithAddress.html#method.deployed_address

This can be used like this:

```
pub fn calculate_address(
    env: Env, 
    deployer: Address,
    salt: BytesN<32>,
) -> Address {
   
    let deployer_with_address = env.deployer().with_address(deployer.clone(), salt);
    
    // Calculate deterministic address:
    // This function can be called at anytime, before or after the contract is deployed, because contract addresses are deterministic.
    // https://docs.rs/soroban-sdk/20.0.0-rc2/soroban_sdk/deploy/struct.DeployerWithAddress.html#method.deployed_address
    let deterministic_address = deployer_with_address.deployed_address();
    deterministic_address
}
```

___
___

### Errors:

In the tests we don't see the panic error, because we test that we panic.
Here is the error when you want to deploy two contracts that will have the same address

```rust
---- test::test_deploy_from_contract_different_wasm_same_salt stdout ----
thread 'test::test_deploy_from_contract_different_wasm_same_salt' panicked at 'HostError: Error(Storage, ExistingValue)

Event log (newest first):
   0: [Diagnostic Event] topics:[error, Error(Storage, ExistingValue)], data:"escalating error to panic"
   1: [Diagnostic Event] topics:[error, Error(Storage, ExistingValue)], data:["contract call failed", deploy, [Address(Contract(b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358)), Bytes(46350c8f2aec668ec7c5123c6c8aed922e213baac24a343dccfcef6218a72e29), Bytes(0000000000000000000000000000000000000000000000000000000000000000), init, [5]]]
   2: [Failed Diagnostic Event (not emitted)] contract:b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358, topics:[error, Error(Storage, ExistingValue)], data:"caught error from function"
   3: [Failed Diagnostic Event (not emitted)] contract:b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358, topics:[error, Error(Storage, ExistingValue)], data:"escalating error to panic"
   4: [Failed Diagnostic Event (not emitted)] contract:b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358, topics:[error, Error(Storage, ExistingValue)], data:["contract already exists", Bytes(e3d6b5af5e363ad37756ebafbca9f7fea6c9dc174a5b6b60539aa2d1b1dfbfdb)]
   5: [Diagnostic Event] topics:[fn_call, Bytes(b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358), deploy], data:[Address(Contract(b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358)), Bytes(46350c8f2aec668ec7c5123c6c8aed922e213baac24a343dccfcef6218a72e29), Bytes(0000000000000000000000000000000000000000000000000000000000000000), init, [5]]
   6: [Diagnostic Event] contract:b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358, topics:[fn_return, deploy], data:[Address(Contract(e3d6b5af5e363ad37756ebafbca9f7fea6c9dc174a5b6b60539aa2d1b1dfbfdb)), Void]
   7: [Diagnostic Event] contract:e3d6b5af5e363ad37756ebafbca9f7fea6c9dc174a5b6b60539aa2d1b1dfbfdb, topics:[fn_return, init], data:Void
   8: [Diagnostic Event] contract:b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358, topics:[fn_call, Bytes(e3d6b5af5e363ad37756ebafbca9f7fea6c9dc174a5b6b60539aa2d1b1dfbfdb), init], data:5
   9: [Diagnostic Event] topics:[fn_call, Bytes(b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358), deploy], data:[Address(Contract(b841dd0f3b3b3cfab39ac7d6feaa1a03997cab709783632472c0f4eef8b1a358)), Bytes(254715ef422bf26928b56c2a4c3a2b7b2e23c38e32c190921b5f7be2a575acca), Bytes(0000000000000000000000000000000000000000000000000000000000000000), init, [5]]

```