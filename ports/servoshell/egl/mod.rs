/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "android")]
mod android;

#[cfg(target_env = "ohos")]
mod ohos;

mod log;

mod app_state;
mod host_trait;
