use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface}};
use fixed::types::I64F64;

use crate::{constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY}, state::{Amm, Pool}};

use super::shared::transfer_token;


impl<'info> WithdrawLiquidity<'info> {
    pub fn withdraw_liquidity(&mut self, amount: u64, bumps: &WithdrawLiquidityBumps) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            &self.pool.amm.to_bytes(),
            &self.a_mint.key().to_bytes(),
            &self.b_mint.key().to_bytes(),
            AUTHORITY_SEED.as_bytes(),
            &[bumps.pool_authority]
        ]];

        let amount_a = I64F64::from_num(amount)
            .checked_mul(I64F64::from_num(self.pool_account_a.amount))
            .unwrap()
            .checked_div(I64F64::from_num(self.mint_liquidity.supply + MINIMUM_LIQUIDITY))
            .unwrap().floor()
            .to_num::<u64>();

        transfer_token(
            self.pool_account_a.to_account_info(), 
            self.deposit_account_a.to_account_info(), 
            self.a_mint.to_account_info(), 
            self.pool_authority.to_account_info(), 
            self.token_program.to_account_info(), 
            amount_a, 
            self.a_mint.decimals, 
            Some(signer_seeds)
        )?;

        let amount_b = I64F64::from_num(amount)
            .checked_mul(I64F64::from_num(self.pool_account_b.amount))
            .unwrap()
            .checked_div(I64F64::from_num(self.mint_liquidity.supply + MINIMUM_LIQUIDITY))
            .unwrap().floor()
            .to_num::<u64>();


        transfer_token(
            self.pool_account_b.to_account_info(), 
            self.deposit_account_b.to_account_info(), 
            self.b_mint.to_account_info(), 
            self.pool_authority.to_account_info(), 
            self.token_program.to_account_info(), 
            amount_b, 
            self.b_mint.decimals, 
            Some(signer_seeds)
        )?;

        burn(
            CpiContext::new(
                self.token_program.to_account_info(), 
                Burn { 
                            mint: self.mint_liquidity.to_account_info(), 
                            from: self.deposit_account_liquidity.to_account_info(), 
                            authority: self.deposit.to_account_info() 
                        }
                ), 
            amount
        )?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub deposit: Signer<'info>,
    pub a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub b_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [amm.id.as_ref()],
        bump
    )]
    pub amm: Box<Account<'info, Amm>>,

    #[account(
        seeds = [
            amm.key().as_ref(),
            a_mint.key().as_ref(),
            b_mint.key().as_ref(),
        ],
        bump,
        has_one = a_mint,
        has_one = b_mint
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Read only authority
    #[account(
        seeds = [
            amm.key().as_ref(),
            a_mint.key().as_ref(), 
            b_mint.key().as_ref(),
            AUTHORITY_SEED.as_bytes()
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            amm.key().as_ref(),
            a_mint.key().as_ref(),
            b_mint.key().as_ref(),
            LIQUIDITY_SEED.as_bytes()
        ],
        bump,
    )]
    pub mint_liquidity: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        associated_token::mint = a_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub pool_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = b_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub pool_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_liquidity,
        associated_token::authority = deposit,
        associated_token::token_program = token_program
    )]
    pub deposit_account_liquidity: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = a_mint,
        associated_token::authority = deposit,
        associated_token::token_program = token_program
    )]
    pub deposit_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = b_mint,
        associated_token::authority = deposit,
        associated_token::token_program = token_program
    )]
    pub deposit_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}
