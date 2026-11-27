# Building a client

The contract is called the same way as any Soroban contract. Two paths are
worth knowing: the CLI (fastest for scripts and debugging) and
`@stellar/stellar-sdk` (what an app uses).

Do not reach for `stellar contract bindings typescript` — in `stellar-cli`
30.0.1 it is deprecated and prints "Deprecated, use the JavaScript Stellar SDK
instead". The SDK's `Client` class reads the contract spec from the network and
exposes the methods directly, so there is nothing to generate.

## Reads with the SDK

Verified against the live testnet instance on 2026-11-27 with
`@stellar/stellar-sdk` 17.4.0:

```js
import { Networks } from "@stellar/stellar-sdk";
import { Client } from "@stellar/stellar-sdk/contract";

const client = await Client.from({
  contractId: "CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV",
  networkPassphrase: Networks.TESTNET,
  rpcUrl: "https://soroban-testnet.stellar.org",
});

const { result } = await client.get_config();
console.log(result.value.admin);   // "GAP6AMMP..."
console.log(result.value.fee_bps); // 250
```

Two things bite people here:

- **`Result` returns are wrapped.** `get_config` and `get_deal` return
  `Result<T, Error>`, so the decoded object is `{ value: T }` (or the error
  variant). `get_deal_count` returns a bare `u64`, so its `result` *is* the
  number.
- **Integers are `BigInt`.** `i128` amounts and `u64` ids/statuses come back as
  BigInt; `JSON.stringify` throws on them until you convert:
  `JSON.stringify(result, (_, v) => (typeof v === "bigint" ? v.toString() : v))`.

A deal read returns the whole struct:

```js
const { result } = await client.get_deal({ deal_id: 1 });
// result.value = {
//   id: 1n, payer: "G…", payee: "G…", arbiter: "G…",
//   amount: 10000000n, deadline: 1795861931n,
//   status: 2, created_at: 1795775534n
// }
```

`status` is the `DealStatus` enum: `0 Created`, `1 Funded`, `2 Released`,
`3 Refunded`, `4 Disputed`.

## Writes

Writes need a signer. For a Node script or CI job, `basicNodeSigner` wraps a
`Keypair` into the two callbacks `Client.from` wants:

```js
import { Keypair, Networks } from "@stellar/stellar-sdk";
import { Client, basicNodeSigner } from "@stellar/stellar-sdk/contract";

const payer = Keypair.fromSecret(process.env.PAYER_SECRET);
const funded = await Client.from({
  contractId: CONTRACT_ID,
  networkPassphrase: Networks.TESTNET,
  rpcUrl: "https://soroban-testnet.stellar.org",
  publicKey: payer.publicKey(),
  ...basicNodeSigner(payer, Networks.TESTNET),
});

const { result } = await funded.fund_deal({
  payer: payer.publicKey(),
  deal_id: 1n,
});
```

In a browser, replace `basicNodeSigner` with a wallet callback — the option
shape is `signTransaction` (and optionally `signAuthEntry`), which is what
Freighter and similar wallets implement. The payer's `require_auth()` produces
an auth entry that the SDK asks your signer to sign; that is why the callback
has to exist before the call, not after.

Amounts go in as BigInt or a plain number; prefer BigInt for `i128` to avoid
precision surprises above `2^53`.

## Reads with the CLI

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet -- get_deal --deal_id 1
```

This prints the decoded struct as JSON with string amounts, which is convenient
for shell scripts and for checking what the SDK *should* return when something
looks wrong.

## Failure modes

- **Simulation fails with an auth error** — the address in your `--source` (or
  your SDK `publicKey`) is not the party the function requires. For example,
  `release` needs the deal's payer; `refund` needs the payee.
- **`InvalidStatus` (code 8)** — the deal already moved. Re-read it before
  retrying; do not retry blindly, it will fail the same way forever.
- **`DealExpired` (code 10)** on funding — the deadline passed. There is no
  grace period and no way to reopen a deal.
- **Transaction succeeds but balance didn't change** — check the asset. The
  contract transfers the `token` from config; if you're watching a different
  asset's balance, nothing will show up.

## Amount decimals

The contract does not know decimals. `amount` is whatever integer the token
accepts. XLM's SAC uses 7 decimals (`10_000_000` = 1 XLM); most stablecoin
issuers on Stellar also use 7. Read the token's `decimals()` before formatting,
and never hardcode 6 because another chain does it that way.
