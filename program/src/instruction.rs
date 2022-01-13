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
    // 1. exchange booth
    // oracle (this is in EB, but you need the actual contents)
    // vault a [W] (this is in EB, but you need the actual contents)
    // vault b [W] (this is in EB, but you need the actual contents)
    // mint a (for decimal places)
    // mint b (for decimal places)
    // customer [S]
    // customer token program [W]
    // pda signer (not signable since to sign things, you need a private key, which doesnt exist for PDA)
    // token program
    Exchange {
        // TODO
    },
    CloseExchangeBooth {
        // TODO
    },
    UpdateOracleExchangeRate {
        exchange_rate_a_to_b: f64
    }
}
