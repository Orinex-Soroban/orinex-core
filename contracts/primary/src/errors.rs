use soroban_sdk::contracterror;

/// Error codes returned by the primary escrow contract.
///
/// Codes are stable and part of the contract's public ABI: never renumber an
/// existing variant, append new ones instead.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `initialize` was called on a contract that already has an admin.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// Fee basis points exceed the configured maximum.
    InvalidFeeBps = 3,
    /// Escrowed amount must be strictly positive.
    InvalidAmount = 4,
    /// Deal deadline must lie in the future.
    InvalidDeadline = 5,
    /// Counterparties and arbiter must be distinct addresses.
    InvalidParty = 6,
    /// No deal exists for the given id.
    DealNotFound = 7,
    /// The deal is not in a status that allows this transition.
    InvalidStatus = 8,
    /// A checked arithmetic operation overflowed.
    ArithmeticOverflow = 9,
    /// The deal deadline has passed and it can no longer be funded.
    DealExpired = 10,
    /// The deal deadline has not passed yet.
    DealNotExpired = 11,
    /// The caller is not authorized for this deal operation.
    Unauthorized = 12,
}
