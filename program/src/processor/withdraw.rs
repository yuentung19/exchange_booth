use solana_program::{
    account_info::{AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey
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
    let exchange_booth = next_account_info(account_info_iter)?;
    let target_vault = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;
    let user_token_account = next_account_info(account_info_iter)?;
    let admin_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    if !admin_account.is_signer {
        msg!("error: Admin account must be signer!");
        return Err(ExchangeBoothError::AccountMustBeSigner.into())
    }
    if !exchange_booth.is_writable {
        msg!("error: exchange booth not writable");
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    if !target_vault.is_writable {
        msg!("error: target vault not writable");
        return Err(ExchangeBoothError::AccountMustBeWritable.into())
    }
    let exchange_booth_struct = ExchangeBooth {
        admin: *admin.key,
        oracle: *oracle.key,
        vault_a: *vault_a.key,
        vault_b: *vault_b.key
    };
    //let exchange_booth_data = &mut exchange_booth.data.borrow_mut();
    //ExchangeBooth::try_from_slice(exchange_booth_data).unwrap();


    Ok(())
}
