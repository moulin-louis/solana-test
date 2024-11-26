use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VoteAccount {
    /// NOTE:: Owner of the Vote, only him can close the Vote
    owner: Pubkey,
    /// NOTE:: Represent the end timestamp
    end_date: i64,
    /// NOTE:: Vectors with all voters pubkey
    /// NOTE:: Max length is 10 for now
    /// TODO:: Create PDA derive from these Pubkey to store information about their vote
    voters: Vec<Pubkey>,
}
