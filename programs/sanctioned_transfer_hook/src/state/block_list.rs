use anchor_lang::prelude::*;

#[account]
pub struct BlockList {
    /// The number of this block list, uses array counting
    /// 
    /// 0 == 1st block list
    /// 1 == 2nd block list
    pub block_list_number: u64,
    pub denied_addresses: Vec<Pubkey>,

}

impl BlockList {
    pub const fn space(max_addresses: usize) -> usize {
        8 +  // discriminator
        32 + // block list number
        4 +  // vec length
        (32 * max_addresses) // addresses
    }
    pub fn transfer_denied(&self, authority: Pubkey) -> bool {
        self.denied_addresses.contains(&authority)
    }
}
