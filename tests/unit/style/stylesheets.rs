/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use cssparser::SourceLocation;
use servo_arc::Arc;
use style::context::QuirksMode;
use style::error_reporting::{ContextualParseError, ParseErrorReporter};
use style::media_queries::MediaList;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{AllowImportRules, Origin, Stylesheet, UrlExtraData};
use url::Url;

#[derive(Debug)]
struct CSSError {
    pub url: Arc<Url>,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

struct TestingErrorReporter {
    errors: RefCell<Vec<CSSError>>,
}

impl TestingErrorReporter {
    pub fn new() -> Self {
        TestingErrorReporter {
            errors: RefCell::new(Vec::new()),
        }
    }

    fn assert_messages_contain(&self, expected_errors: &[(u32, u32, &str)]) {
        let errors = self.errors.borrow();
        for (i, (error, &(line, column, message))) in errors.iter().zip(expected_errors).enumerate()
        {
            assert_eq!(
                (error.line, error.column),
                (line, column),
                "line/column numbers of the {}th error: {:?}",
                i + 1,
                error.message
            );
            assert!(
                error.message.contains(message),
                "{:?} does not contain {:?}",
                error.message,
                message
            );
        }
        if errors.len() < expected_errors.len() {
            panic!("Missing errors: {:#?}", &expected_errors[errors.len()..]);
        }
        if errors.len() > expected_errors.len() {
            panic!("Extra errors: {:#?}", &errors[expected_errors.len()..]);
        }
    }
}

impl ParseErrorReporter for TestingErrorReporter {
    fn report_error(
        &self,
        url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError,
    ) {
        self.errors.borrow_mut().push(CSSError {
            url: url.0.clone(),
            line: location.line,
            column: location.column,
            message: error.to_string(),
        })
    }
}

#[test]
fn test_report_error_stylesheet() {
    let css = r"
    div {
        background-color: red;
        display: invalid;
        background-image: linear-gradient(0deg, black, invalid, transparent);
        invalid: true;
    }
    @media (min-width: 10px invalid 1000px) {}
    @font-face { src: url(), invalid, url(); }
    @counter-style foo { symbols: a 0invalid b }
    @font-feature-values Sans Sans { @foo {} @swash { foo: 1 invalid 2 } }
    @invalid;
    @media screen { @invalid; }
    @supports (color: green) and invalid and (margin: 0) {}
    @keyframes foo { from invalid {} to { margin: 0 invalid 0; } }
    ";
    let url = Url::parse("about::test").unwrap();
    let error_reporter = TestingErrorReporter::new();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(
        css,
        url.clone().into(),
        Origin::UserAgent,
        media,
        lock,
        None,
        Some(&error_reporter),
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    );

    error_reporter.assert_messages_contain(&[
        (
            3,
            18,
            "Unsupported property declaration: 'display: invalid;'",
        ),
        (
            4,
            43,
            "Unsupported property declaration: 'background-image:",
        ), // FIXME: column should be around 56
        (5, 17, "Unsupported property declaration: 'invalid: true;'"),
        (7, 28, "Invalid media rule"),
        // When @counter-style is supported, this should be replaced with two errors
        (9, 19, "Invalid rule: '@counter-style "),
        // When @font-feature-values is supported, this should be replaced with two errors
        (10, 25, "Invalid rule: '@font-feature-values "),
        (11, 13, "Invalid rule: '@invalid'"),
        (12, 29, "Invalid rule: '@invalid'"),
        (13, 34, "Invalid rule: '@supports "),
        (14, 26, "Invalid keyframe rule: 'from invalid '"),
        (
            14,
            52,
            "Unsupported keyframe property declaration: 'margin: 0 invalid 0;'",
        ),
    ]);

    assert_eq!(*error_reporter.errors.borrow()[0].url, url);
}

#[test]
fn test_no_report_unrecognized_vendor_properties() {
    let css = r"
    div {
        -o-background-color: red;
        _background-color: red;
        -moz-background-color: red;
    }
    ";
    let url = Url::parse("about::test").unwrap();
    let error_reporter = TestingErrorReporter::new();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(
        css,
        url.into(),
        Origin::UserAgent,
        media,
        lock,
        None,
        Some(&error_reporter),
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    );

    error_reporter.assert_messages_contain(&[(
        4,
        31,
        "Unsupported property declaration: '-moz-background-color: red;'",
    )]);
}

#[test]
fn test_source_map_url() {
    let tests = vec![
        ("", None),
        (
            "/*# sourceMappingURL=something */",
            Some("something".to_string()),
        ),
    ];

    for test in tests {
        let url = Url::parse("about::test").unwrap();
        let lock = SharedRwLock::new();
        let media = Arc::new(lock.wrap(MediaList::empty()));
        let stylesheet = Stylesheet::from_str(
            test.0,
            url.into(),
            Origin::UserAgent,
            media,
            lock,
            None,
            None,
            QuirksMode::NoQuirks,
            AllowImportRules::Yes,
        );
        let url_opt = stylesheet.contents.source_map_url.read();
        assert_eq!(*url_opt, test.1);
    }
}

#[test]
fn test_source_url() {
    let tests = vec![
        ("", None),
        ("/*# sourceURL=something */", Some("something".to_string())),
    ];

    for test in tests {
        let url = Url::parse("about::test").unwrap();
        let lock = SharedRwLock::new();
        let media = Arc::new(lock.wrap(MediaList::empty()));
        let stylesheet = Stylesheet::from_str(
            test.0,
            url.into(),
            Origin::UserAgent,
            media,
            lock,
            None,
            None,
            QuirksMode::NoQuirks,
            AllowImportRules::Yes,
        );
        let url_opt = stylesheet.contents.source_url.read();
        assert_eq!(*url_opt, test.1);
    }
}
