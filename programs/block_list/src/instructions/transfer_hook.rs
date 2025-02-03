use {
    crate::{block_list::BlockList, error::ErrorCode},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token_2022::{
            spl_token_2022::{extension::StateWithExtensions, state::Account as TokenAccount},
            Token2022,
        },
    },
};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    /// CHECK: validated by token2022 program
    pub source_token: UncheckedAccount<'info>,
    /// CHECK: validated through extra_account_meta_list seed
    pub mint: UncheckedAccount<'info>,
    /// CHECK: validated by token2022 program
    pub destination_token: UncheckedAccount<'info>,
    /// CHECK: owner of source token account, may be a delegated signer
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    //pub block_list_account: Account<'info, BlockList>,
}

impl TransferHook<'_> {
    pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info,TransferHook<'info>>, _amount: u64) -> Result<()> {
        Self::validations(&ctx)
    }
    fn validations<'info>(ctx: &Context<'_, '_, 'info, 'info,TransferHook<'info>>) -> Result<()> {
        require!(
            ctx.accounts.extra_account_meta_list.owner.eq(&crate::ID),
            ErrorCode::InvalidExtraAccountMetasList
        );
        if ctx.remaining_accounts.len() == 0 {
            panic!("unexpected condition")
        }
        // build list of owners to check which will be evaluated against all block lists
        let mut owners_to_check = Vec::with_capacity(3);
        owners_to_check.push(ctx.accounts.owner.key());
        {
            // check to see if source token account owner is denied
            let data = ctx.accounts.source_token.data.try_borrow().unwrap();
            let source_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;
            owners_to_check.push(source_account.base.owner);
        }
        {
            // check to see if receiving token account owner is denied
            let data = ctx.accounts.destination_token.data.try_borrow().unwrap();
            let receiving_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;
            owners_to_check.push(receiving_account.base.owner);
        }

        // evaluate all block lists to see if any of the owners are denied
        for remaining_account in ctx.remaining_accounts.iter() {
            msg!("checking block list");
            let block_list: Account<BlockList> = Account::try_from(remaining_account)?;
            for owner_to_check in &owners_to_check {
                require!(
                    !block_list.transfer_denied(*owner_to_check),
                    ErrorCode::Denied
                );
            }
        }
        Ok(())
    }
}
