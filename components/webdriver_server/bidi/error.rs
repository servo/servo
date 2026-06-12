// TODO: follow WebDriverError

use std::io;

use async_tungstenite::tungstenite;
use embedder_traits::webdriver_bidi::WebDriverBidiToEmbedderMessage;
use webdriver_traits::bidi::{ErrorCode, ErrorResponse};

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
impl_from!(
    crossbeam_channel::SendError<WebDriverBidiToEmbedderMessage>,
    UnknownError
);
