extern crate std;

use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Map, Symbol, Val, Vec};

use crate::storage::MAX_FEE_BPS;
use crate::{DealStatus, Error, PrimaryContract, PrimaryContractClient};

const START: u64 = 1_700_000_000;
const DAY: u64 = 86_400;
const FEE_BPS: u32 = 250;
const AMOUNT: i128 = 10_000;

struct Setup {
    env: Env,
    contract: Address,
    admin: Address,
    token: Address,
    fee_recipient: Address,
    payer: Address,
    payee: Address,
    arbiter: Address,
}

impl Setup {
    fn new() -> Setup {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(START);

        let admin = Address::generate(&env);
        let fee_recipient = Address::generate(&env);
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let arbiter = Address::generate(&env);

        let sac = env.register_stellar_asset_contract_v2(admin.clone());
        let token = sac.address();

        let contract = env.register(PrimaryContract, ());
        let client = PrimaryContractClient::new(&env, &contract);
        client.initialize(&admin, &token, &FEE_BPS, &fee_recipient);

        Setup {
            env,
            contract,
            admin,
            token,
            fee_recipient,
            payer,
            payee,
            arbiter,
        }
    }

    fn client(&self) -> PrimaryContractClient<'_> {
        PrimaryContractClient::new(&self.env, &self.contract)
    }

    fn token_client(&self) -> token::Client<'_> {
        token::Client::new(&self.env, &self.token)
    }

    fn mint(&self, to: &Address, amount: i128) {
        token::StellarAssetClient::new(&self.env, &self.token).mint(to, &amount);
    }

    fn balance(&self, who: &Address) -> i128 {
        self.token_client().balance(who)
    }

    fn create_deal(&self, amount: i128) -> u64 {
        self.client().create_deal(
            &self.payer,
            &self.payee,
            &self.arbiter,
            &amount,
            &(START + DAY),
        )
    }

    /// Creates a deal and deposits its amount, leaving it `Funded`.
    fn funded_deal(&self, amount: i128) -> u64 {
        self.mint(&self.payer, amount);
        let deal_id = self.create_deal(amount);
        self.client().fund_deal(&self.payer, &deal_id);
        deal_id
    }
}

#[test]
fn initialize_records_config() {
    let s = Setup::new();
    let config = s.client().get_config();

    assert_eq!(config.admin, s.admin);
    assert_eq!(config.token, s.token);
    assert_eq!(config.fee_recipient, s.fee_recipient);
    assert_eq!(config.fee_bps, FEE_BPS);
    assert_eq!(s.client().get_deal_count(), 0);
}

#[test]
fn initialize_is_single_shot() {
    let s = Setup::new();
    let result = s
        .client()
        .try_initialize(&s.admin, &s.token, &FEE_BPS, &s.fee_recipient);

    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn initialize_rejects_fee_above_max() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let contract = env.register(PrimaryContract, ());
    let client = PrimaryContractClient::new(&env, &contract);

    let result = client.try_initialize(&admin, &token, &(MAX_FEE_BPS + 1), &admin);
    assert_eq!(result, Err(Ok(Error::InvalidFeeBps)));
}

#[test]
fn calls_before_initialization_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let contract = env.register(PrimaryContract, ());
    let client = PrimaryContractClient::new(&env, &contract);

    assert_eq!(client.try_get_config(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_get_deal(&1), Err(Ok(Error::DealNotFound)));
    assert_eq!(
        client.try_create_deal(&payer, &payee, &arbiter, &AMOUNT, &(START + DAY)),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(
        client.try_fund_deal(&payer, &1),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(
        client.try_raise_dispute(&payer, &1),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn set_fee_bps_updates_config_at_the_cap() {
    let s = Setup::new();

    s.client().set_fee_bps(&MAX_FEE_BPS);
    assert_eq!(s.client().get_config().fee_bps, MAX_FEE_BPS);

    s.client().set_fee_bps(&0);
    assert_eq!(s.client().get_config().fee_bps, 0);
}

#[test]
fn set_fee_bps_rejects_above_cap() {
    let s = Setup::new();
    let result = s.client().try_set_fee_bps(&(MAX_FEE_BPS + 1));

    assert_eq!(result, Err(Ok(Error::InvalidFeeBps)));
    assert_eq!(s.client().get_config().fee_bps, FEE_BPS);
}

#[test]
fn set_fee_recipient_updates_config() {
    let s = Setup::new();
    let new_recipient = Address::generate(&s.env);

    s.client().set_fee_recipient(&new_recipient);

    assert_eq!(s.client().get_config().fee_recipient, new_recipient);
}

#[test]
fn create_deal_stores_created_deal() {
    let s = Setup::new();
    let deal_id = s.create_deal(AMOUNT);

    assert_eq!(deal_id, 1);
    assert_eq!(s.client().get_deal_count(), 1);

    let deal = s.client().get_deal(&deal_id);
    assert_eq!(deal.id, deal_id);
    assert_eq!(deal.payer, s.payer);
    assert_eq!(deal.payee, s.payee);
    assert_eq!(deal.arbiter, s.arbiter);
    assert_eq!(deal.amount, AMOUNT);
    assert_eq!(deal.deadline, START + DAY);
    assert_eq!(deal.created_at, START);
    assert_eq!(deal.status, DealStatus::Created);
}

#[test]
fn create_deal_increments_ids() {
    let s = Setup::new();

    assert_eq!(s.create_deal(AMOUNT), 1);
    assert_eq!(s.create_deal(AMOUNT + 1), 2);
    assert_eq!(s.client().get_deal_count(), 2);
    assert_eq!(s.client().get_deal(&2).amount, AMOUNT + 1);
}

#[test]
fn create_deal_rejects_non_positive_amounts() {
    let s = Setup::new();

    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.arbiter, &0, &(START + DAY)),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.arbiter, &-1, &(START + DAY)),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(s.client().get_deal_count(), 0);
}

#[test]
fn create_deal_rejects_shared_parties() {
    let s = Setup::new();
    let deadline = START + DAY;

    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payer, &s.arbiter, &AMOUNT, &deadline),
        Err(Ok(Error::InvalidParty))
    );
    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.payer, &AMOUNT, &deadline),
        Err(Ok(Error::InvalidParty))
    );
    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.payee, &AMOUNT, &deadline),
        Err(Ok(Error::InvalidParty))
    );
}

#[test]
fn create_deal_rejects_deadline_not_in_future() {
    let s = Setup::new();

    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.arbiter, &AMOUNT, &START),
        Err(Ok(Error::InvalidDeadline))
    );
    assert_eq!(
        s.client()
            .try_create_deal(&s.payer, &s.payee, &s.arbiter, &AMOUNT, &(START - 1)),
        Err(Ok(Error::InvalidDeadline))
    );
}

#[test]
fn fund_deal_moves_funds_into_escrow() {
    let s = Setup::new();
    s.mint(&s.payer, AMOUNT);
    let deal_id = s.create_deal(AMOUNT);

    s.client().fund_deal(&s.payer, &deal_id);

    assert_eq!(s.balance(&s.payer), 0);
    assert_eq!(s.balance(&s.contract), AMOUNT);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Funded);
}

#[test]
fn fund_deal_rejects_wrong_payer() {
    let s = Setup::new();
    s.mint(&s.payer, AMOUNT);
    let deal_id = s.create_deal(AMOUNT);
    let stranger = Address::generate(&s.env);

    assert_eq!(
        s.client().try_fund_deal(&stranger, &deal_id),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(s.balance(&s.payer), AMOUNT);
}

#[test]
fn fund_deal_rejects_double_funding() {
    let s = Setup::new();
    s.mint(&s.payer, AMOUNT * 2);
    let deal_id = s.create_deal(AMOUNT);
    s.client().fund_deal(&s.payer, &deal_id);

    assert_eq!(
        s.client().try_fund_deal(&s.payer, &deal_id),
        Err(Ok(Error::InvalidStatus))
    );
    assert_eq!(s.balance(&s.contract), AMOUNT);
}

#[test]
fn fund_deal_rejects_expired_deal() {
    let s = Setup::new();
    s.mint(&s.payer, AMOUNT);
    let deal_id = s.create_deal(AMOUNT);

    s.env.ledger().set_timestamp(START + DAY);

    assert_eq!(
        s.client().try_fund_deal(&s.payer, &deal_id),
        Err(Ok(Error::DealExpired))
    );
    assert_eq!(s.balance(&s.payer), AMOUNT);
}

#[test]
fn release_pays_payee_net_and_fee_recipient() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    s.client().release(&s.payer, &deal_id);

    let fee = AMOUNT * FEE_BPS as i128 / 10_000;
    assert_eq!(s.balance(&s.payee), AMOUNT - fee);
    assert_eq!(s.balance(&s.fee_recipient), fee);
    assert_eq!(s.balance(&s.contract), 0);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Released);
}

#[test]
fn release_with_zero_fee_pays_full_amount() {
    let s = Setup::new();
    s.client().set_fee_bps(&0);
    let deal_id = s.funded_deal(AMOUNT);

    s.client().release(&s.payer, &deal_id);

    assert_eq!(s.balance(&s.payee), AMOUNT);
    assert_eq!(s.balance(&s.fee_recipient), 0);
}

#[test]
fn release_rejects_non_payer_and_unfunded_deals() {
    let s = Setup::new();
    let stranger = Address::generate(&s.env);
    let deal_id = s.funded_deal(AMOUNT);

    assert_eq!(
        s.client().try_release(&stranger, &deal_id),
        Err(Ok(Error::Unauthorized))
    );

    let unfunded = s.create_deal(AMOUNT);
    assert_eq!(
        s.client().try_release(&s.payer, &unfunded),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn release_twice_is_rejected() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);
    s.client().release(&s.payer, &deal_id);

    assert_eq!(
        s.client().try_release(&s.payer, &deal_id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn refund_returns_full_amount_to_payer() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    s.client().refund(&s.payee, &deal_id);

    assert_eq!(s.balance(&s.payer), AMOUNT);
    assert_eq!(s.balance(&s.payee), 0);
    assert_eq!(s.balance(&s.fee_recipient), 0);
    assert_eq!(s.balance(&s.contract), 0);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Refunded);
}

#[test]
fn refund_rejects_non_payee() {
    let s = Setup::new();
    let stranger = Address::generate(&s.env);
    let deal_id = s.funded_deal(AMOUNT);

    assert_eq!(
        s.client().try_refund(&stranger, &deal_id),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        s.client().try_refund(&s.payer, &deal_id),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn refund_rejects_unfunded_deal() {
    let s = Setup::new();
    let deal_id = s.create_deal(AMOUNT);

    assert_eq!(
        s.client().try_refund(&s.payee, &deal_id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn either_party_can_raise_a_dispute() {
    let s = Setup::new();

    let first = s.funded_deal(AMOUNT);
    s.client().raise_dispute(&s.payer, &first);
    assert_eq!(s.client().get_deal(&first).status, DealStatus::Disputed);

    let second = s.funded_deal(AMOUNT);
    s.client().raise_dispute(&s.payee, &second);
    assert_eq!(s.client().get_deal(&second).status, DealStatus::Disputed);
}

#[test]
fn raise_dispute_rejects_strangers_and_unfunded_deals() {
    let s = Setup::new();
    let stranger = Address::generate(&s.env);
    let deal_id = s.funded_deal(AMOUNT);

    assert_eq!(
        s.client().try_raise_dispute(&stranger, &deal_id),
        Err(Ok(Error::Unauthorized))
    );

    let unfunded = s.create_deal(AMOUNT);
    assert_eq!(
        s.client().try_raise_dispute(&s.payer, &unfunded),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn raise_dispute_twice_is_rejected() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);
    s.client().raise_dispute(&s.payer, &deal_id);

    assert_eq!(
        s.client().try_raise_dispute(&s.payee, &deal_id),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn resolve_dispute_can_release_to_payee() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);
    s.client().raise_dispute(&s.payer, &deal_id);

    s.client().resolve_dispute(&deal_id, &true);

    let fee = AMOUNT * FEE_BPS as i128 / 10_000;
    assert_eq!(s.balance(&s.payee), AMOUNT - fee);
    assert_eq!(s.balance(&s.fee_recipient), fee);
    assert_eq!(s.balance(&s.contract), 0);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Released);
}

#[test]
fn resolve_dispute_can_refund_payer_in_full() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);
    s.client().raise_dispute(&s.payee, &deal_id);

    s.client().resolve_dispute(&deal_id, &false);

    assert_eq!(s.balance(&s.payer), AMOUNT);
    assert_eq!(s.balance(&s.payee), 0);
    assert_eq!(s.balance(&s.fee_recipient), 0);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Refunded);
}

#[test]
fn resolve_dispute_requires_disputed_status() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    assert_eq!(
        s.client().try_resolve_dispute(&deal_id, &true),
        Err(Ok(Error::InvalidStatus))
    );

    s.client().raise_dispute(&s.payer, &deal_id);
    s.client().resolve_dispute(&deal_id, &true);
    assert_eq!(
        s.client().try_resolve_dispute(&deal_id, &false),
        Err(Ok(Error::InvalidStatus))
    );
}

#[test]
fn refund_expired_reclaims_after_deadline() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    s.env.ledger().set_timestamp(START + DAY);
    s.client().refund_expired(&s.payer, &deal_id);

    assert_eq!(s.balance(&s.payer), AMOUNT);
    assert_eq!(s.balance(&s.contract), 0);
    assert_eq!(s.client().get_deal(&deal_id).status, DealStatus::Refunded);
}

#[test]
fn refund_expired_rejects_early_calls() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    s.env.ledger().set_timestamp(START + DAY - 1);
    assert_eq!(
        s.client().try_refund_expired(&s.payer, &deal_id),
        Err(Ok(Error::DealNotExpired))
    );
}

#[test]
fn refund_expired_rejects_non_payer() {
    let s = Setup::new();
    let stranger = Address::generate(&s.env);
    let deal_id = s.funded_deal(AMOUNT);

    s.env.ledger().set_timestamp(START + DAY);
    assert_eq!(
        s.client().try_refund_expired(&stranger, &deal_id),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn get_deal_rejects_unknown_id() {
    let s = Setup::new();

    assert_eq!(s.client().try_get_deal(&99), Err(Ok(Error::DealNotFound)));
}

#[test]
fn release_fee_overflow_is_reported() {
    let s = Setup::new();
    let amount = i128::MAX / 2;
    let deal_id = s.funded_deal(amount);

    assert_eq!(
        s.client().try_release(&s.payer, &deal_id),
        Err(Ok(Error::ArithmeticOverflow))
    );
    assert_eq!(s.balance(&s.contract), amount);
}

#[test]
fn emitting_events() {
    let s = Setup::new();
    let deal_id = s.funded_deal(AMOUNT);

    let mut data: Map<Symbol, Val> = Map::new(&s.env);
    data.set(Symbol::new(&s.env, "amount"), AMOUNT.into_val(&s.env));
    data.set(
        Symbol::new(&s.env, "payer"),
        s.payer.clone().into_val(&s.env),
    );

    let topics: Vec<Val> = Vec::from_array(
        &s.env,
        [
            symbol_short!("deal").into_val(&s.env),
            symbol_short!("funded").into_val(&s.env),
            deal_id.into_val(&s.env),
        ],
    );
    let expected: Vec<(Address, Vec<Val>, Val)> = Vec::from_array(
        &s.env,
        [(s.contract.clone(), topics, data.into_val(&s.env))],
    );

    let events = s.env.events().all().filter_by_contract(&s.contract);
    assert_eq!(events, expected);
}

#[test]
#[should_panic]
fn create_deal_without_payer_auth_panics() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(START);

    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let contract = env.register(PrimaryContract, ());
    let client = PrimaryContractClient::new(&env, &contract);
    client.initialize(&admin, &token, &FEE_BPS, &admin);

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let arbiter = Address::generate(&env);

    env.mock_auths(&[]);
    client.create_deal(&payer, &payee, &arbiter, &AMOUNT, &(START + DAY));
}
