use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum VoteInstruction {
    CreateVote = 0,
    CloseVote = 1,
    AddVote = 2,
    RemoveVote = 3,
}

impl VoteInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        BorshDeserialize::deserialize(&mut &input[..])
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}
