#![no_std]

mod errors;
pub mod events;
pub mod storage;

use soroban_sdk::{contract, contractimpl, token, Address, Env};

pub use errors::Error;
pub use storage::{Config, Deal, DealStatus};

use storage::{bump_instance_ttl, read_config, DataKey, MAX_FEE_BPS};

#[contract]
pub struct PrimaryContract;

#[contractimpl]
impl PrimaryContract {
    /// Configure the escrow contract once.
    ///
    /// `admin` must authorize. Sets the escrowed asset, the protocol fee (in
    /// basis points, capped at [`MAX_FEE_BPS`]) and the fee recipient.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        fee_bps: u32,
        fee_recipient: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        if fee_bps > MAX_FEE_BPS {
            return Err(Error::InvalidFeeBps);
        }

        let storage = env.storage().instance();
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::FeeRecipient, &fee_recipient);
        storage.set(&DataKey::FeeBps, &fee_bps);
        storage.set(&DataKey::DealCount, &0u64);
        bump_instance_ttl(&env);

        events::initialized(&env, &admin, &token, &fee_recipient, fee_bps);
        Ok(())
    }

    /// Update the protocol fee in basis points. Admin only.
    pub fn set_fee_bps(env: Env, fee_bps: u32) -> Result<(), Error> {
        // The admin is only known from storage, so config is read first; the
        // authorization check still precedes every mutation.
        let config = read_config(&env)?;
        config.admin.require_auth();

        if fee_bps > MAX_FEE_BPS {
            return Err(Error::InvalidFeeBps);
        }

        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        bump_instance_ttl(&env);
        events::fee_bps_updated(&env, fee_bps);
        Ok(())
    }

    /// Update the fee recipient. Admin only.
    pub fn set_fee_recipient(env: Env, fee_recipient: Address) -> Result<(), Error> {
        let config = read_config(&env)?;
        config.admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::FeeRecipient, &fee_recipient);
        bump_instance_ttl(&env);
        events::fee_recipient_updated(&env, &fee_recipient);
        Ok(())
    }

    /// Read the current protocol configuration.
    pub fn get_config(env: Env) -> Result<Config, Error> {
        read_config(&env)
    }

    /// Create a deal in the `Created` state; funds move only in [`fund_deal`].
    ///
    /// The payer must authorize. Payer, payee and arbiter must be distinct, the
    /// amount strictly positive, and the deadline in the future.
    pub fn create_deal(
        env: Env,
        payer: Address,
        payee: Address,
        arbiter: Address,
        amount: i128,
        deadline: u64,
    ) -> Result<u64, Error> {
        payer.require_auth();
        storage::ensure_initialized(&env)?;

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if payer == payee || arbiter == payer || arbiter == payee {
            return Err(Error::InvalidParty);
        }

        let created_at = env.ledger().timestamp();
        if deadline <= created_at {
            return Err(Error::InvalidDeadline);
        }

        let deal_id = storage::next_deal_id(&env)?;
        let deal = Deal {
            id: deal_id,
            payer,
            payee,
            arbiter,
            amount,
            deadline,
            status: DealStatus::Created,
            created_at,
        };

        storage::write_deal(&env, &deal);
        events::deal_created(&env, &deal);
        Ok(deal_id)
    }

    /// Deposit the deal amount into escrow, moving it from the payer to the
    /// contract. The payer must authorize.
    pub fn fund_deal(env: Env, payer: Address, deal_id: u64) -> Result<(), Error> {
        payer.require_auth();

        let config = read_config(&env)?;
        let mut deal = storage::read_deal(&env, deal_id)?;

        if deal.payer != payer {
            return Err(Error::Unauthorized);
        }
        if deal.status != DealStatus::Created {
            return Err(Error::InvalidStatus);
        }
        if env.ledger().timestamp() >= deal.deadline {
            return Err(Error::DealExpired);
        }

        let token = token::Client::new(&env, &config.token);
        token.transfer(&payer, env.current_contract_address(), &deal.amount);

        deal.status = DealStatus::Funded;
        storage::write_deal(&env, &deal);
        events::deal_funded(&env, deal_id, &payer, deal.amount);
        Ok(())
    }

    /// Release escrowed funds to the payee, deducting the protocol fee. The
    /// payer must authorize.
    pub fn release(env: Env, payer: Address, deal_id: u64) -> Result<(), Error> {
        payer.require_auth();

        let config = read_config(&env)?;
        let mut deal = storage::read_deal(&env, deal_id)?;

        if deal.payer != payer {
            return Err(Error::Unauthorized);
        }
        if deal.status != DealStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        let fee = fee_amount(deal.amount, config.fee_bps)?;
        let net = deal
            .amount
            .checked_sub(fee)
            .ok_or(Error::ArithmeticOverflow)?;

        let token = token::Client::new(&env, &config.token);
        let contract = env.current_contract_address();
        if fee > 0 {
            token.transfer(&contract, &config.fee_recipient, &fee);
        }
        token.transfer(&contract, &deal.payee, &net);

        deal.status = DealStatus::Released;
        storage::write_deal(&env, &deal);
        events::deal_released(&env, deal_id, &deal.payee, net, fee);
        Ok(())
    }
}

/// Protocol fee for `amount` at `fee_bps` basis points, rounded down.
fn fee_amount(amount: i128, fee_bps: u32) -> Result<i128, Error> {
    let scaled = amount
        .checked_mul(fee_bps as i128)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(scaled / storage::BPS_DENOMINATOR)
}
