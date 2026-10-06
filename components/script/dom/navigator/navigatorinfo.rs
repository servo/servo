/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::RootedDomString;

#[expect(non_snake_case)]
pub(crate) fn Product() -> RootedDomString {
    RootedDomString::from_static("Gecko")
}

#[expect(non_snake_case)]
pub(crate) fn ProductSub() -> RootedDomString {
    RootedDomString::from_static("20100101")
}

#[expect(non_snake_case)]
pub(crate) fn Vendor() -> RootedDomString {
    RootedDomString::new()
}

#[expect(non_snake_case)]
pub(crate) fn VendorSub() -> RootedDomString {
    RootedDomString::new()
}

#[expect(non_snake_case)]
pub(crate) fn TaintEnabled() -> bool {
    false
}

#[expect(non_snake_case)]
pub(crate) fn AppName() -> RootedDomString {
    RootedDomString::from_static("Netscape") // Like Gecko/Webkit
}

#[expect(non_snake_case)]
pub(crate) fn AppCodeName() -> RootedDomString {
    RootedDomString::from_static("Mozilla")
}

#[expect(non_snake_case)]
#[cfg(target_os = "windows")]
pub(crate) fn Platform() -> RootedDomString {
    RootedDomString::from_static("Win32")
}

#[expect(non_snake_case)]
#[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
pub(crate) fn Platform() -> RootedDomString {
    RootedDomString::from_static("Linux")
}

#[expect(non_snake_case)]
#[cfg(target_os = "macos")]
pub(crate) fn Platform() -> RootedDomString {
    RootedDomString::from_static("Mac")
}

#[expect(non_snake_case)]
#[cfg(target_os = "ios")]
pub(crate) fn Platform() -> RootedDomString {
    RootedDomString::from_static("iOS")
}

#[expect(non_snake_case)]
pub(crate) fn UserAgent(user_agent: &str) -> RootedDomString {
    RootedDomString::from(user_agent)
}

#[expect(non_snake_case)]
pub(crate) fn AppVersion() -> RootedDomString {
    RootedDomString::from_static("4.0")
}

#[expect(non_snake_case)]
pub(crate) fn Language() -> RootedDomString {
    RootedDomString::from(net_traits::get_current_locale().0.clone())
}
