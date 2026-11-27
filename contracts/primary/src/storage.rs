use soroban_sdk::{contracttype, Address, Env};

use crate::errors::Error;

/// Ledgers per day at Soroban's 5 second ledger close time.
const DAY_IN_LEDGERS: u32 = 17_280;

/// Extend instance entries when their remaining TTL drops below 7 days.
pub const INSTANCE_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;
/// Extend instance entries out to 30 days.
pub const INSTANCE_TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
/// Extend deal records when their remaining TTL drops below 30 days.
pub const PERSISTENT_TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
/// Extend deal records out to 90 days.
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 90 * DAY_IN_LEDGERS;

/// Maximum protocol fee: 10% (1_000 basis points).
pub const MAX_FEE_BPS: u32 = 1_000;
/// Basis point denominator: 10_000 basis points == 100%.
pub const BPS_DENOMINATOR: i128 = 10_000;

/// Storage keys for contract configuration (instance) and deals (persistent).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// `Address` of the contract admin; also the initialization marker.
    Admin,
    /// `Address` of the SEP-41 asset contract that is escrowed.
    Token,
    /// `Address` that receives protocol fees.
    FeeRecipient,
    /// `u32` protocol fee in basis points.
    FeeBps,
    /// `u64` number of deals created so far.
    DealCount,
    /// `Deal` record keyed by deal id.
    Deal(u64),
}

/// Lifecycle of an escrow deal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DealStatus {
    /// Terms agreed, funds not yet deposited.
    Created = 0,
    /// Funds are held by the contract.
    Funded = 1,
    /// Funds were paid out to the payee (minus fee).
    Released = 2,
    /// Funds were returned to the payer.
    Refunded = 3,
    /// Funds are held and settlement is delegated to the arbiter.
    Disputed = 4,
}

/// Immutable protocol configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub token: Address,
    pub fee_recipient: Address,
    pub fee_bps: u32,
}

/// A single escrow deal.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deal {
    pub id: u64,
    pub payer: Address,
    pub payee: Address,
    pub arbiter: Address,
    /// Amount held in escrow, in the token's smallest unit.
    pub amount: i128,
    /// Unix timestamp after which the deal can no longer be funded.
    pub deadline: u64,
    pub status: DealStatus,
    /// Unix timestamp at which the deal was created.
    pub created_at: u64,
}

/// Extend the TTL of instance entries.
pub fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Fail with `NotInitialized` unless `initialize` has run.
pub fn ensure_initialized(env: &Env) -> Result<(), Error> {
    bump_instance_ttl(env);
    if env.storage().instance().has(&DataKey::Admin) {
        Ok(())
    } else {
        Err(Error::NotInitialized)
    }
}

/// Read the protocol configuration, extending the TTL while doing so.
pub fn read_config(env: &Env) -> Result<Config, Error> {
    bump_instance_ttl(env);
    let storage = env.storage().instance();
    Ok(Config {
        admin: storage.get(&DataKey::Admin).ok_or(Error::NotInitialized)?,
        token: storage.get(&DataKey::Token).ok_or(Error::NotInitialized)?,
        fee_recipient: storage
            .get(&DataKey::FeeRecipient)
            .ok_or(Error::NotInitialized)?,
        fee_bps: storage.get(&DataKey::FeeBps).ok_or(Error::NotInitialized)?,
    })
}

/// Read a deal record, extending its TTL while doing so.
pub fn read_deal(env: &Env, deal_id: u64) -> Result<Deal, Error> {
    let key = DataKey::Deal(deal_id);
    let storage = env.storage().persistent();
    let deal: Deal = storage.get(&key).ok_or(Error::DealNotFound)?;
    storage.extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    Ok(deal)
}

/// Persist a deal record and extend both persistent and instance TTLs.
pub fn write_deal(env: &Env, deal: &Deal) {
    let key = DataKey::Deal(deal.id);
    let storage = env.storage().persistent();
    storage.set(&key, deal);
    storage.extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    bump_instance_ttl(env);
}

/// Allocate the next deal id, persisting the incremented counter.
pub fn next_deal_id(env: &Env) -> Result<u64, Error> {
    bump_instance_ttl(env);
    let storage = env.storage().instance();
    let current: u64 = storage.get(&DataKey::DealCount).unwrap_or(0);
    let next = current.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    storage.set(&DataKey::DealCount, &next);
    Ok(next)
}

/// Total number of deals created.
pub fn deal_count(env: &Env) -> u64 {
    bump_instance_ttl(env);
    env.storage()
        .instance()
        .get(&DataKey::DealCount)
        .unwrap_or(0)
}
