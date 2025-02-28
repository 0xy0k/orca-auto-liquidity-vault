use orca_whirlpools_core::{tick_index_to_sqrt_price, try_get_amount_delta_a, try_get_amount_delta_b};

use crate::errors::VaultError;

pub fn calculate_shares(amount_a: u64, amount_b: u64, total_token_a_amount: u64, total_token_b_amount: u64, price_a: u64, price_b: u64, total_shares: u64, decimal: u8) -> u64 {
    let amount_a_u128 = amount_a as u128;
    let amount_b_u128 = amount_b as u128;
    let total_token_a_amount_u128 = total_token_a_amount as u128;
    let total_token_b_amount_u128 = total_token_b_amount as u128;
    let price_a_u128 = price_a as u128;
    let price_b_u128 = price_b as u128;
    let total_shares_u128 = total_shares as u128;
    let share_amount =     
        if total_shares == 0 {
            (amount_a_u128.checked_mul(price_a_u128).unwrap())
            .checked_add(amount_b_u128.checked_mul(price_b_u128).unwrap())
            .unwrap()
        } else {
            let deposit_value = (amount_a_u128.checked_mul(price_a_u128).unwrap())
                .checked_add(amount_b_u128.checked_mul(price_b_u128).unwrap())
                .unwrap();

            let total_value = (total_token_a_amount_u128.checked_mul(price_a_u128).unwrap())
                .checked_add(total_token_b_amount_u128.checked_mul(price_b_u128).unwrap())
                .unwrap();

            deposit_value.checked_div(total_value).unwrap().checked_mul(total_shares_u128).unwrap()
        };
    
    // Divide by 10^decimal
    let divisor = 10u128.pow(decimal as u32);
    share_amount.checked_div(divisor).unwrap() as u64
}

pub fn calculate_total_tokens(
    vault_token_a_amount: u64,
    vault_token_b_amount: u64,
    position_liquidity: u128,
    current_tick_index: i32,
    sqrt_price: u128,
    tick_lower_index: i32,
    tick_upper_index: i32,
) -> Result<(u64, u64), VaultError> {
    // Calculate tokens in position using Whirlpool math
    let sqrt_price_lower = tick_index_to_sqrt_price(tick_lower_index);
    let sqrt_price_upper = tick_index_to_sqrt_price(tick_upper_index);

    let mut position_token_a_amount: u64 = 0;
    let mut position_token_b_amount: u64 = 0;

    if current_tick_index < tick_lower_index {
        // current tick below position
        position_token_a_amount = try_get_amount_delta_a(sqrt_price_lower, sqrt_price_upper, position_liquidity, true).map_err(|_| VaultError::CalculationFailure)?;
    } else if current_tick_index < tick_upper_index {
        // current tick inside position
        position_token_a_amount = try_get_amount_delta_a(sqrt_price, sqrt_price_upper, position_liquidity, true).map_err(|_| VaultError::CalculationFailure)?;
        position_token_b_amount = try_get_amount_delta_b(sqrt_price_lower, sqrt_price, position_liquidity, true).map_err(|_| VaultError::CalculationFailure)?;
    } else {
        // current tick above position
        position_token_b_amount = try_get_amount_delta_b(sqrt_price_lower, sqrt_price_upper, position_liquidity, true).map_err(|_| VaultError::CalculationFailure)?;
    }

    // Add vault tokens and position tokens
    let total_token_a_amount = vault_token_a_amount
        .checked_add(position_token_a_amount)
        .ok_or(VaultError::CalculationFailure)?;
    
    let total_token_b_amount = vault_token_b_amount
        .checked_add(position_token_b_amount)
        .ok_or(VaultError::CalculationFailure)?;

    Ok((total_token_a_amount, total_token_b_amount))
}