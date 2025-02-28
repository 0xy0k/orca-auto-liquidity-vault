use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub admin: Pubkey,
    pub whirlpool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub share_mint: Pubkey,
    pub token_a_price: u64,
    pub token_b_price: u64,
    pub token_a_decimal: u8,
    pub token_b_decimal: u8,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 1 + 1 + 4 + 4 + 1;
}