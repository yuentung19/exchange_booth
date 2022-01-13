use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, msg, program_error::ProgramError,
    program::{invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar}
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
    let account_info_iter = &mut accounts.iter();
    // 1. admin account [S]
    // 2. vault A [W] pda
    // 3. vault B [W] pda
    // 4. exchange booth [W] pda
    // 5. oracle
    // 6. token_program
    // 7. system_program
    let exchange_booth = next_account_info(account_info_iter)?;
    let oracle = next_account_info(account_info_iter)?;
    let vault_a = next_account_info(account_info_iter)?;
    let vault_b = next_account_info(account_info_iter)?;
    let mint_a = next_account_info(account_info_iter)?;
    let mint_b = next_account_info(account_into_iter)?;
    let admin = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if !admin.is_signer {
        msg!("error: Admin must be signer")
        return Err(ExchangeBoothError::AccountMustBeSigner.into())
    }

    if !exchange_booth.is_writable {
        msg!("error: exchange booth not writable")
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    if !vault_a.is_writable {
        msg!("error: vault-a not writable")
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    if !vault_b.is_writable {
        msg!("error: vault-b not writable")
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }

    let (generated_vault_a_pda_key, bump_seed_a) = Pubkey::find_program_address(
        &[
            admin.key.as_ref(),
            exchange_booth.key.as_ref(),
            // exchange_booth.mint.as_ref(),  TODO: get mint a from ExchangeBooth once defined
        ],
        program_id,
    );

    if generated_vault_a_pda_key != *vault_a.key {
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    }

    let (generated_vault_b_pda_key, bump_seed_b) = Pubkey::find_program_address(
        &[
            admin.key.as_ref(),
            exchange_booth.key.as_ref(),
            // exchange_booth.mint.as_ref(),  TODO: get mint b from ExchangeBooth once defined
        ],
        program_id,
    );

    if generated_vault_b_pda_key != *vault_b.key {
        return Err(ExchangeBoothError::InvalidAccountAddress.into())
    }

    // TODO -- use token program invocations instead of system program for the vaults
    // Now we allocate buffer_size to the vault a pda.
    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            vault_a.key,
            Rent::get()?.minimum_balance(0),
            0 as u64,
            program_id,
        ),
        &[admin.clone(), vault_a.clone(), token_program.clone()],
        &[&[admin.key.as_ref(), exchange_booth.key.as_ref(), mint_a.key.as_ref(),  &[bump_seed_a]]],
    )?;

    // repeat for vault b
    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            vault_b.key,
            Rent::get()?.minimum_balance(0),
            0 as u64,
            program_id,
        ),
        &[admin.clone(), vault_a.clone(), system_program.clone()],
        &[&[admin.key.as_ref(), exchange_booth.key.as_ref(), token_program.key.as_ref(),  &[bump_seed_b]]],
    )?;

    //allocate vaults on the fly

    //what should the seeds for PDA be?
    //cant just use [mint, admin], since if another exchange booth is allocated, it will have the same PDA
    //adding the exchangebooth to the seed guarentees uniqueness. pros and cons to this

    //use [admin, oracle, A, B] to uniquly identify an exchange booth
    Ok(())
}
