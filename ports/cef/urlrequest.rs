/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use types::{cef_request_t, cef_urlrequest_client_t, cef_urlrequest_t};


#[no_mangle]
pub extern "C" fn cef_urlrequest_create(_request: *mut cef_request_t,
                                        _client: *mut cef_urlrequest_client_t)
                                        -> *mut cef_urlrequest_t {
    0 as *mut cef_urlrequest_t
}
