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

    let ledger_path_buf = PathBuf::from("/data/sf-data/mindreader/data/testdata/boot-snapshot");
    let ledger_path = ledger_path_buf.as_path();

    // TODO: load genesis_config
    let genesis_config = GenesisConfig::load(ledger_path).unwrap();

    let (bank, _timings) = bank_from_boot_snapshot(
        &[PathBuf::from(
            "/data/sf-data/mindreader/data/testdata/accounts",
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
    ).unwrap();

    let storages: Vec<_> = bank.get_snapshot_storages();

    storages.iter().for_each(|e| {
        e.iter().for_each(|t| {
	    info!("reference to snapshot storage: {:?}", &t.get_path());
            //t.accounts.set_no_remove_on_drop_unchecked();
        });
    });
    
}

#[cfg(test)]
pub mod test {}
