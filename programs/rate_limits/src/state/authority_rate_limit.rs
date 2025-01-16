use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use crate::error::RateLimitError;
use super::limiters::{LimiterEntry, RateLimitExt};


/// Provides a rate limit implementation that rate limits transfers on a per-authority basis
#[account]
#[derive(Debug)]
pub struct AuthorityRateLimit {
    /// Maximum amount that can be transferred in a single period
    pub period_limit: u64,
    /// The start time of the current period
    pub current_period_start: UnixTimestamp,
    /// Duration of each period in seconds
    pub period_duration: u64,
    /// token mint the rate limit is for
    pub mint: Pubkey,
    /// Vector of rate limit entries for different authorities
    pub entries: Vec<LimiterEntry>,
    #[cfg(test)]
    pub current_time: UnixTimestamp,
}

impl AuthorityRateLimit {
    pub fn new(period_limit: u64, period_duration: u64, start_time: UnixTimestamp, mint: Pubkey) -> Result<Self> {
        require!(period_duration > 0, RateLimitError::InvalidPeriodConfig);
        #[cfg(test)]
        return Ok(Self {
            entries: Vec::new(),
            period_limit,
            current_period_start: start_time,
            period_duration,
            mint,
            current_time: 0,
        });

        #[cfg(not(test))]
        return Ok(Self{
            entries: Vec::new(),
            period_limit,
            current_period_start: start_time,
            mint,
            period_duration,           
        });
    }

    /// Initialize a new rate limit entry for an authority
    pub fn init_limiter_entry(&mut self, authority: Pubkey) {
        if self.limiter_entry(authority).is_none() {
            self.entries.push(LimiterEntry {
                authority,
                value_transferred: 0,
            });
        }
    }

    /// Returns Some(LimiterEntry) for the specific authority if it has a configured entry
    /// 
    /// Returns None for the specific authority if it has no configured entry
    pub fn limiter_entry(&mut self, authority: Pubkey) -> Option<&mut LimiterEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.authority == authority)
    }

    // Add method to update current time (for testing)
    #[cfg(test)]
    pub fn set_current_time(&mut self, time: UnixTimestamp) {
        self.current_time = time;
    }
}

impl RateLimitExt for AuthorityRateLimit {
    fn start_time(&self) -> UnixTimestamp {
        self.current_period_start
    }

    fn end_time(&self) -> UnixTimestamp {
        self.current_period_start.saturating_add(self.period_duration as i64)
    }

    fn period_duration_seconds(&self) -> u64 {
        self.period_duration
    }

    fn roll_over(&mut self) {
        #[cfg(test)]
        let current_time = self.current_time;
        #[cfg(not(test))]
        let current_time = Clock::get().unwrap().unix_timestamp;


        if current_time >= self.end_time() {
            // Calculate how many periods have passed
            let periods_elapsed = (current_time.checked_sub(self.current_period_start).unwrap() as u64)
                .saturating_div(self.period_duration);
            
            // Update the period start time
            self.current_period_start = self.current_period_start
                .saturating_add((periods_elapsed * self.period_duration) as i64);
            
            // Reset all transfer amounts for the new period
            for entry in self.entries.iter_mut() {
                entry.value_transferred = 0;
            }
        }
    }

    fn check_and_update(&mut self, authority: Option<Pubkey>, amount: u64) -> Result<()> {
        let Some(authority) = authority else {
            return Err(RateLimitError::InvalidCheckAndUpdate.into());
        };

        // First check if we need to roll over to a new period
        self.roll_over();

        let period_limit = self.period_limit;

        // Get or create the limiter entry
        let entry = if let Some(entry) = self.limiter_entry(authority) {
            entry
        } else {
            self.init_limiter_entry(authority);
            self.limiter_entry(authority).unwrap()
        };

        // Check if the transfer would exceed the period limit
        if entry.value_transferred.saturating_add(amount) > period_limit {
            return err!(RateLimitError::RateLimitExceeded);
        }

        // Update the transferred amount
        entry.value_transferred = entry.value_transferred.saturating_add(amount);
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let start_time = 1000;
        let mut rate_limit = AuthorityRateLimit::new(100, 3600, start_time, Default::default()).unwrap(); // 100 tokens per hour
        rate_limit.set_current_time(start_time + 1);

        let authority = Pubkey::new_unique();

        // First transfer should work
        assert!(rate_limit.check_and_update(Some(authority), 50).is_ok());
        
        // Second transfer that would exceed limit should fail
        assert!(rate_limit.check_and_update(Some(authority), 51).is_err());
        
        // Small transfer still within limits should work
        assert!(rate_limit.check_and_update(Some(authority), 40).is_ok());
    }

    #[test]
    fn test_period_rollover() {
        let start_time = 1000;
        let mut rate_limit = AuthorityRateLimit::new(100, 3600, start_time, Default::default()).unwrap();
        rate_limit.set_current_time(start_time + 1);
        let authority = Pubkey::new_unique();

        // Use up the limit
        assert!(rate_limit.check_and_update(Some(authority), 100).is_ok());
        
        rate_limit.set_current_time(rate_limit.current_time+3600);


        // This should trigger a rollover and reset the limits
        rate_limit.roll_over();
        
        // Should be able to transfer again
        assert!(rate_limit.check_and_update(Some(authority), 100).is_ok());


        rate_limit.set_current_time(rate_limit.current_time+9600);

        rate_limit.roll_over();;

        assert_eq!(rate_limit.current_period_start, 11800);

    }
}