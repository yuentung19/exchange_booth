use solana_program::{
    account_info::{AccountInfo, next_account_info}, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    program::{invoke_signed, invoke},
};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
    state::ExchangeRate
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint as Mint;


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
    let mint_a_acc = next_account_info(account_info_iter)?;
    let mint_b_acc = next_account_info(account_info_iter)?;
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
    let mint_a = Mint::unpack_from_slice(&mint_a_acc.try_borrow_data()?)?;
    let mint_b = Mint::unpack_from_slice(&mint_b_acc.try_borrow_data()?)?;
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
    if vault_a_token_account.mint != *mint_a_acc.key {
        msg!("Vault A mint address is not equal to mint A");
        return Err(ProgramError::InvalidArgument);
    }
    if vault_b_token_account.mint != *mint_b_acc.key {
        msg!("Vault B mint address is not equal to mint B");
        return Err(ProgramError::InvalidArgument);
    }
    if customer_from_token_account.mint == customer_to_token_account.mint {
        msg!("Customer_From_Mint is equal to Customer_To_Mint");
        return Err(ProgramError::InvalidArgument);
    }
    /*
    if [mint_a_acc.key, mint_b_acc.key].contains(&&customer_from_token_account.mint) {
        msg!("Customer_From_Mint is not one of mint A or mint B");
        return Err(ProgramError::InvalidArgument);
    }
    if [mint_a_acc.key, mint_b_acc.key].contains(&&customer_to_token_account.mint) {
        msg!("Customer_To_Mint is not one of mint A or mint B");
        return Err(ProgramError::InvalidArgument);
    }
    */


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
    let mut rate = exchange_rate.b_to_a;
    let mut from_decimal = mint_b.decimals;
    let mut from_token = "B";
    let mut to_decimal = mint_a.decimals;
    let mut to_token = "A";

    if customer_from_token_account.mint == *mint_a_acc.key {
        exchange_from_a = true;
        rate = exchange_rate.a_to_b;
        from_decimal = mint_a.decimals;
        from_token = "A";
        to_decimal = mint_b.decimals;
        to_token = "B";
    }

    let result = amount * rate;
    let amount_small: u64 = (amount * f64::powf(10., from_decimal.into())) as u64;
    let result_small: u64 = (result * f64::powf(10., to_decimal.into())) as u64;
    msg!("Customer is exchanging {} ({}) token {} for {} ({}) token {} with exchange rate {}",
        amount,
        amount_small,
        from_token,
        result,
        result_small,
        to_token,
        rate
    );

    //debit customers FROM TOKEN account, credit the corresponding vault
    let transfer_from_customer_to_vault_ix = spl_token::instruction::transfer(
        token_program.key,
        customer_from_token_acc.key,
        if exchange_from_a {&exchange_booth.vault_a} else {&exchange_booth.vault_b},
        &customer.key,
        &[&customer.key],
        amount_small
    )?;
    msg!("Transfering token {}", from_token);
    invoke(
        &transfer_from_customer_to_vault_ix,
        &[
            customer.clone(),
            customer_from_token_acc.clone(),
            if exchange_from_a {vault_a.clone()} else{vault_b.clone()},
            token_program.clone(),
        ]
    )?;
   
    
    //generate PDA for the bump seed
    let (_, bump_seed) = Pubkey::find_program_address(
        &[
            b"exchange_booth",
            exchange_booth.admin.as_ref(),
            exchange_booth_acc.key.as_ref(),
            if exchange_from_a {mint_b_acc.key.as_ref()} else{mint_a_acc.key.as_ref()},
        ],
        program_id,
    );
    //debit other vault, and credit the customer's TO TOKEN account
    let transfer_vault_to_to_customer_ix = spl_token::instruction::transfer(
        token_program.key,
        if exchange_from_a {&exchange_booth.vault_b} else {&exchange_booth.vault_a},
        customer_to_token_acc.key,
        if exchange_from_a {&exchange_booth.vault_b} else {&exchange_booth.vault_a},
        &[if exchange_from_a {&exchange_booth.vault_b} else {&exchange_booth.vault_a}],
        result_small
    )?;
    msg!("Transfering token {}", to_token);
    invoke_signed(
        &transfer_vault_to_to_customer_ix,
        &[
            customer.clone(),
            customer_to_token_acc.clone(),
            if exchange_from_a {vault_b.clone()} else{vault_a.clone()},
            token_program.clone(),
        ],
        &[&[
            b"exchange_booth",
            exchange_booth.admin.as_ref(),
            exchange_booth_acc.key.as_ref(),
            if exchange_from_a {mint_b_acc.key.as_ref()} else{mint_a_acc.key.as_ref()},
            &[bump_seed]
        ]],
    )?;


    //spl_token::instruction::initialize_account(token_program_id: &Pubkey, account_pubkey: &Pubkey, mint_pubkey: &Pubkey, owner_pubkey: &Pubkey)
    Ok(())
}