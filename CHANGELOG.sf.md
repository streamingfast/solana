# StreamingFast Fork Changelog

This file tracks StreamingFast-specific changes to this fork of
[anza-xyz/agave](https://github.com/anza-xyz/agave). Upstream changes are documented in
[CHANGELOG.md](CHANGELOG.md).

## Unreleased

### Changed

- Merged upstream `v4.2.0-rc.0` (previously `v4.2.0-beta.1`).

  Released as image `ghcr.io/streamingfast/solana:v4.2.0-rc.0-fh3.0`.

- Release tags now use the Firehose protocol suffix `-fh3.0` instead of `-novote`, so
  Solana matches the tag convention used by the other StreamingFast chain forks. The
  suffix names the Firehose protocol version the build speaks; the novote behaviour
  itself is unchanged.

  Notable upstream changes in this range: vendored `rust-rocksdb` fork
  (`[patch.crates-io]`), `crossbeam-epoch` bump for RUSTSEC-2026-0204, parallel account
  loading for stake-account lt hash updates, `receive_and_buffer` status-cache check
  removal, and XDP packet-drop performance work.

  No change to the Geyser plugin interface: `geyser-plugin-interface`,
  `geyser-plugin-manager`, `transaction-status`, `transaction-context`, `rpc-client` and
  `rpc-client-api` are byte-identical to `v4.2.0-beta.1`. Rust toolchain stays at
  `1.96.1`.

- Added this file to record fork-specific changes without touching the upstream
  `CHANGELOG.md`.

## v4.2.0-beta.1-novote

### Added

- `Dockerfile` and `.github/workflows/docker-publish.yml` building and publishing
  `ghcr.io/streamingfast/solana:<tag>` on every pushed tag.

### Changed

- `geyser-plugin-manager`: never notify plugins of accounts owned by the Vote program
  (the "novote" behaviour), for both live account updates and accounts restored from
  snapshot. Account deletions are still forwarded, since a deletion can carry a
  `write_version` above an ephemeral account creation and must null it out on startup.
