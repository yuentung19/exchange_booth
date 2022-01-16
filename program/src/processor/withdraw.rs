use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, msg,
    pubkey::Pubkey,
    program_pack::{Pack},
    program::{invoke, invoke_signed},
};

use spl_token::state::{Account, Mint};

use crate::{
    error::ExchangeBoothError,
    state::ExchangeBooth,
};

use borsh::{BorshDeserialize, BorshSerialize};


pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: f64
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let exchange_booth = next_account_info(account_info_iter)?;
    let target_vault = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let admin_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    msg!("Amount to transfer/withdraw: {}", amount);
    if !admin_account.is_signer {
        msg!("error: Admin account must be signer!");
        return Err(ExchangeBoothError::AccountMustBeSigner.into())
    }
    if !user_token_account.is_writable {
        msg!("error: User token account is not writable!");
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    if !target_vault.is_writable {
        msg!("error: target vault not writable!");
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    let exchange_booth_data = &mut exchange_booth.try_borrow_data()?;
    let deserialized_eb = ExchangeBooth::try_from_slice(exchange_booth_data).unwrap();

    let (generated_vault_pda_key, bump_seed) = Pubkey::find_program_address(
        &[
            b"exchange_booth",
            admin_account.key.as_ref(),
            exchange_booth.key.as_ref(),
            mint_account.key.as_ref()
        ],
        program_id,
    );

    if *target_vault.key != deserialized_eb.vault_a && *target_vault.key != deserialized_eb.vault_b {
        msg!("Target vault is not in exchange booth!");
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    } 

    // Note: don't check bump seed here because 
    // 1. it is not stored in target_vault anywhere (see the Account struct or the code for the process_initialize_account instruction)
    // 2. transfer_checked won't let us transfer token X to an account that is initialized with mint !X
    if generated_vault_pda_key != *target_vault.key {
        msg!("Target vault PDA key mismatch, check your seeds!");
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    }

    let mint = Mint::unpack(&mint_account.try_borrow_data()?)?;
    let amount_small: u64 = (amount * f64::powf(10., mint.decimals.into())) as u64;
    msg!("amount small: {}", amount_small);
    invoke_signed(
        &spl_token::instruction::transfer_checked(
            &token_program.key,
            &target_vault.key,
            &mint_account.key,
            &user_token_account.key,
            &target_vault.key,
            &[target_vault.key],
            amount_small,
            mint.decimals
        )?,
        &[token_program.clone(), target_vault.clone(), mint_account.clone(), user_token_account.clone()],
        &[&[b"exchange_booth", admin_account.key.as_ref(), exchange_booth.key.as_ref(), mint_account.key.as_ref(), &[bump_seed]]]
    )?;

    // to verify this function, look at the target vault account on explorer immediately after depositing and immediately after 
    // withdrawing. You can also look at the user token account before withdrawing (the client prints this out).
    Ok(())
}
