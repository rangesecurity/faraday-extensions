use anchor_lang::prelude::*;

#[account]
pub struct Management {
    pub authority: Pubkey,
    pub num_block_lists: u64,
}

impl Management {
    pub fn space() -> usize {
        8 + // discriminator
        32 + // authority
        32 // num_block_lists
    }
    pub fn derive_pda() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"management"], &crate::ID)
    }
    pub fn is_authorized(&self, authority: Pubkey) -> bool {
        self.authority == authority
    }
    pub fn increment_and_get_new_list_number(&mut self) -> u64 {
        let new_list_number = self.num_block_lists;
        self.num_block_lists = self.num_block_lists.checked_add(1).unwrap();
        new_list_number
    }
}