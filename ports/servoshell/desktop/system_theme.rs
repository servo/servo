/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Desktop color-scheme detection for Linux via the XDG settings portal.
//!
//! winit has no theme detection on Linux: the X11 backend returns `None`
//! unconditionally, and the Wayland backend returns only the theme the
//! application itself set for its client-side decorations. The desktop's actual
//! preference lives behind the `org.freedesktop.appearance` portal setting, so
//! read it directly.
//!
//! The value is read once and cached. Servo therefore picks up the preference
//! in effect at startup but does not follow a mid-session change; doing that
//! means subscribing to the portal's `SettingChanged` signal and routing it into
//! `WebView::notify_theme_change`.

#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]

use std::sync::OnceLock;

use servo::Theme;

/// Values of the `org.freedesktop.appearance` `color-scheme` setting.
/// <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html>
const COLOR_SCHEME_NO_PREFERENCE: u32 = 0;
const COLOR_SCHEME_PREFER_DARK: u32 = 1;
const COLOR_SCHEME_PREFER_LIGHT: u32 = 2;

/// The desktop's preferred [`Theme`], or `None` when there is no preference or
/// no portal to ask. Cached after the first call.
pub fn preferred_theme() -> Option<Theme> {
    static CACHED: OnceLock<Option<Theme>> = OnceLock::new();
    *CACHED.get_or_init(query_portal)
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn query_portal() -> Option<Theme> {
    let connection = match zbus::blocking::Connection::session() {
        Ok(connection) => connection,
        Err(error) => {
            // No session bus at all (headless CI, a stripped container). Not
            // worth a warning — there is simply nothing to ask.
            log::debug!("No session bus, cannot read desktop color-scheme: {error}");
            return None;
        },
    };

    // `ReadOne` is the current call; `Read` is deprecated but is all that older
    // portals implement. Both return the value wrapped in a variant.
    let scheme =
        read_setting(&connection, "ReadOne").or_else(|| read_setting(&connection, "Read"))?;

    match scheme {
        COLOR_SCHEME_PREFER_DARK => Some(Theme::Dark),
        COLOR_SCHEME_PREFER_LIGHT => Some(Theme::Light),
        // `NO_PREFERENCE` is a real answer meaning "don't care" — fall back to
        // the caller's default rather than inventing a preference.
        COLOR_SCHEME_NO_PREFERENCE => None,
        other => {
            log::warn!("Unknown org.freedesktop.appearance color-scheme value: {other}");
            None
        },
    }
}

#[cfg(not(all(target_os = "linux", not(target_env = "ohos"))))]
fn query_portal() -> Option<Theme> {
    None
}

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn read_setting(connection: &zbus::blocking::Connection, method: &str) -> Option<u32> {
    let reply = connection
        .call_method(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.portal.Settings"),
            method,
            &("org.freedesktop.appearance", "color-scheme"),
        )
        .ok()?;

    let body = reply.body();
    let value = body.deserialize::<zbus::zvariant::Value>().ok()?;
    unwrap_u32(&value)
}

/// `ReadOne` yields the `u32` under one variant; the deprecated `Read` nests it
/// under two. Peel variants until a `u32` appears rather than assuming a depth.
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
fn unwrap_u32(value: &zbus::zvariant::Value) -> Option<u32> {
    match value {
        zbus::zvariant::Value::U32(scheme) => Some(*scheme),
        zbus::zvariant::Value::Value(inner) => unwrap_u32(inner),
        _ => None,
    }
}
