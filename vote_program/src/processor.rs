use crate::state::VoteAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke, program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction,
    system_program, sysvar::Sysvar,
};

pub fn process_create_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let vote_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let size_data = std::mem::size_of::<VoteAccount>();
    msg!("size data alloacted will be : {}", size_data);

    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(size_data);
    invoke(
        &system_instruction::create_account(
            payer_account.key,
            vote_account.key,
            required_lamports,
            size_data as u64,
            program_id,
        ),
        &[
            payer_account.clone(),
            vote_account.clone(),
            system_program.clone(),
        ],
    )?;

    let vote_data = VoteAccount {
        owner: *payer_account.key,
        end_date: -1,
        score: 0,
    };

    let mut account_data = &mut vote_account.data.borrow_mut()[..];
    vote_data.serialize(&mut account_data)?;
    msg!("vote init to {:?}", vote_data);
    Ok(())
}

pub fn process_close_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let source_account = next_account_info(accounts_iter)?;
    let dest_account = next_account_info(accounts_iter)?;

    if source_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let dest_lamports = dest_account.lamports();
    **dest_account.lamports.borrow_mut() = dest_lamports
        .checked_add(source_account.lamports())
        .ok_or(ProgramError::InvalidInstructionData)?;
    **source_account.lamports.borrow_mut() = 0;
    source_account.assign(&system_program::ID);
    source_account.realloc(0, false)?;
    Ok(())
}

pub fn process_remove_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vote_account = next_account_info(accounts_iter)?;

    if vote_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = vote_account.data.borrow_mut();
    let mut vote_data = VoteAccount::try_from_slice(&data)?;

    vote_data.score -= 1;
    vote_data.serialize(&mut &mut data[..])?;
    msg!("one vote removed");
    Ok(())
}

pub fn process_add_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("process_add_vote called");
    let accounts_iter = &mut accounts.iter();
    let vote_account = next_account_info(accounts_iter)?;

    if vote_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = vote_account.data.borrow_mut();
    let mut vote_data = VoteAccount::try_from_slice(&data)?;
    msg!("old score = {}", vote_data.score);

    vote_data.score += 1;
    msg!("new score = {}", vote_data.score);
    vote_data.serialize(&mut &mut data[..])?;
    msg!("one vote added, for a total: {}", vote_data.score);
    Ok(())
}
