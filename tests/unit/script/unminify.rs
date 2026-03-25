/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script::test::unminify::create_output_file;
use servo_url::ServoUrl;

#[test]
fn test_create_output_file_with_query_string() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let url = ServoUrl::parse("https://example.com/app.js?v=abc123&t=456").unwrap();

    let result = create_output_file(dir.path().to_str().unwrap().to_owned(), &url, Some(true));

    assert!(result.is_ok(), "file creation failed: {:?}", result.err());
    assert!(
        dir.path().join("example.com/app.js").exists(),
        "expected file at path without query string"
    );
    assert!(
        !dir.path().join("example.com/app.js?v=abc123&t=456").exists(),
        "file must not have '?' in its name (reserved on Windows)"
    );
}

#[test]
fn test_create_output_file_without_query_string() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let url = ServoUrl::parse("https://example.com/app.js").unwrap();

    let result = create_output_file(dir.path().to_str().unwrap().to_owned(), &url, Some(true));

    assert!(result.is_ok());
    assert!(dir.path().join("example.com/app.js").exists());
}
