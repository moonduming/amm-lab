use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, mint_to, MintTo}
};
use fixed::types::I64F64;
use fixed_sqrt::FixedSqrt;

use crate::{constants::{AUTHORITY_SEED, LIQUIDITY_SEED, MINIMUM_LIQUIDITY}, state::Pool, errors::ErrorCode};

use super::shared::transfer_token;


impl<'info> DepositLiquidity<'info> {
    pub fn deposit_liquidity(&mut self, amount_a: u64, amount_b: u64, bumps: &DepositLiquidityBumps) -> Result<()> {
        // 判断金额是否正确
        let mut amount_a = if amount_a > self.deposit_account_a.amount {
            self.deposit_account_a.amount
        } else {
            amount_a
        };

        let mut amount_b = if amount_b > self.deposit_account_b.amount {
            self.deposit_account_b.amount
        } else {
            amount_b
        };

        // 根据 x * y = K 推导实际需要存入的流动性
        let pool_a = &self.pool_account_a;
        let pool_b = &self.pool_account_b;

        let pool_creation = pool_a.amount == 0 && pool_b.amount == 0;
        (amount_a, amount_b) = if pool_creation {
            (amount_a, amount_b)
        } else {
            let ratio = I64F64::from_num(pool_a.amount)
                .checked_div(I64F64::from_num(pool_b.amount))
                .unwrap();
            if pool_a.amount > pool_b.amount {
                (
                    I64F64::from_num(amount_b)
                        .checked_mul(ratio)
                        .unwrap()
                        .to_num::<u64>(),
                    amount_b,
                )
            } else {
                (
                    amount_a,
                    I64F64::from_num(amount_a)
                        .checked_mul(ratio)
                        .unwrap()
                        .to_num::<u64>()
                )
            }
        };
         
        // 计算即将返给投资者的资金量
        let mut liqidity = I64F64::from_num(amount_a)
            .checked_mul(I64F64::from_num(amount_b))
            .unwrap().sqrt()
            .to_num::<u64>();
        
        // 锁定第一笔存款的最低流动性
        if pool_creation {
            require!(liqidity >= MINIMUM_LIQUIDITY, ErrorCode::DepositTooSmall);
            liqidity -= MINIMUM_LIQUIDITY;
        };

        // 将资金存入流动池
        transfer_token(
            self.deposit_account_a.to_account_info(), 
            self.pool_account_a.to_account_info(), 
            self.a_mint.to_account_info(), 
            self.deposit.to_account_info(), 
            self.token_program.to_account_info(), 
            amount_a, 
            self.a_mint.decimals, 
            None
        )?;
        
        transfer_token(
            self.deposit_account_a.to_account_info(), 
            self.pool_account_b.to_account_info(), 
            self.b_mint.to_account_info(), 
            self.deposit.to_account_info(), 
            self.token_program.to_account_info(), 
            amount_b, 
            self.b_mint.decimals, 
            None
        )?;

        // 将此次出入获取到的代币转给用户
        let signer_seeds: &[&[&[u8]]] = &[&[
            &self.pool.amm.to_bytes(),
            &self.a_mint.key().to_bytes(),
            &self.b_mint.key().to_bytes(),
            AUTHORITY_SEED.as_bytes(),
            &[bumps.pool_authority]
        ]];

        mint_to(
        CpiContext::new_with_signer(
        self.token_program.to_account_info(), 
        MintTo { 
                    mint: self.mint_liquidity.to_account_info(), 
                    to: self.deposit_account_liquidity.to_account_info(), 
                    authority: self.pool_authority.to_account_info() 
                }, 
                signer_seeds
            ),
            liqidity
        )?;
        
        Ok(())
    }
}


#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub deposit: Signer<'info>,
    pub a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub b_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [
            pool.amm.as_ref(),
            pool.a_mint.key().as_ref(), 
            pool.b_mint.key().as_ref(),
        ],
        bump,
        has_one = a_mint,
        has_one = b_mint
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Read only authority
    #[account(
        seeds = [
            pool.amm.as_ref(),
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
            pool.amm.as_ref(),
            a_mint.key().as_ref(), 
            b_mint.key().as_ref(),
            LIQUIDITY_SEED.as_bytes()
        ],
        bump,
    )]
    pub mint_liquidity: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = a_mint,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program
    )]
    pub pool_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
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
