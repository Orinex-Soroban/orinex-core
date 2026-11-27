#![no_std]

mod errors;
pub mod events;
pub mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};

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
}
