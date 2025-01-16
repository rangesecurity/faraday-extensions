use {crate::state::management::Management, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        seeds = [b"management"],
        payer = authority,
        // max relloac space is 10240 bytes which is enough for 320 accounts
        // however the discriminator is 8 bytes, so we can only support 319 accounts
        space = Management::space(),
        bump,
    )]
    pub management: Account<'info, Management>,

    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>) -> Result<()> {
        let management = &mut ctx.accounts.management;

        management.authority = ctx.accounts.authority.key();
        Ok(())
    }
}
