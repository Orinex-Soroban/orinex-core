# Payee guide

You are the side getting paid. Escrow protects you from doing the work and then
being told the invoice "wasn't approved" — the money is already locked when you
start.

## Before you start work

Do not take the payer's word that a deal exists. Read it:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet \
  -- get_deal --deal_id 1
```

Check four fields:

- `status` is `1` (`Funded`). `0` means the money is not in the contract yet — a
  promise, not an escrow.
- `payee` is your address.
- `amount` is what you agreed, in the token's smallest unit.
- `deadline` gives you a time by which the deal must be settled or reclaimed.

Also read the config to see the fee and the asset:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet -- get_config
```

At 250 bps on a 10 XLM deal you receive 9.75 XLM, not 10. Price the fee into
your quote rather than being surprised at settlement.

## Getting paid

You cannot release funds to yourself. Only the payer can call `release`, or the
arbiter can resolve a dispute in your favour. Your leverage is the funded
status: the money is visibly committed.

## If you can't do the work: refund

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- refund --payee <MY_ADDRESS> --deal_id 1
```

You control this path, and it is fee-free. Returning the money promptly is how
you stay usable as a counterparty — a payer who has to wait for a deadline to
recover funds will not fund you again.

## If there's a disagreement: dispute

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- raise_dispute --caller <MY_ADDRESS> --deal_id 1
```

This freezes settlement and hands the decision to the deal's arbiter. The
arbiter can pay you (`amount - fee`) or refund the payer entirely. Give the
arbiter evidence off-chain — a dispute with no evidence is a coin flip, and the
contract only knows what it was told at creation.

## Timings you should know

- Funding requires the current time to be **before** the deadline. After that,
  `fund_deal` fails and the deal can never be funded.
- The payer can reclaim from the deadline instant, not just after a grace
  period. If you expect payment after the deadline, the deal needs a deadline
  beyond that date.
- A dispute freezes the payer's reclaim path: a `Disputed` deal is resolved by
  the arbiter, not by `refund_expired`. That cuts both ways — it also means an
  arbiter who never acts leaves the funds sitting until they do.

## What you cannot do

- Release funds to yourself or anyone else.
- Change the amount, deadline or arbiter.
- Cancel an unfunded deal; only the payer decides whether to fund.
- Reclaim anything — refunds go to the payer by definition; `refund` returns
  their money, it does not pay you.
