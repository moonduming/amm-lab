use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
use fixed::types::I64F64;

use crate::{constants::{AUTHORITY_SEED, LIQUIDITY_SEED}, errors::ErrorCode, instructions::shared::transfer_token, state::{Amm, Pool}};


impl<'info> SwapExactTokensForTokens<'info> {
    pub fn swap_exact_tokens_fro_tokens(
        &mut self, 
        swap_a: bool,
        input_amount: u64,
        min_output_amount: u64,
        bumps: &SwapExactTokensForTokensBumps
    ) -> Result<()> {
        let input = if swap_a && input_amount > self.trader_account_a.amount {
            self.trader_account_a.amount
        } else if !swap_a && input_amount > self.trader_account_b.amount {
            self.trader_account_b.amount
        } else {
            input_amount
        };

        // 计算交易费，用于计算输出
        let taxed_input = input - input * self.amm.fee as u64 / 10000;

        let pool_a = &self.pool_account_a;
        let pool_b = &self.pool_account_b;
        let output = if swap_a {
            I64F64::from_num(taxed_input)
                .checked_mul(I64F64::from_num(pool_b.amount))
                .unwrap()
                .checked_div(
                    I64F64::from_num(pool_a.amount)
                        .checked_add(I64F64::from_num(taxed_input))
                        .unwrap()
                )
                .unwrap()
                .to_num::<u64>()
        } else {
            I64F64::from_num(taxed_input)
                .checked_mul(I64F64::from_num(pool_a.amount))
                .unwrap()
                .checked_div(
                    I64F64::from_num(pool_b.amount)
                        .checked_add(I64F64::from_num(taxed_input))
                        .unwrap()
                    )
                .unwrap()
                .to_num::<u64>()
        };

        require!(output >= min_output_amount, ErrorCode::OutputTooSmall);

        // 计算交易前的不变量
        let invariant = pool_a.amount * pool_b.amount;
        
        let signer_seeds: &[&[&[u8]]] = &[&[
            &self.pool.amm.to_bytes(),
            &self.a_mint.key().to_bytes(),
            &self.b_mint.key().to_bytes(),
            AUTHORITY_SEED.as_bytes(),
            &[bumps.pool_authority]
        ]];

        if swap_a {
            transfer_token(
                self.trader_account_a.to_account_info(), 
                self.pool_account_a.to_account_info(), 
                self.a_mint.to_account_info(), 
                self.trader.to_account_info(), 
                self.token_program.to_account_info(), 
                input, 
                self.a_mint.decimals, 
                None
            )?;
            
            transfer_token(
                self.pool_account_b.to_account_info(), 
                self.trader_account_b.to_account_info(), 
                self.b_mint.to_account_info(), 
                self.pool_authority.to_account_info(), 
                self.token_program.to_account_info(), 
                output, 
                self.b_mint.decimals, 
                Some(signer_seeds)
            )?;

        } else {
            transfer_token(
                self.trader_account_b.to_account_info(), 
                self.pool_account_b.to_account_info(), 
                self.b_mint.to_account_info(), 
                self.trader.to_account_info(), 
                self.token_program.to_account_info(), 
                input, 
                self.b_mint.decimals, 
                None
            )?;
            
            transfer_token(
                self.pool_account_a.to_account_info(), 
                self.trader_account_a.to_account_info(), 
                self.a_mint.to_account_info(), 
                self.pool_authority.to_account_info(), 
                self.token_program.to_account_info(), 
                output, 
                self.a_mint.decimals, 
                Some(signer_seeds)
            )?;
        }

        msg!(
            "Traded {} tokens ({} after fees) for {}",
            input,
            taxed_input,
            output
        );

        self.pool_account_a.reload()?;
        self.pool_account_b.reload()?;

        require!(invariant <= self.pool_account_a.amount * self.pool_account_b.amount, ErrorCode::InvariantViolated);

        Ok(())
    }
}


#[derive(Accounts)]
pub struct SwapExactTokensForTokens<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub trader: Signer<'info>,
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
        has_one = amm,
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
        associated_token::mint = a_mint,
        associated_token::authority = trader,
        associated_token::token_program = token_program
    )]
    pub trader_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = b_mint,
        associated_token::authority = trader,
        associated_token::token_program = token_program
    )]
    pub trader_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}
