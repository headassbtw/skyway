pub enum BlueskyLoginResponseInfo {
    WasntLoggedIn,
    TwoFactorTokenRequired,
}

pub enum BlueskyLoginResponseError {
    Generic(String),
    Network(String),
    InvalidRequest,
    ExpiredToken,
    InvalidToken,
    AccountTakenDown,
    AccountSuspended,
    AccountInactive,
    AccountDeactivated,
    Unauthorized,
}

pub enum BlueskyLoginResponse {
    /// DID then refresh token
    Success(String, String),
    Info(BlueskyLoginResponseInfo),
    Error(BlueskyLoginResponseError),
}
