use crate::pb::codec::{AccountChange, BalanceChange, Batch, Instruction, MessageHeader, Transaction, InstructionError as PbInstructionError, InstructionErrorType, InstructionErrorCustom, TransactionError as PbTransactionError, TransactionErrorType, TransactionInstructionError};
use num_traits::ToPrimitive;
use protobuf::{Message, RepeatedField, SingularPtrField, ProtobufEnum};
use solana_program::hash::Hash;
use solana_sdk::{pubkey::Pubkey};
use std::{borrow::BorrowMut, env, fs::File, str::FromStr, sync::atomic::{AtomicBool, Ordering}};
use solana_program::instruction::InstructionError;
// use std::ops::Deref;
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

pub fn transaction_err_to_i32(error: &TransactionError) -> i32 {
    return match error {
        TransactionError::AccountInUse => 0,
        TransactionError::AccountLoadedTwice => 1,
        TransactionError::AccountNotFound => 2,
        TransactionError::ProgramAccountNotFound => 3,
        TransactionError::InsufficientFundsForFee => 4,
        TransactionError::InvalidAccountForFee => 5,
        TransactionError::DuplicateSignature => 6,
        TransactionError::BlockhashNotFound => 7,
        TransactionError::InstructionError(_, _) => 8,
        TransactionError::CallChainTooDeep => 9,
        TransactionError::MissingSignatureForFee => 10,
        TransactionError::InvalidAccountIndex => 11,
        TransactionError::SignatureFailure => 12,
        TransactionError::InvalidProgramForExecution => 13,
        TransactionError::SanitizeFailure => 14,
        TransactionError::ClusterMaintenance => 15,
    }
}

pub fn instruction_err_to_i32(error: &InstructionError) -> i32 {
    return match error {
        InstructionError::GenericError => 0,
        InstructionError::InvalidArgument => 1,
        InstructionError::InvalidInstructionData => 2,
        InstructionError::InvalidAccountData => 3,
        InstructionError::AccountDataTooSmall => 4,
        InstructionError::InsufficientFunds => 5,
        InstructionError::IncorrectProgramId => 6,
        InstructionError::MissingRequiredSignature => 7,
        InstructionError::AccountAlreadyInitialized => 8,
        InstructionError::UninitializedAccount => 9,
        InstructionError::UnbalancedInstruction => 10,
        InstructionError::ModifiedProgramId => 11,
        InstructionError::ExternalAccountLamportSpend => 12,
        InstructionError::ExternalAccountDataModified => 13,
        InstructionError::ReadonlyLamportChange => 14,
        InstructionError::ReadonlyDataModified => 15,
        InstructionError::DuplicateAccountIndex => 16,
        InstructionError::ExecutableModified => 17,
        InstructionError::RentEpochModified => 18,
        InstructionError::NotEnoughAccountKeys => 19,
        InstructionError::AccountDataSizeChanged => 20,
        InstructionError::AccountNotExecutable => 21,
        InstructionError::AccountBorrowFailed => 22,
        InstructionError::AccountBorrowOutstanding => 23,
        InstructionError::DuplicateAccountOutOfSync => 24,
        InstructionError::Custom(_) => 25,
        InstructionError::InvalidError => 26,
        InstructionError::ExecutableDataModified => 27,
        InstructionError::ExecutableLamportChange => 28,
        InstructionError::ExecutableAccountNotRentExempt => 29,
        InstructionError::UnsupportedProgramId => 30,
        InstructionError::CallDepth => 31,
        InstructionError::MissingAccount => 32,
        InstructionError::ReentrancyNotAllowed => 33,
        InstructionError::MaxSeedLengthExceeded => 34,
        InstructionError::InvalidSeeds => 35,
        InstructionError::InvalidRealloc => 36,
        InstructionError::ComputationalBudgetExceeded => 37,
        InstructionError::PrivilegeEscalation => 38,
        InstructionError::ProgramEnvironmentSetupFailure => 39,
        InstructionError::ProgramFailedToComplete => 40,
        InstructionError::ProgramFailedToCompile => 41,
        InstructionError::Immutable => 42,
        InstructionError::IncorrectAuthority => 43,
    };
}

pub fn inst_err_to_pb(error: &InstructionError) -> Option<PbInstructionError> {
    let pb_inst_error_type_opt = InstructionErrorType::from_i32(instruction_err_to_i32(error));
    if let Some(pb_inst_error_type) = pb_inst_error_type_opt {
        let mut pb_inst_error =  PbInstructionError {
            field_type: pb_inst_error_type,
            ..Default::default()
        };
        if let InstructionError::Custom(error_id) = error {
            let i = &mut InstructionErrorCustom::new();
            i.set_id(error_id.clone());
            let pb_any_res = protobuf::well_known_types::Any::pack_dyn(i);
            match pb_any_res {
                Ok(pb_any) => {
                    pb_inst_error.set_payload(pb_any);
                },
                Err(e) => {}
            }
        }
        return Some(pb_inst_error)
    }

    return None
}

impl Instruction {
    pub fn add_account_change(&mut self, pubkey: Pubkey, pre: &[u8], post: &[u8]) {
        let post_len = post.len();
        let mut account = AccountChange {
            pubkey: format!("{}", pubkey),
            prev_data: Vec::with_capacity(pre.len()),
            new_data: Vec::with_capacity(post_len),
            new_data_length: post_len.to_u64().unwrap_or(0),
            ..Default::default()
        };
        account.prev_data.extend_from_slice(pre);
        account.new_data.extend_from_slice(post);
        self.account_changes.push(account);
    }

    pub fn error(&mut self, error: &InstructionError) {
        if let Some(pb_error) = inst_err_to_pb(error) {
            self.failed = true;
            self.error = SingularPtrField::from_option(Some(pb_error))
        } else {
            panic!(format!("unknown instruction error: {:?}", error));
        }
    }

    pub fn add_lamport_change(&mut self, pubkey: Pubkey, pre: u64, post: u64) {
        self.balance_changes.push(BalanceChange {
            pubkey: format!("{}", pubkey),
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
        program_id: Pubkey,
        keyed_accounts: &[String],
        instruction_data: &[u8],
    ) {
        let accounts: RepeatedField<String> = keyed_accounts
            .into_iter()
            .map(|i| format!("{}", i))
            .collect();

        let parent_ordinal = *self.call_stack.last().unwrap();
        let inst_ordinal = self.pb_transaction.instructions.len() + 1;
        self.call_stack.push(inst_ordinal);

        let mut inst = Instruction {
            program_id: format!("{}", program_id),
            account_keys: accounts,
            data: Vec::with_capacity(instruction_data.len()),
            ordinal: inst_ordinal as u32,
            parent_ordinal: parent_ordinal as u32,
            depth: (self.call_stack.len() - 1) as u32,
            balance_changes: RepeatedField::default(),
            account_changes: RepeatedField::default(),
            ..Default::default()
        };
        inst.data.extend_from_slice(instruction_data);
        self.pb_transaction.instructions.push(inst);
    }

    pub fn end_instruction(&mut self) {
        self.call_stack.pop();
    }

    pub fn error(&mut self, error: &TransactionError) {
        self.pb_transaction.failed = true;
        let pb_trx_error_type_opt = TransactionErrorType::from_i32(transaction_err_to_i32(error));
        if let Some(pb_trx_error_type) = pb_trx_error_type_opt {
            let mut pb_trx_error =  PbTransactionError {
                field_type: pb_trx_error_type,
                ..Default::default()
            };
            if let TransactionError::InstructionError(inst_index, inst_err) = error {
                if let Some(pb_inst_error) = inst_err_to_pb(&inst_err) {
                    let i = &mut TransactionInstructionError::new();
                    i.set_error(pb_inst_error);
                    i.set_Index(*inst_index as u32);
                    let pb_any_res = protobuf::well_known_types::Any::pack_dyn(i);
                    match pb_any_res {
                        Ok(pb_any) => {
                            pb_trx_error.set_payload(pb_any);
                        },
                        Err(e) => {
                            panic!(format!("unable to proto pack: {:?}", e));
                        }
                    }
                } else {
                    panic!(format!("unknown instruction error: {:?}", error));
                }
            }
            self.pb_transaction.error = SingularPtrField::from_option(Some(pb_trx_error))
        } else {
            panic!(format!("unknown transaction error: {:?}", error));
        }
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
    pub filename: String
}

impl<'a> DMBatchContext {
    pub fn new(batch_id: u64, file_number: usize) -> DMBatchContext {
        let filename = format!(
            "dmlog-{}-{}",
            file_number + 1,
            batch_id
        );
        let file_path = format!(
            "{}{}",
            env::var("DEEPMIND_BATCH_FILES_PATH").unwrap_or(String::from_str("/tmp/").unwrap()),
            filename,
        );
        let fl = File::create(&file_path).unwrap();
        DMBatchContext {
            batch_number: batch_id,
            filename: filename,
            trxs: Vec::new(),
            file: fl,
            path: file_path,
        }
    }

    pub fn start_trx(
        &mut self,
        sigs: Vec<String>,
        num_required_signatures: u8,
        num_readonly_signed_accounts: u8,
        num_readonly_unsigned_accounts: u8,
        account_keys: Vec<String>,
        recent_blockhash: Hash,
    ) {
        let header = MessageHeader {
            num_required_signatures: num_required_signatures as u32,
            num_readonly_signed_accounts: num_readonly_signed_accounts as u32,
            num_readonly_unsigned_accounts: num_readonly_unsigned_accounts as u32,
            ..Default::default()
        };
        let trx = Transaction {
            id: sigs[0].clone(),
            additional_signatures: RepeatedField::from_slice(&sigs[1..]),
            header: SingularPtrField::from_option(Some(header)),
            account_keys: RepeatedField::from_vec(account_keys),
            recent_blockhash: format!("{}", recent_blockhash),
            ..Default::default()
        };
        self.trxs.push(DMTransaction {
            pb_transaction: trx,
            call_stack: vec![0],
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
            transactions: RepeatedField::from_vec(
                self.trxs
                    .drain(..)
                    .into_iter()
                    .map(|x| x.pb_transaction)
                    .collect(),
            ),
            ..Default::default()
        };

        if let Err(e) = batch.write_to_writer(&mut self.file) {
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
        program_id: Pubkey,
        keyed_accounts: &[String],
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

    pub fn account_change(&mut self, pubkey: Pubkey, pre: &[u8], post: &[u8]) {
        if let Some(transaction) = self.trxs.last_mut() {
            let instruction = transaction.active_instruction();
            instruction.add_account_change(pubkey, pre, post);
        }
    }
    pub fn lamport_change(&mut self, pubkey: Pubkey, pre: u64, post: u64) {
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
