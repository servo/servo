/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Error scopes and GPUError types

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::wgc;

/// <https://www.w3.org/TR/webgpu/#gpu-error-scope>
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ErrorScope {
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

    // TODO: labels
    // based on https://github.com/gfx-rs/wgpu/blob/trunk/wgpu/src/backend/wgpu_core.rs#L289
    pub fn from_error<E: std::error::Error + 'static>(error: E) -> Self {
        let mut source_opt: Option<&(dyn std::error::Error + 'static)> = Some(&error);
        while let Some(source) = source_opt {
            if let Some(wgc::device::DeviceError::OutOfMemory) =
                source.downcast_ref::<wgc::device::DeviceError>()
            {
                return Self::OutOfMemory(error.to_string());
            }
            source_opt = source.source();
        }
        // TODO: This hack is needed because there are
        // multiple OutOfMemory error variant in wgpu-core
        // and even upstream does not handle them correctly
        if format!("{error:?}").contains("OutOfMemory") {
            return Self::OutOfMemory(error.to_string());
        }
        Self::Validation(error.to_string())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PopError {
    Lost,
    Empty,
}
