use webdriver_traits::bidi::ErrorCode;

pub(crate) struct BidiError {
    pub code: ErrorCode,
    pub message: String,
    pub stacktrace: Option<String>,
}

impl From<ErrorCode> for BidiError {
    fn from(value: ErrorCode) -> Self {
        Self {
            code: value,
            message: Default::default(),
            stacktrace: Default::default(),
        }
    }
}

impl From<uuid::Error> for BidiError {
    fn from(_: uuid::Error) -> Self {
        Self {
            code: ErrorCode::InvalidArgument,
            message: "uuid is required for id".to_string(),
            stacktrace: Default::default(),
        }
    }
}
