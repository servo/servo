/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;

#[expect(non_snake_case)]
pub(crate) fn Product() -> DOMString {
    DOMString::from_static("Gecko")
}

#[expect(non_snake_case)]
pub(crate) fn ProductSub() -> DOMString {
    DOMString::from_static("20100101")
}

#[expect(non_snake_case)]
pub(crate) fn Vendor() -> DOMString {
    DOMString::new()
}

#[expect(non_snake_case)]
pub(crate) fn VendorSub() -> DOMString {
    DOMString::new()
}

#[expect(non_snake_case)]
pub(crate) fn TaintEnabled() -> bool {
    false
}

#[expect(non_snake_case)]
pub(crate) fn AppName() -> DOMString {
    DOMString::from_static("Netscape") // Like Gecko/Webkit
}

#[expect(non_snake_case)]
pub(crate) fn AppCodeName() -> DOMString {
    DOMString::from_static("Mozilla")
}

#[expect(non_snake_case)]
#[cfg(target_os = "windows")]
pub(crate) fn Platform() -> DOMString {
    DOMString::from_static("Win32")
}

#[expect(non_snake_case)]
#[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
pub(crate) fn Platform() -> DOMString {
    DOMString::from_static("Linux")
}

#[expect(non_snake_case)]
#[cfg(target_os = "macos")]
pub(crate) fn Platform() -> DOMString {
    DOMString::from_static("Mac")
}

#[expect(non_snake_case)]
#[cfg(target_os = "ios")]
pub(crate) fn Platform() -> DOMString {
    DOMString::from_static("iOS")
}

#[expect(non_snake_case)]
pub(crate) fn UserAgent(user_agent: &str) -> DOMString {
    DOMString::from(user_agent)
}

#[expect(non_snake_case)]
pub(crate) fn AppVersion() -> DOMString {
    // Firefox intentionally reduces appVersion to a platform hint (for
    // example "5.0 (Windows)") rather than echoing the full user agent.
    // Legacy download pages parse this to decide which installer to offer.
    let platform = match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "Macintosh",
        "android" => "Android",
        "ios" => "iPhone",
        _ => "X11",
    };
    DOMString::from(format!("5.0 ({platform})"))
}

#[expect(non_snake_case)]
pub(crate) fn Oscpu() -> DOMString {
    // Keep these values in sync with Firefox's `navigator.oscpu` so that
    // Firefox-targeted OS detection keeps working.
    let oscpu = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64") {
            "Windows NT 10.0; Win64; x64"
        } else {
            "Windows NT 10.0"
        }
    } else if cfg!(target_os = "macos") {
        "Intel Mac OS X 10.15"
    } else if cfg!(target_os = "android") {
        "Linux armv81"
    } else if cfg!(target_os = "ios") {
        "iPhone"
    } else {
        "Linux x86_64"
    };
    DOMString::from_static(oscpu)
}

#[expect(non_snake_case)]
pub(crate) fn Language() -> DOMString {
    DOMString::from(net_traits::get_current_locale().0.clone())
}
