use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::mem::size_of;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ExchangeRate {
    pub a_to_b: f64,
    pub b_to_a: f64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ExchangeBooth {
    pub admin: Pubkey,
    pub oracle: Pubkey,
    pub vault_a: Pubkey, 
    pub vault_b: Pubkey, 
}
