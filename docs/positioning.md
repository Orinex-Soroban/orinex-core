# Positioning: what `orinex-core` is up against

Written 2026-11-27 from live sources, not memory. Sources are linked at the
bottom; every claim about the approved-repo landscape comes from a page fetched
today.

## 1. Landscape

**The program itself.** Drips runs the Stellar Wave on behalf of SDF: a monthly
sprint where approved repos post issues and contributors get paid from a pool.
Drips reports **$400k+ distributed**, **50k+ issues resolved**, **200+ repos**
program-wide. The Stellar repos page reported **941–964 approved repos** across
two renders today (the count moves as repos are added). Points multipliers of
2x and 4x are shown per repo, so repo *classification* affects contributor
payout, not just visibility.

**What the network is optimising for (2026).** SDF's Q3 2026 report and the
Meridian 2026 leadership interviews point the same direction:

- Payments volume: **$18.5B monthly** in October 2026, up from $1.8B in January.
- Stablecoins: **$25.1B** of stablecoin transfer volume in Q3, +121% QoQ; USDT0
  went live on 2026-09-02.
- RWAs: **past $5B** on network; Chainlink went live 2026-10-14, so price
  oracles for Soroban now exist where they did not a year ago.
- Developers: **3,922 monthly active**, second behind Ethereum per Electric
  Capital; funding runs through SCF v7 tracks (Integration / Open / RFP) plus
  monthly Drips bounties.
- Stated five-year priorities from SDF's President: more builders, more payment
  and stablecoin volume, and assets put to work rather than sitting in wallets.

**Where escrow sits.** Escrow is not white space on Stellar. It is one of the
most crowded corners of the approved list:

| Repo | What it is |
|---|---|
| `Trustless-Work/trustlesswork-smart-contract-stellar` | Permissionless escrow infrastructure on Soroban with USDC; 4x points; default branch is literally `single-release-develop` |
| `safetrustcr/frontend-SafeTrust`, `backend-SafeTrust` | P2P escrow for hotels, rentals and tourism deposits; 4x points |
| `OFFER-HUB/offer-hub-monorepo` | Freelance marketplace escrow, built on Trustless Work's API; 4x points |
| `Stellar-Rent/stellar-rent` | P2P rental escrow with intermediaries removed |
| `kindfi-org/kindfi` | Crowdfunding with milestone escrows |
| `mercato-supply-chain/mercato-dapp` | Investor-funded escrow paying suppliers on proven milestones |
| `akkuea/akkuea` | RWA + DeFi |

The important structural fact: **Trustless Work is escrow-as-a-service**, and at
least three approved products consume it rather than building their own
contracts. The primitive is commoditised at the infrastructure layer; the
remaining competition is at the product layer (a specific vertical, with a UI
and an off-ramp).

## 2. Candidate directions, judged against what this repo already is

`orinex-core` today is a 13-entrypoint single-asset escrow: create → fund →
release / refund / dispute → arbiter resolve, plus deadline reclaim, 250 bps fee
cap at 1000 bps, TTL-bumped storage, 39 tests, live on testnet.

**(a) Generic open escrow protocol.** Direct head-on competition with Trustless
Work, which already has documentation, an SDK surface, and integrations.
**Weak.** No differentiator; being newer and smaller is not one.

**(b) Escrow primitive other Soroban contracts compose against.** Ship it as a
library/interface others call (a `#[contractclient]`-friendly single-asset
escrow with a stable ABI) rather than an end-user product. The repo already has
a clean typed ABI and a published interface check. **Conditional** — real, but
demands adoption work this repo cannot do alone.

**(c) Micro-escrow for machine payments.** Sub-dollar escrow tied to x402-style
agent payments (an approved-list theme: `routedock`, `talos-stellar`). Stellar's
fee floor makes escrow viable at amounts where a percentage fee model on other
chains is not. The current 1000 bps cap and floored fee split already support
this without change. **Strongest of the options, conditional on demand
evidence.**

**(d) Single-asset escrow for a regulated vertical** (freight, agri supply,
construction retainage). Needs domain partners and, in several cases, licensing.
**Blocked** as a Wave submission without a partner; not a scoping problem.

**(e) Confidential escrow.** Would depend on Stellar Private Payments, which is
in developer preview / testnet as of August 2026, not production. **Blocked.**

**(f) Escrow with on-chain arbitration market** (staked arbiters, slashing).
Genuine white space versus the incumbent's named-arbiter model, but it changes
the contract's economics and needs a second contract. **Conditional**, larger
scope.

## 3. Verdict

**As submitted today: WEAK.** Not because the code is weak — the contract is
tested, deployed and documented — but because the *category* is saturated by an
incumbent that other approved repos already build on, and nothing in the repo
states a differentiator a reviewer could score. An escrow contract with no app,
no SDK, no audits and no unique mechanism is the fourth-best version of a solved
problem in this ecosystem.

**Conditional path to STRONG:** pick (c) or (f) and make the differentiator the
repo's first screen — README, issues, and roadmap. Concretely: micro-escrow
amount floors and fee behaviour become the headline, with the arbiter path
demoted to a feature rather than the pitch.

## 4. What is verified vs. not

Verified today: approved-repo membership and descriptions for the repos above;
Drips program figures; SDF Q3 2026 metrics; SCF v7 track structure; that this
repo is public, has admin access from this environment, no topics, no description,
no license, and no release tag.

Not verified: whether `Orinex-Soroban/orinex-core` is already in the Wave
applicant/approved list — the Drips repos page ignores search parameters from an
unauthenticated fetch, and the maintainer dashboard is behind login. Check
Maintainers → Orgs and Repos before submitting.

## Sources

- https://www.drips.network/wave/stellar/repos (fetched 2026-11-27)
- https://www.drips.network/solutions/wave
- https://docs.drips.network/wave/maintainers/participating-in-a-wave
- https://stellar.org/blog/foundation-news/q3-2026-the-rail-put-to-work
- https://www.risein.com/blog/meridian-2026-recap-rise-in-and-the-stellar-ecosystem-in-lisbon
- https://stellar.org/blog/ecosystem/introducing-scf-v7
- https://stellar.org/blog/foundation-news (Chainlink 2026-10-14, USDT0 2026-09-02, Private Payments 2026-08-24)
