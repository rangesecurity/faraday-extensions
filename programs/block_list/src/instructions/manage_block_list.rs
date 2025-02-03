use {
    crate::{error::ErrorCode, management::Management, state::block_list::BlockList, MAX_ADDRESSES_PER_LIST},
    anchor_lang::prelude::*,
    std::collections::HashSet,
};

#[derive(Accounts)]
pub struct ManageBlockList<'info> {
    pub authority: Signer<'info>,
    pub management: Account<'info, Management>,
    #[account(
        mut,
        constraint = management.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub block_list: Account<'info, BlockList>,
}

impl ManageBlockList<'_> {
    pub fn add_handler(ctx: Context<ManageBlockList>, addresses: Vec<Pubkey>) -> Result<()> {
        require!(
            addresses.len() + ctx.accounts.block_list.denied_addresses.len() <= MAX_ADDRESSES_PER_LIST as usize,            ErrorCode::BlockListFull
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
