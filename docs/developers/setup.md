# Local setup and deployment

## Toolchain

| Tool | Version | Why this version |
|---|---|---|
| Rust | 1.92.0 | pinned in `rust-toolchain.toml` |
| `stellar` CLI | 30.0.1 | required by `soroban-sdk` 30 to build wasm |
| wasm target | `wasm32v1-none` | Soroban's build target for SDK 30 |

Version constraints are not arbitrary. `soroban-sdk 30` declares MSRV 1.91.0, but
`stellar-cli` 30.0.1 refuses to build contracts with Rust 1.91.0 specifically
(along with 1.81–1.83). 1.92.0 clears every constraint at once.

```bash
git clone https://github.com/Orinex-Soroban/orinex-core.git
cd orinex-core
rustup show                      # installs and selects 1.92.0
rustup target add wasm32v1-none
cargo install --locked stellar-cli --version 30.0.1
```

If `stellar` isn't found afterwards, `$HOME/.cargo/bin` isn't on your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Build and test

```bash
make verify      # fmt-check, clippy -D warnings, cargo test, wasm build
make build       # artifacts/primary.wasm only
make test        # 39 unit tests
```

`make build` calls `scripts/build.sh`, which runs `stellar contract build`.
A plain `cargo build --target wasm32v1-none` fails on purpose — the SDK's build
script errors with "soroban-sdk requires stellar-cli v30.0.0+ to build a
contract". That is the SDK enforcing the toolchain, not a broken setup.

The built artifact is deterministic for a given source and toolchain: the wasm in
`deployments/testnet.json` has sha256
`deae51926429a785a0b21366e4b3f901135d573702a4261492b763f5c282a77a`, and
rebuilding from this commit reproduces it.

## Deploy to testnet

Required environment variables:

| Variable | Meaning |
|---|---|
| `ADMIN` | address that will own configuration changes; must be the signing identity's address or the `initialize` call fails auth |
| `TOKEN` | SEP-41 asset contract id to escrow (native XLM SAC on testnet: `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`) |
| `FEE_RECIPIENT` | address that receives protocol fees |

Optional:

| Variable | Default |
|---|---|
| `FEE_BPS` | `250` |
| `NETWORK` | `testnet` |
| `IDENTITY` | `orinex-testnet` |
| `WASM` | `artifacts/primary.wasm` |
| `DEPLOYMENTS_FILE` | `deployments/testnet.json` |
| `RPC_URL` | `https://soroban-testnet.stellar.org` |
| `PASSPHRASE` | `Test SDF Network ; September 2015` |

```bash
export ADMIN=G...
export TOKEN=CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
export FEE_RECIPIENT=G...
make deploy-testnet
```

The script:

1. builds the wasm if it's missing,
2. creates and friendbot-funds the CLI identity if it doesn't exist,
3. deploys the wasm and captures the contract id,
4. calls `initialize` with your admin, token, fee and fee recipient,
5. rewrites the deployment manifest with the contract id, wasm hash, config,
   deployer and timestamp.

It does not commit anything. Review `deployments/testnet.json`, then commit it
yourself.

Note the order matters: the identity that signs `initialize` is the deployer, but
`ADMIN` is the address stored as admin. If they differ, the `initialize`
transaction still succeeds only when `ADMIN`'s signature is present — the script
signs with `IDENTITY`, so keep `ADMIN` equal to that identity's address unless
you're driving the CLI by hand.

## Verifying a deployment

```bash
CID=$(python3 -c "import json;print(json.load(open('deployments/testnet.json'))['contract_id'])")

stellar contract invoke --id "$CID" --network testnet -- get_config
stellar contract invoke --id "$CID" --network testnet -- get_deal_count
```

`get_config` should echo what you passed. `get_deal_count` returns `0` on a fresh
deployment.

To prove the contract actually moves the asset, run one full cycle with funded
identities (payer, payee, arbiter), then check the balances:

```bash
stellar contract invoke --id "$CID" --network testnet --source payer \
  -- create_deal --payer <PAYER> --payee <PAYEE> --arbiter <ARBITER> \
  --amount 10000000 --deadline $(( $(date -u +%s) + 86400 ))

stellar contract invoke --id "$CID" --network testnet --source payer \
  -- fund_deal --payer <PAYER> --deal_id 1

stellar contract invoke --id "$CID" --network testnet --source payer \
  -- release --payer <PAYER> --deal_id 1

stellar contract invoke --id "$CID" --network testnet -- get_deal --deal_id 1
```

`status` should be `2` (`Released`), and the payee's balance should have increased
by `amount - fee`.

## Troubleshooting

**`soroban-sdk requires stellar-cli v30.0.0+ to build a contract`** — you ran
`cargo build` for wasm directly. Use `make build` / `scripts/build.sh`.

**`contract builds are only supported with Rust versions ...`** — the active
toolchain is 1.91.0. Run `rustup show` in the repo root so the pinned 1.92.0 is
selected.

**`error: no such command: contract`** — an old `stellar-cli` is on `PATH`
(`brew install stellar-cli` can lag). Check `stellar --version` and reinstall the
pinned version.

**`test_snapshots/` appears in `git status`** — generated on failing test runs,
already gitignored. Don't commit it.

**`initialize` fails with `NotInitialized` on a later call** — you are probably
pointing at the wrong contract id, not a broken deployment; `NotInitialized`
comes from reading config, which a fresh id will also report.
