/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use headers::HeaderMap;

/// <https://fetch.spec.whatwg.org/#concept-header-list-get>
pub fn get_value_from_header_list(name: &str, headers: &HeaderMap) -> Option<String> {
    let values = headers
        .get_all(name)
        .iter()
        .map(|val| val.to_str().unwrap());

    // Step 1
    if values.size_hint() == (0, Some(0)) {
        return None;
    }

    // Step 2
    return Some(values.collect::<Vec<&str>>().join(", "));
}
