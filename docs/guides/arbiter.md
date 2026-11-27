# Arbiter guide

The arbiter is named per deal at creation and is the only address that can settle
a disputed deal. That is the entire role, and it only exists once someone calls
`raise_dispute`.

## What you can do

```bash
# Pay the payee (minus the protocol fee)
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- resolve_dispute --deal_id 1 --release_to_payee true

# Return everything to the payer (no fee)
stellar contract invoke --id <CONTRACT_ID> --network testnet --source me \
  -- resolve_dispute --deal_id 1 --release_to_payee false
```

Those are the only two outcomes. There is no partial release, no split, no
"refund minus costs". If a settlement needs a different number, the parties need
a new deal, not a creative resolution.

## Constraints worth thinking about before you accept the role

- You must act **only** on a `Disputed` deal. Calls on any other status fail
  with `InvalidStatus`, and your signature is checked against the deal's
  `arbiter` field, not against the contract admin.
- Both outcomes are terminal. There is no appeal, no reversal, no second
  resolution. Sign only when you're confident.
- `release_to_payee = true` charges the fee at the current rate; `false` is
  fee-free. The fee goes to the fee recipient, never to you, which is deliberate:
  an arbiter with a financial stake in one outcome is a conflict of interest
  the contract cannot police.
- Your address is public on every deal you're named in. There is no privacy
  between disputes and your identity.
- Nothing enforces that you respond. An inactive arbiter means the funds sit. If
  you're not going to act, do not accept being named.

## Deciding

The contract knows only what it stored: payer, payee, amount, deadline, status.
Every judgement call is off-chain and yours. Before signing, at minimum:

1. Read the deal (`get_deal`) and confirm amount, parties and timeline.
2. Read both sides' evidence, including whether the payee delivered and when.
3. Confirm you have no relationship with either party that you would not want
   disclosed.
4. Decide, then sign once.

Record your reasoning somewhere the parties can see it. The transaction proves
you decided; it does not prove you decided fairly, and that reputation is the
only thing that makes you worth naming again.

## If you were named by mistake

You cannot un-name yourself — the arbiter field is immutable. If a deal you
don't want is funded, say so publicly and let the parties settle it with
`release` or `refund`, which don't need you. Your only forced action is when
someone disputes.
