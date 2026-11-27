<!--
See CONTRIBUTING.md. Keep the PR small enough to review in one sitting, and
reference the issue it addresses (e.g. `Refs #12`).
-->

## What changed
<!-- The smallest change that solves the issue. -->

## Why
<!-- The issue or problem this addresses. Call out any change to event topics,
     payload shapes or error codes and the reason for it. -->

## How it was verified
<!-- Commands you ran and their results. Paste the failing test output if you
     fixed a bug. -->

- [ ] `make verify` passes (`rustfmt`, `clippy -D warnings`, `cargo test`, wasm build)
- [ ] New contract logic has tests in `contracts/primary/src/test.rs` for the success path, each new error path and the auth path
- [ ] No `#[allow(...)]` added to silence a lint and no test `#[ignore]`d
- [ ] If behaviour changed: `docs/protocol-mechanics.md` and `docs/contract-reference.md` are updated
