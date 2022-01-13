use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack
};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::state::Account as TokenAccount;
use crate::state::ExchangeRate;
use crate::state::ExchangeBooth;


pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: f64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let exchange_booth_acc = next_account_info(account_info_iter)?;
    let oracle = next_account_info(account_info_iter)?;
    let vault_a = next_account_info(account_info_iter)?;
    let vault_b = next_account_info(account_info_iter)?;
    let vault_b = next_account_info(account_info_iter)?;
    let mint_a = next_account_info(account_info_iter)?;
    let mint_b = next_account_info(account_info_iter)?;
    let customer = next_account_info(account_info_iter)?;
    let customer_from_token_account = next_account_info(account_info_iter)?;
    let customer_to_token_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    //pulling out data
    let exchange_booth = 
    let vault_a_token_account = TokenAccount::unpack_from_slice(&vault_a.try_borrow_data()?)?;
    let vault_b_token_account = TokenAccount::unpack_from_slice(&vault_b.try_borrow_data()?)?;

    //checking writable/signable
    if !vault_a.is_writable {
        msg!("Vault_A is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !vault_b.is_writable {
        msg!("Vault_B is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer_from_token_account.is_signer {
        msg!("Customer is not set to is_signable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer_from_token_account.is_writable {
        msg!("Customer_From_Account is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer_to_token_account.is_writable {
        msg!("Customer_To_Account is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }

    //checking the mint accounts
    if vault_a_token_account.mint != *mint_a.key {
        msg!("Vault A mint address is not equal to mint A");
        return Err(ProgramError::InvalidArgument);
    }
    if vault_b_token_account.mint != *mint_b.key {
        msg!("Vault B mint address is not equal to mint B");
        return Err(ProgramError::InvalidArgument);
    }

    //check vaults in Exchange Booth are the vaults passed in to the Accounts

    //a_in d_in
    //price decimals
    //a_out d_out

    //Gotchas:
    //1. numerical overflow
    //2. rounding

    //oracle should provide price info (price decimals)
    Ok(())
}