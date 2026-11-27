# orinex-core

Soroban smart contracts for Orinex. Currently one contract: **`primary`**, a
single-asset escrow with arbiter-settled disputes and a protocol fee.

- SDK: `soroban-sdk 30.0.0`
- Toolchain: pinned in `rust-toolchain.toml` (Rust 1.92.0 — the SDK needs ≥1.91
  and `stellar-cli` refuses 1.91.0 for contract builds)
- Target: `wasm32v1-none`, built through `stellar contract build`

## Escrow lifecycle

```text
                 create_deal (payer)
                        │
                        ▼
                    ┌────────┐   fund_deal (payer, before deadline)
                    │Created │──────────────────┐
                    └────────┘                  │
                                                ▼
                                          ┌─────────┐
             refund (payee) ──────────────│ Funded  │────── raise_dispute (payer|payee)
                        │                 └─────────┘                    │
                        │                      │                         ▼
                        ▼                      │ resolve_dispute   ┌──────────┐
                  ┌──────────┐                 │  (arbiter)        │ Disputed │
                  │ Refunded │◄────────────────┤                   └──────────┘
                  └──────────┘   refund_expired│                    │        │
                        ▲        (payer, after │                    │        │
                        │         deadline)    │  release (payer)   │        │
                        │                      ▼                    ▼        ▼
                        └──────────────────┌──────────┐      Refunded    Released
                                           │ Released │
                                           └──────────┘
```

- `create_deal` agrees terms only; funds move in `fund_deal`.
- `release` pays the payee the amount minus the protocol fee and the fee
  recipient the fee.
- `refund` is payee-authorized and returns the full amount to the payer with no
  fee, letting the counterparty cancel a funded deal.
- `raise_dispute` freezes settlement until `resolve_dispute`, where the arbiter
  chooses a fee-bearing payout to the payee or a full refund to the payer.
- `refund_expired` gives the payer a recovery path if the deadline passes while
  funds are still held.

## Entrypoints

| Function | Args | Auth | Notes |
|---|---|---|---|
| `initialize` | `admin, token, fee_bps, fee_recipient` | admin | Single-shot; `fee_bps <= 1000` |
| `set_fee_bps` | `fee_bps` | stored admin | Capped at `MAX_FEE_BPS` (1000) |
| `set_fee_recipient` | `fee_recipient` | stored admin | |
| `create_deal` | `payer, payee, arbiter, amount, deadline` | payer | `amount > 0`, distinct parties, future deadline; returns deal id |
| `fund_deal` | `payer, deal_id` | payer | `Created` only, before deadline |
| `release` | `payer, deal_id` | payer | `Funded` only; splits fee |
| `refund` | `payee, deal_id` | payee | `Funded` only; no fee |
| `raise_dispute` | `caller, deal_id` | payer or payee | `Funded` only |
| `resolve_dispute` | `deal_id, release_to_payee` | deal arbiter | `Disputed` only |
| `refund_expired` | `payer, deal_id` | payer | `Funded` and past deadline |
| `get_config` | – | – | `Config` or `NotInitialized` |
| `get_deal` | `deal_id` | – | `Deal` or `DealNotFound` |
| `get_deal_count` | – | – | Monotonic counter; ids run 1..=count |

Amounts are `i128` in the token's smallest unit. Fees use basis points
(`u32`, `10_000` = 100%) with `amount * fee_bps / 10_000` floored, all in checked
`i128` arithmetic; an overflow surfaces as `ArithmeticOverflow` instead of
wrapping.

## Storage and TTL

- Instance: `Admin`, `Token`, `FeeRecipient`, `FeeBps`, `DealCount`.
- Persistent: `Deal(u64)`.

Instance entries are extended when their TTL drops below 7 days out to 30 days;
deal records below 30 days out to 90 days. Every read and write path bumps TTLs,
so an active escrow cannot archive mid-flight.

## Errors and events

Error codes are `#[repr(u32)]` and append-only (see `contracts/primary/src/errors.rs`):
`1 AlreadyInitialized` … `12 Unauthorized`.

Events are typed `#[contractevent]` types in `events.rs` and are part of the
contract spec:

| Topic | Payload |
|---|---|
| `("init",)` | admin, token, fee_recipient, fee_bps |
| `("cfg", "fee")` | fee_bps |
| `("cfg", "payout")` | fee_recipient |
| `("deal", "created", id)` | payer, payee, arbiter, amount, deadline |
| `("deal", "funded", id)` | payer, amount |
| `("deal", "released", id)` | payee, net, fee |
| `("deal", "refunded", id)` | payer, amount |
| `("deal", "disputed", id)` | caller |
| `("deal", "resolved", id)` | released_to_payee, net, fee |

## Build and test

```bash
make build     # scripts/build.sh -> artifacts/primary.wasm (stellar contract build)
make test      # cargo test --workspace --all-targets --locked
make clippy    # cargo clippy ... -D warnings
make fmt-check # cargo fmt --all -- --check
make verify    # fmt-check + clippy + test + build
```

`soroban-sdk 30` refuses plain `cargo build` for wasm: the build must be driven
by `stellar contract build`, which sets `STELLAR_CLI_VERSION`. Install it with
the release binary or `cargo install stellar-cli --version 30.0.1`.

## Deploy to testnet

```bash
export ADMIN=G...            # contract admin
export TOKEN=C...            # SEP-41 asset contract id (native XLM SAC: CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC)
export FEE_RECIPIENT=G...
export FEE_BPS=250           # optional, defaults to 250
make deploy-testnet
```

`scripts/deploy_testnet.sh` builds the wasm if needed, creates and funds the
`orinex-testnet` CLI identity when absent, deploys, calls `initialize`, and
rewrites `deployments/testnet.json` with the contract id, wasm hash, config and
deployer. Optional overrides: `NETWORK`, `IDENTITY`, `WASM`,
`DEPLOYMENTS_FILE`, `RPC_URL`, `PASSPHRASE`.

Nothing is committed automatically by the script; review the manifest before
committing it.

## Current deployment

`deployments/testnet.json` records the live testnet instance:

- Contract: `CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV`
- Token: native XLM SAC `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`
- Fee: 250 bps; admin/fee recipient is the deployer identity
- Verified with an on-chain `create_deal → fund_deal → release` cycle
  (10 000 000 stroops escrowed; 9 750 000 to payee, 250 000 fee)

## Layout

```text
contracts/primary/src/{lib,errors,storage,events,test}.rs
scripts/{build,deploy_testnet}.sh
deployments/testnet.json
.github/workflows/ci.yml   # fmt, clippy, test, wasm artifact
```

## Notes

- Only the payee can refund, and only the payer can release or reclaim; the
  arbiter is the sole settler of disputes. Deals are immutable once created.
- The contract holds one asset configured at `initialize`; there is no admin
  path to move escrowed funds, so a stuck deal can only be resolved through the
  arbiter or the payer's deadline reclaim.
