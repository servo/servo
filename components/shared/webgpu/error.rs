/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Error scopes and GPUError types

use std::fmt;

use serde::{Deserialize, Serialize};
use wgpu_types::error::{ErrorType, WebGpuError};

/// <https://www.w3.org/TR/webgpu/#gpu-error-scope>
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ErrorScope {
    pub errors: Vec<Error>,
    pub filter: ErrorFilter,
}

impl ErrorScope {
    pub fn new(filter: ErrorFilter) -> Self {
        Self {
            filter,
            errors: Vec::new(),
        }
    }
}

/// <https://www.w3.org/TR/webgpu/#enumdef-gpuerrorfilter>
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ErrorFilter {
    Validation,
    OutOfMemory,
    Internal,
}

/// <https://www.w3.org/TR/webgpu/#gpuerror>
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Error {
    Validation(String),
    OutOfMemory(String),
    Internal(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl Error {
    pub fn filter(&self) -> ErrorFilter {
        match self {
            Error::Validation(_) => ErrorFilter::Validation,
            Error::OutOfMemory(_) => ErrorFilter::OutOfMemory,
            Error::Internal(_) => ErrorFilter::Internal,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Error::Validation(m) => m,
            Error::OutOfMemory(m) => m,
            Error::Internal(m) => m,
        }
    }

    /// Converts a wgpu's errors into a (GPU)Error if possible.
    /// Returns `None` if device is lost,
    /// as those need to be ignored in validation as they are handled by device lost callback
    pub fn from_wgpu_error<E: WebGpuError>(error: E) -> Option<Self> {
        match error.webgpu_error_type() {
            ErrorType::Validation => Some(Self::Validation(error.to_string())),
            ErrorType::OutOfMemory => Some(Self::OutOfMemory(error.to_string())),
            ErrorType::Internal => Some(Self::Internal(error.to_string())),
            ErrorType::DeviceLost => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PopError {
    Lost,
    Empty,
}
