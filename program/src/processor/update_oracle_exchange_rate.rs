use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg
};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeRate,
};

use borsh::{BorshDeserialize, BorshSerialize};


pub fn process(
    accounts: &[AccountInfo],
    exchange_rate_a_to_b: f64
) -> ProgramResult {
    
    let account_info_iter = &mut accounts.iter();
    let oracle_account_info = next_account_info(account_info_iter)?;
    if !oracle_account_info.is_writable {
        msg!("Oracle is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let exchange_rate = ExchangeRate::try_from_slice(&oracle_account_info.data.borrow())
        .map_err(|_| ExchangeBoothError::InvalidAccountData)?;

    // exchange_rate.a_to_b = exchange_rate_a_to_b;
    // exchange_rate.b_to_a = 1.0 / exchange_rate_a_to_b;

    exchange_rate.serialize(&mut *oracle_account_info.data.borrow_mut())?;

    Ok(())
}
