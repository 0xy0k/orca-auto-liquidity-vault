use crate::errors::VaultError;

#[derive(Debug)]
pub struct TokenAmount(u128);

impl TokenAmount {
    pub fn new(amount: u64) -> Self {
        Self(amount as u128)
    }

    pub fn multiply(&self, other: u64) -> Result<Self, VaultError> {
        self.0
            .checked_mul(other as u128)
            .map(Self)
            .ok_or(VaultError::ArithmeticOverflow)
    }

    pub fn divide(&self, other: u64) -> Result<Self, VaultError> {
        self.0
            .checked_div(other as u128)
            .map(Self)
            .ok_or(VaultError::ArithmeticOverflow)
    }

    pub fn to_u64(&self) -> Result<u64, VaultError> {
        self.0
            .try_into()
            .map_err(|_| VaultError::ArithmeticOverflow)
    }
}

pub fn calculate_proportional_amount(
    amount: u64,
    numerator: u64,
    denominator: u64,
) -> Result<u64, VaultError> {
    TokenAmount::new(amount)
        .multiply(numerator)?
        .divide(denominator)?
        .to_u64()
}