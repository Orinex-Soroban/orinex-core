# Indexing events

Every state transition emits a typed contract event. An indexer that follows
events never needs to poll storage, and a UI that follows one deal can filter on
the deal id in the topic.

## Topics and payloads

| Topics | Data |
|---|---|
| `("init",)` | `admin, token, fee_recipient, fee_bps` |
| `("cfg", "fee")` | `fee_bps` |
| `("cfg", "payout")` | `fee_recipient` |
| `("deal", "created", id)` | `payer, payee, arbiter, amount, deadline` |
| `("deal", "funded", id)` | `payer, amount` |
| `("deal", "released", id)` | `payee, net, fee` |
| `("deal", "refunded", id)` | `payer, amount` |
| `("deal", "disputed", id)` | `caller` |
| `("deal", "resolved", id)` | `released_to_payee, net, fee` |

All deal topics share the prefix `("deal", <action>, id)`, so a single
three-segment filter on `id` catches the whole history of one deal. Config
events use the `("cfg", …)` prefix so they never collide with deal ids.

## Watching from the CLI

```bash
stellar events \
  --id CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV \
  --start-ledger 5950378 \
  -c 20 --output plain
```

Real output from the testnet instance:

```text
Event 0025556687498825728-0000000002 [CONTRACT]:
  Ledger:   5950380 (closed at 2026-11-27T10:32:24Z)
  Contract: CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV
  Event:    DealReleased (deal, released)
  Params:
    deal_id: 1
    payee: "GDOZP5EXE2SO5UZ2U47QBFWCEQYEBLI3I7FV2IVWCSRYBLY6BV36SNNP"
    net: "9750000"
    fee: "250000"
```

`--output json` gives the same fields machine-readably. `--topic` accepts
base64-encoded XDR symbol segments with `*` as a wildcard and `**` as a
trailing catch-all, which is precise but tedious to write by hand; filtering by
contract id and decoding in your own code is usually simpler.

## Indexer design that holds up

1. **Store the cursor, not the ledger.** `getEvents` returns a cursor per page.
   Persist it after processing, and restart from it. Restarting from a ledger
   re-emits events you have already handled.
2. **Deduplicate on the event id.** Ids look like
   `0025556687498825728-0000000002` — a ledger ToID plus the event index within
   that transaction — and are stable. A unique constraint on that string is
   enough to make replay safe.
3. **Treat the event as a delta, not the state.** `DealReleased` tells you the
   payout, not the final record. If you need the full deal, call `get_deal`;
   the event tells you *when* to call it.
4. **Handle both terminal payouts.** A deal can end from `Funded` via `release`,
   `refund` or `refund_expired`, and from `Disputed` via `resolve_dispute`.
   A UI that only listens for `released` will think refunds never happened.
5. **Don't infer fees from the amount.** Use `net` and `fee` from the event;
   the rate can change between creation and settlement, so recomputing from the
   current config can produce a number that never matched the transfer.

## What events do not tell you

External transfers are separate events from the asset contract itself (the SAC
emits its own `transfer` events). The escrow's events describe intent and
amounts; if you reconcile against balances, read the token's events or query
balances directly.
