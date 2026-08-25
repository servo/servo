/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};

/// Errors that can be produced by XR.

// TODO: this is currently incomplete!

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    NoMatchingDevice,
    CommunicationError,
    ThreadCreationError,
    InlineSession,
    UnsupportedFeature(String),
    BackendSpecific(String),
}
