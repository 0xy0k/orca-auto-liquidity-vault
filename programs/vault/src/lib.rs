use anchor_lang::prelude::*;
pub mod instructions;
pub mod state;
pub mod util;
pub mod errors;

use instructions::*;
use util::*;

declare_id!("3A6RUfKQe3NDKYX9aFyBS21juUnbtkZi3djaoHmgXTef");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        token_a_price: u64,
        token_b_price: u64,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<()> {
        instructions::initialize_vault(ctx, token_a_price, token_b_price, lower_tick, upper_tick)
    }

    pub fn update_prices(
        ctx: Context<UpdatePrices>,
        token_a_price: u64,
        token_b_price: u64,
    ) -> Result<()> {
        instructions::update_prices(ctx, token_a_price, token_b_price)
    }

    pub fn update_ticks(
        ctx: Context<UpdateTicks>,
        lower_tick: i32,
        upper_tick: i32,
    ) -> Result<()> {
        instructions::update_ticks(ctx, lower_tick, upper_tick)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
    ) -> Result<()> {
        instructions::open_position(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        liquidity_amount: u128,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        instructions::add_liquidity(ctx, liquidity_amount, token_a_amount, token_b_amount)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
    ) -> Result<()> {
        instructions::remove_liquidity(ctx)
    }

    pub fn collect_fees(
        ctx: Context<CollectFees>,
    ) -> Result<()> {
        instructions::collect_fees(ctx)
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
    ) -> Result<()> {
        instructions::close_position(ctx)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        instructions::deposit(ctx, amount_a, amount_b)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        share_amount: u64,
        token_min_a_amount: u64,
        token_min_b_amount: u64,
    ) -> Result<()> {
        instructions::withdraw(ctx, share_amount, token_min_a_amount, token_min_b_amount)
    }
}