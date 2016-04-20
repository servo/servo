/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::chrome_loader::resolve_chrome_url;
use url::Url;

#[test]
fn test_relative() {
    let url = Url::parse("chrome://../something").unwrap();
    assert!(resolve_chrome_url(&url).is_err());
}

#[test]
fn test_relative_2() {
    let url = Url::parse("chrome://subdir/../something").unwrap();
    assert!(resolve_chrome_url(&url).is_err());
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_absolute() {
    let url = Url::parse("chrome:///etc/passwd").unwrap();
    assert!(resolve_chrome_url(&url).is_err());
}

#[test]
#[cfg(target_os = "windows")]
fn test_absolute_2() {
    let url = Url::parse("chrome://C:\\Windows").unwrap();
    assert!(resolve_chrome_url(&url).is_err());
}

#[test]
#[cfg(target_os = "windows")]
fn test_absolute_3() {
    let url = Url::parse("chrome://\\\\server/C$").unwrap();
    assert!(resolve_chrome_url(&url).is_err());
}

#[test]
fn test_valid() {
    let url = Url::parse("chrome://badcert.jpg").unwrap();
    let resolved = resolve_chrome_url(&url).unwrap();
    assert_eq!(resolved.scheme, "file");
}
