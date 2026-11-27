#!/usr/bin/env bash
# Applies the repository settings that need administration rights: description,
# topics, and branch protection on main.
#
# These cannot be done with a GitHub App / integration token ("Resource not
# accessible by integration"), so run this yourself with an admin-authenticated
# gh session:
#
#   gh auth login            # as an org owner or repo admin
#   scripts/setup_repo.sh
#
# Safe to re-run: every call is an idempotent PUT/PATCH.
set -euo pipefail

REPO="${REPO:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
BRANCH="${BRANCH:-main}"

# About box: what a user gets, in one sentence, then the stack and status.
# 350 char limit. Competitor repos (Trustless Work, SafeTrust, OfferHub) all
# describe the product, not the feature list — match that before the pitch.
DESCRIPTION="Escrow smart contract for Stellar: two parties agree terms, a named arbiter settles disputes, the fee is capped at 10%, and the payer can reclaim after the deadline. Rust / Soroban, MIT, deployed on testnet."

# Status check names must match the job names in .github/workflows/ci.yml.
CHECKS=("rustfmt" "clippy" "cargo test" "wasm build")

echo "repo: ${REPO}"

echo "→ description"
if ! gh api --method PATCH "repos/${REPO}" -f description="$DESCRIPTION" >/dev/null 2>&1; then
  echo "  error: cannot set description (needs admin scope)" >&2
  exit 1
fi

# Topics are how people browse, so use the words they actually filter by:
# the two stack terms, the category, and the asset domain. Lowercase and
# hyphenated only; 20 is the GitHub cap, 10-12 is the useful range.
echo "→ topics"
gh api --method PUT "repos/${REPO}/topics" \
  -f "names[]=stellar" \
  -f "names[]=soroban" \
  -f "names[]=escrow" \
  -f "names[]=smart-contracts" \
  -f "names[]=rust" \
  -f "names[]=payments" \
  -f "names[]=defi" \
  -f "names[]=blockchain" >/dev/null

# Homepage: add once the GitBook is published, e.g.
#   gh api --method PATCH "repos/${REPO}" -f homepage="https://orinex.gitbook.io/docs"
# Leave it empty until that URL resolves — a dead link in the About box reads
# worse than a blank one.

echo "→ branch protection on ${BRANCH}"
checks_json="$(printf '"%s",' "${CHECKS[@]}")"
checks_json="[${checks_json%,}]"

gh api --method PUT "repos/${REPO}/branches/${BRANCH}/protection" \
  --input - >/dev/null <<JSON
{
  "required_status_checks": {
    "strict": true,
    "contexts": ${checks_json}
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
JSON

echo
echo "Applied. Summary:"
gh api "repos/${REPO}" --jq '"description: \(.description)\ntopics: \(.topics | join(", "))"'
gh api "repos/${REPO}/branches/${BRANCH}/protection" \
  --jq '"checks: \(.required_status_checks.contexts | join(", "))\nreviews: \(.required_pull_request_reviews.required_approving_review_count)"'
