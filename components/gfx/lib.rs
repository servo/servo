/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

pub mod font;
pub mod font_cache_thread;
pub mod font_context;
pub mod font_store;
pub mod font_template;
#[allow(unsafe_code)]
pub mod platform;
pub mod rendering_context;
pub mod text;
