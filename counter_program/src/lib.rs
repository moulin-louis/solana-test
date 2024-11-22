use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint,
    entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, sysvar::Sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    count: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum CounterInstruction {
    InitCounter { init_value: i64 } = 0,
    IncCounter = 1,
    DecCounter = 2,
}
impl Into<u8> for CounterInstruction {
    fn into(self) -> u8 {
        match self {
            Self::InitCounter { init_value: _ } => 0,
            Self::IncCounter => 1,
            Self::DecCounter => 2,
        }
    }
}

impl CounterInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        BorshDeserialize::deserialize(&mut &input[..])
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_value: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_accounmt = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;

    let required_lamports = rent.minimum_balance(8);
    invoke(
        &system_instruction::create_account(
            payer_account.key,    //account paying for the new account
            counter_accounmt.key, //account to be created
            required_lamports,    // amount of lamport given to the new account
            8u64,                 //size in bytes to allocate for the data field
            program_id,           //program owner to be out program
        ),
        &[
            payer_account.clone(),
            counter_accounmt.clone(),
            system_program.clone(),
        ],
    )?;

    let counter_data = CounterAccount {
        count: initial_value,
    };

    let mut account_data = &mut counter_accounmt.data.borrow_mut()[..];
    counter_data.serialize(&mut account_data)?;
    msg!("counter init to {}", initial_value);
    Ok(())
}

fn process_change_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    direction: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = counter_account.data.borrow_mut();
    let mut counter_data: CounterAccount = CounterAccount::try_from_slice(&data)?;

    counter_data.count = counter_data
        .count
        .checked_add(direction)
        .ok_or(ProgramError::InvalidAccountData)?;
    counter_data.serialize(&mut &mut data[..])?;
    msg!("counter incremented to: {}", counter_data.count);
    Ok(())
}

entrypoint!(main);
pub fn main(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CounterInstruction::unpack(instruction_data)?;
    match instruction {
        CounterInstruction::InitCounter { init_value } => {
            process_initialize_counter(program_id, accounts, init_value)
        }
        CounterInstruction::IncCounter => process_change_counter(program_id, accounts, 1),
        CounterInstruction::DecCounter => process_change_counter(program_id, accounts, -1),
    }
}

#[cfg(test)]
mod test {
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
            ProgramTest::new("counter_program", program_id, processor!(main))
                .start()
                .await;

        let counter_keypair = Keypair::new();
        let init_val = 42i64;

        let mut init_instruction_data: Vec<u8> =
            vec![CounterInstruction::InitCounter { init_value: 0 }.into()];
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
            &[CounterInstruction::IncCounter.into()],
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
            &[CounterInstruction::DecCounter.into()],
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
            info!("counter incremented successfully to {}", counter.count);
        } else {
            panic_log(format!("No counter account found"));
        }
    }
}
