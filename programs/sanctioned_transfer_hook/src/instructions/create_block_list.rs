use {
    crate::{
        error::ErrorCode, management::Management, state::block_list::BlockList,
        MAX_ADDRESSES_PER_LIST,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{program::invoke, system_instruction},
    },
    spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList},
    spl_transfer_hook_interface::instruction::{
        ExecuteInstruction, UpdateExtraAccountMetaListInstruction,
    },
    spl_type_length_value::state::TlvStateBorrowed,
};

#[derive(Accounts)]
#[instruction(list_number: u64)]
pub struct CreateBlockList<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"management"],
        bump,
    )]
    pub management: Account<'info, Management>,
    /// CHECK: validated through account metas
    pub mint: AccountInfo<'info>,
    #[account(
        init,
        seeds = [b"block_list", list_number.to_le_bytes().as_ref()],
        payer = authority,
        // max relloac space is 10240 bytes which is enough for 320 accounts
        // however the discriminator is 8 bytes, so we can only support 319 accounts
        space = BlockList::space(MAX_ADDRESSES_PER_LIST as usize),
        bump,
    )]
    pub block_list: Account<'info, BlockList>,
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl CreateBlockList<'_> {
    pub fn handler(ctx: Context<CreateBlockList>, _list_number: u64) -> Result<()> {
        Self::validations(&ctx)?;
        let new_list_number = {
            let management = &mut ctx.accounts.management;
            management.increment_and_get_new_list_number()
        };
        let block_list = &mut ctx.accounts.block_list;
        block_list.denied_addresses = Vec::new();
        block_list.block_list_number = new_list_number;

        // get current accoutns
        let mut account_metas: Vec<ExtraAccountMeta> = {
            let data = ctx.accounts.extra_account_meta_list.try_borrow_data()?;
            let tlv_state = TlvStateBorrowed::unpack(&data)?;
            let extra_accounts =
                ExtraAccountMetaList::unpack_with_tlv_state::<ExecuteInstruction>(&tlv_state)?;
            extra_accounts.data().to_vec()
        };
        // add new account
        account_metas.push(ExtraAccountMeta::new_with_pubkey(
            &ctx.accounts.block_list.key(),
            false,
            false,
        )?)
        ;
        let account_size = ctx.accounts.extra_account_meta_list.data_len();
        // calculate account size
        let data_to_add = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
        let new_account_size = account_size + data_to_add as usize;
        // Current balance of the account
        let current_balance = ctx.accounts.extra_account_meta_list.lamports();
        // calculate minimum required lamports
        let minimum_balance = Rent::get()?.minimum_balance(new_account_size as usize);
        // If we need more lamports for rent exemption
        if minimum_balance > current_balance {
            let lamports_to_add = minimum_balance - current_balance;
            invoke(
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
            )?;
        }

        // Reallocate the account to the new size
        ctx.accounts
            .extra_account_meta_list
            .realloc(new_account_size as usize, false)?;

        ExtraAccountMetaList::update::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &account_metas,
        )?;

        Ok(())
    }
    fn validations(ctx: &Context<CreateBlockList>) -> Result<()> {
        require!(
            ctx.accounts
                .management
                .is_authorized(ctx.accounts.authority.key()),
            ErrorCode::Unauthorized
        );
        Ok(())
    }
}
