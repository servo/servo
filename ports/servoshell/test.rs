/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;
use std::fs::File;
use std::env;
use servo::{embedder_traits, servo_url::ServoUrl};
use crate::parser::{parse_url_or_filename, get_default_url, sanitize_url};

#[cfg(not(target_os = "windows"))]
const FAKE_CWD: &'static str = "/fake/cwd";

#[cfg(target_os = "windows")]
const FAKE_CWD: &'static str = "C:/fake/cwd";

#[test]
fn test_argument_parsing() {
    let fake_cwd: &Path = Path::new(FAKE_CWD);
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
    let fake_cwd: &Path = Path::new(FAKE_CWD);

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

#[test]
fn url_should_resolve_in_commad_line() {
    embedder_traits::resources::set_for_tests();
    let input = "resources/public_domains.txt";
    let cwd = "../../";
    env::set_current_dir(&cwd).expect("Failed to set current directory");

    let result = get_default_url(Some(input.to_string()));
    assert_eq!(result.scheme(), "file");
}

#[test]
fn url_should_resolve_in_location_bar() {
    embedder_traits::resources::set_for_tests();
    let input = "resources/public_domains.txt";
    let expected_result = ServoUrl::parse("https://resources/public_domains.txt").ok();
    let result = sanitize_url(input);
    assert_eq!(result, expected_result);
}

#[test]
// if a file named with keyword exists, it should be trated as file path
fn no_dots_keyword_should_resolve_in_commad_line_as_file_path() {
    embedder_traits::resources::set_for_tests();

    let cwd = env::current_dir().expect("Failed to get current directory");

    // Create a temporary file called dragonfruit
    let file_path = cwd.join("dragonfruit");
    let file = File::create(&file_path).expect("Failed to create dragonfruit file");

    let path =  file_path.to_string_lossy().to_string();
    let input = "dragonfruit";

    let url = get_default_url(Some(input.to_string()));

    let path_segments = url.path_segments().unwrap().collect::<Vec<_>>();
    let contains_input = path_segments.contains(&input);

    assert_eq!(url.scheme(), "file");
    assert!(contains_input);

    // Remove the temporary dragonfruit file
    std::fs::remove_file(&file_path).expect("Failed to remove dragonfruit file");
}

#[test]
fn no_dots_keyword_should_resolve_as_search() {
    embedder_traits::resources::set_for_tests();
    let input = "dragonfruit";
    let input1 = "README.md";

    // in location bar
    let location_bar_url = sanitize_url(input);
    let binding = location_bar_url.clone().unwrap();

    assert_eq!(binding.scheme(), "https");
    assert_eq!(binding.domain(), Some("duckduckgo.com"));
    assert_eq!(binding.query(), Some("q=dragonfruit"));


    let expected_result = ServoUrl::parse("https://README.md").ok();
    let location_bar_url1 = sanitize_url(input1);
    assert_eq!(location_bar_url1, expected_result);


    // in command line
    let command_line_url = get_default_url(Some(input.to_string()));

    // if no file named with keyword exists locally, it should be trated as search keyword
    assert_eq!(command_line_url.scheme(), "https");
    assert_eq!(command_line_url.domain(), Some("duckduckgo.com"));
    assert_eq!(command_line_url.query(), Some("q=dragonfruit"));

    let cwd = "./";
    env::set_current_dir(&cwd).expect("Failed to set current directory");
    let command_line_url1 = get_default_url(Some(input1.to_string()));
    let path_segments = command_line_url1.path_segments().unwrap().collect::<Vec<_>>();
    let contains_input = path_segments.contains(&input1);

    assert_eq!(command_line_url1.scheme(), "file");
    assert!(contains_input);

}

#[test]
fn should_resolve_url() {
    embedder_traits::resources::set_for_tests();
    let input = "nic.md/ro"; // known tld
    let input1= "foo.txt/ro"; // not a known tld

    // This is the expected result form cmdline_url as well because this file doesn't exists locally
    let result = "https://nic.md/ro";
    let result_for_input1  = "https://foo.txt/ro";

    // location bar
    let location_url = sanitize_url(input).unwrap();
    let location_url_for_input1 = sanitize_url(input1).unwrap();
    assert_eq!(location_url.into_string(), result);
    assert_eq!(location_url_for_input1.into_string(), result_for_input1);

    // cmdline url
    let cmdline_url = get_default_url(Some(input.to_string()));
    let cmdline_url_for_input1 = get_default_url(Some(input1.to_string()));
    assert_eq!(cmdline_url.into_string(), result);
    assert_eq!(cmdline_url_for_input1.into_string(), result_for_input1);
}

#[cfg(target_os = "linux")]
#[test]
fn parse_url_command_line() {
    let input = "/dev/null";
    let url = get_default_url(Some(input.to_string()));
    let expected_result = "file:///dev/null";

    assert_eq!(url.scheme(), "file");
    assert_eq!(url.into_string(), expected_result);
}
