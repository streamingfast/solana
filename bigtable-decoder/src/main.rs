use {
    clap::Parser,
    prost::Message,
    solana_storage_bigtable::{
        bigtable::{deserialize_protobuf_or_bincode_cell_data, CellData},
        StoredConfirmedBlock,
    },
    solana_storage_proto::convert::generated,
    solana_transaction_status::{ConfirmedBlock, TransactionStatusMeta, TransactionWithMetadata},
    std::io::{self, BufRead},
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// enable hex output
    #[clap(short, long)]
    hex: bool,
}

fn main() {
    let args = Args::parse();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let decoded = hex::decode(line.unwrap()).expect("Decoding failed");
        let deserialized = deserialize_protobuf_or_bincode_cell_data::<
            StoredConfirmedBlock,
            generated::ConfirmedBlock,
        >(&[("bin".to_string(), decoded.clone())], "", "".to_string())
        .unwrap();
        if let CellData::Bincode(stored_confirmed_block) = deserialized {
            let StoredConfirmedBlock {
                previous_blockhash,
                blockhash,
                parent_slot,
                transactions,
                rewards,
                block_time,
                block_height,
            } = stored_confirmed_block;

            let confirm_block = ConfirmedBlock {
                previous_blockhash,
                blockhash,
                parent_slot,
                transactions: transactions
                    .into_iter()
                    .map(|tx| {
                        let mut tm = TransactionWithMetadata {
                            transaction: tx.transaction,
                            meta: TransactionStatusMeta {
                                status: Ok(()),
                                fee: 0,
                                pre_balances: vec![],
                                post_balances: vec![],
                                inner_instructions: None,
                                log_messages: None,
                                pre_token_balances: None,
                                post_token_balances: None,
                                rewards: None,
                            },
                        };
                        if let Some(me) = tx.meta {
                            tm.meta.fee = me.fee;
                            tm.meta.pre_balances = me.pre_balances;
                            tm.meta.post_balances = me.post_balances;
                        }
                        return tm;
                    })
                    .collect(),
                rewards: rewards.into_iter().map(|reward| reward.into()).collect(),
                block_time,
                block_height,
            };

            let protobuf_block = confirmed_block_into_protobuf(confirm_block);
            let mut buf = Vec::with_capacity(protobuf_block.encoded_len());
            protobuf_block.encode(&mut buf).unwrap();
            if args.hex {
                println!("0x{}", hex::encode(buf));
            } else {
                println!("{:?}", buf);
            }
        } else {
            panic!("deserialization should produce CellData::Bincode");
        }
    }
}

fn confirmed_block_into_protobuf(confirmed_block: ConfirmedBlock) -> generated::ConfirmedBlock {
    generated::ConfirmedBlock::from(confirmed_block)
}
