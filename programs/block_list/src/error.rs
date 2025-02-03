use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Blocklist has no more room")]
    BlockListFull,
    #[msg("Unauthorized signer")]
    Unauthorized,
    #[msg("Authority is denied")]
    Denied,
    #[msg("Provided account meta list account is invalid")]
    InvalidExtraAccountMetasList,
}
