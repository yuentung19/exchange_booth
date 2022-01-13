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
    state::ExchangeRate
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::state::Account as TokenAccount;


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
    let mint_a = next_account_info(account_info_iter)?;
    let mint_b = next_account_info(account_info_iter)?;
    let customer = next_account_info(account_info_iter)?;
    let customer_from_token_acc = next_account_info(account_info_iter)?;
    let customer_to_token_acc = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    //pulling out data
    let exchange_booth = ExchangeBooth::try_from_slice(&exchange_booth_acc.data.borrow())?;
    let exchange_rate = ExchangeRate::try_from_slice(&oracle.data.borrow())?;
    let vault_a_token_account = TokenAccount::unpack_from_slice(&vault_a.try_borrow_data()?)?;
    let vault_b_token_account = TokenAccount::unpack_from_slice(&vault_b.try_borrow_data()?)?;
    let customer_from_token_account = TokenAccount::unpack_from_slice(&customer_from_token_acc.try_borrow_data()?)?;
    let customer_to_token_account = TokenAccount::unpack_from_slice(&customer_to_token_acc.try_borrow_data()?)?;

    //checking writable/signable
    if !vault_a.is_writable {
        msg!("Vault_A is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !vault_b.is_writable {
        msg!("Vault_B is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer.is_signer {
        msg!("Customer is not set to is_signable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer_from_token_acc.is_writable {
        msg!("Customer_From_Account is not set to is_writable");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !customer_to_token_acc.is_writable {
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
    if customer_from_token_account.mint == customer_to_token_account.mint {
        msg!("Customer_From_Mint is equal to Customer_To_Mint");
        return Err(ProgramError::InvalidArgument);
    }
    if [*mint_a.key, *mint_b.key].contains(&customer_from_token_account.mint) {
        msg!("Customer_From_Mint is not one of mint A or mint B");
        return Err(ProgramError::InvalidArgument);
    }
    if [*mint_a.key, *mint_b.key].contains(&customer_to_token_account.mint) {
        msg!("Customer_To_Mint is not one of mint A or mint B");
        return Err(ProgramError::InvalidArgument);
    }


    //check vaults in Exchange Booth are the vaults passed in to the Accounts
    if exchange_booth.vault_a != *vault_a.key {
        msg!("ExchangeBooth vault A pubkey not equal to vault A pub key");
        return Err(ProgramError::InvalidArgument);
    }
    if exchange_booth.vault_b != *vault_b.key {
        msg!("ExchangeBooth vault B pubkey not equal to vault B pub key");
        return Err(ProgramError::InvalidArgument);
    }

    //figure out the direction
    let mut exchange_from_a: bool = false;
    if customer_from_token_account.mint == *mint_a.key {
        exchange_from_a = true;
    }


    //a_in d_in
    //price decimals
    //a_out d_out

    //Gotchas:
    //1. numerical overflow
    //2. rounding

    //oracle should provide price info (price decimals)
    Ok(())
}