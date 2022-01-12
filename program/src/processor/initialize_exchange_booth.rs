use solana_program::{
};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};


pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    // ???
) -> ProgramResult {
    //allocate vaults on the fly

    //what should the seeds for PDA be?
    //cant just use [mint, admin], since if another exchange booth is allocated, it will have the same PDA
    //adding the exchangebooth to the seed guarentees uniqueness. pros and cons to this

    //use [admin, oracle, A, B] to uniquly identify an exchange booth
    Ok(())
}
