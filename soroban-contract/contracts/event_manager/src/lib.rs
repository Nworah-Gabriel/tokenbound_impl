#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, IntoVal, String,
    Symbol, Vec,
};

// Error handling
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    EventNotFound = 2,
    EventAlreadyCanceled = 3,
    CannotSellMoreTickets = 4,
    InvalidStartDate = 5,
    InvalidEndDate = 6,
    NegativeTicketPrice = 7,
    InvalidTicketCount = 8,
    CounterOverflow = 9,
    FactoryNotInitialized = 10,
    InvalidTierIndex = 11,
    TierSoldOut = 12,
    InvalidTierConfig = 13,
}

// Storage keys
#[contracttype]
pub enum DataKey {
    Event(u32),
    EventCounter,
    TicketFactory,
    RefundClaimed(u32, Address),
    EventBuyers(u32),
    EventTiers(u32), // event_id -> Vec<TicketTier>
}

/// A single ticket tier (e.g. VIP, General, Early Bird)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicketTier {
    pub name: String,
    pub price: i128,
    pub total_quantity: u128,
    pub sold_quantity: u128,
}

/// Input config for creating a tier
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub name: String,
    pub price: i128,
    pub total_quantity: u128,
}

/// Parameters for creating a new event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateEventParams {
    pub organizer: Address,
    pub theme: String,
    pub event_type: String,
    pub start_date: u64,
    pub end_date: u64,
    pub ticket_price: i128,
    pub total_tickets: u128,
    pub payment_token: Address,
    pub tiers: Vec<TierConfig>,
}

// Event structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub id: u32,
    pub theme: String,
    pub organizer: Address,
    pub event_type: String,
    pub total_tickets: u128,
    pub tickets_sold: u128,
    pub ticket_price: i128,
    pub start_date: u64,
    pub end_date: u64,
    pub is_canceled: bool,
    pub ticket_nft_addr: Address,
    pub payment_token: Address,
}

#[contract]
pub struct EventManager;

#[contractimpl]
impl EventManager {
    /// Initialize the contract with the ticket factory address
    pub fn initialize(env: Env, ticket_factory: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::TicketFactory) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&DataKey::TicketFactory, &ticket_factory);
        env.storage().instance().set(&DataKey::EventCounter, &0u32);
        Ok(())
    }

    /// Create a new event.
    /// `params.tiers`: optional multi-tier config. If empty, falls back to single tier using
    /// `params.ticket_price` and `params.total_tickets` (backward-compatible).
    pub fn create_event(env: Env, params: CreateEventParams) -> Result<u32, Error> {
        params.organizer.require_auth();

        Self::validate_event_params(
            &env,
            params.start_date,
            params.end_date,
            params.ticket_price,
            params.total_tickets,
        )?;

        // Build resolved tiers
        let resolved_tiers = if params.tiers.is_empty() {
            // Backward-compatible: single default tier
            let mut v = Vec::new(&env);
            v.push_back(TicketTier {
                name: String::from_str(&env, "General"),
                price: params.ticket_price,
                total_quantity: params.total_tickets,
                sold_quantity: 0,
            });
            v
        } else {
            // Validate and build tiers; derive total_tickets from sum
            let mut v = Vec::new(&env);
            for cfg in params.tiers.iter() {
                if cfg.price < 0 {
                    return Err(Error::NegativeTicketPrice);
                }
                if cfg.total_quantity == 0 {
                    return Err(Error::InvalidTierConfig);
                }
                v.push_back(TicketTier {
                    name: cfg.name.clone(),
                    price: cfg.price,
                    total_quantity: cfg.total_quantity,
                    sold_quantity: 0,
                });
            }
            v
        };

        // Compute aggregate totals from tiers
        let agg_total: u128 = resolved_tiers.iter().map(|t| t.total_quantity).sum();
        let agg_price = resolved_tiers
            .first()
            .map(|t| t.price)
            .unwrap_or(params.ticket_price);

        let event_id = Self::get_and_increment_counter(&env)?;

        let ticket_nft_addr =
            Self::deploy_ticket_nft(&env, event_id, params.theme.clone(), agg_total)?;

        let event = Event {
            id: event_id,
            theme: params.theme.clone(),
            organizer: params.organizer.clone(),
            event_type: params.event_type,
            total_tickets: agg_total,
            tickets_sold: 0,
            ticket_price: agg_price,
            start_date: params.start_date,
            end_date: params.end_date,
            is_canceled: false,
            ticket_nft_addr: ticket_nft_addr.clone(),
            payment_token: params.payment_token,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
        env.storage()
            .persistent()
            .set(&DataKey::EventTiers(event_id), &resolved_tiers);

        env.storage().persistent().extend_ttl(
            &DataKey::Event(event_id),
            30 * 24 * 60 * 60 / 5,
            100 * 24 * 60 * 60 / 5,
        );

        env.events().publish(
            (Symbol::new(&env, "event_created"),),
            (event_id, params.organizer, ticket_nft_addr),
        );

        Ok(event_id)
    }

    /// Get event by ID
    pub fn get_event(env: Env, event_id: u32) -> Result<Event, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)
    }

    /// Get tiers for an event
    pub fn get_event_tiers(env: Env, event_id: u32) -> Result<Vec<TicketTier>, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::EventTiers(event_id))
            .ok_or(Error::EventNotFound)
    }

    /// Get total number of events
    pub fn get_event_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EventCounter)
            .unwrap_or(0)
    }

    /// Get all events
    pub fn get_all_events(env: Env) -> Vec<Event> {
        let count = Self::get_event_count(env.clone());
        let mut events = Vec::new(&env);
        for i in 0..count {
            if let Some(event) = env.storage().persistent().get(&DataKey::Event(i)) {
                events.push_back(event);
            }
        }
        events
    }

    /// Cancel an event
    pub fn cancel_event(env: Env, event_id: u32) -> Result<(), Error> {
        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        event.organizer.require_auth();

        if event.is_canceled {
            return Err(Error::EventAlreadyCanceled);
        }

        event.is_canceled = true;
        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);

        env.events()
            .publish((Symbol::new(&env, "event_canceled"),), event_id);

        Ok(())
    }

    /// Claim refund for a canceled event (pull model)
    pub fn claim_refund(env: Env, claimer: Address, event_id: u32) {
        claimer.require_auth();

        let event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic!("Event not found"));

        if !event.is_canceled {
            panic!("Event is not canceled");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::RefundClaimed(event_id, claimer.clone()))
        {
            panic!("Refund already claimed");
        }

        let buyers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::EventBuyers(event_id))
            .unwrap_or_else(|| Vec::new(&env));

        let mut found = false;
        for buyer in buyers.iter() {
            if buyer == claimer {
                found = true;
                break;
            }
        }

        if !found {
            panic!("Claimer did not purchase a ticket for this event");
        }

        env.storage()
            .persistent()
            .set(&DataKey::RefundClaimed(event_id, claimer.clone()), &true);

        if event.ticket_price > 0 {
            let token_client = soroban_sdk::token::Client::new(&env, &event.payment_token);
            token_client.transfer(&event.organizer, &claimer, &event.ticket_price);
        }

        env.events().publish(
            (Symbol::new(&env, "refund_claimed"),),
            (event_id, claimer, event.ticket_price),
        );
    }

    /// Update event details
    pub fn update_event(
        env: Env,
        event_id: u32,
        theme: Option<String>,
        ticket_price: Option<i128>,
        total_tickets: Option<u128>,
        start_date: Option<u64>,
        end_date: Option<u64>,
    ) {
        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic!("Event not found"));

        event.organizer.require_auth();

        if event.is_canceled {
            panic!("Cannot update a canceled event");
        }

        let current_time = env.ledger().timestamp();

        if let Some(t) = theme {
            event.theme = t;
        }

        if let Some(p) = ticket_price {
            if p < 0 {
                panic!("Ticket price cannot be negative");
            }
            event.ticket_price = p;
        }

        if let Some(t) = total_tickets {
            if t == 0 {
                panic!("Total tickets must be greater than 0");
            }
            if t < event.tickets_sold {
                panic!("Cannot reduce total_tickets below tickets_sold");
            }
            event.total_tickets = t;
        }

        let effective_end = end_date.unwrap_or(event.end_date);
        if let Some(s) = start_date {
            if s < current_time {
                panic!("Start date cannot be in the past");
            }
            if s >= effective_end {
                panic!("Start date must be before end date");
            }
            event.start_date = s;
        }

        let effective_start = start_date.unwrap_or(event.start_date);
        if let Some(e) = end_date {
            if e < current_time {
                panic!("End date cannot be in the past");
            }
            if e <= effective_start {
                panic!("End date must be after start date");
            }
            event.end_date = e;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);

        env.storage().persistent().extend_ttl(
            &DataKey::Event(event_id),
            30 * 24 * 60 * 60 / 5,
            100 * 24 * 60 * 60 / 5,
        );

        env.events().publish(
            (Symbol::new(&env, "event_updated"),),
            (event_id, event.organizer.clone()),
        );
    }

    /// Update tickets sold (called by ticket NFT contract)
    pub fn update_tickets_sold(env: Env, event_id: u32, amount: u128) -> Result<(), Error> {
        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        event.ticket_nft_addr.require_auth();

        event.tickets_sold = event
            .tickets_sold
            .checked_add(amount)
            .ok_or(Error::CounterOverflow)?;

        if event.tickets_sold > event.total_tickets {
            return Err(Error::CannotSellMoreTickets);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);

        Ok(())
    }

    /// Purchase a ticket for an event.
    /// `tier_index`: index into the event's tiers Vec. Pass 0 for single-tier events.
    pub fn purchase_ticket(env: Env, buyer: Address, event_id: u32, tier_index: u32) {
        buyer.require_auth();

        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .unwrap_or_else(|| panic!("Event not found"));

        if event.is_canceled {
            panic!("Event is canceled");
        }

        // Load and update the specific tier
        let mut tiers: Vec<TicketTier> = env
            .storage()
            .persistent()
            .get(&DataKey::EventTiers(event_id))
            .unwrap_or_else(|| panic!("Event tiers not found"));

        if tier_index as usize >= tiers.len() as usize {
            panic!("Invalid tier index");
        }

        let mut tier = tiers.get(tier_index).unwrap();

        if tier.sold_quantity >= tier.total_quantity {
            panic!("Tier is sold out");
        }

        let price = tier.price;

        // Handle payment
        if price > 0 {
            let token_client = soroban_sdk::token::Client::new(&env, &event.payment_token);
            token_client.transfer(&buyer, &event.organizer, &price);
        }

        // Mint ticket NFT
        env.invoke_contract::<u128>(
            &event.ticket_nft_addr,
            &Symbol::new(&env, "mint_ticket_nft"),
            soroban_sdk::vec![&env, buyer.into_val(&env)],
        );

        // Update tier sold count
        tier.sold_quantity += 1;
        tiers.set(tier_index, tier);
        env.storage()
            .persistent()
            .set(&DataKey::EventTiers(event_id), &tiers);

        // Track buyer for refund purposes (store price paid for this buyer)
        let mut buyers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::EventBuyers(event_id))
            .unwrap_or_else(|| Vec::new(&env));
        buyers.push_back(buyer.clone());
        env.storage()
            .persistent()
            .set(&DataKey::EventBuyers(event_id), &buyers);

        // Update aggregate event counters
        event.tickets_sold += 1;
        // Keep ticket_price reflecting the last purchased tier price for refund logic
        event.ticket_price = price;

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);

        env.storage().persistent().extend_ttl(
            &DataKey::Event(event_id),
            30 * 24 * 60 * 60 / 5,
            100 * 24 * 60 * 60 / 5,
        );

        env.events().publish(
            (Symbol::new(&env, "ticket_purchased"),),
            (event_id, buyer, event.ticket_nft_addr, tier_index),
        );
    }

    // ========== Helper Functions ==========

    fn validate_event_params(
        env: &Env,
        start_date: u64,
        end_date: u64,
        ticket_price: i128,
        total_tickets: u128,
    ) -> Result<(), Error> {
        let current_time = env.ledger().timestamp();

        if start_date < current_time {
            return Err(Error::InvalidStartDate);
        }
        if end_date <= start_date {
            return Err(Error::InvalidEndDate);
        }
        if ticket_price < 0 {
            return Err(Error::NegativeTicketPrice);
        }
        if total_tickets == 0 {
            return Err(Error::InvalidTicketCount);
        }

        Ok(())
    }

    fn get_and_increment_counter(env: &Env) -> Result<u32, Error> {
        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::EventCounter)
            .unwrap_or(0);

        let next = current.checked_add(1).ok_or(Error::CounterOverflow)?;
        env.storage().instance().set(&DataKey::EventCounter, &next);

        Ok(current)
    }

    fn deploy_ticket_nft(
        env: &Env,
        _event_id: u32,
        _theme: String,
        _total_supply: u128,
    ) -> Result<Address, Error> {
        let factory_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TicketFactory)
            .ok_or(Error::FactoryNotInitialized)?;

        let salt = BytesN::from_array(env, &[0u8; 32]);
        let mut args = Vec::new(env);
        args.push_back(env.current_contract_address().to_val());
        args.push_back(salt.to_val());

        let nft_addr: Address =
            env.invoke_contract(&factory_addr, &Symbol::new(env, "deploy_ticket"), args);

        Ok(nft_addr)
    }
}

mod test;
