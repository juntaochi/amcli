# Archived v0.1.0 Manual Release Notes

> **Obsolete for current releases.** This file records the manual recovery plan used around the first `v0.1.0` release. Do not use it as the active release checklist for the current codebase.

Current release work should use:

- `Cargo.toml` for the package version (currently `0.3.0` at the time of this cleanup)
- `CHANGELOG.md` for user-facing release notes
- `RELEASE.md` for the active version-agnostic release checklist
- `.github/workflows/release.yml` for automated macOS release asset generation
- `.github/workflows/backfill-release-asset.yml` for authorized release asset backfills
- `homebrew/README.md` and `homebrew/amcli.rb` for Homebrew distribution notes/template

## Historical Context

The original contents of this file described a one-off manual recovery path for `v0.1.0`, including local ARM64 tarballs under `/tmp`, manual GitHub release uploads, and first-time Homebrew tap setup. Those instructions are no longer safe as active guidance because the repository has since moved to later versions and has automated release/backfill workflows.

## Current Recovery Pattern

If a future release is missing assets or Homebrew metadata:

1. Confirm the intended version from `Cargo.toml` and the matching git tag.
2. Run local verification first: `make verify`.
3. Prefer the GitHub release workflow or the backfill workflow over ad-hoc local binaries.
4. Verify both Apple Silicon (`arm64`) and Intel (`x86_64`) artifacts when the release promises both.
5. Update the Homebrew formula only after artifact URLs and SHA256 checksums are confirmed.
6. Do not push tags, publish releases, or update a remote tap without explicit operator authorization.

## Migration Note

The detailed `v0.1.0` commands were intentionally removed from the active workspace docs to avoid misleading future `v0.3.x+` release decisions. Use git history if the exact historical manual steps are needed for forensic purposes.
