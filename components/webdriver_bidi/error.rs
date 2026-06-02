#[derive(Debug)]
pub enum Error {
    Serialize(serde_json::Error),
    Transport(async_tungstenite::tungstenite::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Serialize(error) => write!(f, "fail to serialize: {}", error),
            Error::Transport(error) => write!(f, "fail to transport: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Serialize(error) => Some(error),
            Error::Transport(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value)
    }
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(value: async_tungstenite::tungstenite::Error) -> Self {
        Self::Transport(value)
    }
}
