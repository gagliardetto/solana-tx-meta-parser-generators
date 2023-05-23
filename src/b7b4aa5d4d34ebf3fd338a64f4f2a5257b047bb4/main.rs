#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_generate;
use serde_reflection::{Registry, Samples, Tracer, TracerConfig};
use solana_sdk::deserialize_utils::default_on_eof;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::short_vec;
use solana_sdk::transaction_context::TransactionReturnData;
//use solana_storage_proto::StoredTransactionStatusMeta;
use std::fmt::{self};
use std::io::Write;
use {
    solana_account_decoder::{
        parse_token::{real_number_string_trimmed, UiTokenAmount},
        StringAmount,
    },
    solana_transaction_status::{Reward, RewardType, TransactionTokenBalance},
    std::str::FromStr,
};

use std::default::Default;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
fn main() {
    println!("Hello, world!");
    gen();
}

fn gen() {
    println!("started");
    let conf = TracerConfig::default().record_samples_for_structs(true);
    let mut tracer = Tracer::new(conf);

    let mut samples = Samples::new();
    println!("samples created");

    {
        let v = TransactionStatusMeta {
            status: Result::Ok(()),
            fee: 500,
            pre_balances: vec![1, 2, 3],
            post_balances: vec![1, 2, 3],
        };
        let reg = tracer.trace_value::<TransactionStatusMeta>(&mut samples, &v);
        println!("tracer created");
        if let Err(e) = reg {
            panic!("error: {}", e);
        }
    }
    {
        // iterate over all the variants of the enum TransactionError:
        //

        for te in TransactionError::iter() {
            match te {
                // if it's InstructionError, then iterate over all the variants of InstructionError:
                TransactionError::InstructionError(_a, _b) => {
                    for ie in InstructionError::iter() {
                        let v = TransactionStatusMeta {
                            status: Result::Err(TransactionError::InstructionError(123, ie)),
                            fee: 500,
                            pre_balances: vec![1, 2, 3],
                            post_balances: vec![1, 2, 3],
                        };
                        let reg = tracer.trace_value::<TransactionStatusMeta>(&mut samples, &v);
                        println!("tracer created");
                        if let Err(e) = reg {
                            panic!("error: {}", e);
                        }
                    }
                }
                _ => {
                    let v = TransactionStatusMeta {
                        status: Result::Err(te),
                        fee: 500,
                        pre_balances: vec![1, 2, 3],
                        post_balances: vec![1, 2, 3],
                    };
                    let reg = tracer.trace_value::<TransactionStatusMeta>(&mut samples, &v);
                    println!("tracer created");
                    if let Err(e) = reg {
                        panic!("error: {}", e);
                    }
                }
            }
        }
    }

    let registry = tracer.registry();
    if let Err(ref e) = registry {
        panic!("error: {}", e);
    };
    println!("registry created");

    let name = "parse_legacy_transaction_status_meta";

    // Create Golang definitions.
    let mut source = Vec::new();
    let config = serde_generate::CodeGeneratorConfig::new(name.to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode]);
    println!("config created");

    let generator = serde_generate::golang::CodeGenerator::new(&config);
    println!("generator created");
    let registry = registry.unwrap();
    println!("registry unwrapped");
    generator.output(&mut source, &registry).unwrap();
    println!("output created");

    // Write the generated code to disk.
    std::fs::write(name.to_string() + ".go", source).unwrap();
}

// https://github.com/solana-labs/solana/blob/b7b4aa5d4d34ebf3fd338a64f4f2a5257b047bb4/transaction-status/src/lib.rs
// https://github.com/solana-labs/solana/blob/b7b4aa5d4d34ebf3fd338a64f4f2a5257b047bb4/sdk/src/transaction.rs
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatusMeta {
    pub status: Result<()>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
}
use std::result;

pub type Result<T> = result::Result<T, TransactionError>;
/// Reasons a transaction might be rejected.
#[derive(Serialize, Deserialize, Default, EnumIter)]
pub enum TransactionError {
    /// An account is already being processed in another transaction in a way
    /// that does not support parallelism
    #[default]
    AccountInUse,

    /// A `Pubkey` appears twice in the transaction's `account_keys`.  Instructions can reference
    /// `Pubkey`s more than once but the message must contain a list with no duplicate keys
    AccountLoadedTwice,

    /// Attempt to debit an account but found no record of a prior credit.
    AccountNotFound,

    /// Attempt to load a program that does not exist
    ProgramAccountNotFound,

    /// The from `Pubkey` does not have sufficient balance to pay the fee to schedule the transaction
    InsufficientFundsForFee,

    /// This account may not be used to pay transaction fees
    InvalidAccountForFee,

    /// The bank has seen this `Signature` before. This can occur under normal operation
    /// when a UDP packet is duplicated, as a user error from a client not updating
    /// its `recent_blockhash`, or as a double-spend attack.
    DuplicateSignature,

    /// The bank has not seen the given `recent_blockhash` or the transaction is too old and
    /// the `recent_blockhash` has been discarded.
    BlockhashNotFound,

    /// An error occurred while processing an instruction. The first element of the tuple
    /// indicates the instruction index in which the error occurred.
    InstructionError(u8, InstructionError),

    /// Loader call chain is too deep
    CallChainTooDeep,

    /// Transaction requires a fee but has no signature present
    MissingSignatureForFee,

    /// Transaction contains an invalid account reference
    InvalidAccountIndex,

    /// Transaction did not pass signature verification
    SignatureFailure,

    /// This program may not be used for executing instructions
    InvalidProgramForExecution,
}

use thiserror::Error;
/// Reasons the runtime might have rejected an instruction.
#[derive(Serialize, Deserialize, Debug, Error, PartialEq, Eq, Clone, Default, EnumIter)]
pub enum InstructionError {
    /// Deprecated! Use CustomError instead!
    /// The program instruction returned an error
    #[default]
    #[error("generic instruction error")]
    GenericError,

    /// The arguments provided to a program were invalid
    #[error("invalid program argument")]
    InvalidArgument,

    /// An instruction's data contents were invalid
    #[error("invalid instruction data")]
    InvalidInstructionData,

    /// An account's data contents was invalid
    #[error("invalid account data for instruction")]
    InvalidAccountData,

    /// An account's data was too small
    #[error("account data too small for instruction")]
    AccountDataTooSmall,

    /// An account's balance was too small to complete the instruction
    #[error("insufficient funds for instruction")]
    InsufficientFunds,

    /// The account did not have the expected program id
    #[error("incorrect program id for instruction")]
    IncorrectProgramId,

    /// A signature was required but not found
    #[error("missing required signature for instruction")]
    MissingRequiredSignature,

    /// An initialize instruction was sent to an account that has already been initialized.
    #[error("instruction requires an uninitialized account")]
    AccountAlreadyInitialized,

    /// An attempt to operate on an account that hasn't been initialized.
    #[error("instruction requires an initialized account")]
    UninitializedAccount,

    /// Program's instruction lamport balance does not equal the balance after the instruction
    #[error("sum of account balances before and after instruction do not match")]
    UnbalancedInstruction,

    /// Program modified an account's program id
    #[error("instruction modified the program id of an account")]
    ModifiedProgramId,

    /// Program spent the lamports of an account that doesn't belong to it
    #[error("instruction spent from the balance of an account it does not own")]
    ExternalAccountLamportSpend,

    /// Program modified the data of an account that doesn't belong to it
    #[error("instruction modified data of an account it does not own")]
    ExternalAccountDataModified,

    /// Read-only account modified lamports
    #[error("instruction changed balance of a read-only account")]
    ReadonlyLamportChange,

    /// Read-only account modified data
    #[error("instruction modified data of a read-only account")]
    ReadonlyDataModified,

    /// An account was referenced more than once in a single instruction
    // Deprecated, instructions can now contain duplicate accounts
    #[error("instruction contains duplicate accounts")]
    DuplicateAccountIndex,

    /// Executable bit on account changed, but shouldn't have
    #[error("instruction changed executable bit of an account")]
    ExecutableModified,

    /// Rent_epoch account changed, but shouldn't have
    #[error("instruction modified rent epoch of an account")]
    RentEpochModified,

    /// The instruction expected additional account keys
    #[error("insufficient account key count for instruction")]
    NotEnoughAccountKeys,

    /// A non-system program changed the size of the account data
    #[error("non-system instruction changed account size")]
    AccountDataSizeChanged,

    /// The instruction expected an executable account
    #[error("instruction expected an executable account")]
    AccountNotExecutable,

    /// Failed to borrow a reference to account data, already borrowed
    #[error("instruction tries to borrow reference for an account which is already borrowed")]
    AccountBorrowFailed,

    /// Account data has an outstanding reference after a program's execution
    #[error("instruction left account with an outstanding reference borrowed")]
    AccountBorrowOutstanding,

    /// The same account was multiply passed to an on-chain program's entrypoint, but the program
    /// modified them differently.  A program can only modify one instance of the account because
    /// the runtime cannot determine which changes to pick or how to merge them if both are modified
    #[error("instruction modifications of multiply-passed account differ")]
    DuplicateAccountOutOfSync,

    /// Allows on-chain programs to implement program-specific error types and see them returned
    /// by the Solana runtime. A program-specific error may be any type that is represented as
    /// or serialized to a u32 integer.
    #[error("program error: {0}")]
    CustomError(u32),

    /// The return value from the program was invalid.  Valid errors are either a defined builtin
    /// error value or a user-defined error in the lower 32 bits.
    #[error("program returned invalid error code")]
    InvalidError,
}
