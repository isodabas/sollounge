use anchor_lang::prelude::error_code;
#[error_code]
pub enum ClaimError {
    #[msg("Bet do not belong to the game!")]
    GameDoesntMatch,
    #[msg("Already claimed!")]
    AlreadyClaimed,
    #[msg("Game is still live!")]
    GameIsOn,
    #[msg("Your prediction was wrong!")]
    WinningSideDoesntMatch,
    #[msg("User can only claim his/her rewards!")]
    WrongSigner,
}

#[error_code]
pub enum EndError {
    #[msg("Already Ended!")]
    AlreadyEnded,
    #[msg("Choose a valid winner!")]
    InvalidWinner,
}
