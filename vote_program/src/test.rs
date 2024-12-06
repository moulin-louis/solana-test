use borsh::BorshDeserialize;
use env_logger;
use log::info;
use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel,
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use state::VoteAccount;

use super::*;

fn setup() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter(None, log::LevelFilter::Off)
        .filter(
            Some("solana_runtime::message_processor::stable_log"),
            log::LevelFilter::Trace,
        )
        .filter_module("vote_program::test", log::LevelFilter::Trace)
        .try_init();
}

#[tokio::test]
async fn test_vote_lifecycle() {
    setup();

    let program_id = Pubkey::new_unique();
    let (mut bank_clients, payer, recent_blockhash) =
        ProgramTest::new("vote_program", program_id, processor!(entrypoints))
            .start()
            .await;

    // Create a new vote account
    let vote_keypair = Keypair::new();
    info!("Testing vote account creation...");

    let init_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::CreateVote as u8],
        vec![
            AccountMeta::new(vote_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &vote_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    // Verify initial state
    let account = bank_clients
        .get_account(vote_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let vote = VoteAccount::try_from_slice(&account.data).unwrap();
    assert_eq!(vote.owner, payer.pubkey());
    assert_eq!(vote.score, 0);
    info!("Vote account created successfully: {:?}", vote);

    // Test adding votes
    info!("Testing vote addition...");
    let add_vote_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::AddVote as u8],
        vec![AccountMeta::new(vote_keypair.pubkey(), true)],
    );

    let mut tx = Transaction::new_with_payer(&[add_vote_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &vote_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    // Verify vote was added
    let account = bank_clients
        .get_account(vote_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let vote = VoteAccount::try_from_slice(&account.data).unwrap();
    assert_eq!(vote.score, 1);
    info!("Vote added successfully, new score: {}", vote.score);

    // Test removing votes
    info!("Testing vote removal...");
    let remove_vote_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::RemoveVote as u8],
        vec![AccountMeta::new(vote_keypair.pubkey(), true)],
    );

    let mut tx = Transaction::new_with_payer(&[remove_vote_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &vote_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    // Verify vote was removed
    let account = bank_clients
        .get_account(vote_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();
    let vote = VoteAccount::try_from_slice(&account.data).unwrap();
    assert_eq!(vote.score, 0);
    info!("Vote removed successfully, new score: {}", vote.score);

    // Test closing vote account
    info!("Testing vote account closure...");
    let close_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::CloseVote as u8],
        vec![
            AccountMeta::new(vote_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[close_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &vote_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    // Verify account was closed
    let account = bank_clients
        .get_account(vote_keypair.pubkey())
        .await
        .unwrap();
    assert!(account.is_none(), "Vote account should be closed");
    info!("Vote account closed successfully");
}

#[tokio::test]
async fn test_multiple_votes() {
    setup();

    let program_id = Pubkey::new_unique();
    let (mut bank_clients, payer, recent_blockhash) =
        ProgramTest::new("vote_program", program_id, processor!(entrypoints))
            .start()
            .await;

    let vote_keypair = Keypair::new();

    // Create vote account
    let init_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::CreateVote as u8],
        vec![
            AccountMeta::new(vote_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );
    {
        let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
        tx.sign(&[&payer, &vote_keypair], recent_blockhash);
        bank_clients
            .process_transaction_with_commitment(tx, CommitmentLevel::Finalized)
            .await
            .unwrap();
        info!("vote account created");
    }

    // Add multiple votes
    for i in 0..5 {
        info!("Trying to add vote, i  = {}", i);
        let add_vote_instruction = Instruction::new_with_bytes(
            program_id,
            &[VoteInstruction::AddVote as u8],
            vec![AccountMeta::new(vote_keypair.pubkey(), true)],
        );
        info!("instruction created");

        let mut tx = Transaction::new_with_payer(&[add_vote_instruction], Some(&payer.pubkey()));
        info!("tx created");
        tx.sign(&[&payer, &vote_keypair], recent_blockhash);
        info!("tx signed");
        bank_clients.process_transaction(tx).await.unwrap();
        info!("tx processed");

        let account = bank_clients
            .get_account(vote_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
        let vote = VoteAccount::try_from_slice(&account.data).unwrap();
        assert_eq!(vote.score, i + 1);
        info!("Added vote #{}, total score: {}", i + 1, vote.score);
    }

    // Remove some votes
    for i in 0..3 {
        info!("trying to remove vote");
        let remove_vote_instruction = Instruction::new_with_bytes(
            program_id,
            &[VoteInstruction::RemoveVote as u8],
            vec![AccountMeta::new(vote_keypair.pubkey(), true)],
        );

        let mut tx = Transaction::new_with_payer(&[remove_vote_instruction], Some(&payer.pubkey()));
        tx.sign(&[&payer, &vote_keypair], recent_blockhash);
        bank_clients.process_transaction(tx).await.unwrap();

        let account = bank_clients
            .get_account(vote_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
        let vote = VoteAccount::try_from_slice(&account.data).unwrap();
        assert_eq!(vote.score, 4 - i);
        info!("Removed vote #{}, total score: {}", i + 1, vote.score);
    }

    info!("Closing vote account...");
    let close_instruction = Instruction::new_with_bytes(
        program_id,
        &[VoteInstruction::CloseVote as u8],
        vec![
            AccountMeta::new(vote_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[close_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer, &vote_keypair], recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    // Verify account was closed
    let account = bank_clients
        .get_account(vote_keypair.pubkey())
        .await
        .unwrap();
    assert!(account.is_none(), "Vote account should be closed");
    info!("Vote account closed successfully");
}
