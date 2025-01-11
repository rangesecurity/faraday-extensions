use {
    crate::{block_list::BlockList, error::ErrorCode}, anchor_lang::{
        prelude::*,
        system_program::{create_account, CreateAccount},
    }, anchor_spl::{
        associated_token::AssociatedToken,  token_2022::{
            spl_token_2022::{
                state::{Account as TokenAccount, Mint},
                extension::StateWithExtensions
            }, Token2022
        }, token_interface::{transfer_checked, TokenInterface, TransferChecked}
    }, spl_tlv_account_resolution::{
        account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
    }, spl_transfer_hook_interface::instruction::ExecuteInstruction
};


#[derive(Accounts)]
pub struct TransferHook<'info> {
    /// CHECK: validated by token2022 program
    //#[account(
    //    token::mint = mint,
    //    token::authority = owner,
    //)]
    pub source_token: UncheckedAccount<'info>,
    /// CHECK: validated through extra_account_meta_list seed
    pub mint: UncheckedAccount<'info>,
    /// CHECK: validated by token2022 program
    //#[account(
    //    token::mint = mint,
    //)]
    pub destination_token: UncheckedAccount<'info>,
    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        seeds = [b"delegate"],
        bump
    )]
    pub delegate: SystemAccount<'info>,
    /// CHECK: validated by state unpacking and token2022 program
    #[account(
        mut,
        //token::mint = mint,
        //token::authority = delegate,
    )]
    pub delegate_token_account: UncheckedAccount<'info>,
    /// CHECK: validated by state unpacking and token2022 program
    #[account(
        mut,
        //token::mint = mint,
        //token::authority = owner,
    )]
    pub sender_token_account: UncheckedAccount<'info>,
    pub block_list_account: Account<'info, BlockList>,
}

impl TransferHook<'_> {
    pub fn handler(
        ctx: Context<TransferHook>,
        amount: u64
    ) -> Result<()> {
        // check if any of the source or destination token accounts are denied
        // check if the delegate is denied
        {
            // check to see if source token account owner is denied
            let data = ctx.accounts.sender_token_account.data.try_borrow().unwrap();
            let source_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;

            require!(
                !ctx.accounts.block_list_account.transfer_denied(
                    source_account.base.owner
                ),
                ErrorCode::Denied
            );

        }
        {
            // check to see if receivint token account owner is denied
            let data = ctx.accounts.delegate_token_account.data.try_borrow().unwrap();
            let receiving_account = StateWithExtensions::<TokenAccount>::unpack(&data)?;

            require!(
                !ctx.accounts.block_list_account.transfer_denied(
                    receiving_account.base.owner
                ),
                ErrorCode::Denied
            );
        }
        let decimals = {
            let data = ctx.accounts.mint.data.try_borrow().unwrap();
            StateWithExtensions::<Mint>::unpack(
                &data
            )?.base.decimals
        };
        // check to see if delegate is denied
        require!(
            !ctx.accounts.block_list_account.transfer_denied(
                *ctx.accounts.delegate.owner
            ),
            ErrorCode::Denied
        );
        msg!("Transfer tokens using delegate PDA");

        let signer_seeds: &[&[&[u8]]] = &[&[b"delegate", &[ctx.bumps.delegate]]];
    
        // transfer tokens from sender to delegate token account using delegate PDA
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.sender_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.delegate_token_account.to_account_info(),
                    authority: ctx.accounts.delegate.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            amount,
            decimals,
        )?;
        Ok(())
    }
}