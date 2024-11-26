use instructions::VoteInstruction;
use processor::*;
use solana_program::entrypoint;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub mod error;
pub mod instructions;
pub mod processor;
pub mod state;

entrypoint!(entrypoints);
pub fn entrypoints(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VoteInstruction::unpack(instruction_data)?;
    match instruction {
        VoteInstruction::CreateVote => process_create_vote(program_id, accounts),
        VoteInstruction::CloseVote => process_close_vote(program_id, accounts),
        VoteInstruction::AddVote => process_add_vote(program_id, accounts),
        VoteInstruction::RemoveVote => process_remove_vote(program_id, accounts),
    }
}

#[cfg(test)]
mod test;
