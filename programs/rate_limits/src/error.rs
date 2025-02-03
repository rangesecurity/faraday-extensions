use anchor_lang::prelude::*;

#[error_code]
pub enum RateLimitError {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Unauthorized signer")]
    Unauthorized,
    #[msg("Transfer would exceed rate limit for this period")]
    RateLimitExceeded,
    #[msg("Invalid period configuration")]
    InvalidPeriodConfig,
    #[msg("Invalid check and update")]
    InvalidCheckAndUpdate,
    #[msg("Invalid rate limit account provided")]
    InvalidRateLimitAccount,
    #[msg("Invalid rate limit type provided")]
    InvalidRateLimitType,
    #[msg("Provided account meta list account is invalid")]
    InvalidExtraAccountMetasList,
}
