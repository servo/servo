/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

fn main() {
    cfg_aliases::cfg_aliases! {
        // Platforms
        android: { target_os = "android" },
        macos: { target_os = "macos" },
        ios: { target_os = "ios" },
        // windows: { target_os = "windows" },
        apple: { any(target_os = "ios", target_os = "macos") },
        linux: { all(unix, not(apple), not(android)) },
    }
}
