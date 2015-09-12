/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;
use util::opts::parse_url_or_filename;

#[test]
fn test_argument_parsing() {
    let fake_cwd = Path::new("/fake/cwd");
    assert!(parse_url_or_filename(fake_cwd, "http://example.net:invalid").is_err());

    let url = parse_url_or_filename(fake_cwd, "http://example.net").unwrap();
    assert_eq!(url.scheme, "http");

    let url = parse_url_or_filename(fake_cwd, "file:///foo/bar.html").unwrap();
    assert_eq!(url.scheme, "file");
    assert_eq!(url.path().unwrap(), ["foo", "bar.html"]);

    let url = parse_url_or_filename(fake_cwd, "bar.html").unwrap();
    assert_eq!(url.scheme, "file");
    assert_eq!(url.path().unwrap(), ["fake", "cwd", "bar.html"]);

    // '?' and '#' have a special meaning in URLs...
    let url = parse_url_or_filename(fake_cwd, "file:///foo/bar?baz#buzz.html").unwrap();
    assert_eq!(&*url.to_file_path().unwrap(), Path::new("/foo/bar"));
    assert_eq!(url.scheme, "file");
    assert_eq!(url.path().unwrap(), ["foo", "bar"]);
    assert_eq!(url.query.unwrap(), "baz");
    assert_eq!(url.fragment.unwrap(), "buzz.html");

    // but not in file names.
    let url = parse_url_or_filename(fake_cwd, "./bar?baz#buzz.html").unwrap();
    assert_eq!(&*url.to_file_path().unwrap(), Path::new("/fake/cwd/bar?baz#buzz.html"));
    assert_eq!(url.scheme, "file");
    assert_eq!(url.path().unwrap(), ["fake", "cwd", "bar%3Fbaz%23buzz.html"]);
    assert!(url.query.is_none());
    assert!(url.fragment.is_none());
}
