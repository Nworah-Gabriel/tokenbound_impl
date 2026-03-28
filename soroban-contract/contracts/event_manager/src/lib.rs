#![no_std]

use core::convert::TryFrom;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, IntoVal, String,
    Symbol, Vec,
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    EventNotCanceled = 11,
    RefundAlreadyClaimed = 12,
    NotABuyer = 13,
    EventSoldOut = 14,
    TicketsBelowSold = 15,
}

#[contracttype]
pub enum DataKey {
    Event(u32),
    EventCounter,
    TicketFactory,
    RefundClaimed(u32, Address),
    EventBuyers(u32),
    BuyerPurchase(u32, Address),
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuyerPurchase {
    pub quantity: u128,
    pub total_paid: i128,
}

#[contract]
pub struct EventManager;

#[contractimpl]
impl EventManager {
    pub fn initialize(env: Env, ticket_factory: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::TicketFactory) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&DataKey::TicketFactory, &ticket_factory);
        env.storage().instance().set(&DataKey::EventCounter, &0u32);
        env.storage()
            .instance()
            .extend_ttl(Self::ttl_threshold(), Self::ttl_extend_to());

        Ok(())
    }

    pub fn create_event(
        env: Env,
        organizer: Address,
        theme: String,
        event_type: String,
        start_date: u64,
        end_date: u64,
        ticket_price: i128,
        total_tickets: u128,
        payment_token: Address,
    ) -> Result<u32, Error> {
        organizer.require_auth();

        Self::validate_event_params(&env, start_date, end_date, ticket_price, total_tickets)?;

        let event_id = Self::get_and_increment_counter(&env)?;
        let ticket_nft_addr =
            Self::deploy_ticket_nft(&env, event_id).ok_or(Error::FactoryNotInitialized)?;

        // Validate inputs
        Self::validate_event_params(&env, start_date, end_date, ticket_price, total_tickets).unwrap_or_else(|e| panic!("Validation failed: {:?}", e));

        // Get and increment event counter
        let event_id = Self::get_and_increment_counter(&env).unwrap_or_else(|e| panic!("Counter error: {:?}", e));

        // Deploy ticket NFT contract via factory
        let ticket_nft_addr = Self::deploy_ticket_nft(&env, event_id, theme.clone(), total_tickets).unwrap_or_else(|e| panic!("Deploy failed: {:?}", e));

        // Create event struct
        let event = Event {
            id: event_id,
            theme,
            organizer: organizer.clone(),
            event_type,
            total_tickets,
            tickets_sold: 0,
            ticket_price,
            start_date,
            end_date,
            is_canceled: false,
            ticket_nft_addr: ticket_nft_addr.clone(),
            payment_token,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
        Self::extend_persistent_ttl(&env, &DataKey::Event(event_id));

        env.events().publish(
            (Symbol::new(&env, "event_created"),),
            (event_id, organizer, ticket_nft_addr),
        );

        event_id
    }

    pub fn get_event(env: Env, event_id: u32) -> Result<Event, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)
    }

    pub fn get_event_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EventCounter)
            .unwrap_or(0)
    }

    pub fn get_all_events(env: Env) -> Vec<Event> {
        let count = Self::get_event_count(env.clone());
        let mut events = Vec::new(&env);

        for event_id in 0..count {
            if let Some(event) = env.storage().persistent().get(&DataKey::Event(event_id)) {
                events.push_back(event);
            }
        }

        events
    }

    pub fn get_buyer_purchase(env: Env, event_id: u32, buyer: Address) -> Option<BuyerPurchase> {
        env.storage()
            .persistent()
            .get(&DataKey::BuyerPurchase(event_id, buyer))
    }

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
        Self::extend_persistent_ttl(&env, &DataKey::Event(event_id));

        env.events()
            .publish((Symbol::new(&env, "event_canceled"),), event_id);

        Ok(())
    }

    /// Claim refund for a canceled event (pull model)
    /// Only works for canceled events, prevents double-refund claims
    pub fn claim_refund(env: Env, claimer: Address, event_id: u32) -> Result<(), Error> {
    pub fn claim_refund(env: Env, claimer: Address, event_id: u32) {
        claimer.require_auth();

        let event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        if !event.is_canceled {
            return Err(Error::EventNotCanceled);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::RefundClaimed(event_id, claimer.clone()))
        {
            return Err(Error::RefundAlreadyClaimed);
        }

        let purchase: BuyerPurchase = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerPurchase(event_id, claimer.clone()))
            .unwrap_or_else(|| panic!("Claimer did not purchase a ticket for this event"));

        if !found {
            return Err(Error::NotABuyer);
        }

        // Mark refund as claimed (prevent double-refund)
        env.storage()
            .persistent()
            .set(&DataKey::RefundClaimed(event_id, claimer.clone()), &true);
        Self::extend_persistent_ttl(&env, &DataKey::RefundClaimed(event_id, claimer.clone()));

        if purchase.total_paid > 0 {
            let token_client = soroban_sdk::token::Client::new(&env, &event.payment_token);
            token_client.transfer(&event.organizer, &claimer, &purchase.total_paid);
        }

        env.events().publish(
            (Symbol::new(&env, "refund_claimed"),),
            (event_id, claimer, purchase.quantity, purchase.total_paid),
        );

        Ok(())
    }

    pub fn update_event(
        env: Env,
        event_id: u32,
        theme: Option<String>,
        ticket_price: Option<i128>,
        total_tickets: Option<u128>,
        start_date: Option<u64>,
        end_date: Option<u64>,
    ) -> Result<(), Error> {
        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        event.organizer.require_auth();

        if event.is_canceled {
            return Err(Error::EventAlreadyCanceled);
        }

        let current_time = env.ledger().timestamp();

        if let Some(next_theme) = theme {
            event.theme = next_theme;
        }

        // Apply ticket_price if provided (must be non-negative)
        if let Some(p) = ticket_price {
            if p < 0 {
                return Err(Error::NegativeTicketPrice);
        if let Some(next_price) = ticket_price {
            if next_price < 0 {
                panic!("Ticket price cannot be negative");
            }
            event.ticket_price = next_price;
        }

        // Apply total_tickets if provided (cannot be below tickets_sold)
        if let Some(t) = total_tickets {
            if t == 0 {
                return Err(Error::InvalidTicketCount);
            }
            if t < event.tickets_sold {
                return Err(Error::TicketsBelowSold);
        if let Some(next_total) = total_tickets {
            if next_total == 0 {
                panic!("Total tickets must be greater than 0");
            }
            if next_total < event.tickets_sold {
                panic!("Cannot reduce total_tickets below tickets_sold");
            }
            event.total_tickets = next_total;
        }

        let effective_end = end_date.unwrap_or(event.end_date);
        // Apply start_date if provided
        if let Some(s) = start_date {
            if s < current_time {
                return Err(Error::InvalidStartDate);
            }
            if s >= effective_end {
                return Err(Error::InvalidEndDate);
        if let Some(next_start) = start_date {
            if next_start < current_time {
                panic!("Start date cannot be in the past");
            }
            if next_start >= effective_end {
                panic!("Start date must be before end date");
            }
            event.start_date = next_start;
        }

        let effective_start = start_date.unwrap_or(event.start_date);
        // Apply end_date if provided
        if let Some(e) = end_date {
            if e < current_time {
                return Err(Error::InvalidEndDate);
            }
            if e <= effective_start {
                return Err(Error::InvalidEndDate);
        if let Some(next_end) = end_date {
            if next_end < current_time {
                panic!("End date cannot be in the past");
            }
            if next_end <= effective_start {
                panic!("End date must be after start date");
            }
            event.end_date = next_end;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
        Self::extend_persistent_ttl(&env, &DataKey::Event(event_id));

        env.events().publish(
            (Symbol::new(&env, "event_updated"),),
            (event_id, event.organizer),
        );

        // Emit event_updated event
        env.events()
            .publish((Symbol::new(&env, "event_updated"),), (event_id, event.organizer.clone()));

        Ok(())
    }

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
        Self::extend_persistent_ttl(&env, &DataKey::Event(event_id));

        Ok(())
    }

    /// Purchase a ticket for an event
    pub fn purchase_ticket(env: Env, buyer: Address, event_id: u32) -> Result<(), Error> {
    pub fn purchase_ticket(env: Env, buyer: Address, event_id: u32) {
        Self::purchase_tickets(env, buyer, event_id, 1);
    }

    pub fn purchase_tickets(env: Env, buyer: Address, event_id: u32, quantity: u128) {
        buyer.require_auth();

        if quantity == 0 {
            panic!("Quantity must be greater than 0");
        }

        let mut event: Event = env
            .storage()
            .persistent()
            .get(&DataKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        if event.is_canceled {
            return Err(Error::EventAlreadyCanceled);
        }

        if event.tickets_sold >= event.total_tickets {
            return Err(Error::EventSoldOut);
        let next_tickets_sold = event
            .tickets_sold
            .checked_add(quantity)
            .unwrap_or_else(|| panic!("Ticket quantity overflow"));

        if next_tickets_sold > event.total_tickets {
            panic!("Event is sold out");
        }

        let total_price = Self::calculate_total_price(event.ticket_price, quantity);
        if total_price > 0 {
            let token_client = soroban_sdk::token::Client::new(&env, &event.payment_token);
            token_client.transfer(&buyer, &event.organizer, &total_price);
        }

        for _ in 0..quantity {
            env.invoke_contract::<u128>(
                &event.ticket_nft_addr,
                &Symbol::new(&env, "mint_ticket_nft"),
                soroban_sdk::vec![&env, buyer.clone().into_val(&env)],
            );
        }

        Self::record_purchase(&env, event_id, buyer.clone(), quantity, total_price);

        event.tickets_sold = next_tickets_sold;
        env.storage()
            .persistent()
            .set(&DataKey::Event(event_id), &event);
        Self::extend_persistent_ttl(&env, &DataKey::Event(event_id));

        env.events().publish(
            (Symbol::new(&env, "ticket_purchased"),),
            (event_id, buyer, quantity, total_price, event.ticket_nft_addr),
        );

        Ok(())
    }

    fn validate_event_params(
        env: &Env,
        start_date: u64,
        end_date: u64,
        ticket_price: i128,
        total_tickets: u128,
    ) -> Result<(), Error> {
        let current_time = env.ledger().timestamp();

        if start_date <= current_time {
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
        env.storage()
            .instance()
            .extend_ttl(Self::ttl_threshold(), Self::ttl_extend_to());

        Ok(current)
    }

    fn deploy_ticket_nft(env: &Env, event_id: u32, theme: String, total_supply: u128) -> Result<Address, Error> {
        let factory_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TicketFactory)
            .ok_or(Error::FactoryNotInitialized)?;

        // Call the factory contract to deploy a new NFT contract

        Ok(nft_addr)
    fn deploy_ticket_nft(env: &Env, event_id: u32) -> Option<Address> {
        let factory_addr: Address = env.storage().instance().get(&DataKey::TicketFactory)?;
        let mut salt_bytes = [0u8; 32];
        salt_bytes[28..32].copy_from_slice(&event_id.to_be_bytes());
        let salt = BytesN::from_array(env, &salt_bytes);

        Some(env.invoke_contract::<Address>(
            &factory_addr,
            &Symbol::new(env, "deploy_ticket"),
            soroban_sdk::vec![
                env,
                env.current_contract_address().into_val(env),
                salt.into_val(env)
            ],
        ))
    }

    fn record_purchase(
        env: &Env,
        event_id: u32,
        buyer: Address,
        quantity: u128,
        total_paid: i128,
    ) {
        let key = DataKey::BuyerPurchase(event_id, buyer.clone());
        let existing = env.storage().persistent().get::<_, BuyerPurchase>(&key);

        if let Some(mut purchase) = existing {
            purchase.quantity = purchase
                .quantity
                .checked_add(quantity)
                .unwrap_or_else(|| panic!("Purchase quantity overflow"));
            purchase.total_paid = purchase
                .total_paid
                .checked_add(total_paid)
                .unwrap_or_else(|| panic!("Purchase total overflow"));
            env.storage().persistent().set(&key, &purchase);
        } else {
            let purchase = BuyerPurchase {
                quantity,
                total_paid,
            };
            env.storage().persistent().set(&key, &purchase);

            let buyers_key = DataKey::EventBuyers(event_id);
            let mut buyers: Vec<Address> = env
                .storage()
                .persistent()
                .get(&buyers_key)
                .unwrap_or_else(|| Vec::new(env));
            buyers.push_back(buyer.clone());
            env.storage().persistent().set(&buyers_key, &buyers);
            Self::extend_persistent_ttl(env, &buyers_key);
        }

        Self::extend_persistent_ttl(env, &key);
    }

    fn calculate_total_price(ticket_price: i128, quantity: u128) -> i128 {
        if ticket_price <= 0 {
            return 0;
        }
    fn deploy_ticket_nft(env: &Env, event_id: u32, _theme: String, _total_supply: u128) -> Result<Address, Error> {
        let factory_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TicketFactory)
            .ok_or(Error::FactoryNotInitialized)?;

        // Create a unique salt from the event_id
        let mut salt_bytes = [0u8; 32];
        let id_bytes = event_id.to_be_bytes();
        salt_bytes[..4].copy_from_slice(&id_bytes);
        let salt = BytesN::from_array(env, &salt_bytes);

        // Call the factory contract to deploy a new NFT contract
        let mut args = Vec::new(env);
        args.push_back(env.current_contract_address().to_val());
        args.push_back(salt.to_val());

        let nft_addr: Address =
            env.invoke_contract(&factory_addr, &Symbol::new(env, "deploy_ticket"), args);

        Ok(nft_addr)
    }
}

#[cfg(test)]
mod update_event_tests {
    use super::*;
    use crate::test::MockContract;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger};

    fn setup_event_for_update(env: &Env) -> (EventManagerClient<'_>, Address, u32) {
        let contract_id = env.register(EventManager, ());
        let client = EventManagerClient::new(env, &contract_id);
        let mock_addr = env.register(MockContract, ());
        let organizer = Address::generate(env);
        env.mock_all_auths();
        client.initialize(&mock_addr);

        let start_date = env.ledger().timestamp() + 86400;
        let end_date = start_date + 86400;
        let event_id = client.create_event(
            &organizer,
            &String::from_str(env, "Original Theme"),
            &String::from_str(env, "Conference"),
            &start_date,
            &end_date,
            &1000_0000000,
            &100,
            &Address::generate(env),
        );
        (client, organizer, event_id)
    }

    #[test]
    fn test_update_event_theme() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);

        client.update_event(
            &event_id,
            &Option::Some(String::from_str(&env, "Updated Theme")),
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::None,
        );

        let event = client.get_event(&event_id);
        assert_eq!(event.theme, String::from_str(&env, "Updated Theme"));
    }

    #[test]
    fn test_update_event_ticket_price() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);
        let new_price = 2000_0000000i128;

        client.update_event(
            &event_id,
            &Option::None,
            &Option::Some(new_price),
            &Option::None,
            &Option::None,
            &Option::None,
        );

        let event = client.get_event(&event_id);
        assert_eq!(event.ticket_price, new_price);
    }

    #[test]
    fn test_update_event_total_tickets() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);

        client.update_event(
            &event_id,
            &Option::None,
            &Option::None,
            &Option::Some(200u128),
            &Option::None,
            &Option::None,
        );

        let event = client.get_event(&event_id);
        assert_eq!(event.total_tickets, 200);
    }

    #[test]
    fn test_update_event_dates() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);
        let new_start = env.ledger().timestamp() + 172800;
        let new_end = new_start + 86400;

        client.update_event(
            &event_id,
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::Some(new_start),
            &Option::Some(new_end),
        );

        let event = client.get_event(&event_id);
        assert_eq!(event.start_date, new_start);
        assert_eq!(event.end_date, new_end);
    }

    #[test]
    fn test_update_event_emits_event() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);

        client.update_event(
            &event_id,
            &Option::Some(String::from_str(&env, "Emit Test")),
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::None,
        );

        // Update completed successfully; event_updated is emitted in the same code path
        let event = client.get_event(&event_id);
        assert_eq!(event.theme, String::from_str(&env, "Emit Test"));
    }

    #[test]
    fn test_update_event_canceled_fails() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);
        client.cancel_event(&event_id);

        let result = client.try_update_event(
            &event_id,
            &Option::Some(String::from_str(&env, "Should fail")),
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_event_total_tickets_below_sold_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(EventManager, ());
        let client = EventManagerClient::new(&env, &contract_id);
        let mock_addr = env.register(MockContract, ());
        let organizer = Address::generate(&env);
        let buyer = Address::generate(&env);
        client.initialize(&mock_addr);

        let start_date = env.ledger().timestamp() + 86400;
        let end_date = start_date + 86400;
        let event_id = client.create_event(
            &organizer,
            &String::from_str(&env, "Event"),
            &String::from_str(&env, "Type"),
            &start_date,
            &end_date,
            &100i128,
            &10u128,
            &mock_addr,
        );
        client.purchase_ticket(&buyer, &event_id);
        client.purchase_ticket(&Address::generate(&env), &event_id);

        let result = client.try_update_event(
            &event_id,
            &Option::None,
            &Option::None,
            &Option::Some(1u128),
            &Option::None,
            &Option::None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_event_start_date_past_fails() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);
        env.ledger()
            .set_timestamp(env.ledger().timestamp() + 86400 * 2);
        let past_start = env.ledger().timestamp() - 3600;

        let result = client.try_update_event(
            &event_id,
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::Some(past_start),
            &Option::None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_event_end_before_start_fails() {
        let env = Env::default();
        let (client, _organizer, event_id) = setup_event_for_update(&env);
        let start_date = env.ledger().timestamp() + 86400;
        let end_before_start = start_date - 3600;

        let result = client.try_update_event(
            &event_id,
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::Some(start_date),
            &Option::Some(end_before_start),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_update_event_not_found_fails() {
        let env = Env::default();
        let contract_id = env.register(EventManager, ());
        let client = EventManagerClient::new(&env, &contract_id);
        let mock_addr = env.register(MockContract, ());
        env.mock_all_auths();
        client.initialize(&mock_addr);

        let result = client.try_update_event(
            &999u32,
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::None,
            &Option::None,
        );
        assert!(result.is_err());
        let quantity_i128 =
            i128::try_from(quantity).unwrap_or_else(|_| panic!("Quantity exceeds pricing range"));
        let subtotal = ticket_price
            .checked_mul(quantity_i128)
            .unwrap_or_else(|| panic!("Price overflow"));

        let discount_bps = if quantity >= 10 {
            1_000i128
        } else if quantity >= 5 {
            500i128
        } else {
            0i128
        };

        subtotal
            .checked_mul(10_000i128 - discount_bps)
            .and_then(|value| value.checked_div(10_000))
            .unwrap_or_else(|| panic!("Discount calculation overflow"))
    }

    fn extend_persistent_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, Self::ttl_threshold(), Self::ttl_extend_to());
    }

    const fn ttl_threshold() -> u32 {
        30 * 24 * 60 * 60 / 5
    }

    const fn ttl_extend_to() -> u32 {
        100 * 24 * 60 * 60 / 5
    }
}

#[cfg(test)]
mod test;
