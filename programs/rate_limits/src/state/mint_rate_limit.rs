use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use crate::error::ErrorCode;
use super::limiters::{LimiterEntry, RateLimitExt};


/// Provides a rate limit implementation that rate limits transfers on a per-mint basis
#[account]
#[derive(Debug)]
pub struct MintRateLimit {
    /// Maximum amount that can be transferred in a single period
    pub period_limit: u64,
    /// The start time of the current period
    pub current_period_start: UnixTimestamp,
    /// Duration of each period in seconds
    pub period_duration: u64,
    /// token mint the rate limit is for
    pub mint: Pubkey,
    /// The value that has been transferred in the current period
    pub value_transferred: u64,
    #[cfg(test)]
    pub current_time: UnixTimestamp,
}

impl MintRateLimit {
    pub fn derive_pda(mint: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"mint_based",
                mint.as_ref(),
            ],
            &crate::ID
        )
    }
    pub fn space() -> usize {
        8 //discriminator
        + 8 // period_limit
        + 8 // current_period_start   
        + 8 // period_duration
        + 32 // mint
        + 8 // value_transferred
    }
    pub fn initialize(&mut self, period_limit: u64, period_duration: u64, start_time: UnixTimestamp, mint: Pubkey) -> Result<()> {
        let rate_limit = Self::new(period_limit, period_duration, start_time, mint)?;
        *self = rate_limit;

        Ok(())
    }
    // Add method to update current time (for testing)
    #[cfg(test)]
    pub fn set_current_time(&mut self, time: UnixTimestamp) {
        self.current_time = time;
    }
    fn new(period_limit: u64, period_duration: u64, start_time: UnixTimestamp, mint: Pubkey) -> Result<Self> {
        require!(period_duration > 0, ErrorCode::InvalidPeriodConfig);
        #[cfg(test)]
        return Ok(Self {
            period_limit,
            current_period_start: start_time,
            period_duration,
            mint,
            current_time: 0,
            value_transferred: 0,
        });

        #[cfg(not(test))]
        return Ok(Self{
            period_limit,
            current_period_start: start_time,
            mint,
            period_duration,
            value_transferred: 0,
        });
    }
}

impl RateLimitExt for MintRateLimit {
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

            // reset the value transferred
            self.value_transferred = 0;
        }
    }

    fn check_and_update(&mut self, _authority: Option<Pubkey>, amount: u64) -> Result<()> {
        // First check if we need to roll over to a new period
        self.roll_over();

        let period_limit = self.period_limit;


        let new_value_transferred = self.value_transferred.saturating_add(amount);

        // Check if the transfer would exceed the period limit
        if new_value_transferred > period_limit {
            return err!(ErrorCode::RateLimitExceeded);
        }

        self.value_transferred = new_value_transferred;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let start_time = 1000;
        let mut rate_limit = MintRateLimit {
            period_duration: 0,
            period_limit: 0,
            current_period_start: 0,
            mint: Default::default(),
            value_transferred: 0,
            current_time: 0,
        };
        rate_limit.initialize(100, 3600, start_time, Default::default()).unwrap(); // 100 tokens per hour
        rate_limit.set_current_time(start_time + 1);

        let authority = Pubkey::new_unique();

        // First transfer should work
        assert!(rate_limit.check_and_update(None, 50).is_ok());
        
        // Second transfer that would exceed limit should fail
        assert!(rate_limit.check_and_update(None, 51).is_err());
        
        // Small transfer still within limits should work
        assert!(rate_limit.check_and_update(None, 40).is_ok());
    }

    #[test]
    fn test_period_rollover() {
        let start_time = 1000;
        let mut rate_limit = MintRateLimit {
            period_duration: 0,
            period_limit: 0,
            current_period_start: 0,
            mint: Default::default(),
            value_transferred: 0,
            current_time: 0,
        };
        rate_limit.initialize(100, 3600, start_time, Default::default()).unwrap(); // 100 tokens per hour
        rate_limit.set_current_time(start_time + 1);

        // Use up the limit
        assert!(rate_limit.check_and_update(None, 100).is_ok());
        
        rate_limit.set_current_time(rate_limit.current_time+3600);


        // This should trigger a rollover and reset the limits
        rate_limit.roll_over();
        
        // Should be able to transfer again
        assert!(rate_limit.check_and_update(None, 100).is_ok());


        rate_limit.set_current_time(rate_limit.current_time+9600);

        rate_limit.roll_over();;

        assert_eq!(rate_limit.current_period_start, 11800);

    }
}