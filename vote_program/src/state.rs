use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VoteAccount {
    /// NOTE:: Owner of the Vote, only him can close the Vote
    /// NOTE:: 32 (1 * 32) bytes
    pub owner: Pubkey,
    /// NOTE:: Represent the end timestamp
    /// NOTE:: 8 bytes
    pub end_date: i64,
    ///NOTE:: Reprensent the score of the Vote
    pub score: i64,
}
