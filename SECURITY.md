# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in RolyPoly, please report it
responsibly using
[GitHub Security Advisories](https://github.com/IamMikeHelsel/rolypoly/security/advisories/new)
rather than opening a public issue.

Please include:
- A description of the vulnerability
- Steps to reproduce
- Potential impact

You can expect an initial response within 72 hours. Once the issue is
confirmed, a fix will be prioritised and released as a patch version.

## Security Considerations

RolyPoly is a ZIP archiver and handles untrusted archive files. The
following classes of vulnerability are in scope:

- **Path traversal (Zip Slip)** – an archive entry whose name contains
  `../` segments could write files outside the intended output directory.
- **Zip bombs** – a small archive that expands to an extremely large size,
  exhausting disk space or memory.
- **Invalid or malformed archives** – crafted entries that trigger panics,
  infinite loops, or excessive resource consumption during listing,
  extraction, or validation.

## Automated Security Scanning

Every pull request and push to `main` is checked by:

| Tool | Purpose |
| ---- | ------- |
| `cargo audit` | Scans Rust dependencies for known CVEs |
| `cargo deny` | Enforces license policy and bans unsafe sources |
| `cargo clippy` | Catches common correctness and safety issues |
| Dependabot | Proposes dependency and GitHub Actions version updates |
