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
use solana_sdk::{transaction::Result, transaction_context::TransactionReturnData};
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

fn main() {
    println!("Hello, world!");
    gen();
}

fn gen() {
    println!("started");
    let mut tracer = Tracer::new(TracerConfig::default());

    let mut samples = Samples::new();
    println!("samples created");

    let v = TransactionStatusMeta {
        status: Ok(()),
        fee: 500,
        pre_balances: vec![1, 2, 3],
        post_balances: vec![1, 2, 3],
        inner_instructions: Some(vec![InnerInstructions {
            index: 0,
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![1, 2, 3],
                data: vec![1, 2, 3],
            }],
        }]),
    };
    let re = tracer.trace_value::<TransactionStatusMeta>(&mut samples, &v);
    println!("tracer created");

    if let Err(e) = re {
        panic!("error: {}", e);
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

// https://github.com/solana-labs/solana/blob/ce598c5c98e7384c104fe7f5121e32c2c5a2d2eb/transaction-status/src/lib.rs
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstructions {
    /// Transaction instruction index
    pub index: u8,
    /// List of inner instructions
    pub instructions: Vec<CompiledInstruction>,
}

/// An instruction to execute a program
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
