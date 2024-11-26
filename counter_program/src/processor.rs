use crate::state::CounterAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction,
    sysvar::Sysvar,
};

pub fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_value: i64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let size_data = i64::BITS / 8;

    let rent = Rent::get()?;

    let required_lamports = rent.minimum_balance(size_data as usize);
    invoke(
        &system_instruction::create_account(
            payer_account.key,   //account paying for the new account
            counter_account.key, //account to be created
            required_lamports,   // amount of lamport given to the new account
            size_data.into(),    //size in bytes to allocate for the data field
            program_id,          //program owner is set to our program
        ),
        &[
            payer_account.clone(),
            counter_account.clone(),
            system_program.clone(),
        ],
    )?;

    let counter_data = CounterAccount {
        count: initial_value,
    };

    let mut account_data = &mut counter_account.data.borrow_mut()[..];
    counter_data.serialize(&mut account_data)?;
    msg!("counter init to {}", initial_value);
    Ok(())
}

pub fn process_change_counter(
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
