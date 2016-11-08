/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::test::resolve_chrome_url;
use url::Url;

fn c(s: &str) -> Result<Url, ()> {
    resolve_chrome_url(&Url::parse(s).unwrap())
}

#[test]
fn test_resolve_chrome_url() {
    assert_eq!(c("chrome://resources/nonexistent.jpg"), Err(()));
    assert_eq!(c("chrome://not-resources/badcert.jpg"), Err(()));
    assert_eq!(c("chrome://resources/badcert.jpg").unwrap().scheme(), "file");
    assert_eq!(c("chrome://resources/subdir/../badcert.jpg").unwrap().scheme(), "file");
    assert_eq!(c("chrome://resources/subdir/../../badcert.jpg").unwrap().scheme(), "file");
    assert_eq!(c("chrome://resources/../badcert.jpg").unwrap().scheme(), "file");
    assert_eq!(c("chrome://resources/../README.md"), Err(()));
    assert_eq!(c("chrome://resources/%2e%2e/README.md"), Err(()));

    assert_eq!(c("chrome://resources/etc/passwd"), Err(()));
    assert_eq!(c("chrome://resources//etc/passwd"), Err(()));
    assert_eq!(c("chrome://resources/%2Fetc%2Fpasswd"), Err(()));

    assert_eq!(c("chrome://resources/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources/C:\\Windows\\notepad.exe"), Err(()));

    assert_eq!(c("chrome://resources/localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources//localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources///localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources/\\\\localhost\\C:\\Windows\\notepad.exe"), Err(()));

    assert_eq!(c("chrome://resources/%3F/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources//%3F/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources///%3F/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources/\\\\%3F\\C:\\Windows\\notepad.exe"), Err(()));

    assert_eq!(c("chrome://resources/%3F/UNC/localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources//%3F/UNC/localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources///%3F/UNC/localhost/C:/Windows/notepad.exe"), Err(()));
    assert_eq!(c("chrome://resources/\\\\%3F\\UNC\\localhost\\C:\\Windows\\notepad.exe"), Err(()));
}
