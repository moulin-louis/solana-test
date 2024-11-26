use borsh::BorshDeserialize;
use env_logger;
use log::{error, info};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use state::CounterAccount;

use super::*;

fn panic_log(content: String) -> ! {
    error!("{}", content);
    panic!("{}", content);
}

fn setup() {
    env_logger::builder()
        .is_test(true)
        //disable all logs by defaykt
        .filter(None, log::LevelFilter::Off)
        //enable solana program logs
        .filter(
            Some("solana_runtime::message_processor::stable_log"),
            log::LevelFilter::Trace,
        )
        //enable our test log
        .filter_module("counter_program::test", log::LevelFilter::Trace)
        .init();
}

#[tokio::test]
async fn test_init() {
    setup();

    let program_id = Pubkey::new_unique();
    let (mut bank_clients, payer, recent_blockhash) =
        ProgramTest::new("counter_program", program_id, processor!(entrypoints))
            .start()
            .await;

    let counter_keypair = Keypair::new();
    let init_val = 0i64;

    let mut init_instruction_data: Vec<u8> = vec![CounterInstruction::InitCounter as u8];
    init_instruction_data.extend_from_slice(&init_val.to_le_bytes());

    let init_instruction = Instruction::new_with_bytes(
        program_id,
        &init_instruction_data,
        vec![
            AccountMeta::new(counter_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &counter_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair.pubkey()).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        let counter = CounterAccount::try_from_slice(&account_data.data)
            .expect("failed to deserialize counter data");
        assert_eq!(counter.count, init_val);
        info!("counter init successfully with value {}", counter.count);
    } else {
        panic_log("No counter account found".to_string());
    }

    info!("Testing counter increment...");

    let inc_instructionb = Instruction::new_with_bytes(
        program_id,
        &[CounterInstruction::IncCounter as u8],
        vec![AccountMeta::new(counter_keypair.pubkey(), true)],
    );

    let mut tx = Transaction::new_with_payer(&[inc_instructionb], Some(&payer.pubkey()));
    tx.sign(&[&payer, &counter_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair.pubkey()).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        let counter = CounterAccount::try_from_slice(&account_data.data)
            .expect("failed to deserialize counter data");
        assert_eq!(counter.count, init_val + 1);
        info!("counter incremented successfully to {}", counter.count);
    } else {
        panic_log(format!("No counter account found"));
    }

    info!("Testing counter decrement...");

    let dec_instruction = Instruction::new_with_bytes(
        program_id,
        &[CounterInstruction::DecCounter as u8],
        vec![AccountMeta::new(counter_keypair.pubkey(), true)],
    );

    let mut tx = Transaction::new_with_payer(&[dec_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &counter_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair.pubkey()).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        let counter = CounterAccount::try_from_slice(&account_data.data)
            .expect("failed to deserialize counter data");
        assert_eq!(counter.count, init_val);
        info!("counter decremented successfully to {}", counter.count);
    } else {
        panic_log(format!("No counter account found"));
    }
}
