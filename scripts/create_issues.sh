#!/usr/bin/env bash
# Creates the planned issue backlog in one run.
#
# Usage:
#   scripts/create_issues.sh            # create labels + issues
#   DRY_RUN=1 scripts/create_issues.sh  # show what would be created
#   REPO=owner/name scripts/create_issues.sh
#
# Safe to re-run: an issue whose title already exists (open or closed) is skipped.
set -euo pipefail

REPO="${REPO:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
DRY_RUN="${DRY_RUN:-0}"

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI not found" >&2
  exit 1
fi
if ! gh auth status >/dev/null 2>&1; then
  echo "error: gh is not authenticated (run: gh auth login)" >&2
  exit 1
fi

echo "repo: ${REPO}"

ensure_label() {
  local name="$1" color="$2" description="$3"
  if [ "$DRY_RUN" = "1" ]; then
    echo "label: ${name}"
    return
  fi
  gh label create "$name" --repo "$REPO" --color "$color" --description "$description" --force >/dev/null
}

ensure_label "contract" "1D76DB" "Soroban contract code and interface changes"
ensure_label "tests" "0E8A16" "Test coverage for the contract"
ensure_label "docs" "5319E7" "Documentation and the docs site"
ensure_label "infra" "FBCA04" "CI, scripts, deployment and repo tooling"
ensure_label "design" "D4C5F9" "Needs a scoping or architecture decision first"
ensure_label "good first issue" "7057FF" "Scoped, low-context work suitable for a new contributor"

create_issue() {
  local title="$1" labels="$2" body="$3"
  local existing
  existing="$(gh issue list --repo "$REPO" --state all --search "${title} in:title" \
    --json number --jq 'map(.number) | first // empty')"
  if [ -n "$existing" ]; then
    echo "skip  #${existing} ${title}"
    return
  fi
  if [ "$DRY_RUN" = "1" ]; then
    echo "would create: ${title}  [${labels}]"
    return
  fi
  local number
  number="$(gh issue create --repo "$REPO" --title "$title" --body "$body" --label "$labels" \
    --json number --jq .number)"
  echo "create #${number} ${title}"
}

# ---------------------------------------------------------------- contract ---

create_issue "feat(primary): add admin transfer entrypoint" "contract,design" "$(cat <<'BODY'
## Summary
The admin address is set once in `initialize` and can never change. The current
testnet admin is the deployer key that lives only in a local Stellar CLI config,
so there is no way to move administration to an operational key without
redeploying and losing the deal history.

Add `transfer_admin(new_admin: Address)` that requires the stored admin's
authorization, writes the new address, emits an `AdminTransferred` event, and
rejects transfer to the current admin.

## Acceptance Criteria
- [ ] `transfer_admin(new_admin)` is exposed in the contract interface
- [ ] Auth is checked against the stored admin before any mutation
- [ ] Event `("cfg", "admin")` carries `old_admin` and `new_admin`
- [ ] A test proves a non-admin caller gets `Unauthorized` and storage is unchanged
- [ ] A test proves the old admin is rejected by `set_fee_bps` after transfer
- [ ] No existing error code is renumbered (append-only policy)

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0, stellar-cli 30.0.1, `wasm32v1-none`
BODY
)"

create_issue "refactor(primary): extract the shared payout path from release and dispute resolution" "contract" "$(cat <<'BODY'
## Summary
`release` and `resolve_dispute(release_to_payee = true)` perform the same
operation: compute the fee, transfer net to the payee, transfer the fee to the
fee recipient, mark the deal released. They are currently written out twice.

Extract one private helper so the payout maths and event emission exist in one
place, and a change to fee handling cannot drift between the two paths.

## Acceptance Criteria
- [ ] A single private function performs transfer + fee split + status write
- [ ] `release` and the payee branch of `resolve_dispute` both call it
- [ ] Public behaviour, error codes and emitted events are unchanged
- [ ] All existing tests pass without modification
- [ ] No new public entrypoints are introduced

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0
BODY
)"

create_issue "feat(primary): add per-party deal listing for clients" "contract,design" "$(cat <<'BODY'
## Summary
A client can only read deals by id, and ids are not discoverable from an
address. A UI has to walk `1..get_deal_count()` and filter locally, which gets
expensive fast and is the main reason an app would still need a full indexer.

Decide between two options, record the decision in the issue, then implement:

1. **On-chain index**: `DataKey::PartyDeals(Address)` holding `Vec<u64>`,
   appended in `create_deal`. Costs extra persistent writes on create and grows
   unbounded per address.
2. **Explicit refusal**: document that discovery is an indexer responsibility and
   close the issue. The event stream already carries payer, payee and arbiter on
   `DealCreated`.

## Acceptance Criteria
- [ ] Decision recorded with reasoning before code is written
- [ ] If implemented: `get_deals_by_party(address, cursor, limit)` exists, pages
      results, and has a test with more than one page
- [ ] If implemented: `create_deal` cost increase is measured and stated in the PR
- [ ] Docs updated to match whichever option is chosen

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0
BODY
)"

# ------------------------------------------------------------------- tests ---

create_issue "test(primary): cover TTL extension on every read and write path" "tests" "$(cat <<'BODY'
## Summary
Every read and write is supposed to bump TTLs — instance entries under 7 days
out to 30, deal records under 30 days out to 90. That policy is currently only
asserted indirectly by tests happening to pass before expiry. A regression that
drops a `bump_instance_ttl` call would be silent until funds strand in a live
deployment.

## Acceptance Criteria
- [ ] Test asserts instance TTL after `initialize`, `create_deal` and `get_config`
- [ ] Test asserts `Deal(id)` TTL after `create_deal` and after a read via `get_deal`
- [ ] A test proves a deal read at the TTL threshold extends the entry
- [ ] Tests fail if any individual bump call is removed (state this check in the PR)
- [ ] No production code change is required; if one is, it is called out explicitly

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0 test environment
BODY
)"

create_issue "test(primary): add fee property tests for the dispute resolution path" "tests" "$(cat <<'BODY'
## Summary
`fee_rounding_holds_across_amounts_and_rates` covers `release`. Dispute
resolution pays through a different code path, so the same invariants are not
proven there: `net + fee == amount`, `fee <= amount`, floors instead of rounding
up, and a zero contract balance afterwards.

## Acceptance Criteria
- [ ] Property test sweeps at least 5 fee rates across amount boundaries
      (1, 2, 7, 10, 99, 100, 101, 9_999, 10_000, 10_001)
- [ ] Test goes through `raise_dispute` + `resolve_dispute(release_to_payee = true)`
- [ ] Test asserts payee balance, fee recipient balance and contract balance after each resolution
- [ ] Test asserts the refund branch of `resolve_dispute` charges no fee
- [ ] Suite runtime stays under 30 seconds

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0
BODY
)"

create_issue "test(primary): fuzz fee arithmetic across the i128 range" "tests" "$(cat <<'BODY'
## Summary
The fee path is `amount * fee_bps / 10_000` with checked arithmetic. Hand-picked
boundary tests cover the obvious edges, but the failure mode to rule out is an
input where multiplication overflows while the fee itself would be valid, or
where flooring produces a fee greater than the amount.

## Acceptance Criteria
- [ ] A generative test (proptest or a deterministic seeded generator) covers
      `amount` in `1..=i128::MAX` and `fee_bps` in `0..=1_000`
- [ ] Invariant asserted: `fee + (amount - fee) == amount` and `0 <= fee <= amount`
- [ ] Overflow is asserted to surface as `ArithmeticOverflow`, never a panic
- [ ] Any new dev-dependency is justified in the PR and pinned
- [ ] Test runs in CI within the existing timeout

## Tech Stack
Rust 1.92.0, soroban-sdk 30.0.0, proptest (if used)
BODY
)"

# -------------------------------------------------------------------- infra ---

create_issue "feat(deploy): verify the on-chain wasm hash matches the local artifact" "infra" "$(cat <<'BODY'
## Summary
The manifest records `wasm_sha256` of the local file, but nothing checks what was
actually deployed. A stale build, a wrong file, or a deploy from a different
commit would be recorded as if it were correct.

After deploying, fetch the wasm back from the network and compare hashes; fail
loudly on mismatch instead of writing the manifest.

## Acceptance Criteria
- [ ] `scripts/deploy_testnet.sh` fetches the deployed wasm
      (`stellar contract fetch --id "$CONTRACT_ID" -o <tmp>`) and compares sha256
- [ ] On mismatch the script exits non-zero and writes no manifest
- [ ] On success the manifest is written as it is today
- [ ] Script still works when `WASM` points at a prebuilt artifact
- [ ] Failure path is exercised once and the output pasted in the PR

## Tech Stack
bash, stellar-cli 30.0.1, sha256sum
BODY
)"

create_issue "feat(scripts): smoke-test the dispute and expiry flows against testnet" "infra" "$(cat <<'BODY'
## Summary
Deployment verification covers create → fund → release. The paths most likely to
break in a real deployment — arbiter resolution and deadline reclaim — are only
covered by unit tests against the test host, not against a live network.

Add a script that runs a disputed deal to resolution and an expired deal to
reclaim, asserting final balances and statuses.

## Acceptance Criteria
- [ ] New script `scripts/smoke_test_testnet.sh` takes a contract id and three funded identities
- [ ] Covers: fund → dispute → arbitrate to payee (fee applied)
- [ ] Covers: fund → wait past deadline → `refund_expired` (no fee)
- [ ] Asserts deal status and token balances after each flow
- [ ] Exits non-zero on the first mismatch, with the failing expectation printed
- [ ] Script is referenced from `docs/developers/setup.md`

## Tech Stack
bash, stellar-cli 30.0.1
BODY
)"

create_issue "chore(ci): cache the stellar-cli download in the wasm job" "infra,good first issue" "$(cat <<'BODY'
## Summary
The `wasm build` job downloads the `stellar-cli` release tarball on every run.
It is the slowest non-compilation step in CI and it is fetched from a third
party each time.

## Acceptance Criteria
- [ ] CLI install is cached with `actions/cache`, keyed on the pinned CLI version
- [ ] Cache miss path still installs a working binary and prints `stellar --version`
- [ ] Job time is compared before and after, with both numbers posted in the PR
- [ ] No change to the pinned versions in `rust-toolchain.toml` or `Makefile`

## Tech Stack
GitHub Actions, stellar-cli 30.0.1
BODY
)"

create_issue "chore(repo): add issue and pull request templates" "infra,good first issue" "$(cat <<'BODY'
## Summary
Every issue in this repo follows the same shape (Summary, Acceptance Criteria,
Tech Stack) because it is written by hand each time. Contributors opening issues
or PRs get an empty text box.

## Acceptance Criteria
- [ ] `.github/ISSUE_TEMPLATE/feature.md` and `bug.md` capture Summary, Acceptance Criteria, Tech Stack
- [ ] `.github/pull_request_template.md` asks for what changed, why, and how it was verified
- [ ] Template fields match `CONTRIBUTING.md` (no new process invented in a template)
- [ ] Labels referenced by templates exist in the repo

## Tech Stack
GitHub repository configuration, Markdown/YAML
BODY
)"

# --------------------------------------------------------------------- docs ---

create_issue "docs(site): publish the GitBook site and link it from the README" "docs,good first issue" "$(cat <<'BODY'
## Summary
`docs/` is already structured for GitBook (`SUMMARY.md`, `.gitbook.yaml`) and
covers the protocol, the full contract reference, three user guides and the
developer path. It is only readable in the repository today.

## Acceptance Criteria
- [ ] GitBook space (or equivalent) connected to this repo and built from `docs/`
- [ ] All pages in `SUMMARY.md` resolve, with no broken internal links
- [ ] Intro page renders the banner image
- [ ] Published URL added to `README.md` and to `docs/developers/setup.md`
- [ ] A note states the docs track `main` and are not versioned per release

## Tech Stack
GitBook, GitHub sync, Markdown
BODY
)"

create_issue "docs(repo): add a mainnet readiness checklist" "docs" "$(cat <<'BODY'
## Summary
The contract is deployed on testnet and explicitly unaudited. Before anyone
points real funds at it, there is a set of decisions and checks that are
currently scattered across `docs/security.md` and the positioning notes.

Write a single checklist that a reviewer can work through, with each item
linked to evidence or marked as open.

## Acceptance Criteria
- [ ] `docs/mainnet-readiness.md` lists: audit, arbitration policy, key management,
      monitoring, incident response, upgrade/migration plan, asset allow-list
- [ ] Each item states done / open and links to the issue or document that covers it
- [ ] The admin key question is answered concretely (where it lives, who holds it)
- [ ] Page is added to `docs/SUMMARY.md`
- [ ] README links to it from the deployment section

## Tech Stack
Markdown, GitBook
BODY
)"

echo
if [ "$DRY_RUN" = "1" ]; then
  echo "dry run complete; nothing was created"
else
  echo "done: $(gh issue list --repo "$REPO" --state open --json number --jq 'length') open issues"
fi
