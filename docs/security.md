# Security

## Audit status

The contracts are **unaudited**. They have 39 unit tests, a clean clippy run and
a live testnet deployment — that is a starting point, not an assurance. Do not
deploy this with real funds before an independent audit.

## Trust model

| Party | Can | Cannot |
|---|---|---|
| Payer | create, fund, release, dispute, reclaim after deadline | refund a funded deal, release to anyone but the payee |
| Payee | refund, dispute | release funds to themselves, change terms |
| Arbiter (per deal) | settle a disputed deal either way | touch any other deal, act before a dispute exists |
| Admin | change fee (≤ 10%), change fee recipient | move escrowed funds, cancel deals, replace arbiters |
| Contract | hold one asset, transfer exactly once per deal | — |

The absence of an admin escape hatch is deliberate. Any "emergency withdraw"
would make this custodial, and the guarantee to the payee — funds are committed
and only the agreed paths move them — would stop being true.

## Known limitations

- **One asset per deployment.** Mixed-asset escrow means separate contract
  instances and separate trust in each deployment's config.
- **No partial settlement.** The full amount goes one way. Milestone payments
  need multiple deals.
- **Arbiter availability is not enforced.** An inactive arbiter leaves a
  disputed deal frozen; nothing times out a dispute.
- **Fee changes are not locked at creation.** The rate at settlement applies.
  Capped at 10%, but not fixed per deal.
- **Deadline is absolute.** There is no grace period, and the payer can reclaim
  from the deadline instant.
- **No pause switch.** If a bug is found, there is no way to freeze the
  contract; the migration path would be deploying a new instance.

## Reporting

Use GitHub's private advisory flow, described in
[`SECURITY.md`](https://github.com/Orinex-Soroban/orinex-core/blob/main/SECURITY.md)
in the repository root. Do not open a public issue.

## Before mainnet

- External audit of `lib.rs` and `storage.rs`, including the TTL behaviour under
  adversarial access patterns.
- Fuzz fee arithmetic across the full `i128` range and all `fee_bps` values.
- Decide and document the arbitration policy — reputation, staking, or
  explicitly "trusted third party".
- A monitoring plan for `DisputeResolved` and failed settlements.
