# Contributing to orinex-core

Thanks for picking up an issue. This repo is a Soroban contract workspace; the
bar is that `make verify` passes and nobody has to guess what a change means.

## Before you start

- Read `docs/positioning.md` for what this project is competing with, and
  `README.md` for the contract surface.
- Comment on the issue you intend to work on so two people don't duplicate work.
- If an issue is ambiguous, ask in the issue before writing code. Scope changes
  mid-PR are the main cause of rejected work here.

## Local setup

```bash
rustup show                 # installs the pinned toolchain from rust-toolchain.toml
rustup target add wasm32v1-none
cargo install --locked stellar-cli --version 30.0.1
```

Rust 1.92.0 is pinned deliberately: `soroban-sdk 30` needs ≥1.91, and
`stellar-cli` refuses 1.91.0 for contract builds. Do not bump the toolchain in
an unrelated PR.

## Verification

Every PR must pass the same four checks CI runs (`rustfmt`, `clippy`,
`cargo test`, `wasm build`):

```bash
make verify
```

`make verify` is `fmt-check → clippy → test → build`. Do not add
`#[allow(...)]` to silence a lint; fix the code or, if the lint is genuinely
wrong for the case, explain it in the PR. Do not `#[ignore]` a failing test.

New contract logic needs tests in `contracts/primary/src/test.rs` covering the
success path, each new error path, and the auth path. The suite uses `try_*`
calls and asserts `Err(Ok(Error::X))`.

## Commit conventions

Conventional commits, one logical unit per commit:

```
feat(primary): add admin transfer entrypoint
fix(primary): reject zero-amount deals before funding
test(primary): cover ttl bump on deal read
docs(repo): document dispute resolution flow
chore(ci): cache the wasm build target
```

- Stage specific files. `git add .` and `git add -A` are not used in this repo.
- Keep contract code, tests and docs in separate commits when they are separate
  changes.
- Reference the issue in the commit body when one exists (`Refs #12`).

## Pull requests

- Branch from `main`; `main` requires a PR with passing CI. Branch protection is
  applied by `scripts/setup_repo.sh` and needs an admin-scoped `gh` session.
- PR description: what changed, why, and how you verified it. Paste the failing
  test output if you fixed a bug.
- Keep PRs small enough to review in one sitting. A 1,000-line PR will sit.

## What not to submit

- `unwrap()` / `expect()` / `panic!()` outside `#[cfg(test)]`.
- Floating point anywhere in contract code. Amounts are `i128`, fees are basis
  points (`u32`, `10_000` = 100%).
- New public functions that no user flow calls.
- `todo!()`, `unimplemented!()`, stub modules, or commented-out logic.
- Changes to error codes that renumber existing variants — the code list is
  append-only because downstream clients match on the numbers.
- Silent changes to event topics or payload shapes; they are part of the
  contract spec.

## Security

Do not open a public issue for a vulnerability. See `SECURITY.md`.
