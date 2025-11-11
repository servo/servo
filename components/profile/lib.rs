/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod mem;
#[cfg_attr(
    not(any(target_os = "windows", target_env = "ohos")),
    expect(unsafe_code)
)]
pub mod system_reporter;
pub mod time;
pub mod trace_dump;
