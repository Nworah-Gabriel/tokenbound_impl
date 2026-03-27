# TBA Registry Contract

The **TBA Registry Contract** is the factory and directory for creating and tracking TBA accounts. It ensures deterministic address calculation and provides a single source of truth for TBA creation.

## Core Functions

### `create_account`
Deploys a new TBA account for a specific NFT.

```rust
fn create_account(
    env: Env,
    implementation_hash: u256,
    token_contract: Address,
    token_id: u128,
    salt: u256
) -> Address
```

- **`implementation_hash`**: The hash of the TBA account implementation contract.
- **`token_contract`**: The address of the NFT contract.
- **`token_id`**: The token ID of the NFT.
- **`salt`**: A unique salt for address derivation.
- **Returns**: The address of the newly deployed TBA.

### `get_account`
Calculates the deterministic address of a TBA account without deploying it.

```rust
fn get_account(
    env: Env,
    implementation_hash: u256,
    token_contract: Address,
    token_id: u128,
    salt: u256
) -> Address
```

### `total_deployed_accounts`
Returns the total number of TBA accounts deployed for a specific NFT.

```rust
fn total_deployed_accounts(
    env: Env,
    token_contract: Address,
    token_id: u128
) -> u32
```

## Storage Pattern

| Key | Storage Type | Description |
|-----|--------------|-------------|
| `AccountCount` | Instance | Total accounts per NFT |
| `Account` | Persistent | Mapping of NFT + params to TBA address |
| `TotalAccounts` | Instance | Total accounts created globally |
