/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::ops::Deref;

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

/// A thin wrapper around [`http::StatusCode`]. Default value is OK
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct HttpStatus(
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    http::StatusCode,
);

impl Deref for HttpStatus {
    type Target = http::StatusCode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for HttpStatus {
    fn default() -> Self {
        Self(http::StatusCode::OK)
    }
}

impl PartialEq<http::StatusCode> for HttpStatus {
    fn eq(&self, other: &http::StatusCode) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<HttpStatus> for http::StatusCode {
    fn eq(&self, other: &HttpStatus) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<http::StatusCode> for &HttpStatus {
    fn eq(&self, other: &http::StatusCode) -> bool {
        self.0.eq(other)
    }
}

impl From<http::StatusCode> for HttpStatus {
    fn from(value: http::StatusCode) -> Self {
        HttpStatus(value)
    }
}
