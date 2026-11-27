# Payer guide

You are the side paying. Escrow protects you from paying for nothing — but only
if you fund a deal you can still cancel.

## Before you create a deal

Agree three things off-chain first: the amount, the deadline, and who the
arbiter is. The arbiter is a real Stellar address; if you name an address nobody
controls, a disputed deal can never be resolved and your money sits until you
reclaim it after the deadline.

Check the fee and the asset before you commit:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet -- get_config
```

You get back `admin`, `token`, `fee_recipient`, `fee_bps`. Confirm `token` is the
asset you think you're paying (an asset contract id, not an issuer account) and
that `fee_bps` is the rate you agreed — 250 means 2.5%.

## Create

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- create_deal \
  --payer <MY_ADDRESS> \
  --payee <PAYEE_ADDRESS> \
  --arbiter <ARBITER_ADDRESS> \
  --amount 10000000 \
  --deadline 1795861931
```

`amount` is in the token's smallest unit — `10_000_000` is 1 XLM, or 10 USDC if
the token has 7 decimals like XLM's stroop convention. Check the token's
decimals; getting this wrong by a factor of 10 is the most common mistake.

The call returns the new deal id. Nothing has moved yet: a `Created` deal costs
you nothing and can simply be ignored.

## Fund

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- fund_deal --payer <MY_ADDRESS> --deal_id 1
```

After this the money is in the contract. Until the deadline, the payee can
refund you, and you can release. You cannot unilaterally take the money back
before the deadline — that is the point of the escrow, and it is what makes the
payee willing to start work.

## Settle

Work delivered, you're satisfied:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- release --payer <MY_ADDRESS> --deal_id 1
```

The payee gets `amount - fee`. At 1 XLM and 250 bps that is 9.75 XLM, with
0.25 XLM to the fee recipient. The deal is now terminal — there is no undo. Make
sure you're satisfied before you sign.

Work not delivered, deadline passed:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- refund_expired --payer <MY_ADDRESS> --deal_id 1
```

Full amount back, no fee. Allowed from the deadline instant onwards.

Work disputed:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- raise_dispute --caller <MY_ADDRESS> --deal_id 1
```

This freezes the deal. From here only the arbiter can move it, and they can
decide against you — a dispute is not a refund. Raise one when you have evidence
the terms weren't met, not as a way to stall.

## What you cannot do

- Refund a deal yourself. Only the payee can refund.
- Release to anyone but the named payee.
- Change the amount, payee, arbiter or deadline after creation.
- Cancel a `Created` deal — just never fund it.
- Reclaim before the deadline. If the payee has gone silent and the deadline is
  far away, you chose that deadline.

## Costs

One transaction fee to create, one to fund, one to settle. The escrow fee
applies only on release, never on refund or expired reclaim.
