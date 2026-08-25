/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::{CspList, PolicyDisposition};
use http::header::HeaderMap;
use hyper_serde::Serde;
use net_traits::fetch::headers::get_decode_and_split_header_name;
use rustc_hash::FxHashSet;
use servo_url::MutableOrigin;

use crate::dom::node::NodeTraits;
use crate::dom::window::Window;

/// <https://html.spec.whatwg.org/multipage/#check-a-navigation-response%027s-adherence-to-x-frame-options>
pub(crate) fn check_a_navigation_response_adherence_to_x_frame_options(
    window: &Window,
    csp_list: Option<&CspList>,
    destination_origin: &MutableOrigin,
    headers: Option<&Serde<HeaderMap>>,
) -> bool {
    // Step 1. If navigable is not a child navigable, then return true.
    if window.window_proxy().parent().is_none() {
        return true;
    }
    // Step 2. For each policy of cspList:
    if let Some(csp_list) = csp_list {
        for policy in csp_list.0.iter() {
            // Step 2.1. If policy's disposition is not "enforce", then continue.
            if policy.disposition != PolicyDisposition::Enforce {
                continue;
            }
            // Step 2.2. If policy's directive set contains a frame-ancestors directive, then return true.
            if policy.contains_a_directive_whose_name_is("frame-ancestors") {
                return true;
            }
        }
    }
    // Step 3. Let rawXFrameOptions be the result of getting, decoding,
    // and splitting `X-Frame-Options` from response's header list.
    let Some(headers) = headers else {
        return true;
    };
    let Some(raw_xframe_options) = get_decode_and_split_header_name("X-Frame-Options", headers)
    else {
        return true;
    };
    // Step 4. Let xFrameOptions be a new set.
    // Step 5. For each value of rawXFrameOptions, append value, converted to ASCII lowercase, to xFrameOptions.
    let x_frame_options =
        FxHashSet::from_iter(raw_xframe_options.iter().map(|value| value.to_lowercase()));
    // Step 6. If xFrameOptions's size is greater than 1,
    // and xFrameOptions contains any of "deny", "allowall", or "sameorigin", then return false.
    if x_frame_options.len() > 1 &&
        x_frame_options
            .iter()
            .any(|value| value == "deny" || value == "allowall" || value == "sameorigin")
    {
        return false;
    }
    // Step 7. If xFrameOptions's size is greater than 1, then return true.
    if x_frame_options.len() > 1 {
        return true;
    }
    let Some(first_item) = x_frame_options.iter().next() else {
        return true;
    };
    // Step 8. If xFrameOptions[0] is "deny", then return false.
    if first_item == "deny" {
        return false;
    }
    // Step 9. If xFrameOptions[0] is "sameorigin", then:
    if first_item == "sameorigin" {
        let mut window_proxy = window.window_proxy();
        // Step 9.2. While containerDocument is not null:
        while let Some(container_element) = window_proxy.frame_element() {
            // Step 9.1. Let containerDocument be navigable's container document.
            let container_document = container_element.owner_document();
            // Step 9.2.1. If containerDocument's origin is not same origin with destinationOrigin, then return false.
            if !container_document.origin().same_origin(destination_origin) {
                return false;
            }
            // Step 9.2.2. Set containerDocument to containerDocument's container document.
            window_proxy = container_document.window().window_proxy()
        }
        // If the `frame_element` is None, it could be two options:
        // 1. There is no parent, in which case we are top-level. We can stop the loop
        // 2. There is a parent, but it isn't same origin. Therefore, we need to double
        //    check here that we actually cover this case.
        if window_proxy.parent().is_some() {
            return false;
        }
    }
    // Step 10. Return true.
    true
}
