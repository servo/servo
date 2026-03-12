/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "windows", target_os = "macos"))]
static COMMON_PLUGINS: &[&str] = &include!("gstreamer_plugin_lists/common.rs.in");
#[cfg(target_os = "windows")]
static WINDOWS_PLUGINS: &[&str] = &include!("gstreamer_plugin_lists/windows.rs.in");
#[cfg(target_os = "macos")]
static MACOS_PLUGINS: &[&str] = &include!("gstreamer_plugin_lists/macos.rs.in");

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(crate) fn gstreamer_plugins() -> Vec<String> {
    let mut plugins = Vec::from(COMMON_PLUGINS);
    #[cfg(target_os = "windows")]
    plugins.extend_from_slice(WINDOWS_PLUGINS);
    #[cfg(target_os = "macos")]
    plugins.extend_from_slice(MACOS_PLUGINS);

    let (prefix, suffix) = if cfg!(target_os = "windows") {
        ("", ".dll")
    } else if cfg!(target_os = "macos") {
        ("lib", ".dylib")
    } else {
        unreachable!("This function is only for macOS and Windows.")
    };

    plugins
        .iter()
        .map(|basename| format!("{prefix}{basename}{suffix}"))
        .collect()
}
