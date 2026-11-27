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

DESCRIPTION="Single-asset escrow for Soroban on Stellar — arbiter-settled disputes, capped protocol fee, deadline reclaim. Rust contract, 39 tests, deployed on testnet."

# Status check names must match the job names in .github/workflows/ci.yml.
CHECKS=("rustfmt" "clippy" "cargo test" "wasm build")

echo "repo: ${REPO}"

echo "→ description"
if ! gh api --method PATCH "repos/${REPO}" -f description="$DESCRIPTION" >/dev/null 2>&1; then
  echo "  error: cannot set description (needs admin scope)" >&2
  exit 1
fi

echo "→ topics"
gh api --method PUT "repos/${REPO}/topics" \
  -f "names[]=stellar" \
  -f "names[]=soroban" \
  -f "names[]=escrow" \
  -f "names[]=smart-contracts" \
  -f "names[]=rust" \
  -f "names[]=stellar-wave" \
  -f "names[]=blockchain" >/dev/null

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
