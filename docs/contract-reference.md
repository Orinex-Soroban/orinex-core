# Smart contract reference

Contract: `primary` (`contracts/primary`). Live testnet instance:
`CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV`.

Every function returns `Result`; failures are `#[contracterror]` codes, not
panics. Amounts are `i128` in the escrowed token's smallest unit. Fees are
`u32` basis points with `10_000` = 100% and a hard cap of `1_000` (10%).

## Types

```rust
pub enum DealStatus { Created = 0, Funded = 1, Released = 2, Refunded = 3, Disputed = 4 }

pub struct Config {
    pub admin: Address,
    pub token: Address,
    pub fee_recipient: Address,
    pub fee_bps: u32,
}

pub struct Deal {
    pub id: u64,
    pub payer: Address,
    pub payee: Address,
    pub arbiter: Address,
    pub amount: i128,
    pub deadline: u64,   // unix seconds; funding closes at this instant
    pub status: DealStatus,
    pub created_at: u64, // unix seconds, ledger timestamp at creation
}
```

## Functions

### `initialize`

```rust
fn initialize(env: Env, admin: Address, token: Address, fee_bps: u32, fee_recipient: Address) -> Result<(), Error>
```

Sets the admin, the escrowed asset, the fee and the fee recipient. Callable
once: the admin key doubles as the initialization marker, so a second call fails
with `AlreadyInitialized`.

- Auth: `admin` must sign.
- Errors: `AlreadyInitialized` (1), `InvalidFeeBps` (3, when `fee_bps > 1000`).
- Event: `Initialized`.
- Note: `token` is any SEP-41 contract that the escrow will call `transfer` on.
  Nothing validates it beyond use; a wrong address means funds are stuck.

### `set_fee_bps`

```rust
fn set_fee_bps(env: Env, fee_bps: u32) -> Result<(), Error>
```

Changes the protocol fee. Applies to deals settled after the change; deals
already created are not grandfathered.

- Auth: stored admin.
- Errors: `NotInitialized` (2), `InvalidFeeBps` (3).
- Event: `FeeBpsUpdated`.

### `set_fee_recipient`

```rust
fn set_fee_recipient(env: Env, fee_recipient: Address) -> Result<(), Error>
```

- Auth: stored admin.
- Errors: `NotInitialized` (2).
- Event: `FeeRecipientUpdated`.

### `create_deal`

```rust
fn create_deal(env: Env, payer: Address, payee: Address, arbiter: Address, amount: i128, deadline: u64) -> Result<u64, Error>
```

Creates a `Created` deal and returns its id (`DealCount + 1`, starting at 1).
No funds move here.

- Auth: `payer`.
- Validation: `amount > 0`; `deadline > ledger timestamp`; `payer`, `payee` and
  `arbiter` all distinct.
- Errors: `NotInitialized` (2), `InvalidAmount` (4), `InvalidDeadline` (5),
  `InvalidParty` (6), `ArithmeticOverflow` (9).
- Event: `DealCreated`.

### `fund_deal`

```rust
fn fund_deal(env: Env, payer: Address, deal_id: u64) -> Result<(), Error>
```

Transfers `amount` from the payer to the contract and moves the deal to
`Funded`. One funding per deal.

- Auth: `payer`.
- Preconditions: deal exists, `status == Created`, `payer` matches the deal
  payer, `now < deadline`.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `DealExpired` (10), `Unauthorized` (12).
- Event: `DealFunded`.

### `release`

```rust
fn release(env: Env, payer: Address, deal_id: u64) -> Result<(), Error>
```

Pays the payee `amount - fee` and the fee recipient `fee`, then marks the deal
`Released`. The fee is whatever `fee_bps` is at call time.

- Auth: `payer`.
- Preconditions: `status == Funded`, `payer` matches.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `ArithmeticOverflow` (9), `Unauthorized` (12).
- Event: `DealReleased` with `net` and `fee`.

### `refund`

```rust
fn refund(env: Env, payee: Address, deal_id: u64) -> Result<(), Error>
```

Payee-initiated cancellation: returns the full amount to the payer, no fee, deal
`Refunded`.

- Auth: `payee`.
- Preconditions: `status == Funded`, `payee` matches.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `Unauthorized` (12).
- Event: `DealRefunded`.

### `raise_dispute`

```rust
fn raise_dispute(env: Env, caller: Address, deal_id: u64) -> Result<(), Error>
```

Freezes a funded deal for arbitration. Funds stay in the contract; no transfer.

- Auth: `caller`, who must be the deal's payer or payee.
- Preconditions: `status == Funded`.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `Unauthorized` (12).
- Event: `DealDisputed`.

### `resolve_dispute`

```rust
fn resolve_dispute(env: Env, deal_id: u64, release_to_payee: bool) -> Result<(), Error>
```

Arbiter settlement. `release_to_payee = true` pays payee `amount - fee` and the
fee recipient `fee` (`Released`); `false` returns the full amount to the payer
(`Refunded`, no fee).

- Auth: the deal's arbiter.
- Preconditions: `status == Disputed`.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `ArithmeticOverflow` (9).
- Event: `DisputeResolved` with `released_to_payee`, `net`, `fee`.

### `refund_expired`

```rust
fn refund_expired(env: Env, payer: Address, deal_id: u64) -> Result<(), Error>
```

Payer reclaim once the deadline has passed with funds still held. Full amount,
no fee, deal `Refunded`.

- Auth: `payer`.
- Preconditions: `status == Funded`, `now >= deadline`.
- Errors: `NotInitialized` (2), `DealNotFound` (7), `InvalidStatus` (8),
  `DealNotExpired` (11), `Unauthorized` (12).
- Event: `DealRefunded`.

### `get_config`

```rust
fn get_config(env: Env) -> Result<Config, Error>
```

- Auth: none. Errors: `NotInitialized` (2).

### `get_deal`

```rust
fn get_deal(env: Env, deal_id: u64) -> Result<Deal, Error>
```

- Auth: none. Errors: `DealNotFound` (7), `NotInitialized` (2 when the contract
  was never initialized, because config is read first).

### `get_deal_count`

```rust
fn get_deal_count(env: Env) -> u64
```

- Auth: none. Returns `0` before any deal exists. Deal ids run `1..=count`.

## Errors

| Code | Name | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` called twice |
| 2 | `NotInitialized` | contract has no admin/config |
| 3 | `InvalidFeeBps` | fee above `MAX_FEE_BPS` (1000) |
| 4 | `InvalidAmount` | amount `<= 0` |
| 5 | `InvalidDeadline` | deadline not in the future at creation |
| 6 | `InvalidParty` | payer, payee and arbiter not all distinct |
| 7 | `DealNotFound` | no deal with that id |
| 8 | `InvalidStatus` | transition not allowed from current status |
| 9 | `ArithmeticOverflow` | checked `i128` arithmetic overflowed |
| 10 | `DealExpired` | deadline passed; cannot fund |
| 11 | `DealNotExpired` | deadline not reached; cannot reclaim |
| 12 | `Unauthorized` | caller is not the required party |

Codes are append-only. Clients match on numbers.

## Events

Topics are `(string, string, u64)` for deal events, with the deal id as the
third topic so a single filter catches every event for one deal.

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

## Storage layout

| Key | Space | Type | Purpose |
|---|---|---|---|
| `Admin` | instance | `Address` | admin + init marker |
| `Token` | instance | `Address` | escrowed SEP-41 asset |
| `FeeRecipient` | instance | `Address` | fee sink |
| `FeeBps` | instance | `u32` | current fee |
| `DealCount` | instance | `u64` | id counter |
| `Deal(u64)` | persistent | `Deal` | one entry per deal |

TTL: instance entries are extended when under 7 days out to 30 days; deal
records when under 30 days out to 90 days. One day is `17_280` ledgers.

## Calling it

Read a deal:

```bash
stellar contract invoke \
  --id CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV \
  --network testnet --source <identity> \
  -- get_deal --deal_id 1
```

Create and fund:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source payer \
  -- create_deal \
  --payer G... --payee G... --arbiter G... \
  --amount 10000000 --deadline 1795861931

stellar contract invoke --id <CONTRACT_ID> --network testnet --source payer \
  -- fund_deal --payer G... --deal_id 1
```

Argument names are exactly the Rust parameter names. `--source` selects the
identity that signs, and the JS/CLI layer will fail the simulation if that
address is not the one the contract requires.

Return values are typed `ScVal`: `Result<(), Error>` prints `null` on success,
`Result<Deal, Error>` prints the struct as JSON.
