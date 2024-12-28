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

pub struct LoginInformation {
    pub did: String,
    pub refresh_token: String,
}

pub enum BlueskyLoginResponse {
    Success(LoginInformation),
    Info(BlueskyLoginResponseInfo),
    Error(BlueskyLoginResponseError),
}
