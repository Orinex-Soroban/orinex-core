# Introduction

Orinex is an escrow contract for Stellar's Soroban smart-contract platform. Two
parties agree on terms, one of them locks the money in the contract, and the
money leaves only in one of three ways: the payer releases it, the payee refunds
it, or a named arbiter decides. Nobody — including the admin — can reach into a
funded deal and move the money.

## The problem it addresses

Cross-border work and small-ticket trade run on trust that the two sides don't
have yet. The traditional answer is a platform in the middle holding the money,
which brings its own problems: account freezes, withdrawal delays, and a
take-it-or-leave-it fee.

The scale of what's moving on Stellar is the reason this is worth doing on-chain
rather than through another platform. Stellar's Q3 2026 report puts monthly
payment volume at **$18.5 billion in October 2026**, up from $1.8 billion in
January, with **$25.1 billion** of stablecoin transfer volume in the quarter.
The money is already here; the settlement layer for conditional payments is
what's missing in most workflows.

Escrow on Stellar is not new — Trustless Work, SafeTrust and KindFi all run
escrow products on this network. `docs/positioning.md` says plainly where this
contract competes and where it doesn't. Read that before assuming this is the
right tool.

## How it works

1. **Agree terms.** The payer calls `create_deal` with the payee, an arbiter, an
   amount, and a deadline. Nothing moves yet. The contract assigns a deal id.
2. **Fund.** The payer calls `fund_deal` before the deadline. The full amount
   moves into the contract; the deal is now live.
3. **Settle.** One of:
   - the payer calls `release` — the payee gets the amount minus the protocol
     fee, the fee recipient gets the fee;
   - the payee calls `refund` — the payer gets everything back, no fee;
   - either party calls `raise_dispute`, which freezes settlement until the
     arbiter calls `resolve_dispute` and chooses a release or a refund.
4. **If nobody settles.** Once the deadline passes with funds still held, the
   payer can call `refund_expired` and take the money back. The payee can still
   ask the arbiter to settle.

Every step is authorised against the specific address it belongs to, and every
step emits an event, so a UI or indexer can reconstruct state without reading
storage.

## Who it's for

- **Platforms** that need conditional payouts without building escrow custody
  themselves.
- **Payers** who want proof that funds are committed but release only on terms
  they approve.
- **Payees** who want the money visibly locked before starting work, and a
  dispute path that isn't "ask the platform nicely".
- **Arbiters** — people or services named per deal who settle disagreements.

## Status

Live on Stellar testnet, unaudited. Contract
`CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV`, verified with a full
create → fund → release cycle. Do not deploy this with real funds before an
independent audit; see [Security](security.md).
