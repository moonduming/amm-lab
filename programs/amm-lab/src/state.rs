use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct Amm {
    pub id: Pubkey,
    pub admin: Pubkey,
    pub fee: u16
}


#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub amm: Pubkey,
    pub a_mint: Pubkey,
    pub b_mint: Pubkey
}
