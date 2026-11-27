# Protocol mechanics

One contract instance holds escrowed funds for **one asset** and settles
**many deals**. The asset is chosen once at `initialize` and cannot change; a
second asset means a second deployment.

## Deal states

| Status | Value | Money is | Can become |
|---|---|---|---|
| `Created` | `0` | still with the payer | `Funded` |
| `Funded` | `1` | inside the contract | `Released`, `Refunded`, `Disputed` |
| `Released` | `2` | with the payee and fee recipient | terminal |
| `Refunded` | `3` | back with the payer | terminal |
| `Disputed` | `4` | inside the contract | `Released`, `Refunded` |

Terminal states are terminal: a second release, refund or dispute on the same
deal fails with `InvalidStatus` (code 8). That is the whole settlement-safety
argument — the contract will not pay twice.

## Transitions

| Call | Who signs | Required status | Extra conditions | Effect |
|---|---|---|---|---|
| `create_deal` | payer | — | `amount > 0`, `deadline > now`, payer/payee/arbiter all distinct | Stores `Created` deal, id = `DealCount + 1` |
| `fund_deal` | payer | `Created` | `now < deadline` | Pulls `amount` from payer via the token contract, status `Funded` |
| `release` | payer | `Funded` | — | Pays payee `amount - fee`, fee recipient `fee`, status `Released` |
| `refund` | payee | `Funded` | — | Pays payer the full amount, status `Refunded` |
| `raise_dispute` | payer or payee | `Funded` | — | Status `Disputed`, no transfer |
| `resolve_dispute` | deal's arbiter | `Disputed` | — | `release_to_payee` decides: release path or full refund to payer |
| `refund_expired` | payer | `Funded` | `now >= deadline` | Pays payer the full amount, status `Refunded` |

Deadline comparisons are deliberate and specific: funding requires
`now < deadline` (at the deadline itself, funding is refused with `DealExpired`),
and the payer's reclaim requires `now >= deadline` (at the deadline, reclaim is
already allowed, so there is no gap where neither is possible).

Note what a refund is: it is **payee-authorised**. If the work can't happen, the
payee can end the deal and give the money back without needing the payer awake.
The payer cannot unilaterally refund a funded deal — that would let them pull
the rug after the payee starts work.

## Fees

- Fee is stored as basis points on the instance: `u32`, `10_000` = 100%.
- `MAX_FEE_BPS = 1_000` (10%). `initialize` and `set_fee_bps` both reject
  anything above it with `InvalidFeeBps` (code 3).
- Computation is `fee = amount * fee_bps / 10_000` in checked `i128`
  arithmetic, **floored**. `net = amount - fee`. The invariant is
  `net + fee == amount`, exactly, always.
- Fees apply only to payee releases: `release` and `resolve_dispute` with
  `release_to_payee = true`. Refunds and expired reclaims are fee-free.
- The fee is read from config **at release time**, not at creation. A deal
  created under 250 bps can settle under a different rate if the admin changes
  it in between. Admin fee changes are capped at 10%, so the worst case is
  bounded.

### Worked numbers

Escrow of **1 XLM** (`10_000_000` stroops) at the deployed 250 bps:

```
fee = 10_000_000 * 250 / 10_000 = 250_000
net = 10_000_000 - 250_000      = 9_750_000
```

Payee receives `9_750_000`, fee recipient receives `250_000`, contract balance
returns to zero. This is the cycle that was executed against the testnet
deployment.

Fee by rate at the same amount:

| Fee | Basis points | Fee charged | Payee receives |
|---|---|---|---|
| 0% | 0 | 0 | 10,000,000 |
| 2.5% | 250 | 250,000 | 9,750,000 |
| 5% | 500 | 500,000 | 9,500,000 |
| 10% (cap) | 1,000 | 1,000,000 | 9,000,000 |

### Rounding at small amounts

Flooring means the protocol takes nothing below one fee unit, and never more
than the amount:

| Amount | Rate | Fee | Payee net |
|---|---|---|---|
| 9 | 1,000 bps | 1.9 → **0** | 9 |
| 10 | 1,000 bps | 1.0 → **1** | 9 |
| 19 | 1,000 bps | 1.9 → **1** | 18 |
| 1,000,000,007 | 1,000 bps | 100,000,000.7 → **100,000,000** | 900,000,007 |

This is honest for micro-escrow: at 10% and an amount of 9 units, the fee is
zero rather than rounding up to 1 (which would be a 11% effective rate).

### Overflow

`soroban-sdk` checked arithmetic is used throughout. An amount large enough to
overflow `amount * fee_bps` returns `ArithmeticOverflow` (code 9) instead of
wrapping. The largest safe amount at the 10% cap is `i128::MAX / 1_000`; the
test suite releases exactly that amount.

## Storage and lifetime

Configuration lives in instance storage; each deal lives in persistent storage
under `DataKey::Deal(id)`.

| Entry | Extend when remaining TTL < | Extend to |
|---|---|---|
| Instance (config, counters) | 7 days (`120_960` ledgers) | 30 days (`518_400`) |
| Deal record | 30 days (`518_400`) | 90 days (`1_555_200`) |

One day is `17_280` ledgers at 5-second close time. TTLs are bumped on every
read and write path, so an escrow that is being touched cannot archive
mid-flight. A deal nobody touches for 30 days can archive — its record can be
restored from the network afterwards, but a UI should not treat "missing deal"
as "never existed" without checking.

## What the admin can and cannot do

Can: change the fee recipient, change the fee within 0–1,000 bps, both
immediately.

Cannot: move escrowed funds, cancel a deal, release or refund on anyone's
behalf, change deal terms, seize the token balance, or replace the arbiter on an
existing deal. There is no function in the contract that transfers escrowed
tokens to the admin.

This is a deliberate limitation, not an oversight: any admin escape hatch would
make the escrow custodial and change what users are being promised.
