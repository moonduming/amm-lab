use anchor_lang::prelude::*;

mod state;
mod instructions;
mod errors;
mod constants;

pub use instructions::*;

declare_id!("BLWvcgaBfsQLkfxcxg4afZzfQWZZKD5L5QcJDb9n6ag3");

#[program]
pub mod amm_lab {
    use super::*;

    pub fn create_amm(ctx: Context<CreateAmm>, id: Pubkey, fee: u16) -> Result<()> {
        ctx.accounts.create_amm(id, fee)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        ctx.accounts.create_pool()
    }

    pub fn deposit_liquidity(ctx: Context<DepositLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        ctx.accounts.deposit_liquidity(amount_a, amount_b, &ctx.bumps)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw_liquidity(amount, &ctx.bumps)
    }

    pub fn swap_exact_tokens_for_tokens(
        ctx: Context<SwapExactTokensForTokens>,
        swap_a: bool,
        input_amount: u64,
        min_output_amount: u64
    ) -> Result<()> {
        ctx.accounts.swap_exact_tokens_fro_tokens(swap_a, input_amount, min_output_amount, &ctx.bumps)
    }
}

