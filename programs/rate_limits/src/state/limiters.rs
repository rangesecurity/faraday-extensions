use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use crate::error::RateLimitError;

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
    /// Checks to see if a transfer can be performed.
    /// 
    /// `authority` is optional as some rate limiting implementations may not want to do per-authority lrate limiting
    /// Checks to see if `authority` can transfer `amount` of tokens. 
    /// 
    /// If the authority is not rate limited, updates the addresses rate limit entry and retunrs Ok.
    /// 
    /// If the authority is rate limited, returns an error.
    fn check_and_update(&mut self, authority: Option<Pubkey>, amount: u64) -> Result<()>;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct LimiterEntry {
    /// The address which this particular rate limit entry corresponds to
    pub authority: Pubkey,
    /// The amount of value this authority has transferred in the current period
    pub value_transferred: u64,
}


/// Denotes the possible types of rate limits which can be created
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
#[repr(u8)]
pub enum RateLimitType {
    AuthorityBased,
    MintBased,
}

impl TryFrom<u8> for RateLimitType {
    type Error = RateLimitError;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(RateLimitType::AuthorityBased),
            1 => Ok(RateLimitType::MintBased),
            _ => Err(RateLimitError::InvalidRateLimitType)
        }
    }
}