use crate::pb::codec::{
    AccountChange, BalanceChange, Batch, Instruction, InstructionError as PbInstructionError,
    MessageHeader, Transaction, TransactionError as PbTransactionError,
};
use num_traits::ToPrimitive;
use solana_program::hash::Hash;
use solana_program::instruction::InstructionError;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::{
    borrow::BorrowMut,
    env,
    fs::File,
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};
use std::io::Write;
use prost::Message;
use crate::transaction::TransactionError;

pub static DEEPMIND_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_deepmind() {
    DEEPMIND_ENABLED.store(true, Ordering::Relaxed)
}

pub fn disable_deepmind() {
    DEEPMIND_ENABLED.store(false, Ordering::Relaxed)
}

pub fn deepmind_enabled() -> bool {
    return DEEPMIND_ENABLED.load(Ordering::Relaxed);
}

pub fn inst_err_to_pb(error: &InstructionError) -> Option<PbInstructionError> {
    return Some(PbInstructionError {
        error: error.to_string()
    });
}

impl Instruction {
    pub fn add_account_change(&mut self, pubkey: &Pubkey, pre: &[u8], post: &[u8]) {
        self.account_changes.push(AccountChange {
            pubkey: pubkey.as_ref().to_vec(),
            prev_data: pre.to_vec(),
            new_data: post.to_vec(),
            new_data_length: post.len().to_u64().expect("length is not a valid size"),
            ..Default::default()
        });
    }

    pub fn error(&mut self, error: &InstructionError) {
        if let Some(pb_error) = inst_err_to_pb(error) {
            self.failed = true;
            self.error = Some(pb_error)
        } else {
            panic!("unknown instruction error: {:?}", error);
        }
    }

    pub fn add_lamport_change(&mut self, pubkey: &Pubkey, pre: u64, post: u64) {
        self.balance_changes.push(BalanceChange {
            pubkey: pubkey.as_ref().to_vec(),
            prev_lamports: pre,
            new_lamports: post,
            ..Default::default()
        });
    }
}

#[derive(Default)]
pub struct DMTransaction {
    pub pb_transaction: Transaction,

    pub call_stack: Vec<usize>,
}

impl DMTransaction {
    pub fn start_instruction(
        &mut self,
        program_id: &Pubkey,
        keyed_accounts: &mut dyn Iterator<Item=&Pubkey>,
        instruction_data: &[u8],
    ) {
        let parent_ordinal = *self.call_stack.last().unwrap();
        let inst_ordinal = self.pb_transaction.instructions.len() + 1;
        self.call_stack.push(inst_ordinal);

        self.pb_transaction.instructions.push(Instruction {
            program_id: program_id.to_bytes().to_vec(),
            account_keys: keyed_accounts.map(|key| key.to_bytes().to_vec()).collect(),
            data: instruction_data.to_vec(),
            ordinal: inst_ordinal as u32,
            parent_ordinal: parent_ordinal as u32,
            depth: (self.call_stack.len() - 1) as u32,
            balance_changes: Vec::new(),
            account_changes: Vec::new(),
            ..Default::default()
        });
    }

    pub fn end_instruction(&mut self) {
        self.call_stack.pop();
    }

    pub fn error(&mut self, error: &TransactionError) {
        let pb_trx_error = PbTransactionError {
            error: error.to_string()
        };
        self.pb_transaction.failed = true;
        self.pb_transaction.error = Some(pb_trx_error)
    }

    pub fn add_log(&mut self, log: String) {
        self.pb_transaction.log_messages.push(log)
    }

    pub fn active_instruction(&mut self) -> &mut Instruction {
        return self.pb_transaction.instructions[(self.call_stack.last().unwrap() - 1)]
            .borrow_mut();
    }
}

pub struct DMBatchContext {
    pub batch_number: u64,
    pub trxs: Vec<DMTransaction>,
    pub file: File,
    pub path: String,
    pub filename: String,
}

impl<'a> DMBatchContext {
    pub fn new(batch_id: u64, file_number: usize) -> DMBatchContext {
        let filename = format!("dmlog-{}-{}", file_number + 1, batch_id);
        let file_path = format!(
            "{}{}",
            env::var("DEEPMIND_BATCH_FILES_PATH").unwrap_or(String::from_str("/tmp/").unwrap()),
            filename,
        );
        let fl = File::create(&file_path).unwrap();
        DMBatchContext {
            batch_number: batch_id,
            filename,
            trxs: Vec::new(),
            file: fl,
            path: file_path,
        }
    }

    pub fn start_trx(
        &mut self,
        sigs: &Vec<&Signature>,
        num_required_signatures: u8,
        num_readonly_signed_accounts: u8,
        num_readonly_unsigned_accounts: u8,
        account_keys: &Vec<&Pubkey>,
        recent_blockhash: &Hash,
    ) {
        let header = MessageHeader {
            num_required_signatures: num_required_signatures as u32,
            num_readonly_signed_accounts: num_readonly_signed_accounts as u32,
            num_readonly_unsigned_accounts: num_readonly_unsigned_accounts as u32,
            ..Default::default()
        };

        self.trxs.push(DMTransaction {
            call_stack: vec![0],
            pb_transaction: Transaction {
                id: sigs[0].as_ref().to_vec(),
                additional_signatures: sigs[1..].iter().map(|sig| sig.as_ref().to_vec()).collect(),
                header: Some(header),
                account_keys: account_keys
                    .iter()
                    .map(|key| key.to_bytes().to_vec())
                    .collect(),
                recent_blockhash: recent_blockhash.as_ref().to_vec(),
                ..Default::default()
            },
        })
    }

    pub fn error_trx(&mut self, error: &TransactionError) {
        if let Some(transaction) = self.trxs.last_mut() {
            transaction.error(error)
        }
        // Do we panic here? this should never happen?
    }

    pub fn flush(&mut self) {
        // loop through transations, and instructions, and logs and whateve, and print it all out
        // in a format ConsoleReader appreciated.

        let batch = Batch {
            transactions: self.trxs
                .drain(..)
                .into_iter()
                .map(|x| x.pb_transaction)
                .collect(),
            ..Default::default()
        };

        let mut buf = Vec::new();
        buf.reserve(batch.encoded_len());
        if let Err(e) = batch.encode(&mut buf) {
            println!("DMLOG ERROR FILE {}", e);
            return;
        }

        if let Err(e) = self.file.write(&mut buf) {
            println!("DMLOG ERROR FILE {}", e);
            return;
        }

        if let Err(e) = self.file.sync_all() {
            println!("DMLOG ERROR FILE {}", e);
            return;
        }

        drop(&self.file);
        println!("DMLOG BATCH_FILE {}", self.filename);
    }

    pub fn start_instruction(
        &mut self,
        program_id: &Pubkey,
        keyed_accounts: &mut dyn Iterator<Item=&Pubkey>,
        instruction_data: &[u8],
    ) {
        if let Some(transaction) = self.trxs.last_mut() {
            transaction.start_instruction(program_id, keyed_accounts, instruction_data)
        }
        // Do we panic here? this should never happen?
    }

    pub fn end_instruction(&mut self) {
        if let Some(transaction) = self.trxs.last_mut() {
            transaction.end_instruction()
        }
    }

    pub fn error_instruction(&mut self, error: &InstructionError) {
        if let Some(transaction) = self.trxs.last_mut() {
            let instruction = transaction.active_instruction();
            instruction.error(error);
        }
    }

    pub fn account_change(&mut self, pubkey: &Pubkey, pre: &[u8], post: &[u8]) {
        if let Some(transaction) = self.trxs.last_mut() {
            let instruction = transaction.active_instruction();
            instruction.add_account_change(pubkey, pre, post);
        }
    }
    pub fn lamport_change(&mut self, pubkey: &Pubkey, pre: u64, post: u64) {
        if let Some(transaction) = self.trxs.last_mut() {
            let instruction = transaction.active_instruction();
            instruction.add_lamport_change(pubkey, pre, post);
        }
    }

    pub fn add_log(&mut self, log: String) {
        if let Some(transaction) = self.trxs.last_mut() {
            transaction.add_log(log);
        }
    }
}
