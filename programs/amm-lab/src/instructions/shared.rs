use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TransferChecked, transfer_checked};



pub fn transfer_token<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    signer_seeds: Option<&[&[&[u8]]]>
) -> Result<()> {
    let accounts = TransferChecked {
        from,
        to,
        mint,
        authority
    };

    let cpi_ctx = match signer_seeds {
        Some(seeds) => {
            CpiContext::new_with_signer(token_program, accounts, seeds)
        },
        None => {
            CpiContext::new(token_program, accounts)
        }
    };

    transfer_checked(cpi_ctx, amount, decimals)
}
