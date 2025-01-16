use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";


/// Maximum addresses per block list based on realloc limits
#[constant]
pub const MAX_ADDRESSES_PER_LIST: u64 = 318;