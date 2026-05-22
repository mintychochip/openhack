# Contributing to OpenHack

Thank you for your interest in contributing! This document covers setup, code style, and PR process.

## Development Setup

1. **Prerequisites**: Rust 1.88+, Node.js 20+, Docker, PostgreSQL 16
2. **Clone & Run**: `git clone https://github.com/mintychochip/openhack.git && cd openhack`
3. **Start infrastructure**: `docker compose up -d postgres redis pgbouncer`
4. **Build services**: `cargo build --workspace`
5. **Run tests**: `cargo test --workspace`
6. **Start stack**: `docker compose --profile full up -d`

## Code Style

### Rust
- Run `cargo fmt --all` before committing — CI enforces formatting.
- Run `cargo clippy --workspace -- -D clippy::pedantic -D warnings` — zero warnings allowed.
- Every public function must have a doc comment with **Expected Behavior**, **Raises**, and **Side Effects** sections (see `CLAUDE.md`).
- No `unwrap()` in production code — use `?` or explicit error handling.

### TypeScript (web frontend)
- ESLint is configured; run `npx eslint src/` before committing.
- Use TypeScript strict mode — no `any` types.

### SQL
- All migrations use `IF NOT EXISTS` / `IF NOT EXISTS` for idempotency.
- Migrations are numbered sequentially per schema (e.g., `auth/001_`, `auth/002_`).
- Add new migration files to the appropriate `docker/postgres/<schema>/` directory and include them via `\i` in `docker/postgres/init.sql`.

## Branch Naming

- `feature/<short-description>` — new features
- `fix/<short-description>` — bug fixes
- `refactor/<short-description>` — code rewrites
- `docs/<short-description>` — documentation changes

## Commit Messages

Use conventional commits:

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `ci`
Scopes: service name (e.g., `auth`, `core`, `gateway`, `web`, `cli`, `infra`)

Examples:
- `feat(auth): add MFA TOTP support`
- `fix(gateway): prevent SSE storm on Redis disconnect`
- `docs(helm): add NOTES.txt post-install instructions`

## Pull Request Process

1. Fork the repository and create a feature branch.
2. Make your changes with appropriate tests.
3. Ensure all CI checks pass: `cargo fmt`, `cargo clippy`, `cargo test`, `docker compose config`.
4. Open a PR against the `develop` branch.
5. Request review from a maintainer.
6. Squash-merge on approval.

## Testing

- **Unit tests**: `cargo test --workspace` — all 59+ tests must pass.
- **Docker build**: Verify your service's Docker image builds: `docker compose build <service>`.
- **Integration**: Start the full stack (`docker compose --profile full up -d`) and verify health endpoints.

## Reporting Issues

Open a GitHub Issue with:
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs and environment info
