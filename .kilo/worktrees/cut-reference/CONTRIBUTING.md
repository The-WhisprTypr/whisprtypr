# Contributing To WhisprTypr

Thanks for helping improve WhisprTypr. This guide keeps contributions predictable and reviewable.

## Development Setup

Install the required tools:

- Node.js LTS
- pnpm
- Rust stable, minimum Rust 1.81
- Platform dependencies required by Tauri

WhisprTypr currently targets Windows and macOS. Linux desktop builds are not supported.

Install dependencies:

```sh
pnpm install
```

Run the app:

```sh
pnpm tauri:dev
```

## Checks

Before opening a pull request, run:

```sh
pnpm run typecheck
cd src-tauri
cargo test -j 1
```

Use `-j 1` on Windows if parallel Rust builds run out of paging-file space while compiling native AI runtime dependencies.

## Pull Requests

- Keep pull requests focused on one change.
- Include tests for behavior changes.
- Update documentation when user-facing behavior, setup, or configuration changes.
- Do not commit generated release artifacts, signing keys, model files, local databases, or build outputs.
- Do not include real license keys, certificates, access tokens, or private URLs.

## Code Style

- Follow the existing Rust, TypeScript, and React patterns in the repository.
- Prefer small, explicit backend helpers over large command bodies when adding testable logic.
- Keep backend tests deterministic. Use local mock servers instead of real external services.
- Keep UI copy user-facing and concise.

## Reporting Bugs

Use the bug report issue template. Include:

- Operating system and version
- WhisprTypr version or commit
- Steps to reproduce
- Expected behavior
- Actual behavior
- Relevant logs or screenshots, without secrets

## Feature Requests

Open a feature request issue and describe the workflow you want to improve. Small, incremental proposals are easier to review and ship.
