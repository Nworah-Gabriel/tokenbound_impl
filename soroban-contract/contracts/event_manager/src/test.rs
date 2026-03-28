#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, BytesN, Env};

#[contract]
pub struct MockContract;

#[contractimpl]
impl MockContract {
    pub fn deploy_ticket(env: Env, _minter: Address, _salt: BytesN<32>) -> Address {
        env.current_contract_address()
    }

    pub fn mint_ticket_nft(_env: Env, _recipient: Address) -> u128 {
        1
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

// ========== Helpers ==========

fn setup(env: &Env) -> (EventManagerClient<'_>, Address) {
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    env.mock_all_auths();
    client.initialize(&mock_addr);
    (client, mock_addr)
}

fn make_params(
    env: &Env,
    mock_addr: &Address,
    tiers: Vec<TierConfig>,
) -> (Address, CreateEventParams) {
    let organizer = Address::generate(env);
    let start = env.ledger().timestamp() + 86400;
    let end = start + 86400;
    let params = CreateEventParams {
        organizer: organizer.clone(),
        theme: String::from_str(env, "Test Event"),
        event_type: String::from_str(env, "Conference"),
        start_date: start,
        end_date: end,
        ticket_price: 100i128,
        total_tickets: 10u128,
        payment_token: mock_addr.clone(),
        tiers,
    };
    (organizer, params)
}

fn make_event(
    env: &Env,
    client: &EventManagerClient<'_>,
    mock_addr: &Address,
    tiers: Vec<TierConfig>,
) -> (Address, u32) {
    let (organizer, params) = make_params(env, mock_addr, tiers);
    let event_id = client.create_event(&params);
    (organizer, event_id)
}

// ========== Existing Tests ==========

#[test]
fn test_create_event() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let organizer = Address::generate(&env);
    let start_date = env.ledger().timestamp() + 86400;

    let event_id = client.create_event(&CreateEventParams {
        organizer: organizer.clone(),
        theme: String::from_str(&env, "Rust Conference 2026"),
        event_type: String::from_str(&env, "Conference"),
        start_date,
        end_date: start_date + 86400,
        ticket_price: 1000_0000000,
        total_tickets: 500,
        payment_token: mock_addr,
        tiers: Vec::new(&env),
    });

    assert_eq!(event_id, 0);

    let event = client.get_event(&event_id);
    assert_eq!(event.id, 0);
    assert_eq!(event.organizer, organizer);
    assert_eq!(event.total_tickets, 500);
    assert_eq!(event.tickets_sold, 0);
    assert!(!event.is_canceled);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn setup() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

#[test]
fn test_create_event_past_date() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let organizer = Address::generate(&env);

    env.ledger().set_timestamp(1000);

    client.create_event(&CreateEventParams {
        organizer,
        theme: String::from_str(&env, "Past Event"),
        event_type: String::from_str(&env, "Conference"),
        start_date: 500, // past
        end_date: 1500,
        ticket_price: 1000_0000000,
        total_tickets: 100,
        payment_token: mock_addr,
        tiers: Vec::new(&env),
    });
    let theme = String::from_str(&env, "Past Event");
    let event_type = String::from_str(&env, "Conference");
    let start_date = 500; // Past date
    let end_date = 1500;

    let result = client.try_create_event(
        &organizer,
        &theme,
        &event_type,
        &start_date,
        &end_date,
        &1000_0000000,
        &100,
        &Address::generate(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_cancel_event() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));

    client.cancel_event(&event_id);

    let event = client.get_event(&event_id);
    assert!(event.is_canceled);
}

#[test]
fn test_purchase_ticket() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));
    let buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id, &0u32);

    let event = client.get_event(&event_id);
    assert_eq!(event.tickets_sold, 1);
}

// ========== Refund Tests ==========

#[test]
fn test_claim_refund_successful() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));
    let buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id, &0u32);
    client.cancel_event(&event_id);
    client.claim_refund(&buyer, &event_id);
    assert_eq!(event.is_canceled, true);
fn create_sample_event(env: &Env, client: &EventManagerClient<'_>, payment_token: &Address) -> u32 {
    let organizer = Address::generate(env);
    client.create_event(
        &organizer,
        &String::from_str(env, "Stellar Meetup"),
        &String::from_str(env, "Conference"),
        &(env.ledger().timestamp() + 86_400),
        &(env.ledger().timestamp() + 172_800),
        &100i128,
        &20u128,
        payment_token,
    )
}

#[test]
fn test_create_event() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let organizer = Address::generate(&env);

    let event_id = client.create_event(
        &organizer,
        &String::from_str(&env, "Rust Conference 2026"),
        &String::from_str(&env, "Conference"),
        &(env.ledger().timestamp() + 86_400),
        &(env.ledger().timestamp() + 172_800),
        &1000_0000000,
        &500u128,
        &mock_addr,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event_id, 0);
    assert_eq!(event.id, 0);
    assert_eq!(event.total_tickets, 500);
    assert_eq!(event.tickets_sold, 0);
    assert_eq!(event.payment_token, mock_addr);
}

#[test]
fn test_claim_refund_event_not_canceled() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));
    let buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id, &0u32);
    client.claim_refund(&buyer, &event_id);
    env.mock_all_auths();

#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_create_event_rejects_past_start_date() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    env.ledger().set_timestamp(1000);

    let organizer = Address::generate(&env);
    client.create_event(
        &organizer,
        &String::from_str(&env, "Past Event"),
        &String::from_str(&env, "Conference"),
        &999u64,
        &2000u64,
        &1000_0000000,
        &100u128,
        &mock_addr,
    );

    client.purchase_ticket(&buyer, &event_id);

    // Try to claim refund without canceling event
    let result = client.try_claim_refund(&buyer, &event_id);
    assert!(result.is_err());
}

#[test]
fn test_claim_refund_double_claim() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));
    let buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id, &0u32);
    env.mock_all_auths();

}

#[test]
fn test_cancel_event_marks_event_canceled() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let event_id = create_sample_event(&env, &client, &mock_addr);

    client.cancel_event(&event_id);
    client.claim_refund(&buyer, &event_id);
    client.claim_refund(&buyer, &event_id);

    // Try to claim again (should fail)
    let result = client.try_claim_refund(&buyer, &event_id);
    assert!(result.is_err());
}

#[test]
fn test_claim_refund_no_ticket_purchased() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));
    let buyer = Address::generate(&env);
    let non_buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id, &0u32);
    client.cancel_event(&event_id);
    client.claim_refund(&non_buyer, &event_id);
}

#[test]
fn test_claim_refund_free_ticket() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let organizer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let start = env.ledger().timestamp() + 86400;

    let event_id = client.create_event(&CreateEventParams {
        organizer,
        theme: String::from_str(&env, "Free Event"),
        event_type: String::from_str(&env, "Conference"),
        start_date: start,
        end_date: start + 86400,
        ticket_price: 0,
        total_tickets: 10,
        payment_token: mock_addr,
        tiers: Vec::new(&env),
    });

    client.purchase_ticket(&buyer, &event_id, &0u32);
    client.cancel_event(&event_id);
    client.claim_refund(&buyer, &event_id);
}

#[test]
fn test_multiple_refund_claims() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));

    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    let buyer3 = Address::generate(&env);

    client.purchase_ticket(&buyer1, &event_id, &0u32);
    client.purchase_ticket(&buyer2, &event_id, &0u32);
    client.purchase_ticket(&buyer3, &event_id, &0u32);

    assert_eq!(client.get_event(&event_id).tickets_sold, 3);

    client.cancel_event(&event_id);
    client.claim_refund(&buyer1, &event_id);
    client.claim_refund(&buyer2, &event_id);
    client.claim_refund(&buyer3, &event_id);
}

#[test]
#[should_panic(expected = "Event not found")]
fn test_claim_refund_nonexistent_event() {
    let env = Env::default();
    let (client, _) = setup(&env);
    let buyer = Address::generate(&env);
    client.claim_refund(&buyer, &999u32);
}

// ========== Multi-Tier Tests ==========

fn make_tiers(env: &Env) -> Vec<TierConfig> {
    let mut tiers = Vec::new(env);
    tiers.push_back(TierConfig {
        name: String::from_str(env, "Early Bird"),
        price: 50i128,
        total_quantity: 5u128,
    });
    tiers.push_back(TierConfig {
        name: String::from_str(env, "General"),
        price: 100i128,
        total_quantity: 10u128,
    });
    tiers.push_back(TierConfig {
        name: String::from_str(env, "VIP"),
        price: 300i128,
        total_quantity: 3u128,
    });
    tiers
}

#[test]
fn test_create_event_with_tiers() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let tiers = make_tiers(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, tiers);

    let event = client.get_event(&event_id);
    // total_tickets = 5 + 10 + 3 = 18
    assert_eq!(event.total_tickets, 18);
    assert_eq!(event.tickets_sold, 0);

    let stored_tiers = client.get_event_tiers(&event_id);
    assert_eq!(stored_tiers.len(), 3);
    assert_eq!(stored_tiers.get(0).unwrap().price, 50);
    assert_eq!(stored_tiers.get(1).unwrap().price, 100);
    assert_eq!(stored_tiers.get(2).unwrap().price, 300);
}

#[test]
fn test_purchase_ticket_specific_tier() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, make_tiers(&env));
    let buyer = Address::generate(&env);

    // Buy a VIP ticket (tier index 2)
    client.purchase_ticket(&buyer, &event_id, &2u32);

    let stored_tiers = client.get_event_tiers(&event_id);
    assert_eq!(stored_tiers.get(2).unwrap().sold_quantity, 1);
    assert_eq!(stored_tiers.get(0).unwrap().sold_quantity, 0);
    assert_eq!(stored_tiers.get(1).unwrap().sold_quantity, 0);

    assert_eq!(client.get_event(&event_id).tickets_sold, 1);
}

#[test]
fn test_per_tier_inventory_tracking() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, make_tiers(&env));

    // Buy all 5 Early Bird tickets (tier 0)
    for _ in 0..5 {
        client.purchase_ticket(&Address::generate(&env), &event_id, &0u32);
    }

    let stored_tiers = client.get_event_tiers(&event_id);
    assert_eq!(stored_tiers.get(0).unwrap().sold_quantity, 5);
    assert_eq!(stored_tiers.get(0).unwrap().total_quantity, 5);
    assert_eq!(stored_tiers.get(1).unwrap().sold_quantity, 0);
    assert_eq!(stored_tiers.get(2).unwrap().sold_quantity, 0);
}

#[test]
#[should_panic(expected = "Tier is sold out")]
fn test_purchase_ticket_tier_sold_out() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, make_tiers(&env));

    // Exhaust VIP (3 tickets)
    for _ in 0..3 {
        client.purchase_ticket(&Address::generate(&env), &event_id, &2u32);
    }

    // 4th VIP purchase should fail
    client.purchase_ticket(&Address::generate(&env), &event_id, &2u32);
}

#[test]
#[should_panic(expected = "Invalid tier index")]
fn test_purchase_ticket_invalid_tier_index() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, make_tiers(&env));

    client.purchase_ticket(&Address::generate(&env), &event_id, &99u32);
}

#[test]
fn test_backward_compat_single_tier() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));

    let tiers = client.get_event_tiers(&event_id);
    assert_eq!(tiers.len(), 1);
    assert_eq!(
        tiers.get(0).unwrap().name,
        String::from_str(&env, "General")
    );
    assert_eq!(tiers.get(0).unwrap().price, 100);
    assert_eq!(tiers.get(0).unwrap().total_quantity, 10);
}

// ========== Update Event Tests ==========

fn setup_event_for_update(env: &Env) -> (EventManagerClient<'_>, Address, u32) {
    let (client, mock_addr) = setup(env);
    let (organizer, event_id) = make_event(env, &client, &mock_addr, Vec::new(env));
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

    client.update_event(
        &event_id,
        &Option::None,
        &Option::Some(2000_0000000i128),
        &Option::None,
        &Option::None,
        &Option::None,
    );

    assert_eq!(client.get_event(&event_id).ticket_price, 2000_0000000i128);
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

    assert_eq!(client.get_event(&event_id).total_tickets, 200);
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

    assert_eq!(
        client.get_event(&event_id).theme,
        String::from_str(&env, "Emit Test")
    );
}

#[test]
#[should_panic(expected = "Cannot update a canceled event")]
fn test_update_event_canceled_fails() {
    let env = Env::default();
    let (client, _organizer, event_id) = setup_event_for_update(&env);
    client.cancel_event(&event_id);

    client.update_event(
        &event_id,
        &Option::Some(String::from_str(&env, "Should fail")),
        &Option::None,
        &Option::None,
        &Option::None,
        &Option::None,
    );
}

#[test]
#[should_panic(expected = "Cannot reduce total_tickets below tickets_sold")]
fn test_update_event_total_tickets_below_sold_fails() {
    let event = client.get_event(&event_id);
    assert!(event.is_canceled);
}

#[test]
fn test_purchase_ticket_increments_tickets_sold() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let event_id = create_sample_event(&env, &client, &mock_addr);
    let buyer = Address::generate(&env);

    client.purchase_ticket(&buyer, &event_id);

    // Try to claim refund without purchasing
    let result = client.try_claim_refund(&non_buyer, &event_id);
    assert!(result.is_err());
    let event = client.get_event(&event_id);
    let purchase = client.get_buyer_purchase(&event_id, &buyer).unwrap();

    assert_eq!(event.tickets_sold, 1);
    assert_eq!(purchase.quantity, 1);
    assert_eq!(purchase.total_paid, 100);
}

#[test]
fn test_purchase_tickets_applies_group_discount() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let event_id = create_sample_event(&env, &client, &mock_addr);
    let buyer = Address::generate(&env);

    client.purchase_tickets(&buyer, &event_id, &5u128);

    let event = client.get_event(&event_id);
    let purchase = client.get_buyer_purchase(&event_id, &buyer).unwrap();

    assert_eq!(event.tickets_sold, 5);
    assert_eq!(purchase.quantity, 5);
    assert_eq!(purchase.total_paid, 475);
}

#[test]
fn test_batch_purchase_refund_uses_total_paid() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let event_id = create_sample_event(&env, &client, &mock_addr);
    let buyer = Address::generate(&env);

    client.purchase_tickets(&buyer, &event_id, &10u128);
    client.cancel_event(&event_id);
    client.claim_refund(&buyer, &event_id);
}

#[test]
fn test_claim_refund_nonexistent_event() {
    let env = Env::default();
    let (client, mock_addr) = setup(&env);
    let (_, event_id) = make_event(&env, &client, &mock_addr, Vec::new(&env));

    client.purchase_ticket(&Address::generate(&env), &event_id, &0u32);
    client.purchase_ticket(&Address::generate(&env), &event_id, &0u32);

    client.update_event(
        &event_id,
        &Option::None,
        &Option::None,
        &Option::Some(1u128),
        &Option::None,
        &Option::None,
    );
}

#[test]
#[should_panic(expected = "Start date cannot be in the past")]
fn test_update_event_start_date_past_fails() {
    let env = Env::default();
    let (client, _organizer, event_id) = setup_event_for_update(&env);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 86400 * 2);

    client.update_event(
        &event_id,
        &Option::None,
        &Option::None,
        &Option::None,
        &Option::Some(env.ledger().timestamp() - 3600),
        &Option::None,
    );
}

#[test]
#[should_panic(expected = "Start date must be before end date")]
fn test_update_event_end_before_start_fails() {
    let env = Env::default();
    let (client, _organizer, event_id) = setup_event_for_update(&env);
    let start_date = env.ledger().timestamp() + 86400;

    client.update_event(
        &event_id,
        &Option::None,
        &Option::None,
        &Option::None,
        &Option::Some(start_date),
        &Option::Some(start_date - 3600),
    );
}

#[test]
#[should_panic(expected = "Event not found")]
fn test_update_event_not_found_fails() {
    let env = Env::default();
    let (client, _) = setup(&env);

    client.update_event(
        &999u32,
        &Option::None,
        &Option::None,
        &Option::None,
        &Option::None,
        &Option::None,
    );
#[should_panic(expected = "Refund already claimed")]
fn test_refund_cannot_be_claimed_twice() {
    let env = setup();
    let contract_id = env.register(EventManager, ());
    let client = EventManagerClient::new(&env, &contract_id);
    let mock_addr = env.register(MockContract, ());
    client.initialize(&mock_addr);
    let event_id = create_sample_event(&env, &client, &mock_addr);
    let buyer = Address::generate(&env);

    // Try to claim refund for non-existent event
    let result = client.try_claim_refund(&buyer, &999u32);
    assert!(result.is_err());
    client.purchase_tickets(&buyer, &event_id, &2u128);
    client.cancel_event(&event_id);
    client.claim_refund(&buyer, &event_id);
    client.claim_refund(&buyer, &event_id);
}
