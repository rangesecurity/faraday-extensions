use {
    crate::{
        error::RateLimitError, limiters::RateLimitType, management::Management,
        mint_rate_limit::MintRateLimit,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{
            program::{invoke, invoke_signed},
            system_instruction,
        },
    },
    spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList},
    spl_transfer_hook_interface::instruction::{
        ExecuteInstruction, UpdateExtraAccountMetaListInstruction,
    },
    spl_type_length_value::state::TlvStateBorrowed,
};

#[derive(Accounts)]
pub struct CreateMintBasedRateLimit<'info> {
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
    /// CHECK: vlaidated manually
    #[account(
        init,
        seeds = [b"mint_based", mint.key.as_ref()],
        payer = authority,
        space = MintRateLimit::space(),
        bump
    )]
    pub rate_limit: Account<'info, MintRateLimit>,
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl CreateMintBasedRateLimit<'_> {
    /// Creates and initializes a rate limit account, which sets the current period start to the current time
    pub fn handler(
        ctx: Context<CreateMintBasedRateLimit>,
        period_limit: u64,
        period_duration: u64,
    ) -> Result<()> {
        Self::validations(&ctx)?;

        // initialize the rate limit
        {
            let rate_limit= &mut ctx.accounts.rate_limit;
            rate_limit.initialize(
                period_limit,
                period_duration,
                Clock::get()?.unix_timestamp,
                ctx.accounts.mint.key(),
            )?;
        }

        // get current accounts
        let mut account_metas: Vec<ExtraAccountMeta> = {
            let data = ctx.accounts.extra_account_meta_list.try_borrow_data()?;
            let tlv_state = TlvStateBorrowed::unpack(&data)?;
            let extra_accounts =
                ExtraAccountMetaList::unpack_with_tlv_state::<ExecuteInstruction>(&tlv_state)?;
            extra_accounts.data().to_vec()
        };
        // add new account
        account_metas.push(ExtraAccountMeta::new_with_pubkey(
            &ctx.accounts.rate_limit.key(),
            false,
            false,
        )?);
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
    // returns the nocne used to derive the rate limit account
    fn validations(
        ctx: &Context<CreateMintBasedRateLimit>,
    ) -> Result<()> {
        require!(
            ctx.accounts
                .management
                .is_authorized(ctx.accounts.authority.key()),
                RateLimitError::Unauthorized
        );

        Ok(())
    }
}
