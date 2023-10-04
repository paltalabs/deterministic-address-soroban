#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Val, Vec};

#[contract]
pub struct Deployer;

#[contractimpl]
impl Deployer {
    /// Deploy the contract Wasm and after deployment invoke the init function
    /// of the contract with the given arguments.
    ///
    /// This has to be authorized by `deployer` (unless the `Deployer` instance
    /// itself is used as deployer). This way the whole operation is atomic
    /// and it's not possible to frontrun the contract initialization.
    ///
    /// Returns the contract ID and result of the init function.
    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        // Skip authorization if deployer is the current contract.
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        // Deploy the contract using the uploaded Wasm with given hash.
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);

        // Invoke the init function with the given arguments.
        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        // Return the contract ID of the deployed contract and the result of
        // invoking the init result.
        (deployed_address, res)
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
    
}

mod test;
