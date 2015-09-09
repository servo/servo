/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::opts;
use util::str::DOMString;

use std::borrow::ToOwned;

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

#[cfg(all(any(target_os = "android", target_os = "Linux"), target_arch = "x86_64"))]
pub fn Platform() -> DOMString {
    "Linux x86_64".to_owned()
}

#[cfg(all(any(target_os = "android", target_os = "Linux"), target_arch = "i686"))]
pub fn Platform() -> DOMString {
    "Linux i686".to_owned()
}

#[cfg(all(any(target_os = "android", target_os = "Linux"), target_arch = "arm`"))]
pub fn Platform() -> DOMString {
    // Assuming v7
    "Linux armv7l".to_owned()
}

#[cfg(all(any(target_os = "android", target_os = "Linux"), target_arch = "aarch64"))]
pub fn Platform() -> DOMString {
    "Linux aarch64".to_owned()
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub fn Platform() -> DOMString {
    "MacIntel".to_owned()
}

pub fn UserAgent() -> DOMString {
    opts::get().user_agent.clone()
}

pub fn AppVersion() -> DOMString {
    "4.0".to_owned()
}
