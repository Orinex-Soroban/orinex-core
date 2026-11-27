# Security policy

## Audit status

**The contracts in this repository are unaudited.** They have been tested
locally (39 unit tests, clippy-clean) and deployed to Stellar testnet, and
nothing more. Do not put real funds in a deployment of this code without an
independent audit first.

## Reporting a vulnerability

Do not open a public issue, PR, or discussion for a security problem.

Use GitHub's private reporting: **Security → Advisories → Report a
vulnerability** on this repository
(`https://github.com/Orinex-Soroban/orinex-core/security/advisories/new`). That
channel is private to the maintainers and gives us a place to coordinate a fix
and a disclosure timeline with you.

Include, as far as you can:

- The affected function(s) and contract version or commit hash.
- A description of the impact: funds at risk, auth bypass, wrong payout,
  storage exhaustion, etc.
- Reproduction steps, ideally a failing test against
  `contracts/primary/src/test.rs`.
- Whether the issue is already public anywhere.

## Scope

In scope:

- `contracts/primary/src/**` — escrow logic, auth checks, fee math, TTL
  handling, event emission.
- `scripts/deploy_testnet.sh` — anything that could deploy a misconfigured or
  unintended contract, or leak a secret into the manifest.
- The deployed testnet instance recorded in `deployments/testnet.json`.

Out of scope:

- Test network availability, RPC provider outages, and Stellar protocol issues.
- Test keys committed nowhere and referenced in the manifest only as public
  addresses.
- Dependency vulnerabilities that are already fixed in a newer `soroban-sdk`
  release (report the version issue, not an exploit chain).
- Anything requiring the admin key, which is trusted by design: the admin can
  change the fee within the cap and the fee recipient, but has no path to move
  escrowed funds.

## What we consider a real finding

- Any way for a non-party to move escrowed funds.
- Any way for the payer to take funds back after release, or for the payee to
  take funds that are not theirs.
- A fee computation that lets the fee exceed `MAX_FEE_BPS` of the amount, or
  that makes `net + fee != amount`.
- A state transition that skips `require_auth` or accepts a deal in the wrong
  status.
- Storage that can be made to archive mid-escrow, stranding funds.

## Response

We aim to acknowledge a report within 72 hours, confirm or dispute the finding
with reasoning, and ship a fix plus a regression test before any public
disclosure. Credit is given in the advisory unless you ask otherwise.
