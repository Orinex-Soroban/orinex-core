#![no_std]

mod errors;

use soroban_sdk::contract;

pub use errors::Error;

#[contract]
pub struct PrimaryContract;
