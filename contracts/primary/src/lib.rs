#![no_std]

mod errors;
pub mod events;
pub mod storage;

use soroban_sdk::contract;

pub use errors::Error;
pub use storage::{Config, Deal, DealStatus};

#[contract]
pub struct PrimaryContract;
