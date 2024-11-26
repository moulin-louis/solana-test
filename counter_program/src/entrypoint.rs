use instructions::{CounterInstruction, Unpack};
use processor::{process_change_counter, process_initialize_counter};
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
    let instruction = CounterInstruction::unpack(instruction_data)?;
    match instruction {
        CounterInstruction::InitCounter => process_initialize_counter(program_id, accounts, 0),
        CounterInstruction::IncCounter => process_change_counter(program_id, accounts, true),
        CounterInstruction::DecCounter => process_change_counter(program_id, accounts, false),
    }
}

#[cfg(test)]
mod test;
