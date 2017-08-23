/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::DOMString;
use servo_config::opts;

pub fn Product() -> DOMString {
    DOMString::from("Gecko")
}

pub fn TaintEnabled() -> bool {
    false
}

pub fn AppName() -> DOMString {
    DOMString::from("Netscape") // Like Gecko/Webkit
}

pub fn AppCodeName() -> DOMString {
    DOMString::from("Mozilla")
}

#[cfg(target_os = "windows")]
pub fn Platform() -> DOMString {
    DOMString::from("Win32")
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn Platform() -> DOMString {
    DOMString::from("Linux")
}

#[cfg(target_os = "macos")]
pub fn Platform() -> DOMString {
    DOMString::from("Mac")
}

#[cfg(target_os = "ios")]
pub fn Platform() -> DOMString {
    DOMString::from("iOS")
}

pub fn UserAgent() -> DOMString {
    DOMString::from(&*opts::get().user_agent)
}

pub fn AppVersion() -> DOMString {
    DOMString::from("4.0")
}

pub fn Language() -> DOMString {
    DOMString::from("en-US")
}
