use solana_sdk::genesis_config::GenesisConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use {
    clap::{crate_description, crate_name, value_t_or_exit, App, Arg},
    log::*,
    solana_runtime::accounts_db::AccountShrinkThreshold,
    solana_runtime::accounts_index::{AccountIndex, AccountSecondaryIndexes},
    solana_runtime::snapshot_utils::bank_from_boot_snapshot,
};

fn main() {
    solana_logger::setup_with_default("solana=info");

    let ledger_path_buf = PathBuf::from("/Users/abourget/dev/solana/testdata");
    let ledger_path = ledger_path_buf.as_path();

    // TODO: load genesis_config
    let genesis_config = GenesisConfig::load(ledger_path).unwrap();

    let res = bank_from_boot_snapshot(
        &[PathBuf::from(
            "/Users/abourget/dev/solana/testdata/accounts",
        )],
        &[],
        &ledger_path,
        &genesis_config,
        None,
        None,
        AccountSecondaryIndexes {
            keys: None,
            indexes: HashSet::<AccountIndex>::new(),
        },
        true,
        None,
        AccountShrinkThreshold::TotalSpace { shrink_ratio: 0.0 },
        false,
        false,
        None,
    );
}

#[cfg(test)]
pub mod test {}
