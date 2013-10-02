/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::stylesheets::parse_stylesheet;

#[test]
fn test_bootstrap() {
    // Test that parsing bootstrap does not trigger an assertion or otherwise fail.
    let stylesheet = parse_stylesheet(include_str!("bootstrap-v3.0.0.css"));
    assert!(stylesheet.rules.len() > 100);  // This depends on whet selectors are supported.
}
