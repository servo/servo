/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod body;
#[allow(
    clippy::module_inception,
    reason = "Will be refactored later after file move"
)]
pub(crate) mod fetch;
mod mime_multipart;
pub(crate) mod network_listener;
