use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use vote_program::instructions::VoteInstruction;

#[tokio::main]
async fn main() {
    let keypair = read_keypair_file("./target/deploy/counter_program-keypair.json").unwrap();
    let program_pubkey = keypair.pubkey();

    // Connect to the Solana devnet
    let rpc_url = String::from("http://127.0.0.1:8899");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // Generate a new keypair for the payer
    let payer = Keypair::new();
    let counter_keypair = Keypair::new();

    // Request airdrop
    let airdrop_amount = 1_000_000_000; // 1 SOL
    let signature = client
        .request_airdrop(&payer.pubkey(), airdrop_amount)
        .expect("Failed to request airdrop");

    // Wait for airdrop confirmation
    loop {
        let confirmed = client.confirm_transaction(&signature).unwrap();
        if confirmed {
            break;
        }
    }
    let tx_init = VoteInstruction::CreateVote;
    // Create the instruction
    let ix = Instruction::new_with_borsh(
        program_pubkey,
        &tx_init,
        vec![
            AccountMeta::new(counter_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(system_program::id(), false),
        ],
    );

    // Add the instruction to new transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(
        &[&payer, &counter_keypair],
        client.get_latest_blockhash().unwrap(),
    );

    // Send and confirm the transaction
    match client.send_and_confirm_transaction(&tx) {
        Ok(signature) => println!("Success Init Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }
}
