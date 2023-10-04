NETWORK="$1"

case "$NETWORK" in
standalone) 
echo "Using standalone network"
SOROBAN_RPC_HOST="http://stellar:8000"
SOROBAN_RPC_URL="$SOROBAN_RPC_HOST/soroban/rpc"
SOROBAN_NETWORK_PASSPHRASE="Standalone Network ; February 2017"
FRIENDBOT_URL="$SOROBAN_RPC_HOST/friendbot"
NEW_TITLE="I love Standalone"
;;
futurenet)
echo "Using Futurenet network"
SOROBAN_RPC_HOST="http://stellar:8000"
SOROBAN_RPC_URL="$SOROBAN_RPC_HOST/soroban/rpc"
SOROBAN_NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
FRIENDBOT_URL="https://friendbot-futurenet.stellar.org/"
NEW_TITLE="I prefer Futurenet"
;;
*)
echo "Usage: $0 standalone|futurenet"
exit 1
;;
esac


echo Add the $NETWORK network to cli client
    soroban config network add "$NETWORK" \
        --rpc-url "$SOROBAN_RPC_URL" \
        --network-passphrase "$SOROBAN_NETWORK_PASSPHRASE"
    echo "--"
    echo "--"

echo Creating the admin identity
    soroban config identity generate admin
    TOKEN_ADMIN_ADDRESS="$(soroban config identity address admin)"
    echo "--"
    echo "--"

# This will fail if the account already exists, but it'll still be fine.
echo Fund admin account from friendbot
    curl  -X POST "$FRIENDBOT_URL?addr=$TOKEN_ADMIN_ADDRESS"
    echo "--"
    echo "--"

ARGS="--network $NETWORK --source admin"
echo "Using ARGS: $ARGS"
    echo "--"
    echo "--"


echo Compile all contracts
    cd /workspace/deployer/contract
    make build
    cd ../contract_b
    make build
    cd ../deployer
    make build
    cd /workspace
    echo "--"
    echo "--"

echo Deploy the first dummy contract
    WASM_DUMMY_PATH="/workspace/deployer/contract/target/wasm32-unknown-unknown/release/soroban_deployer_test_contract.wasm"
    DUMMY="$(soroban contract deploy $ARGS --wasm $WASM_DUMMY_PATH)"
    echo "Contract deployed in $NETWORK network succesfully with ID: $DUMMY"
    echo "--"
    echo "--"

echo Deploy the second dummy contract
    WASM_DUMMY_BPATH="/workspace/deployer/contract_b/target/wasm32-unknown-unknown/release/soroban_deployer_test_contract_b.wasm"
    DUMMY_B="$(soroban contract deploy $ARGS --wasm $WASM_DUMMY_BPATH)"
    echo "Contract deployed in $NETWORK network succesfully with ID: $DUMMY_B"
    echo "--"
    echo "--"

echo Deploy deployer contract
    WASM_DEPLOYER_PATH="/workspace/deployer/deployer/target/wasm32-unknown-unknown/release/soroban_deployer_contract.wasm"
    DEPLOYER="$(soroban contract deploy $ARGS --wasm $WASM_DEPLOYER_PATH)"
    echo "Contract deployed in $NETWORK network succesfully with ID: $DEPLOYER"
    echo "--"
    echo "--"

echo Set variables and salt
    SALT="my_salt"
    init_fn="init"
    init_fn_args="3"

echo Calculate deployed contract address

    soroban contract invoke \
        $ARGS \
        --wasm $WASM_DEPLOYER_PATH \
        --id $DEPLOYER \
        -- \
        calculate_address \
        --deployer $DEPLOYER \
        --salt $SALT
