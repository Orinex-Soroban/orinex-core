#![no_std]

mod errors;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;

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

    /// Return the escrowed amount to the payer without a fee. The payee must
    /// authorize, so only the counterparty can cancel a funded deal.
    pub fn refund(env: Env, payee: Address, deal_id: u64) -> Result<(), Error> {
        payee.require_auth();

        let config = read_config(&env)?;
        let mut deal = storage::read_deal(&env, deal_id)?;

        if deal.payee != payee {
            return Err(Error::Unauthorized);
        }
        if deal.status != DealStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        let token = token::Client::new(&env, &config.token);
        token.transfer(&env.current_contract_address(), &deal.payer, &deal.amount);

        deal.status = DealStatus::Refunded;
        storage::write_deal(&env, &deal);
        events::deal_refunded(&env, deal_id, &deal.payer, deal.amount);
        Ok(())
    }

    /// Escalate a funded deal to the arbiter. Either the payer or the payee may
    /// call this; settlement is then decided by the arbiter.
    pub fn raise_dispute(env: Env, caller: Address, deal_id: u64) -> Result<(), Error> {
        caller.require_auth();
        storage::ensure_initialized(&env)?;

        let mut deal = storage::read_deal(&env, deal_id)?;

        if caller != deal.payer && caller != deal.payee {
            return Err(Error::Unauthorized);
        }
        if deal.status != DealStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        deal.status = DealStatus::Disputed;
        storage::write_deal(&env, &deal);
        events::deal_disputed(&env, deal_id, &caller);
        Ok(())
    }

    /// Settle a disputed deal. The arbiter must authorize.
    ///
    /// With `release_to_payee` the payee receives the amount net of the protocol
    /// fee and the fee recipient the remainder; otherwise the payer is refunded
    /// in full with no fee taken.
    pub fn resolve_dispute(env: Env, deal_id: u64, release_to_payee: bool) -> Result<(), Error> {
        let config = read_config(&env)?;
        let mut deal = storage::read_deal(&env, deal_id)?;

        // The arbiter address lives in the deal record, so the record is read
        // before the authorization check, which still precedes every transfer
        // and storage mutation.
        deal.arbiter.require_auth();

        if deal.status != DealStatus::Disputed {
            return Err(Error::InvalidStatus);
        }

        let token = token::Client::new(&env, &config.token);
        let contract = env.current_contract_address();

        let (net, fee) = if release_to_payee {
            let fee = fee_amount(deal.amount, config.fee_bps)?;
            let net = deal
                .amount
                .checked_sub(fee)
                .ok_or(Error::ArithmeticOverflow)?;
            if fee > 0 {
                token.transfer(&contract, &config.fee_recipient, &fee);
            }
            token.transfer(&contract, &deal.payee, &net);
            (net, fee)
        } else {
            token.transfer(&contract, &deal.payer, &deal.amount);
            (deal.amount, 0)
        };

        deal.status = if release_to_payee {
            DealStatus::Released
        } else {
            DealStatus::Refunded
        };
        storage::write_deal(&env, &deal);
        events::dispute_resolved(&env, deal_id, release_to_payee, net, fee);
        Ok(())
    }

    /// Reclaim escrow once the deal deadline has passed. The payer must
    /// authorize; the payee keeps the option to refund at any time.
    pub fn refund_expired(env: Env, payer: Address, deal_id: u64) -> Result<(), Error> {
        payer.require_auth();

        let config = read_config(&env)?;
        let mut deal = storage::read_deal(&env, deal_id)?;

        if deal.payer != payer {
            return Err(Error::Unauthorized);
        }
        if deal.status != DealStatus::Funded {
            return Err(Error::InvalidStatus);
        }
        if env.ledger().timestamp() < deal.deadline {
            return Err(Error::DealNotExpired);
        }

        let token = token::Client::new(&env, &config.token);
        token.transfer(&env.current_contract_address(), &deal.payer, &deal.amount);

        deal.status = DealStatus::Refunded;
        storage::write_deal(&env, &deal);
        events::deal_refunded(&env, deal_id, &deal.payer, deal.amount);
        Ok(())
    }

    /// Read a deal by id.
    pub fn get_deal(env: Env, deal_id: u64) -> Result<Deal, Error> {
        storage::read_deal(&env, deal_id)
    }

    /// Total number of deals created so far; deal ids run from 1 to this value.
    pub fn get_deal_count(env: Env) -> u64 {
        storage::deal_count(&env)
    }
}

/// Protocol fee for `amount` at `fee_bps` basis points, rounded down.
fn fee_amount(amount: i128, fee_bps: u32) -> Result<i128, Error> {
    let scaled = amount
        .checked_mul(fee_bps as i128)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(scaled / storage::BPS_DENOMINATOR)
}
