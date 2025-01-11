use {
    crate::block_list::BlockList, anchor_lang::{
        prelude::*,
        system_program::{create_account, CreateAccount},
    }, anchor_spl::{
        associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}, token_2022::Token2022, token_interface::{transfer_checked, TokenInterface, TransferChecked}
    }, spl_tlv_account_resolution::{
        account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
    }, spl_transfer_hook_interface::instruction::ExecuteInstruction
};

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    /// CHECK: mint of the token we the transferhook is for
    pub mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub block_list: Account<'info, BlockList>,
}

impl InitializeExtraAccountMetaList<'_> {

pub fn handler(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
    // index 0-3 are the accounts required for token transfer (source, mint, destination, owner)
    // index 4 is address of ExtraAccountMetaList account
    let account_metas = vec![
        // index 5, token mint
        //ExtraAccountMeta::new_with_pubkey(&ctx.accounts.mint.key(), false, false)?,
        // index 6, token program
        ExtraAccountMeta::new_with_pubkey(&ctx.accounts.token_program.key(), false, false)?,
        // index 7, associated token program
        ExtraAccountMeta::new_with_pubkey(
            &ctx.accounts.associated_token_program.key(),
            false,
            false,
        )?,
        // index 8 
        ExtraAccountMeta::new_with_pubkey(
            &ctx.accounts.block_list.key(),
            false,
            false
        )?,
        /*
        // index 8, delegate PDA
        ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal {
                bytes: "delegate".as_bytes().to_vec(),
            }],
            false, // is_signer
            false, // is_writable
        )?,
        // index 9, delegate token account
        ExtraAccountMeta::new_external_pda_with_seeds(
            7, // associated token program index
            &[
                Seed::AccountKey { index: 8 }, // owner index (delegate PDA)
                Seed::AccountKey { index: 6 }, // token program index
                Seed::AccountKey { index: 5 }, // mint index
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 10, sender token account
        ExtraAccountMeta::new_external_pda_with_seeds(
            7, // associated token program index
            &[
                Seed::AccountKey { index: 3 }, // owner index
                Seed::AccountKey { index: 6 }, // token program index
                Seed::AccountKey { index: 5 }, // mint index
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 11, deny list
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: "block_list".as_bytes().to_vec()
                }
            ],
            false,
            false
        )?,*/
    ];

    // calculate account size
    let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
    // calculate minimum required lamports
    let lamports = Rent::get()?.minimum_balance(account_size as usize);

    let mint = ctx.accounts.mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"extra-account-metas",
        &mint.as_ref(),
        &[ctx.bumps.extra_account_meta_list],
    ]];

    // create ExtraAccountMetaList account
    create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.extra_account_meta_list.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        lamports,
        account_size,
        ctx.program_id,
    )?;

    // initialize ExtraAccountMetaList account with extra accounts
    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
        &account_metas,
    )?;

    msg!("extra account {}", ctx.accounts.extra_account_meta_list.key);
    Ok(())
}
}
