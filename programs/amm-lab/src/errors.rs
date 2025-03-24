use anchor_lang::prelude::*;


#[error_code]
pub enum ErrorCode {
    #[msg("Invalid fee value")]
    InvalidFee,

    #[msg("Depositing too little liquidity")]
    DepositTooSmall,

    #[msg("Output is below the minimum expected")]
    OutputTooSmall,

    #[msg("Invariant does not hold")]
    InvariantViolated
} 
