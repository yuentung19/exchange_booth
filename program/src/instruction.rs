use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ExchangeBoothInstruction {
    /// Accounts:
    /// | index | writable | signer | description                                                                                         |
    /// |-------|----------|--------|-----------------------------------------------------------------------------------------------------|
    /// | 0     | ❌       | ❌     | exchange_booth: contains the ExchangeBooth struct in the data
    /// | 1     | ❌       | ❌     | oracle: contains the ExchangeRate struct in the data
    /// | 2     | ✅       | ❌     | vault_A: account that the ExchangeBooth::vault_a address points to (PDA, this is to debit/credit)
    /// | 3     | ✅       | ❌     | vault_B: account that the ExchangeBooth::vault_b address points to (PDA, this is to debit/credit)
    /// | 4     | ❌       | ❌     | mint_A: mint address of token A (required to get decimal places)
    /// | 5     | ❌       | ❌     | mint_B: mint address of token B (required to get decimal places)
    /// | 6     | ❌       | ✅     | admin account
    /// | 7     | ❌       | ❌     | system_program
    /// | 8     | ❌       | ❌     | token_program
    InititializeExchangeBooth {
        // no instruction data on purpose
     },
    Deposit {
        // TODO
    },
    /// Accounts:
    /// | index | writable | signer | description                                                                                         |
    /// |-------|----------|--------|-----------------------------------------------------------------------------------------------------|
    /// | 0     | ❌       | ❌     | exchange_booth: contains the ExchangeBooth struct in the data
    /// | 1     | ✅       | ❌     | target_vault: one of the booth vault addresses from which we withdraw amount 
    /// | 2     | ❌       | ❌     | mint: mint address of withdraw token
    /// | 3     | ✅       | ❌     | user_token_account: writable deposit address
    /// | 4     | ❌       | ✅     | admin account
    /// | 5     | ❌       | ❌     | token_program
    Withdraw {
        amount: f64
    },
    /// Accounts:
    /// | index | writable | signer | description                                                                                         |
    /// |-------|----------|--------|-----------------------------------------------------------------------------------------------------|
    /// | 0     | ❌       | ❌     | exchange_booth: contains the ExchangeBooth struct in the data
    /// | 1     | ❌       | ❌     | oracle: contains the ExchangeRate struct in the data
    /// | 2     | ✅       | ❌     | vault_A: account that the ExchangeBooth::vault_a address points to (PDA, this is to debit/credit)
    /// | 3     | ✅       | ❌     | vault_B: account that the ExchangeBooth::vault_b address points to (PDA, this is to debit/credit)
    /// | 4     | ❌       | ❌     | mint_A: mint address of token A (required to get decimal places)
    /// | 5     | ❌       | ❌     | mint_B: mint address of token B (required to get decimal places)
    /// | 6     | ❌       | ✅     | customer: needed to be the signer of the transaction to debit customer token account
    /// | 7     | ✅       | ❌     | customer_from_token_account: the token account that the exchange program will DEBIT
    /// | 8     | ✅       | ❌     | customer_to_token_account: the token account that the exchange program will CREDIT
    /// | 9     | ❌       | ❌     | system_program
    /// | 10    | ❌       | ❌     | token_program
    Exchange {
        amount: f64,
    },
    CloseExchangeBooth {
        // TODO
    },
    /// Accounts:
    /// | index | writable | signer | description                                                                                         |
    /// |-------|----------|--------|-----------------------------------------------------------------------------------------------------|
    /// | 0     | ✅       | ❌     | oracle: contains the ExchangeRate struct in the data
    UpdateOracleExchangeRate {
        exchange_rate_a_to_b: f64
    }
}
