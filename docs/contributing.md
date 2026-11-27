# Contributing

The full workflow lives in
[`CONTRIBUTING.md`](https://github.com/Orinex-Soroban/orinex-core/blob/main/CONTRIBUTING.md).
This page is the short version.

## Pick up an issue

Issues are labelled by area (`contract`, `tests`, `docs`, `infra`) and size
(`good first issue` for scoped, low-context work). Comment on the issue before
starting so two people don't build the same thing.

## The non-negotiables

- `make verify` must pass: `rustfmt`, `clippy -D warnings`, 39 tests, wasm build.
- Conventional commits, one logical unit per commit, no `git add .`.
- No `unwrap()`/`expect()`/`panic!()` outside `#[cfg(test)]`.
- No floating point in contract code; `i128` amounts, `u32` basis points.
- New error codes are appended, never renumbered — clients match on numbers.
- Event topics and payload shapes are part of the public interface; changing
  one is a breaking change and needs a reason in the PR.

## What a good contract PR contains

1. The smallest change that solves the issue.
2. A test for the new path, including the error path and the auth path.
3. Doc comments where behaviour is non-obvious — especially anything about
   timing, TTL, or who is allowed to call what.
4. No dead code, no speculative functions, no `#[allow(...)]` to silence lints.

## Reviewing

`main` takes PRs only, with one approval and green CI; the protection rules are
applied by `scripts/setup_repo.sh`. Reviewers look for:
does the change match the issue, does the test actually exercise the failure it
claims to, and does anything new widen what an admin or a party can do.

## Documentation changes

Docs are ordinary PRs. If you change behaviour, update
`docs/protocol-mechanics.md` and `docs/contract-reference.md` in the same PR —
stale docs about money paths are worse than no docs.
