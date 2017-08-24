/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::parse;
use style::properties::longhands::background_size;

#[test]
fn background_size_should_reject_negative_values() {
    assert!(parse(|c, _, i| background_size::parse(c, i), "-40% -40%").is_err());
}
