# Ticket Factory Contract

The **Ticket Factory Contract** is responsible for deploying isolated Ticket NFT contracts for each event.

## Core Functions

### `deploy_ticket`
Deploys a new Ticket NFT contract instance.

```rust
fn deploy_ticket(
    env: Env,
    minter: Address,
    salt: u256
) -> Address
```

- **`minter`**: The address authorized to mint tickets (usually the Event Manager).
- **`salt`**: A unique salt for deterministic contract deployment.
- **Returns**: The address of the new Ticket NFT contract.

## Storage Pattern

| Key | Storage Type | Description |
|-----|--------------|-------------|
| `TicketCount` | Instance | Total ticket contracts deployed |
| `TicketContract` | Persistent | Mapping of event ID to ticket contract address |
