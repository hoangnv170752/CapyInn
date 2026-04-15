# Contributing to MHM

Thanks for contributing.

## Project Layout

- `mhm/` is the core PMS application.
- `docs/plans/` contains public implementation plans and release prep notes.

## Prerequisites

- macOS 12 or newer
- Node.js 20 or newer
- Rust stable via `rustup`
- Xcode Command Line Tools

## Local Setup

```bash
git clone https://github.com/chuanman2707/Hotel-Manager.git
cd Hotel-Manager/mhm
npm ci
```

## Development

```bash
npm run tauri dev
```

## Verification

Run these before opening a PR:

```bash
cd mhm
npm test
npm run build

cd src-tauri
cargo check
cargo test
```

If you changed Rust code, also run:

```bash
cargo clippy --all-targets -- -D warnings
```

## Coding Conventions

- Keep changes scoped and easy to review.
- Prefer editing existing files over rewriting large areas.
- TypeScript should stay strict and type-safe.
- Rust should compile cleanly and pass clippy.
- Avoid committing secrets, local paths, exported browser cookies, or internal agent files.

## Commits

Use Conventional Commits where practical:

- `feat:`
- `fix:`
- `docs:`
- `refactor:`
- `test:`
- `chore:`

## Pull Requests

Before opening a PR:

- explain the problem and the chosen approach
- list user-visible changes
- list verification commands and results
- note any follow-up work or known limitations

Prefer small, focused PRs over broad mixed-scope changes.

## Issues

- Use bug reports for concrete defects with repro steps.
- Use feature requests for user-facing improvements.
- Use Discussions for open-ended questions and design discussion if enabled.

## Security

Do not open public issues for security-sensitive findings. See [SECURITY.md](SECURITY.md).
