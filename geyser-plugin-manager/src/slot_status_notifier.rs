use {
    crate::geyser_plugin_manager::GeyserPluginManager,
    log::*,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        SlotStatus,
        ReplicaBlockInfoV3, 
        ReplicaBlockInfoVersions,
    },
    solana_transaction_status::Reward,
    solana_measure::measure::Measure,
    solana_metrics::*,
    solana_sdk::{clock::{Slot, UnixTimestamp}, hash::Hash as SolHash},
    std::sync::{Arc, RwLock},
};

pub trait SlotStatusNotifierInterface {
    /// Notified when a slot is optimistically confirmed
    fn notify_slot_confirmed(&self, slot: Slot, parent: Option<Slot>);

    /// Notified when a slot is marked frozen.
    fn notify_slot_processed(&self, slot: Slot, parent: Option<Slot>, hash: SolHash, parent_hash: SolHash, timestamp: UnixTimestamp);

    /// Notified when a slot is rooted.
    fn notify_slot_rooted(&self, slot: Slot, parent: Option<Slot>);
}

pub type SlotStatusNotifier = Arc<RwLock<dyn SlotStatusNotifierInterface + Sync + Send>>;

pub struct SlotStatusNotifierImpl {
    plugin_manager: Arc<RwLock<GeyserPluginManager>>,
}

impl SlotStatusNotifierInterface for SlotStatusNotifierImpl {
    fn notify_slot_confirmed(&self, slot: Slot, parent: Option<Slot>) {
        self.notify_slot_status(slot, parent, SlotStatus::Confirmed, None, None, None);
    }

    fn notify_slot_processed(&self, slot: Slot, parent: Option<Slot>, hash: SolHash, parent_hash: SolHash, timestamp: UnixTimestamp) {
        self.notify_slot_status(slot, parent, SlotStatus::Processed, Some(hash), Some(parent_hash), Some(timestamp));
    }

    fn notify_slot_rooted(&self, slot: Slot, parent: Option<Slot>) {
        self.notify_slot_status(slot, parent, SlotStatus::Rooted, None, None, None);
    }
}

impl SlotStatusNotifierImpl {
    pub fn new(plugin_manager: Arc<RwLock<GeyserPluginManager>>) -> Self {
        Self { plugin_manager }
    }

    pub fn notify_slot_status(&self, slot: Slot, parent_slot: Option<Slot>, slot_status: SlotStatus, blockhash: Option<SolHash>, parent_blockhash: Option<SolHash>, timestamp: Option<UnixTimestamp>) {
        let plugin_manager = self.plugin_manager.read().unwrap();
        if plugin_manager.plugins.is_empty() {
            return;
        }

        for plugin in plugin_manager.plugins.iter() {
            if let Some(block_hash) = blockhash {
                let blk_hash = block_hash.to_string();
                let parent_blk_hash = parent_blockhash.unwrap_or_default().to_string();

                let block_info = ReplicaBlockInfoV3 {
                    parent_slot: parent_slot.unwrap_or_default().into(),
                    parent_blockhash: parent_blk_hash.as_ref(),
                    slot: slot,
                    blockhash: blk_hash.as_ref(),
                    rewards: &[Reward{
                        pubkey: String::from(""),
                        lamports: 0,
                        post_balance: 0,
                        reward_type: None,
                        commission: None,
                    }],
                    block_time: Some(timestamp.unwrap_or_default()),
                    block_height: None,
                    executed_transaction_count: 0,
                    entry_count: 0,
                };
                let _ =  plugin.notify_block_metadata(ReplicaBlockInfoVersions::V0_0_3(&block_info));
            }

            let mut measure = Measure::start("geyser-plugin-update-slot");
            match plugin.update_slot_status(slot, parent_slot, slot_status) {
                Err(err) => {
                    error!(
                        "Failed to update slot status at slot {}, error: {} to plugin {}",
                        slot,
                        err,
                        plugin.name()
                    )
                }
                Ok(_) => {
                    trace!(
                        "Successfully updated slot status at slot {} to plugin {}",
                        slot,
                        plugin.name()
                    );
                }
            }
            measure.stop();
            inc_new_counter_debug!(
                "geyser-plugin-update-slot-us",
                measure.as_us() as usize,
                1000,
                1000
            );
        }
    }
}
