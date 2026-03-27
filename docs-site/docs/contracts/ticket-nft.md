# Ticket NFT Contract

The **Ticket NFT Contract** represents event tickets as NFTs. It implements ERC721-equivalent functionality on Soroban and enforces business rules like one ticket per user.

## Core Functions

### `mint_ticket_nft`
Mints a new ticket NFT to the specified recipient. Only the designated minter (Event Manager) can call this.

```rust
fn mint_ticket_nft(env: Env, recipient: Address) -> u128
```

- **`recipient`**: The address receiving the ticket.
- **Returns**: The newly minted token ID.

### `owner_of`
Returns the current owner of a specific ticket.

```rust
fn owner_of(env: Env, token_id: u128) -> Address
```

### `balance_of`
Returns the number of tickets owned by a specific address.

```rust
fn balance_of(env: Env, owner: Address) -> u128
```

### `transfer_from`
Transfers a ticket from one address to another.

```rust
fn transfer_from(env: Env, from: Address, to: Address, token_id: u128)
```

## Storage Pattern

| Key | Storage Type | Description |
|-----|--------------|-------------|
| `NextTokenId` | Instance | Auto-incrementing token ID |
| `Owner` | Persistent | Mapping of token ID to owner address |
| `Balance` | Persistent | Mapping of owner to ticket count |
| `MinterRole` | Instance | The authorized minter (e.g., Event Manager) |
