# Event Manager Contract

The **Event Manager Contract** handles the lifecycle of event ticketing, including creation, ticket sales, cancellations, and refunds.

## Core Functions

### `create_event`
Initializes a new event and deploys a dedicated Ticket NFT contract.

```rust
fn create_event(
    env: Env,
    theme: String,
    organizer: Address,
    event_type: String,
    total_tickets: u128,
    ticket_price: i128,
    start_date: u64,
    end_date: u64
) -> u32
```

### `purchase_ticket`
Allows a user to purchase a ticket for an event. Triggers the minting of an NFT and creation of a TBA.

```rust
fn purchase_ticket(env: Env, event_id: u32)
```

### `claim_ticket_refund`
Processes a refund for a ticket if the event was canceled. The refund is sent directly to the TBA.

```rust
fn claim_ticket_refund(env: Env, event_id: u32)
```

### `cancel_event`
Allows the event organizer to cancel the event, enabling refunds.

```rust
fn cancel_event(env: Env, event_id: u32)
```

### `reschedule_event`
Updates the start and end dates of an event.

```rust
fn reschedule_event(env: Env, event_id: u32, start_date: u64, end_date: u64)
```

## Storage Pattern

| Key | Storage Type | Description |
|-----|--------------|-------------|
| `EventCount` | Instance | Total events created |
| `Event` | Persistent | Event metadata by ID |
| `UserTicketTokenId` | Persistent | Mapping of user to ticket ID per event |
| `UserClaimedRefund` | Temporary | Tracking if a user has already claimed a refund |
