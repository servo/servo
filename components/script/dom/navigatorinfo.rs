/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use util::opts;
use util::str::DOMString;

pub fn Product() -> DOMString {
    "Gecko".to_owned()
}

pub fn TaintEnabled() -> bool {
    false
}

pub fn AppName() -> DOMString {
    "Netscape".to_owned() // Like Gecko/Webkit
}

pub fn AppCodeName() -> DOMString {
    "Mozilla".to_owned()
}

#[cfg(target_os = "windows")]
pub fn Platform() -> DOMString {
    "Win32".to_owned()
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn Platform() -> DOMString {
    "Linux".to_owned()
}

#[cfg(target_os = "macos")]
pub fn Platform() -> DOMString {
    "Mac".to_owned()
}

pub fn UserAgent() -> DOMString {
    opts::get().user_agent.clone()
}

pub fn AppVersion() -> DOMString {
    "4.0".to_owned()
}
