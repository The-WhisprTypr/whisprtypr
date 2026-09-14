# Security Policy

## Supported Versions

Security fixes are applied to the default branch. Tagged releases may receive fixes when maintainers decide a patch release is needed.

## Reporting A Vulnerability

Please do not open a public issue for a suspected vulnerability.

Use GitHub's private vulnerability reporting feature if it is available for this repository. If private reporting is not available, contact the repository owner through GitHub and share only enough information to establish a secure reporting channel.

Include:

- A clear description of the issue
- Steps to reproduce
- Affected platform and version
- Impact and likely attack scenario
- Any proposed fix, if you have one

## Scope

Security-sensitive areas include:

- Tauri command validation and filesystem access
- Model download and update handling
- License cache encryption and device-bound data
- SQLite storage and import/export logic
- Text injection and clipboard behavior
- Release signing and updater metadata

## Secrets

Never commit signing keys, certificates, access tokens, license keys, model credentials, private API keys, or local databases. The repository ignores common secret and release-artifact paths, but contributors are still responsible for checking their changes before opening a pull request.
