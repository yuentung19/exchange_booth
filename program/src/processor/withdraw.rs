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
    amount: u64
) -> ProgramResult {
    msg!("hello");
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
    let exchange_booth_data = &mut exchange_booth.data.borrow_mut();
    let target_vault_data = &mut target_vault.data.borrow_mut();
    let deserialized_eb = ExchangeBooth::try_from_slice(exchange_booth_data).unwrap();
    msg!("Passed deserialize of ExchangeBooth!");
    msg!("bump seed: {}", target_vault_data[0]);
    msg!("admin acc: {}", admin_account.key);
    msg!("exchange acc: {}", exchange_booth.key);
    msg!("mint acc: {}", mint_account.key);
    let (generated_vault_pda_key, bump_seed) = Pubkey::find_program_address(
        &[
            b"exchange_booth",
            admin_account.key.as_ref(),
            exchange_booth.key.as_ref(),
            mint_account.key.as_ref()
        ],
        program_id,
    );
    msg!("generated bump seed: {}", bump_seed);
    if *target_vault.key != deserialized_eb.vault_a && *target_vault.key != deserialized_eb.vault_b {
        msg!("Target vault is not in exchange booth!");
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    } 
    msg!("generated vault pda: {}", generated_vault_pda_key);
    msg!("target_vault_key: {}", *target_vault.key);
    if generated_vault_pda_key != *target_vault.key || bump_seed != target_vault_data[0] {
        msg!("Target vault PDA key mismatch, check your seeds!");
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    }
    msg!("Passed PDA checks");
    // TODO: Does withdraw fail if amount is not enough? 
    // invoke_signed(
    //     &spl_token::instruction::transfer(
    //         &token_program.key,
    //         &target_vault.key,
    //         &user_token_account.key,
    //         &target_vault.key,
    //         &[],
    //         amount
    //     ),
    //     &[token_program.clone(), target_vault.clone(), user_token_account.clone(), admin_account.clone()],
    //     &[&[b"exchange_booth", admin_account.key.as_ref(), exchange_booth.key.as_ref(), mint.key.as_ref()]]
    // )?;
    let mint = Mint::unpack(&mint_account.data.borrow_mut())?;
    invoke_signed(
        &spl_token::instruction::transfer_checked(
            &token_program.key,
            &target_vault.key,
            &mint_account.key,
            &user_token_account.key,
            &target_vault.key,
            &[],
            amount,
            mint.decimals
        )?,
        &[token_program.clone(), target_vault.clone(), user_token_account.clone(), admin_account.clone(), mint_account.clone()],
        &[&[b"exchange_booth", admin_account.key.as_ref(), exchange_booth.key.as_ref(), mint_account.key.as_ref()]]
    )?;
    msg!("Passed CPI transfer call!");
    Ok(())
}
