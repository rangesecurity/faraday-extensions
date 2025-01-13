use anchor_lang::prelude::*;

#[account]
pub struct BlockList {
    pub authority: Pubkey,
    pub denied_addresses: Vec<Pubkey>,
}

impl BlockList {
    pub const fn space(max_addresses: usize) -> usize {
        8 +  // discriminator
        32 + // authority
        4 +  // vec length
        (32 * max_addresses) // addresses
    }
    pub fn transfer_denied(&self, authority: Pubkey) -> bool {
        self.denied_addresses.contains(&authority)
    }
    pub fn derive_pda(mint: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"extra-account-metas", mint.as_ref()], &crate::ID)
    }
}
