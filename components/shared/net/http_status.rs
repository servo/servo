/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::ops::Deref;

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

/// A thin wrapper around [`http::StatusCode`] with a custom message. Default value is OK
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct HttpStatus {
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    code: http::StatusCode,
    /// Custom message. This can be different than canonical_reason
    message: Vec<u8>,
}

impl HttpStatus {
    /// Creates a new [`HttpStatus`] with a potential message
    pub fn try_new(code: u16, message: Option<Vec<u8>>) -> Option<Self> {
        let code = http::StatusCode::from_u16(code).ok()?;
        Some(HttpStatus {
            code,
            message: message.unwrap_or_default(),
        })
    }

    /// The inner message will construct the canonical reason if none given.
    pub fn message(&self) -> &[u8] {
        if self.message.is_empty() {
            self.code
                .canonical_reason()
                .map(|reason| reason.as_bytes())
                .unwrap_or_default()
        } else {
            &self.message
        }
    }
}

impl Deref for HttpStatus {
    type Target = http::StatusCode;

    fn deref(&self) -> &Self::Target {
        &self.code
    }
}

impl Default for HttpStatus {
    fn default() -> Self {
        Self {
            code: http::StatusCode::OK,
            message: vec![],
        }
    }
}

impl PartialEq<http::StatusCode> for HttpStatus {
    fn eq(&self, other: &http::StatusCode) -> bool {
        self.code.eq(other)
    }
}

impl PartialEq<HttpStatus> for http::StatusCode {
    fn eq(&self, other: &HttpStatus) -> bool {
        self.eq(&other.code)
    }
}

impl PartialEq<http::StatusCode> for &HttpStatus {
    fn eq(&self, other: &http::StatusCode) -> bool {
        self.code.eq(other)
    }
}

impl From<http::StatusCode> for HttpStatus {
    fn from(value: http::StatusCode) -> Self {
        HttpStatus {
            code: value,
            message: vec![],
        }
    }
}
