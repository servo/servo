/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::ops::RangeBounds;

use serde::{Deserialize, Serialize};

use crate::{MallocSizeOf, StatusCode};

/// A representation of a HTTP Status Code and Message that can be used for
/// DOM Response objects and other cases.
/// These objects are immutable once created.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct HttpStatus {
    code: u16,
    message: Vec<u8>,
}

impl HttpStatus {
    /// Creates a new HttpStatus for a valid status code.
    pub fn new(code: StatusCode, message: Vec<u8>) -> Self {
        Self {
            code: code.as_u16(),
            message,
        }
    }

    /// Creates a new HttpStatus from a raw status code, but will panic
    /// if the code is not in the 100 to 599 valid range.
    pub fn new_raw(code: u16, message: Vec<u8>) -> Self {
        if !(100..=599).contains(&code) {
            panic!(
                "HttpStatus code must be in the range 100 to 599, inclusive, but is {}",
                code
            );
        }

        Self { code, message }
    }

    /// Creates an instance that represents a Response.error() instance.
    pub fn new_error() -> Self {
        Self {
            code: 0,
            message: vec![],
        }
    }

    /// Returns the StatusCode for non-error cases, panics otherwise.
    pub fn code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).expect("HttpStatus code is 0, can't return a StatusCode")
    }

    /// Returns the StatusCode if not an error instance, or None otherwise.
    pub fn try_code(&self) -> Option<StatusCode> {
        StatusCode::from_u16(self.code).ok()
    }

    /// Returns the u16 representation of the access code. This is usable both for
    /// valid HTTP status codes and in the error case.
    pub fn raw_code(&self) -> u16 {
        self.code
    }

    /// Get access to a reference of the message part.
    pub fn message(&self) -> &[u8] {
        &self.message
    }

    /// Helper that relays is_success() from the underlying code.
    pub fn is_success(&self) -> bool {
        StatusCode::from_u16(self.code).is_ok_and(|s| s.is_success())
    }

    /// True when the object was created with `new_error`.
    pub fn is_error(&self) -> bool {
        self.code == 0
    }

    /// Returns true if this status is in the given range.
    /// Always return false for error statuses.
    pub fn in_range<T: RangeBounds<u16>>(&self, range: T) -> bool {
        self.code != 0 && range.contains(&self.code)
    }
}

impl Default for HttpStatus {
    /// The default implementation creates a "200 OK" response.
    fn default() -> Self {
        Self {
            code: 200,
            message: b"OK".to_vec(),
        }
    }
}

impl PartialEq<StatusCode> for HttpStatus {
    fn eq(&self, other: &StatusCode) -> bool {
        self.code == other.as_u16()
    }
}

impl From<StatusCode> for HttpStatus {
    fn from(code: StatusCode) -> Self {
        Self {
            code: code.as_u16(),
            message: code
                .canonical_reason()
                .unwrap_or_default()
                .as_bytes()
                .to_vec(),
        }
    }
}
