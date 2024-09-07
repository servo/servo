/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use headers::HeaderMap;
use net_traits::fetch::headers::get_decode_and_split_header_name;

/// <https://fetch.spec.whatwg.org/#determine-nosniff>
pub fn determine_nosniff(headers: &HeaderMap) -> bool {
    let values = get_decode_and_split_header_name("x-content-type-options", headers);

    match values {
        None => false,
        Some(values) => !values.is_empty() && values[0].eq_ignore_ascii_case("nosniff"),
    }
}
