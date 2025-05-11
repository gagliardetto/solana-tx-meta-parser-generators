use serde::{Deserialize, Serialize};
use serde_generate;
use serde_reflection::{Samples, Tracer, TracerConfig};
use solana_sdk::deserialize_utils::default_on_eof;
use solana_sdk::short_vec;

use std::default::Default;
use strum::IntoEnumIterator;

fn main() {
    let commit = "agave";
    println!("Starting go generation for {}...", commit);
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
        let sample_meta = solana_transaction_status_client_types::TransactionStatusMeta {
            status: Result::Ok(()),
            fee: 500,
            pre_balances: vec![1, 2, 3],
            post_balances: vec![1, 2, 3],
            inner_instructions: Some(vec![
                solana_transaction_status_client_types::InnerInstructions {
                    index: 11,
                    instructions: vec![solana_transaction_status_client_types::InnerInstruction {
                        instruction: solana_message::compiled_instruction::CompiledInstruction {
                            program_id_index: 1,
                            accounts: Vec::from([1u8, 2, 3, 4, 123]),
                            data: Vec::from([1u8, 2, 3]),
                        },
                        stack_height: Some(123),
                    }],
                },
            ]),
            log_messages: Some(vec![
                "log message 1".to_string(),
                "log message 2".to_string(),
            ]),
            return_data: Some(solana_transaction_context::TransactionReturnData {
                program_id: solana_sdk::pubkey::new_rand(),
                data: vec![1, 2, 3],
            }),
            rewards: Some(vec![solana_transaction_status_client_types::Reward {
                pubkey: "pubkey".to_string(),
                lamports: 100,
                post_balance: 200,
                reward_type: Some(solana_reward_info::RewardType::Rent),
                commission: Some(123),
            }]),
            compute_units_consumed: Some(100),
            loaded_addresses: solana_message::v0::LoadedAddresses {
                writable: vec![
                    solana_sdk::pubkey::new_rand(),
                    solana_sdk::pubkey::new_rand(),
                ],
                readonly: vec![
                    solana_sdk::pubkey::new_rand(),
                    solana_sdk::pubkey::new_rand(),
                ],
            },
            pre_token_balances: Some(vec![
                solana_transaction_status_client_types::TransactionTokenBalance {
                    account_index: 1,
                    mint: "mint".to_string(),
                    ui_token_amount: solana_account_decoder_client_types::token::UiTokenAmount {
                        ui_amount: Some(1.0),
                        decimals: 2,
                        amount: "100".to_string(),
                        ui_amount_string: "1.0".to_string(),
                    },
                    owner: "owner".to_string(),
                    program_id: "program_id".to_string(),
                },
            ]),
            post_token_balances: Some(vec![
                solana_transaction_status_client_types::TransactionTokenBalance {
                    account_index: 1,
                    mint: "mint".to_string(),
                    ui_token_amount: solana_account_decoder_client_types::token::UiTokenAmount {
                        ui_amount: Some(1.0),
                        decimals: 2,
                        amount: "100".to_string(),
                        ui_amount_string: "1.0".to_string(),
                    },
                    owner: "owner".to_string(),
                    program_id: "program_id".to_string(),
                },
            ]),
        };
        {
            let mut reward_iter = RewardTypeIter::new();
            for reward in reward_iter.by_ref() {
                let mut sample_meta = sample_meta.clone();
                sample_meta.rewards = Some(vec![solana_transaction_status_client_types::Reward {
                    pubkey: "pubkey".to_string(),
                    lamports: 100,
                    post_balance: 200,
                    reward_type: Some(reward),
                    commission: Some(123),
                }]);
                let reg = tracer
                    .trace_value::<solana_transaction_status_client_types::TransactionStatusMeta>(
                        &mut samples,
                        &sample_meta,
                    );
                println!("registered sample");
                if let Err(e) = reg {
                    panic!("error: {}", e);
                }
            }
            let reg = tracer
                .trace_value::<solana_transaction_status_client_types::TransactionStatusMeta>(
                    &mut samples,
                    &sample_meta,
                );
            println!("registered sample");
            if let Err(e) = reg {
                panic!("error: {}", e);
            }
        }
        {
            // register all solana_transaction_error::TransactionError variants individually
            let mut te_iter = TransactionErrorIter::new();
            for te in te_iter.by_ref() {
                let te_name = format!("{:?}", te);
                let sample_err = te;
                let reg = tracer.trace_value::<solana_transaction_error::TransactionError>(
                    &mut samples,
                    &sample_err,
                );
                println!("TransactionError registered: {}", te_name);
                if let Err(e) = reg {
                    panic!("error: {}", e);
                }
            }
        }
        let mut te_iter = TransactionErrorIter::new();
        // Sample cases with errors (all possible):
        for te in te_iter.by_ref() {
            let te_name = format!("{:?}", te);
            match te {
                // if it's InstructionError, then iterate over all the variants of InstructionError:
                solana_transaction_error::TransactionError::InstructionError(_a, _b) => {
                    println!("TransactionError registered: {}", te_name);

                    let mut i_iter = InstructionErrorIter::new();
                    for ie in i_iter.by_ref() {
                        let ie_name = format!("{:?}", ie);
                        let mut sample_meta = sample_meta.clone();
                        sample_meta.status = Result::Err(
                            solana_transaction_error::TransactionError::InstructionError(123, ie),
                        );

                        let reg = tracer.trace_value::<solana_transaction_status_client_types::TransactionStatusMeta>(&mut samples, &sample_meta);
                        println!("InstructionError type registered: {}", ie_name);
                        if let Err(e) = reg {
                            panic!("error: {}", e);
                        }
                    }
                }
                _ => {
                    let mut another_sample = sample_meta.clone();
                    another_sample.status = Result::Err(te);
                    let reg = tracer.trace_value::<solana_transaction_status_client_types::TransactionStatusMeta>(&mut samples, &another_sample);
                    println!("TransactionError registered: {}", te_name);
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

    let name = "transaction_status_meta_serde_".to_string() + commit;

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

    let dir = format!("generated/{}", commit);
    // Create the directory if it doesn't exist
    if !std::path::Path::new(&dir).exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let save_to_path = format!("generated/{}/{}.go", commit, name);

    // Write the generated code to disk.
    std::fs::write(save_to_path, source).unwrap();
}

use solana_transaction_error::TransactionError;

#[derive(Debug, Clone)]
struct TransactionErrorIter {
    index: usize,
}

impl TransactionErrorIter {
    fn new() -> Self {
        TransactionErrorIter { index: 0 }
    }
}
// Predefined array of all variants you want to iterate
fn all_transaction_errors() -> &'static [TransactionError] {
    use TransactionError::*;

    &[
        AccountInUse,
        AccountLoadedTwice,
        AccountNotFound,
        ProgramAccountNotFound,
        InsufficientFundsForFee,
        InvalidAccountForFee,
        AlreadyProcessed,
        BlockhashNotFound,
        InstructionError(1, solana_sdk::instruction::InstructionError::GenericError),
        CallChainTooDeep,
        MissingSignatureForFee,
        InvalidAccountIndex,
        SignatureFailure,
        InvalidProgramForExecution,
        SanitizeFailure,
        ClusterMaintenance,
        AccountBorrowOutstanding,
        WouldExceedMaxBlockCostLimit,
        UnsupportedVersion,
        InvalidWritableAccount,
        WouldExceedMaxAccountCostLimit,
        WouldExceedAccountDataBlockLimit,
        TooManyAccountLocks,
        AddressLookupTableNotFound,
        InvalidAddressLookupTableOwner,
        InvalidAddressLookupTableData,
        InvalidAddressLookupTableIndex,
        InvalidRentPayingAccount,
        WouldExceedMaxVoteCostLimit,
        WouldExceedAccountDataTotalLimit,
        DuplicateInstruction(12),
        InsufficientFundsForRent { account_index: 123 },
        MaxLoadedAccountsDataSizeExceeded,
        InvalidLoadedAccountsDataSizeLimit,
        ResanitizationNeeded,
        ProgramExecutionTemporarilyRestricted { account_index: 123 },
        UnbalancedTransaction,
        ProgramCacheHitMaxLimit,
        CommitCancelled,
    ]
}
impl Iterator for TransactionErrorIter {
    type Item = TransactionError;

    fn next(&mut self) -> Option<Self::Item> {
        let errors = all_transaction_errors();
        if self.index >= errors.len() {
            return None;
        }
        let item = errors[self.index].clone(); // Requires TransactionError: Clone
        self.index += 1;
        Some(item)
    }
}

use solana_sdk::instruction::InstructionError;

#[derive(Debug, Clone)]
struct InstructionErrorIter {
    index: usize,
}

impl InstructionErrorIter {
    fn new() -> Self {
        InstructionErrorIter { index: 0 }
    }
}
use once_cell::sync::Lazy;

static INSTRUCTION_ERRORS: Lazy<Vec<InstructionError>> = Lazy::new(|| {
    use InstructionError::*;

    vec![
        GenericError,
        InvalidArgument,
        InvalidInstructionData,
        InvalidAccountData,
        AccountDataTooSmall,
        InsufficientFunds,
        IncorrectProgramId,
        MissingRequiredSignature,
        AccountAlreadyInitialized,
        UninitializedAccount,
        UnbalancedInstruction,
        ModifiedProgramId,
        ExternalAccountLamportSpend,
        ExternalAccountDataModified,
        ReadonlyLamportChange,
        ReadonlyDataModified,
        DuplicateAccountIndex,
        ExecutableModified,
        RentEpochModified,
        NotEnoughAccountKeys,
        AccountDataSizeChanged,
        AccountNotExecutable,
        AccountBorrowFailed,
        AccountBorrowOutstanding,
        DuplicateAccountOutOfSync,
        Custom(999),
        InvalidError,
        ExecutableDataModified,
        ExecutableLamportChange,
        ExecutableAccountNotRentExempt,
        UnsupportedProgramId,
        CallDepth,
        MissingAccount,
        ReentrancyNotAllowed,
        MaxSeedLengthExceeded,
        InvalidSeeds,
        InvalidRealloc,
        ComputationalBudgetExceeded,
        PrivilegeEscalation,
        ProgramEnvironmentSetupFailure,
        ProgramFailedToComplete,
        ProgramFailedToCompile,
        Immutable,
        IncorrectAuthority,
        BorshIoError("some error".into()),
        AccountNotRentExempt,
        InvalidAccountOwner,
        ArithmeticOverflow,
        UnsupportedSysvar,
        IllegalOwner,
        MaxAccountsDataAllocationsExceeded,
        MaxAccountsExceeded,
        MaxInstructionTraceLengthExceeded,
        BuiltinProgramsMustConsumeComputeUnits,
    ]
});

fn all_instruction_errors() -> &'static [InstructionError] {
    &INSTRUCTION_ERRORS
}

impl Iterator for InstructionErrorIter {
    type Item = InstructionError;

    fn next(&mut self) -> Option<Self::Item> {
        let variants = all_instruction_errors();
        if self.index >= variants.len() {
            return None;
        }
        let item = variants[self.index].clone(); // Requires InstructionError: Clone
        self.index += 1;
        Some(item)
    }
}

use solana_reward_info::RewardType;

#[derive(Debug, Clone)]
struct RewardTypeIter {
    index: usize,
}
impl RewardTypeIter {
    fn new() -> Self {
        RewardTypeIter { index: 0 }
    }
}

fn all_reward_types() -> &'static [RewardType] {
    use RewardType::*;

    &[Fee, Rent, Staking, Voting]
}
impl Iterator for RewardTypeIter {
    type Item = RewardType;

    fn next(&mut self) -> Option<Self::Item> {
        let variants = all_reward_types();
        if self.index >= variants.len() {
            return None;
        }
        let item = variants[self.index].clone(); // Requires RewardType: Clone
        self.index += 1;
        Some(item)
    }
}
