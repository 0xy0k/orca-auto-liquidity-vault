# Automated liquidity vault for Orca

## Specifications

1. Admin can create a vault. Each vault need to store relevant Orca Pool info.

2. Admin can create a position using lower_tick and upper_tick.

3. Admin can create a position using deposited tokens.

4. Admin can collect fee, withdraw liquidity and close a position for reposition.

5. User can deposit tokens and get share token minted. Token prices are set by admin, it will be replaced with oracle value later.

6. User can withdraw tokens from the vault.

7. Unit test for main functions by forking Solana mainnet.
