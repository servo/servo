/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

use crate::parser::{get_default_url, location_bar_input_to_url, parse_url_or_filename};

#[cfg(not(target_os = "windows"))]
const FAKE_CWD: &str = "/fake/cwd";

#[cfg(target_os = "windows")]
const FAKE_CWD: &'static str = "C:/fake/cwd";

#[test]
fn test_argument_parsing() {
    let fake_cwd = Path::new(FAKE_CWD);
    assert!(parse_url_or_filename(fake_cwd, "http://example.net:invalid").is_err());

    let url = parse_url_or_filename(fake_cwd, "http://example.net").unwrap();
    assert_eq!(url.scheme(), "http");

    let url = parse_url_or_filename(fake_cwd, "file:///foo/bar.html").unwrap();
    assert_eq!(url.scheme(), "file");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["foo", "bar.html"]
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_file_path_parsing() {
    let fake_cwd = Path::new(FAKE_CWD);

    let url = parse_url_or_filename(fake_cwd, "bar.html").unwrap();
    assert_eq!(url.scheme(), "file");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["fake", "cwd", "bar.html"]
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_file_path_parsing() {
    let fake_cwd = Path::new(FAKE_CWD);

    let url = parse_url_or_filename(fake_cwd, "bar.html").unwrap();
    assert_eq!(url.scheme(), "file");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["C:", "fake", "cwd", "bar.html"]
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
// Windows file paths can't contain ?
fn test_argument_parsing_special() {
    let fake_cwd = Path::new(FAKE_CWD);

    // '?' and '#' have a special meaning in URLs...
    let url = parse_url_or_filename(fake_cwd, "file:///foo/bar?baz#buzz.html").unwrap();
    assert_eq!(&*url.to_file_path().unwrap(), Path::new("/foo/bar"));
    assert_eq!(url.scheme(), "file");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["foo", "bar"]
    );
    assert_eq!(url.query(), Some("baz"));
    assert_eq!(url.fragment(), Some("buzz.html"));

    // but not in file names.
    let url = parse_url_or_filename(fake_cwd, "./bar?baz#buzz.html").unwrap();
    assert_eq!(
        &*url.to_file_path().unwrap(),
        Path::new("/fake/cwd/bar?baz#buzz.html")
    );
    assert_eq!(url.scheme(), "file");
    assert_eq!(
        url.path_segments().unwrap().collect::<Vec<_>>(),
        ["fake", "cwd", "bar%3Fbaz%23buzz.html"]
    );
    assert_eq!(url.query(), None);
    assert_eq!(url.fragment(), None);
}

// Helper function to test url
fn test_url(input: &str, location: &str, cmdline_if_exists: &str, cmdline_otherwise: &str) {
    assert_eq!(
        location_bar_input_to_url(input).unwrap().into_string(),
        location
    );
    assert_eq!(
        get_default_url(Some(input), FAKE_CWD, |_| true).into_string(),
        cmdline_if_exists
    );
    assert_eq!(
        get_default_url(Some(input), FAKE_CWD, |_| false).into_string(),
        cmdline_otherwise
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_cmdline_and_location_bar_url() {
    test_url(
        "data:text/html,a",
        "data:text/html,a",
        "data:text/html,a",
        "data:text/html,a",
    );
    test_url(
        "README.md",
        "https://readme.md/",
        "file:///fake/cwd/README.md",
        "https://readme.md/",
    );
    test_url(
        "nic.md",
        "https://nic.md/",
        "file:///fake/cwd/nic.md",
        "https://nic.md/",
    );
    test_url(
        "nic.md/ro",
        "https://nic.md/ro",
        "file:///fake/cwd/nic.md/ro",
        "https://nic.md/ro",
    );
    test_url(
        "foo.txt",
        "https://foo.txt/",
        "file:///fake/cwd/foo.txt",
        "https://foo.txt/",
    );
    test_url(
        "foo.txt/ro",
        "https://foo.txt/ro",
        "file:///fake/cwd/foo.txt/ro",
        "https://foo.txt/ro",
    );
    test_url(
        "resources/public_domains.txt",
        "https://resources/public_domains.txt",
        "file:///fake/cwd/resources/public_domains.txt",
        "https://resources/public_domains.txt",
    );
    test_url(
        "dragonfruit",
        "https://duckduckgo.com/html/?q=dragonfruit",
        "file:///fake/cwd/dragonfruit",
        "https://duckduckgo.com/html/?q=dragonfruit",
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_cmdline_and_location_bar_url() {
    test_url(
        "data:text/html,a",
        "data:text/html,a",
        "data:text/html,a",
        "data:text/html,a",
    );
    test_url(
        "README.md",
        "https://readme.md/",
        "file:///C:/fake/cwd/README.md",
        "https://readme.md/",
    );
    test_url(
        "nic.md",
        "https://nic.md/",
        "file:///C:/fake/cwd/nic.md",
        "https://nic.md/",
    );
    test_url(
        "nic.md/ro",
        "https://nic.md/ro",
        "file:///C:/fake/cwd/nic.md/ro",
        "https://nic.md/ro",
    );
    test_url(
        "foo.txt",
        "https://foo.txt/",
        "file:///C:/fake/cwd/foo.txt",
        "https://foo.txt/",
    );
    test_url(
        "foo.txt/ro",
        "https://foo.txt/ro",
        "file:///C:/fake/cwd/foo.txt/ro",
        "https://foo.txt/ro",
    );
    test_url(
        "resources/public_domains.txt",
        "https://resources/public_domains.txt",
        "file:///C:/fake/cwd/resources/public_domains.txt",
        "https://resources/public_domains.txt",
    );
    test_url(
        "dragonfruit",
        "https://duckduckgo.com/html/?q=dragonfruit",
        "file:///C:/fake/cwd/dragonfruit",
        "https://duckduckgo.com/html/?q=dragonfruit",
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_cmd_and_location_bar_url() {
    test_url(
        "/dev/null",
        "file:///dev/null",
        "file:///dev/null",
        "file:///dev/null",
    );
}
