use anchor_lang::prelude::*;

use crate::{errors::ErrorCode, state::Amm};


impl<'info> CreateAmm<'info> {
    pub fn create_amm(&mut self, id: Pubkey, fee: u16) -> Result<()> {
        let amm_account = &mut self.amm_acount;
        amm_account.id = id;
        amm_account.fee = fee;
        amm_account.admin = self.admin.key();
        
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(id: Pubkey, fee: u16)]
pub struct CreateAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    ///CHECK: Read only, delegatable creation
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer = signer,
        space = Amm::INIT_SPACE,
        seeds = [id.as_ref()],
        bump,
        constraint = fee < 10000 @ ErrorCode::InvalidFee,
    )]
    pub amm_acount: Box<Account<'info, Amm>>,

    pub system_program: Program<'info, System>
}
