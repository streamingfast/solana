# StreamingFast Fork Changelog

This file tracks StreamingFast-specific changes to this fork of
[anza-xyz/agave](https://github.com/anza-xyz/agave). Upstream changes are documented in
[CHANGELOG.md](CHANGELOG.md).

## v4.3.0-beta.2-fh3.0

### Changed

- Merged upstream `v4.3.0-beta.2` (previously `v4.2.1`), moving the fork from the `v4.2`
  release line to `v4.3`. `v4.3.0-beta.2` is the most recent upstream tag with published
  binaries that is actually running on devnet (`v4.3.0-beta.1` has no artifacts and its
  crates were yanked after a `cargo audit` failure on RUSTSEC-2026-0258).

  Released as image `ghcr.io/streamingfast/solana:v4.3.0-beta.2-fh3.0`.

  Because `v4.2` and `v4.3` diverged upstream and `v4.2.1` fixes reached `v4.3` as
  cherry-picks rather than merges, the merge base is far behind both tags. The merge was
  therefore resolved by taking the upstream `v4.3.0-beta.2` tree wholesale and
  re-applying the fork delta on top, so the tree differs from upstream only by the four
  fork-owned files (`Dockerfile`, `.github/workflows/docker-publish.yml`,
  `CHANGELOG.sf.md`, and the novote guard in
  `geyser-plugin-manager/src/accounts_update_notifier.rs`).

- The novote guard follows upstream's split of `notify_plugins_of_account_update` into
  `notify_plugins_of_account_update_for_bank` and
  `notify_plugins_of_account_update_from_snapshot`. Behaviour is unchanged: vote-owned
  accounts are never forwarded to plugins, while deletions still are.

### Geyser plugin interface

`v4.3` is an Alpenglow release and changes the plugin interface substantially, though
backwards-compatibly — every new callback has a default implementation delegating to the
old one:

- `update_account`, `notify_transaction`, `notify_entry` and `notify_block_metadata` are
  deprecated since `4.3.0` in favour of `update_account_from_snapshot`,
  `update_account_for_bank`, `notify_transaction_for_bank`, `notify_entry_for_bank` and
  `notify_block_metadata_for_bank`, which carry a `BankId` identifying the concrete bank
  instance.
- `update_slot_status` is now only called for statuses with no bank (`FirstShredReceived`,
  `Completed`, `Dead`); bank-scoped statuses (`Confirmed`, `Processed`, `Rooted`,
  `CreatedBank`) go to the new `update_bank_status`.
- New Alpenglow callbacks: `notify_block_footer` (gated on
  `block_footer_notifications_enabled`), `notify_entry_update_parent` and
  `notify_deshred_update_parent`, with the matching
  `ReplicaBlockFooterInfo`, `ReplicaEntryUpdateParentInfo` and
  `ReplicaDeshredUpdateParentInfo` types.

Other consumer-facing crates that changed: `transaction-status` (large rewrite of
confidential-transfer, confidential-mint-burn and permissioned-burn token extension
parsing), `account-decoder` (`parse_sysvar`), and `transaction-context`.

Rust toolchain moves from `1.96.1` to `1.97.1`.

## v4.2.0-fh3.0

### Changed

- Merged upstream `v4.2.0` (previously `v4.2.0-rc.0`), covering `v4.2.0-rc.1` and the
  final release. The merge was conflict-free: no upstream commit in this range touches
  any fork-specific file.

  Released as image `ghcr.io/streamingfast/solana:v4.2.0-fh3.0`.

  Notable upstream changes in this range: migration to `spl-token-2022-interface` 3.x
  with parsing for the `PermissionedBurn` and token `Batch` instructions, `runtime` no
  longer rewriting inactive stakes, a larger stack size for the accounts-hasher rayon
  pool, PoH waking replay after controller-message completion, and the default
  incremental snapshot interval dropping to 200 slots.

  No change to the Geyser plugin interface: `geyser-plugin-interface`,
  `geyser-plugin-manager`, `transaction-context`, `rpc-client` and `rpc-client-api` are
  byte-identical to `v4.2.0-rc.0`. `transaction-status` and `account-decoder` did change
  — the token-2022 3.x migration adds `PermissionedBurn` extension parsing and new parsed
  instruction variants, so consumers decoding parsed token instructions see additional
  variants. Rust toolchain stays at `1.96.1`.

## v4.2.0-rc.0-fh3.0

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
