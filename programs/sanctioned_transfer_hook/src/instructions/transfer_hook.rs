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
    pub block_list_account: Account<'info, BlockList>,
}

impl TransferHook<'_> {
    pub fn handler(ctx: Context<TransferHook>, _amount: u64) -> Result<()> {
        Self::validations(&ctx)
    }
    fn validations(ctx: &Context<TransferHook>) -> Result<()> {
        require!(
            ctx.accounts.extra_account_meta_list.owner.eq(&crate::ID),
            ErrorCode::InvalidExtraAccountMetasList
        );
        require!(
            BlockList::derive_pda(ctx.accounts.mint.key())
                .0
                .eq(ctx.accounts.extra_account_meta_list.key),
            ErrorCode::InvalidExtraAccountMetasList
        );
        {
            // check to see if source token account owner is denied
            let data = ctx.accounts.source_token.data.try_borrow().unwrap();
            let source_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;

            require!(
                !ctx.accounts
                    .block_list_account
                    .transfer_denied(source_account.base.owner),
                ErrorCode::Denied
            );
        }
        {
            // check to see if receiving token account owner is denied
            let data = ctx.accounts.destination_token.data.try_borrow().unwrap();
            let receiving_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;

            require!(
                !ctx.accounts
                    .block_list_account
                    .transfer_denied(receiving_account.base.owner),
                ErrorCode::Denied
            );
        }
        {
            // check to see if owner is denied
            require!(
                !ctx.accounts
                    .block_list_account
                    .transfer_denied(ctx.accounts.owner.key()),
                ErrorCode::Denied
            );
        }
        Ok(())
    }
}
