use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{state::{Amm, Pool}, constants::{AUTHORITY_SEED, LIQUIDITY_SEED}};


impl<'info> CreatePool<'info> {
    pub fn create_pool(&mut self) -> Result<()> {
        let pool = &mut self.pool;
        pool.amm = self.amm.key();
        pool.a_mint = self.a_mint.key();
        pool.b_mint = self.b_mint.key();
        
        Ok(())
    }
}


#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub a_mint: Box<InterfaceAccount<'info, Mint>>,
    pub b_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [amm.id.as_ref()],
        bump
    )]
    pub amm: Box<Account<'info, Amm>>,

    #[account(
        init,
        payer = signer,
        space = Pool::INIT_SPACE,
        seeds = [
            amm.key().as_ref(),
            a_mint.key().as_ref(), 
            b_mint.key().as_ref(),
        ],
        bump
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
        init,
        payer = signer,
        seeds = [
            amm.key().as_ref(),
            a_mint.key().as_ref(), 
            b_mint.key().as_ref(),
            LIQUIDITY_SEED.as_bytes()
        ],
        bump,
        mint::decimals = 6,
        mint::authority = pool_authority
    )]
    pub mint_liquidity: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>
}
