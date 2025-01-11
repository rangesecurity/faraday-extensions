use {
    anchor_lang::prelude::*,
    crate::state::block_list::BlockList,
};


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        seeds = [b"block_list"],
        payer = authority,
        space = BlockList::space(300),
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
