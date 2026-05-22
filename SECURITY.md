# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability in OpenHack, please report it privately:

- **Email**: security@openhack.dev (or open a GitHub Security Advisory)
- **Do not** open a public GitHub Issue for security vulnerabilities.

We ask that you:

1. Report vulnerabilities responsibly — give us 90 days to fix before public disclosure.
2. Avoid accessing or modifying other users' data.
3. Do not degrade service availability during testing.

## What We Scan

- **Rust dependencies**: `cargo audit` runs on every push to `main`/`develop`
- **Docker images**: Trivy container scanning for CRITICAL/HIGH CVEs
- **Secrets**: TruffleHog scans git history for leaked credentials
- **Infrastructure**: Checkov validates Terraform and Helm configurations

## Supported Versions

| Version | Supported |
|---------|-----------|
| `main`  | Yes       |
| `develop` | Yes     |
| Older   | No        |
