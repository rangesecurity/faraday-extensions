use {crate::{state::block_list::BlockList, MAX_ADDRESSES_PER_LIST}, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [b"block_list"],
        payer = authority,
        // max relloac space is 10240 bytes which is enough for 320 accounts
        // however the discriminator is 8 bytes, so we can only support 319 accounts
        space = BlockList::space(MAX_ADDRESSES_PER_LIST as usize),
        bump,
    )]
    pub block_list: Account<'info, BlockList>,

    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>) -> Result<()> {
        let block_list = &mut ctx.accounts.block_list;
        block_list.authority = ctx.accounts.authority.key();
        block_list.denied_addresses = Vec::new();
        Ok(())
    }
}
