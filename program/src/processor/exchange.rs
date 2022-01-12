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


    //a_in d_in
    //price decimals
    //a_out d_out

    //Gotchas:
    //1. numerical overflow
    //2. rounding

    //oracle should provide price info (price decimals)
    Ok(())
}