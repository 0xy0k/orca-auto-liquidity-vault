use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    CalculationFailure,
    SlippageExceeded,
    ArithmeticOverflow
}