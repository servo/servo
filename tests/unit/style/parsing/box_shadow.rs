/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::properties::longhands::box_shadow;

#[test]
fn blur_radius_should_not_accept_negavite_values() {
    let result = parse_longhand!(box_shadow, "normal");
    assert_eq!(result, "");
}
