use {
    crate::{
        error::ErrorCode,
        state::block_list::BlockList,
    }, anchor_lang::prelude::*, std::collections::HashSet
};

#[derive(Accounts)]
pub struct ManageBlockList<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = block_list.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub block_list: Account<'info, BlockList>,
}

impl ManageBlockList<'_> {
    pub fn add_handler(
        ctx: Context<ManageBlockList>,
        addresses: Vec<Pubkey>,
    ) -> Result<()> {
        require!(
            addresses.len() + ctx.accounts.block_list.denied_addresses.len() <= 1000,
            ErrorCode::BlockListFull
        );

        let mut current_addresses: HashSet<_> = ctx
            .accounts
            .block_list
            .denied_addresses
            .iter()
            .cloned()
            .collect();

        for address in addresses {
            current_addresses.insert(address);
        }

        ctx.accounts.block_list.denied_addresses = current_addresses.into_iter().collect();
        Ok(())       
    }
    pub fn remove_handler(ctx: Context<ManageBlockList>, addresses: Vec<Pubkey>) -> Result<()> {
        let remove_set: HashSet<_> = addresses.into_iter().collect();
        ctx.accounts
            .block_list
            .denied_addresses
            .retain(|addr| !remove_set.contains(addr));
        Ok(())
    }
}