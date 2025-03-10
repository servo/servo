/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
use std::path::{Path, PathBuf};

use servo::net_traits::pub_domains::is_reg_domain;
use servo::servo_url::ServoUrl;

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn parse_url_or_filename(cwd: &Path, input: &str) -> Result<ServoUrl, ()> {
    match ServoUrl::parse(input) {
        Ok(url) => Ok(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            url::Url::from_file_path(&*cwd.join(input)).map(ServoUrl::from_url)
        },
        Err(_) => Err(()),
    }
}

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn get_default_url(
    url_opt: Option<&str>,
    cwd: impl AsRef<Path>,
    exists: impl FnOnce(&PathBuf) -> bool,
    preferences: &crate::prefs::ServoShellPreferences,
) -> ServoUrl {
    // If the url is not provided, we fallback to the homepage in prefs,
    // or a blank page in case the homepage is not set either.
    let mut new_url = None;
    let cmdline_url = url_opt.map(|s| s.to_string()).and_then(|url_string| {
        parse_url_or_filename(cwd.as_ref(), &url_string)
            .inspect_err(|&error| {
                log::warn!("URL parsing failed ({:?}).", error);
            })
            .ok()
    });

    if let Some(url) = cmdline_url.clone() {
        // Check if the URL path corresponds to a file
        match (url.scheme(), url.host(), url.to_file_path()) {
            ("file", None, Ok(ref path)) if exists(path) => {
                new_url = cmdline_url;
            },
            _ => {},
        }
    }

    if new_url.is_none() && url_opt.is_some() {
        new_url = location_bar_input_to_url(url_opt.unwrap(), &preferences.searchpage);
    }

    let pref_url = parse_url_or_filename(cwd.as_ref(), &preferences.homepage).ok();
    let blank_url = ServoUrl::parse("about:blank").ok();

    new_url.or(pref_url).or(blank_url).unwrap()
}

/// Interpret an input URL.
///
/// If this is not a valid URL, try to "fix" it by adding a scheme or if all else fails,
/// interpret the string as a search term.
pub(crate) fn location_bar_input_to_url(request: &str, searchpage: &str) -> Option<ServoUrl> {
    let request = request.trim();
    ServoUrl::parse(request)
        .ok()
        .or_else(|| try_as_file(request))
        .or_else(|| try_as_domain(request))
        .or_else(|| try_as_search_page(request, searchpage))
}

fn try_as_file(request: &str) -> Option<ServoUrl> {
    if request.starts_with('/') {
        return ServoUrl::parse(&format!("file://{}", request)).ok();
    }
    None
}

fn try_as_domain(request: &str) -> Option<ServoUrl> {
    fn is_domain_like(s: &str) -> bool {
        !s.starts_with('/') && s.contains('/') ||
            (!s.contains(' ') && !s.starts_with('.') && s.split('.').count() > 1)
    }

    if !request.contains(' ') && is_reg_domain(request) || is_domain_like(request) {
        return ServoUrl::parse(&format!("https://{}", request)).ok();
    }
    None
}

fn try_as_search_page(request: &str, searchpage: &str) -> Option<ServoUrl> {
    if request.is_empty() {
        return None;
    }
    ServoUrl::parse(&searchpage.replace("%s", request)).ok()
}
