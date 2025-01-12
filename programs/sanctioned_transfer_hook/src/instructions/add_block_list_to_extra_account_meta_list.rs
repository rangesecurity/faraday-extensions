use {
    crate::{block_list::BlockList, error::ErrorCode, management::Management},
    anchor_lang::{
        prelude::*, solana_program::{program::invoke_signed, system_instruction}, system_program::{create_account, CreateAccount}
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token_2022::{
            spl_token_2022::{extension::StateWithExtensions, state::Mint},
            Token2022,
        },
    },
    spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList},
    spl_transfer_hook_interface::instruction::{ExecuteInstruction, UpdateExtraAccountMetaListInstruction},
};

#[derive(Accounts)]
pub struct AddBlockListExtraAccountMetaList<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        constraint = management.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub management: Account<'info, Management>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metasA", mint.key().as_ref()],
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

impl AddBlockListExtraAccountMetaList<'_> {
    pub fn handler(ctx: Context<AddBlockListExtraAccountMetaList>) -> Result<()> {
        // index 0-3 are the accounts required for token transfer (source, mint, destination, owner)
        // index 4 is address of ExtraAccountMetaList account
        let new_account_metas = vec![
            ExtraAccountMeta::new_with_pubkey(&ctx.accounts.block_list.key(), false, false)?,
        ];
        let account_size = ctx.accounts.extra_account_meta_list.data_len();
        // calculate account size
        let data_to_add = ExtraAccountMetaList::size_of(new_account_metas.len())? as u64;
        let new_account_size = account_size+ data_to_add as usize;
        // Current balance of the account
        let current_balance = ctx.accounts.extra_account_meta_list.lamports();
        // calculate minimum required lamports
        let minimum_balance = Rent::get()?.minimum_balance(new_account_size as usize);
        // If we need more lamports for rent exemption
        if minimum_balance > current_balance {
            let lamports_to_add = minimum_balance - current_balance;
            invoke_signed(
                &system_instruction::transfer(
                    ctx.accounts.authority.key,
                    ctx.accounts.extra_account_meta_list.key,
                    lamports_to_add,
                ),
                &[
                    ctx.accounts.authority.to_account_info(),
                    ctx.accounts.extra_account_meta_list.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[],
            )?;
        }
    
        // Reallocate the account to the new size
        ctx.accounts.extra_account_meta_list.realloc((account_size+500) as usize, false)?;
        

        ExtraAccountMetaList::update::<UpdateExtraAccountMetaListInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &new_account_metas
        )?;

        Ok(())
    }
}
