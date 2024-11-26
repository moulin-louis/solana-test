use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

pub trait Unpack {
    fn unpack(input: &[u8]) -> Result<Self, ProgramError>
    where
        Self: Sized;
}

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum CounterInstruction {
    InitCounter = 0,
    IncCounter = 1,
    DecCounter = 2,
}

impl Unpack for CounterInstruction {
    fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        BorshDeserialize::deserialize(&mut &input[..])
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}
