/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::RootedDOMString;

#[expect(non_snake_case)]
pub(crate) fn Product() -> RootedDOMString {
    RootedDOMString::from_static("Gecko")
}

#[expect(non_snake_case)]
pub(crate) fn ProductSub() -> RootedDOMString {
    RootedDOMString::from_static("20100101")
}

#[expect(non_snake_case)]
pub(crate) fn Vendor() -> RootedDOMString {
    RootedDOMString::new()
}

#[expect(non_snake_case)]
pub(crate) fn VendorSub() -> RootedDOMString {
    RootedDOMString::new()
}

#[expect(non_snake_case)]
pub(crate) fn TaintEnabled() -> bool {
    false
}

#[expect(non_snake_case)]
pub(crate) fn AppName() -> RootedDOMString {
    RootedDOMString::from_static("Netscape") // Like Gecko/Webkit
}

#[expect(non_snake_case)]
pub(crate) fn AppCodeName() -> RootedDOMString {
    RootedDOMString::from_static("Mozilla")
}

#[expect(non_snake_case)]
#[cfg(target_os = "windows")]
pub(crate) fn Platform() -> RootedDOMString {
    RootedDOMString::from_static("Win32")
}

#[expect(non_snake_case)]
#[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
pub(crate) fn Platform() -> RootedDOMString {
    RootedDOMString::from_static("Linux")
}

#[expect(non_snake_case)]
#[cfg(target_os = "macos")]
pub(crate) fn Platform() -> RootedDOMString {
    RootedDOMString::from_static("Mac")
}

#[expect(non_snake_case)]
#[cfg(target_os = "ios")]
pub(crate) fn Platform() -> RootedDOMString {
    RootedDOMString::from_static("iOS")
}

#[expect(non_snake_case)]
pub(crate) fn UserAgent(user_agent: &str) -> RootedDOMString {
    RootedDOMString::from(user_agent)
}

#[expect(non_snake_case)]
pub(crate) fn AppVersion() -> RootedDOMString {
    RootedDOMString::from_static("4.0")
}

#[expect(non_snake_case)]
pub(crate) fn Language() -> RootedDOMString {
    RootedDOMString::from(net_traits::get_current_locale().0.clone())
}
