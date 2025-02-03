use anchor_lang::prelude::*;

#[account]
pub struct Management {
    pub authority: Pubkey,
}

impl Management {
    pub fn space() -> usize {
        8 + // discriminator
        32 // authority
    }
    pub fn derive_pda() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"management"], &crate::ID)
    }
    pub fn is_authorized(&self, authority: Pubkey) -> bool {
        self.authority == authority
    }
}