# TBA Account Contract

The **TBA Account Contract** represents an individual token-bound account owned by a specific NFT. It leverages Soroban's `CustomAccountInterface` to provide native account abstraction.

## Core Functions

### `initialize`
Initializes the TBA account. Can only be called once.

```rust
fn initialize(
    env: Env,
    token_contract: Address,
    token_id: u128,
    implementation_hash: u256,
    salt: u256
)
```

- **`token_contract`**: The address of the NFT contract that owns this TBA.
- **`token_id`**: The specific token ID of the NFT.
- **`implementation_hash`**: The hash of the TBA implementation contract.
- **`salt`**: A unique salt used for deterministic address calculation.

### `execute`
Executes an arbitrary call from the TBA account. Only the current NFT owner can call this.

```rust
fn execute(
    env: Env, 
    to: Address, 
    func: Symbol, 
    args: Vec<Val>
)
```

- **`to`**: The target contract address.
- **`func`**: The function name to call.
- **`args`**: The arguments for the call.

### `owner`
Returns the current owner of the NFT associated with this TBA.

```rust
fn owner(env: Env) -> Address
```

### `token_contract`
Returns the address of the NFT contract.

```rust
fn token_contract(env: Env) -> Address
```

### `token_id`
Returns the token ID of the associated NFT.

```rust
fn token_id(env: Env) -> u128
```

## Storage Pattern

| Key | Storage Type | Description |
|-----|--------------|-------------|
| `TokenContract` | Instance | Address of the NFT contract |
| `TokenId` | Instance | Specific NFT token ID |
| `ImplementationHash` | Instance | Hash used for deployment |
| `Salt` | Instance | Deployment salt |
| `Initialized` | Instance | Initialization flag |
