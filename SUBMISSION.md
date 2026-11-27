# Submission pack

Everything needed to file this repo with the Stellar Wave program, plus what is
still missing. Written 2026-11-27.

## Status

| Item | State |
|---|---|
| Contract implemented, tested, clippy-clean | done — 39 tests, `make verify` |
| Testnet deployment, verified end to end | done — see links below |
| Documentation site content | done in `docs/`; publishing is issue #11 |
| Issue backlog for contributors | done — issues #1–#12 |
| Repo hygiene (LICENSE, SECURITY, CONTRIBUTING) | done |
| Release `v0.1.0` with deployment details | done — tag pushed |
| Description and topics | blocked: needs admin token, run `scripts/setup_repo.sh` |
| Branch protection | blocked: needs admin token, run `scripts/setup_repo.sh` |
| Application-layer repo (`orinex-app`) | not started — see "Two-repo split" |
| Live app URL | pending |
| Demo video | pending — shot list below |
| Docs site URL | pending (issue #11) |
| Drips application / approval check | must be done in the maintainer dashboard |

## Project description (submission form)

Orinex is an escrow contract for Stellar that lets two parties settle work
without a platform holding their money: the payer locks funds on-chain against
agreed terms, the payee can return them without needing the payer awake, a named
arbiter settles disputes, and a deadline guarantees the payer can always reclaim
what nobody settled. It is written in Rust for Soroban, holds one SEP-41 asset
per deployment, charges a protocol fee in basis points capped at 10% and floored
so it never exceeds the amount, and takes no cuts on refunds. The contract is 13
entrypoints with typed errors and events, covered by 39 tests including fee
rounding properties and the largest non-overflowing amount, and is deployed on
testnet with a verified create → fund → release cycle. Escrow on Stellar is
crowded — Trustless Work, SafeTrust, KindFi and OfferHub all ship versions of it —
so this repo states plainly where it fits instead of claiming novelty; the
near-term differentiator is micro-escrow, where a floored fee at small amounts
matters more than a percentage model.

## Planned issues (for the submission form)

Twelve issues are open, organised by area. All were created with
`scripts/create_issues.sh`, so the scope is real work, not a wish list.

**Contract**
- #1 admin transfer entrypoint — move admin off the deployer key
- #2 extract the shared payout path from release and dispute resolution
- #3 per-party deal listing (design decision recorded before code)

**Testing**
- #4 TTL extension on every read and write path
- #5 fee property tests for the dispute resolution path
- #6 fuzz fee arithmetic across the `i128` range

**Infrastructure**
- #7 verify the on-chain wasm hash matches the local artifact
- #8 smoke-test dispute and expiry flows against testnet
- #9 cache the stellar-cli download in CI
- #10 issue and pull request templates

**Documentation**
- #11 publish the GitBook site and link it from the README
- #12 mainnet readiness checklist

Three carry `good first issue` for newcomers: #9, #10, #11.

## Two-repo split

The playbook's default is two repos so both surfaces are eligible:

- **`orinex-core`** (this repo) — contracts only. Rust workspace, one contract,
  deployment scripts and manifests. Nothing here knows about a frontend.
- **`orinex-app`** (not created) — `packages/sdk`, `apps/web`, `indexer/`. Reads
  the contract through `@stellar/stellar-sdk`'s `Client` (see
  `docs/developers/sdk.md`), writes through a wallet signer, and indexes events
  via cursor pagination (see `docs/developers/events.md`).

How they connect: the app needs `CONTRACT_ID`, the network passphrase
(`Test SDF Network ; September 2015` for testnet) and an RPC URL. The contract
needs nothing from the app. The app repo must not be started before the
positioning decision in `docs/positioning.md` is settled — building a UI for the
fourth-best escrow is the expensive mistake this pack exists to prevent.

## Supporting links

| What | Where |
|---|---|
| Contract (testnet) | https://stellar.expert/explorer/testnet/contract/CA43UELWFPN2YF2H77XIVCWR7KMBG66VHMKOOHO2YMD4Q5ODDGDRABZV |
| Deploy tx | https://stellar.expert/explorer/testnet/tx/0803c5cf077cafda19e7012767e124430266753cf044a49c59f750871f7c4f8a |
| Initialize tx | https://stellar.expert/explorer/testnet/tx/ece407762df693f098693ad29524bb69f75de009158d50845e5366fe4aa2fc89 |
| Create → fund → release (first tx) | https://stellar.expert/explorer/testnet/tx/db3aa96206746ac96ccb8a9fac5ca7f561c240e813de1e50b3c92630ce838770 |
| Repo | https://github.com/Orinex-Soroban/orinex-core |
| Docs (source) | `docs/` in this repo, GitBook-ready |
| App repo | pending |
| Demo video | pending |

Deployment facts to quote accurately: native XLM SAC
`CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`, fee 250 bps, wasm
sha256 `deae51926429a785a0b21366e4b3f901135d573702a4261492b763f5c282a77a`,
reproducible from commit `ea69385` or later.

## Demo video shot list

Two minutes is enough; show the real flow, not slides.

1. `stellar contract invoke ... get_config` — the deployed contract is live.
2. `get_deal --deal_id 1` — status `2`, the completed cycle.
3. New deal: `create_deal` with a funded payer, then `fund_deal`; show the SAC
   `transfer` event and the payer's balance dropping.
4. `release`; show the payee's balance increase by `amount - fee` and the fee
   recipient's by `fee`.
5. A dispute run: `raise_dispute`, then `resolve_dispute --release_to_payee false`
   by the arbiter, payee balance unchanged.
6. `stellar events --id <contract> --start-ledger <n>` — the whole history as
   decoded events.

Record the terminal output at a readable font size; blur no keys, but use
throwaway testnet identities for everything.

## Before filing

0. Run `scripts/setup_repo.sh` with an admin-authenticated `gh` session. The
   integration token used to prepare this repo cannot set the description,
   topics or branch protection (HTTP 403). The script is idempotent and its
   required checks (`rustfmt`, `clippy`, `cargo test`, `wasm build`) are copied
   from the real job names in `.github/workflows/ci.yml`.
1. Confirm on Maintainers → Orgs and Repos that this repo is not already
   approved (the public list ignores search parameters, so it cannot be checked
   from outside).
2. Publish the docs site (issue #11) and paste the URL above.
3. Record the demo video and paste the URL above.
4. Decide the positioning question in `docs/positioning.md`. Filing without it
   means submitting "another escrow contract", which is the verdict this repo's
   own analysis calls weak.
