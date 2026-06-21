// TODO: follow WebDriverError

use std::io;

use async_tungstenite::tungstenite;
use webdriver_traits::bidi::{ErrorCode, ErrorResponse};

pub(crate) type BidiResult<T> = Result<T, BidiError>;

pub(crate) struct BidiError {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    pub(crate) stacktrace: Option<String>,
}

impl Default for BidiError {
    fn default() -> Self {
        Self {
            code: ErrorCode::UnknownError,
            message: Default::default(),
            stacktrace: Default::default(),
        }
    }
}

impl From<ErrorCode> for BidiError {
    fn from(error: ErrorCode) -> Self {
        Self {
            code: error,
            ..Default::default()
        }
    }
}

impl From<ipc_channel::IpcError> for BidiError {
    fn from(value: ipc_channel::IpcError) -> Self {
        ErrorCode::UnknownError.into()
    }
}

// === Error Old ===

/// This is basically a mirror of ruustenium `ErrorResponse` except `id`.
#[derive(Debug)]
pub struct WebDriverBidiError {
    pub error: ErrorCode,
    pub message: String,
    pub stacktrace: Option<String>,
}

impl WebDriverBidiError {
    pub fn into_response(self, id: Option<u64>) -> ErrorResponse {
        let Self {
            error,
            message,
            stacktrace,
        } = self;
        ErrorResponse {
            id,
            error,
            message,
            stacktrace,
            extensible: Default::default(),
        }
    }

    pub fn new(error: ErrorCode, message: impl ToString) -> Self {
        Self {
            error,
            message: message.to_string(),
            stacktrace: None,
        }
    }

    /// Convenience constructor to create `"unknown error"` error.
    ///
    /// Most internal errors are mapped to `"unknown error"`.
    pub fn unknown(message: impl ToString) -> Self {
        Self::new(ErrorCode::UnknownError, message.to_string())
    }
}

macro_rules! impl_from {
    ($err:path, $code:tt) => {
        impl From<$err> for WebDriverBidiError {
            fn from(value: $err) -> Self {
                Self::new(ErrorCode::$code, value)
            }
        }
    };
}

impl_from!(serde_json::Error, InvalidArgument);

impl_from!(io::Error, UnknownError);
impl_from!(tungstenite::Error, UnknownError);
impl_from!(Box<dyn ::core::error::Error>, UnknownError);
