pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GFfVeXyUvceCu1H1PPTBUvwYJeDQjXusgzjrAjHf2kHn");

#[program]
pub mod rate_limits {
    use spl_transfer_hook_interface::instruction::TransferHookInstruction;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Initialize::handler(ctx)
    }
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        InitializeExtraAccountMetaList::handler(ctx)
    }
    pub fn create_mint_rate_limit(
        ctx: Context<CreateMintBasedRateLimit>,
        period_limit: u64,
        period_duration: u64,
    ) -> Result<()> {
        CreateMintBasedRateLimit::handler(ctx, period_limit, period_duration)
    }
    pub fn transfer_hook<'info>(ctx: Context<'_, '_, 'info, 'info,TransferHook<'info>>, amount: u64) -> Result<()> {
        TransferHook::handler(ctx, amount)
    }

    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;

        // match instruction discriminator to transfer hook interface execute instruction
        // token2022 program CPIs this instruction on token transfer
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();

                // invoke transfer hook to check if one of the token accounts, or the owner is in the block list
                __private::__global::transfer_hook(program_id, accounts, &amount_bytes)?;
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
        Ok(())
    }
}
