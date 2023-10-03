#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{Deployer, DeployerClient};
use alloc::vec;
use soroban_sdk::{
    Bytes,
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    xdr::{self, ContractIdPreimage, ContractIdPreimageFromAddress, CreateContractArgs, Uint256},
    Address, BytesN, Env, IntoVal, Val, Vec// deploy::DeployerWithAddress
};

// The contract that will be deployed by the deployer contract.
mod contract {
    soroban_sdk::contractimport!(
        file =
            "../contract/target/wasm32-unknown-unknown/release/soroban_deployer_test_contract.wasm"
    );
}

// The other contract that will be deployed by the deployer contract.
mod contract_b {
    soroban_sdk::contractimport!(
        file =
            "../contract_b/target/wasm32-unknown-unknown/release/soroban_deployer_test_contract_b.wasm"
    );
}

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

#[test]
fn test_deploy_from_contract() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // Deploy contract using deployer, and include an init function to call.
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();
    let (contract_id, init_result) = deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    assert!(init_result.is_void());
    // No authorizations needed - the contract acts as a factory.
    assert_eq!(env.auths(), vec![]);

    // Invoke contract to check that it is initialized.
    let client = contract::Client::new(&env, &contract_id);
    let sum = client.value();
    assert_eq!(sum, 5);
}

#[test]
fn test_deploy_from_address() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // Define a deployer address that needs to authorize the deployment.
    let deployer = Address::random(&env);

    // Deploy contract using deployer, and include an init function to call.
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();
    let (contract_id, init_result) =
        deployer_client.deploy(&deployer, &wasm_hash, &salt, &init_fn, &init_fn_args);

    assert!(init_result.is_void());

    let expected_auth = AuthorizedInvocation {
        // Top-level authorized function is `deploy` with all the arguments.
        function: AuthorizedFunction::Contract((
            deployer_client.address,
            symbol_short!("deploy"),
            (
                deployer.clone(),
                wasm_hash.clone(),
                salt,
                init_fn,
                init_fn_args,
            )
                .into_val(&env),
        )),
        // From `deploy` function the 'create contract' host function has to be
        // authorized.
        sub_invocations: vec![AuthorizedInvocation {
            function: AuthorizedFunction::CreateContractHostFn(CreateContractArgs {
                contract_id_preimage: ContractIdPreimage::Address(ContractIdPreimageFromAddress {
                    address: deployer.clone().try_into().unwrap(),
                    salt: Uint256([0; 32]),
                }),
                executable: xdr::ContractExecutable::Wasm(xdr::Hash(wasm_hash.into_val(&env))),
            }),
            sub_invocations: vec![],
        }],
    };
    assert_eq!(env.auths(), vec![(deployer, expected_auth)]);

    // Invoke contract to check that it is initialized.
    let client = contract::Client::new(&env, &contract_id);
    let sum = client.value();
    assert_eq!(sum, 5);
}


#[test]
fn test_deploy_from_contract_twice_same_wasm_different_salt() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // Deploy contract using deployer, and include an init function to call.
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();
    let (contract_id,_init_result) = deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    // Let's create a new_salt array, of 32 elements, all initializated to value 0
    let mut new_salt = BytesN::from_array(&env, &[0; 32]);
    // Let's make the array different form the previous one:
    new_salt.set(0,1);  // Change the first element to 42
    
    // Let's confirm that the two arrays are different:
    assert_ne!(salt, new_salt);


    // Deploy contract using deployer, and include an init function to call.
    let (new_contract_id,_new_init_result) = deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &new_salt,
        &init_fn,
        &init_fn_args,
    );

    // Let's confirm that the two contracts addresses are different:
    assert_ne!(contract_id, new_contract_id);

}

#[test]
#[should_panic(expected = "escalating error to panic")]
fn test_deploy_from_contract_twice_same_wasm_same_salt_should_panic() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // Deploy contract using deployer, and include an init function to call.
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();
    deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    // Deploy using the same salt should panic
    // Deploy contract using deployer, and include an init function to call.
    deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

}

#[test]
#[should_panic(expected = "escalating error to panic")]
fn test_deploy_from_contract_different_wasm_same_salt_should_panic() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);
    let new_wasm_hash = env.deployer().upload_contract_wasm(contract_b::WASM);

    //Let's confirm that the two wasm are different:
    assert_ne!(wasm_hash, new_wasm_hash);

    // We will have the same salt, init_fn and init_fn_args for the two contracts
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();

    // We'll deploy the first contract
   deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    // We'll try to deploy the second contract with the same salt. This should panic

    deployer_client.deploy(
        &deployer_client.address,
        &new_wasm_hash  ,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    

}


#[test]
fn test_deploy_from_two_contract_deployers_same_wasm_same_salt() {
    let env = Env::default();
    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));
    let deployer_client_b = DeployerClient::new(&env, &env.register_contract(None, Deployer));

    //Let's confirm that they are two different deployers
    assert_ne!(deployer_client.address, deployer_client_b.address);
    
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);


    // We will have the same salt, init_fn and init_fn_args for the two deployments
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();

    // We'll deploy the first contract from the first deployer
    let (contract_id,_init_result) =deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    // We'll deploy the same wasm/salt from another deployer. This should work

    let (contract_id_b,_init_result_b) =deployer_client_b.deploy(
        &deployer_client_b.address,
        &wasm_hash  ,
        &salt,
        &init_fn,
        &init_fn_args,
    );


    // Let's check that the two contracts have different addresses:
    assert_ne!(contract_id, contract_id_b);

}


#[test]
fn test_calculate_address() {
    let env = Env::default();
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    let deployer_client = DeployerClient::new(&env, &env.register_contract(None, Deployer));
    let salt = BytesN::from_array(&env, &[0; 32]);
    let init_fn = symbol_short!("init");
    let init_fn_args: Vec<Val> = (5u32,).into_val(&env);
    env.mock_all_auths();

    // Pre-calculate the address before deploying
    let calculated_address=calculate_address(env.clone(), deployer_client.address.clone(), salt.clone());

    // Deploy
    let (contract_id,_init_result) =deployer_client.deploy(
        &deployer_client.address,
        &wasm_hash,
        &salt,
        &init_fn,
        &init_fn_args,
    );

    // Check that pre calculated address is the same as the new deployed contract address
    assert_eq!(calculated_address, contract_id);
}


 