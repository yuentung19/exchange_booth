use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ExchangeBoothInstruction {
    // 1. admin account [S]
    // 2. mint A
    // 3. mint B
    // 4. vault A [W] pda
    // 5. vault B [W] pda
    // 6. exchange booth [W] pda
    // 7. oracle
    // 8. token_program
    // 9. system_program
    InititializeExchangeBooth {
        // TODO
     },
    Deposit {
        // TODO
    },
    Withdraw {
        // TODO
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
}
