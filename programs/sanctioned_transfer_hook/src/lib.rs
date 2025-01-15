pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("D3Cna2aGhRxzfeoiCaQMU2LZPxPuWXpsJAFqiMdkhXCo");

#[program]
pub mod sanctioned_transfer_hook {
    use spl_transfer_hook_interface::instruction::TransferHookInstruction;

    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Initialize::handler(ctx)
    }
    pub fn create_block_listt(ctx: Context<CreateBlockList>, list_number: u64) -> Result<()> {
        CreateBlockList::handler(ctx, list_number)
    }
    pub fn add_to_block_list(ctx: Context<ManageBlockList>, addresses: Vec<Pubkey>) -> Result<()> {
        ManageBlockList::add_handler(ctx, addresses)
    }
    pub fn remove_from_block_list(
        ctx: Context<ManageBlockList>,
        addresses: Vec<Pubkey>,
    ) -> Result<()> {
        ManageBlockList::remove_handler(ctx, addresses)
    }
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        InitializeExtraAccountMetaList::handler(ctx)
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
