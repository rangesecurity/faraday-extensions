use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};

/// Trait that defines the interface a rate limit must conform to.
/// 
/// Using traits allows for customizing the underlying logic of the rate limit
pub trait RateLimitExt {
    /// Returns the starting time for the current rate limit period
    fn start_time(&self) -> UnixTimestamp;
    /// Returns the ending time for the current rate limit period
    fn end_time(&self) -> UnixTimestamp;
    /// Returns the amount of time in seconds a period lasts
    fn period_duration_seconds(&self) -> u64;
    /// If the current time is passed the end time of the previous period, roll the period over to the new one
    fn roll_over(&mut self);
    /// Checks to see if `authority` can transfer `amount` of tokens. 
    /// 
    /// If the authority is not rate limited, updates the addresses rate limit entry and retunrs Ok.
    /// 
    /// If the authority is rate limited, returns an error.
    fn check_and_update(&mut self, authority: Pubkey, amount: u64) -> Result<()>;
    /// Returns Some(LimiterEntry) for the specific authority if it has a configured entry
    /// 
    /// Returns None for the specific authority if it has no configured entry
    fn limiter_entry(&mut self, authority: Pubkey) -> Option<&mut LimiterEntry>;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct LimiterEntry {
    /// The address which this particular rate limit entry corresponds to
    pub authority: Pubkey,
    /// The amount of value this authority has transferred in the current period
    pub value_transferred: u64,
}