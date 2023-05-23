use serde::{Deserialize, Serialize};
use serde_generate;
use serde_reflection::{Samples, Tracer, TracerConfig};
use solana_sdk::deserialize_utils::default_on_eof;
use solana_sdk::short_vec;

use std::default::Default;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() {
    let commit = "ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb";
    println!("Starting generation for {}...", commit);
    // From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/transaction-status/src/lib.rs#L140-L147
    // This is the last version of the TransactionStatusMeta struct before it
    // started using Protobufs.
    // History taken from from https://github.com/solana-labs/solana/commits/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/transaction-status/src/lib.rs
    generate_bindings(commit);
}

fn generate_bindings(commit: &str) {
    println!("started");
    let conf = TracerConfig::default().record_samples_for_structs(true);
    let mut tracer = Tracer::new(conf);

    let mut samples = Samples::new();
    println!("samples created");

    // Sample cases with success:
    {
        let v = TransactionStatusMeta {
            status: Result::Ok(()),
            fee: 500,
            pre_balances: vec![1, 2, 3],
            post_balances: vec![1, 2, 3],
            inner_instructions: Some(vec![InnerInstructions {
                index: 11,
                instructions: vec![CompiledInstruction {
                    program_id_index: 1,
                    accounts: vec![1, 2, 3],
                    data: vec![1, 2, 3],
                }],
            }]),
        };
        let reg = tracer.trace_value::<TransactionStatusMeta>(&mut samples, &v);
        println!("tracer created");
        if let Err(e) = reg {
            panic!("error: {}", e);
        }
    }
    // Sample cases with errors (all possible):
    {
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
                            inner_instructions: Some(vec![InnerInstructions {
                                index: 11,
                                instructions: vec![CompiledInstruction {
                                    program_id_index: 1,
                                    accounts: vec![1, 2, 3],
                                    data: vec![1, 2, 3],
                                }],
                            }]),
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
                        inner_instructions: Some(vec![InnerInstructions {
                            index: 11,
                            instructions: vec![CompiledInstruction {
                                program_id_index: 1,
                                accounts: vec![1, 2, 3],
                                data: vec![1, 2, 3],
                            }],
                        }]),
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

    let name = "parse_legacy_transaction_status_meta_".to_string() + commit;

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

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/transaction-status/src/lib.rs#L140-L147
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatusMeta {
    pub status: Result<()>,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    #[serde(deserialize_with = "default_on_eof")]
    pub inner_instructions: Option<Vec<InnerInstructions>>,
}

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/transaction-status/src/lib.rs#L96-L101
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstructions {
    /// Transaction instruction index
    pub index: u8,
    /// List of inner instructions
    pub instructions: Vec<CompiledInstruction>,
}

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/sdk/src/instruction.rs#L225-L234
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledInstruction {
    /// Index into the transaction keys array indicating the program account that executes this instruction
    pub program_id_index: u8,
    /// Ordered indices into the transaction keys array indicating which accounts to pass to the program
    #[serde(with = "short_vec")]
    pub accounts: Vec<u8>,
    /// The program input data
    #[serde(with = "short_vec")]
    pub data: Vec<u8>,
}

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/sdk/src/transaction.rs#L95
pub type Result<T> = result::Result<T, TransactionError>;
use std::result;
use thiserror::Error;

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/sdk/src/transaction.rs#L22-L93
#[derive(Error, Debug, Serialize, Deserialize, Default, EnumIter)]
pub enum TransactionError {
    /// An account is already being processed in another transaction in a way
    /// that does not support parallelism
    #[default]
    #[error("Account in use")]
    AccountInUse,

    /// A `Pubkey` appears twice in the transaction's `account_keys`.  Instructions can reference
    /// `Pubkey`s more than once but the message must contain a list with no duplicate keys
    #[error("Account loaded twice")]
    AccountLoadedTwice,

    /// Attempt to debit an account but found no record of a prior credit.
    #[error("Attempt to debit an account but found no record of a prior credit.")]
    AccountNotFound,

    /// Attempt to load a program that does not exist
    #[error("Attempt to load a program that does not exist")]
    ProgramAccountNotFound,

    /// The from `Pubkey` does not have sufficient balance to pay the fee to schedule the transaction
    #[error("Insufficient funds for fee")]
    InsufficientFundsForFee,

    /// This account may not be used to pay transaction fees
    #[error("This account may not be used to pay transaction fees")]
    InvalidAccountForFee,

    /// The bank has seen this `Signature` before. This can occur under normal operation
    /// when a UDP packet is duplicated, as a user error from a client not updating
    /// its `recent_blockhash`, or as a double-spend attack.
    #[error("The bank has seen this signature before")]
    DuplicateSignature,

    /// The bank has not seen the given `recent_blockhash` or the transaction is too old and
    /// the `recent_blockhash` has been discarded.
    #[error("Blockhash not found")]
    BlockhashNotFound,

    /// An error occurred while processing an instruction. The first element of the tuple
    /// indicates the instruction index in which the error occurred.
    #[error("Error processing Instruction {0}: {1}")]
    InstructionError(u8, InstructionError),

    /// Loader call chain is too deep
    #[error("Loader call chain is too deep")]
    CallChainTooDeep,

    /// Transaction requires a fee but has no signature present
    #[error("Transaction requires a fee but has no signature present")]
    MissingSignatureForFee,

    /// Transaction contains an invalid account reference
    #[error("Transaction contains an invalid account reference")]
    InvalidAccountIndex,

    /// Transaction did not pass signature verification
    #[error("Transaction did not pass signature verification")]
    SignatureFailure,

    /// This program may not be used for executing instructions
    #[error("This program may not be used for executing instructions")]
    InvalidProgramForExecution,

    /// Transaction failed to sanitize accounts offsets correctly
    /// implies that account locks are not taken for this TX, and should
    /// not be unlocked.
    #[error("Transaction failed to sanitize accounts offsets correctly")]
    SanitizeFailure,

    #[error("Transactions are currently disabled due to cluster maintenance")]
    ClusterMaintenance,
}

// From https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/sdk/src/instruction.rs#L11-L170
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

    /// Read-only account's lamports modified
    #[error("instruction changed the balance of a read-only account")]
    ReadonlyLamportChange,

    /// Read-only account's data was modified
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
    #[error("insufficient account keys for instruction")]
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
    #[error("custom program error: {0:#x}")]
    Custom(u32),

    /// The return value from the program was invalid.  Valid errors are either a defined builtin
    /// error value or a user-defined error in the lower 32 bits.
    #[error("program returned invalid error code")]
    InvalidError,

    /// Executable account's data was modified
    #[error("instruction changed executable accounts data")]
    ExecutableDataModified,

    /// Executable account's lamports modified
    #[error("instruction changed the balance of a executable account")]
    ExecutableLamportChange,

    /// Executable accounts must be rent exempt
    #[error("executable accounts must be rent exempt")]
    ExecutableAccountNotRentExempt,

    /// Unsupported program id
    #[error("Unsupported program id")]
    UnsupportedProgramId,

    /// Cross-program invocation call depth too deep
    #[error("Cross-program invocation call depth too deep")]
    CallDepth,

    /// An account required by the instruction is missing
    #[error("An account required by the instruction is missing")]
    MissingAccount,

    /// Cross-program invocation reentrancy not allowed for this instruction
    #[error("Cross-program invocation reentrancy not allowed for this instruction")]
    ReentrancyNotAllowed,

    /// Length of the seed is too long for address generation
    #[error("Length of the seed is too long for address generation")]
    MaxSeedLengthExceeded,

    /// Provided seeds do not result in a valid address
    #[error("Provided seeds do not result in a valid address")]
    InvalidSeeds,

    /// Failed to reallocate account data of this length
    #[error("Failed to reallocate account data")]
    InvalidRealloc,

    /// Computational budget exceeded
    #[error("Computational budget exceeded")]
    ComputationalBudgetExceeded,
}
