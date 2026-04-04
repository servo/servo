/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
pub mod avplayer;
#[cfg(not(sdk_api_21))]
pub mod dummy_source;
#[cfg(sdk_api_21)]
pub mod source;
pub mod source_builder;
