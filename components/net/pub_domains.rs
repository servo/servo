/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of public domain matching.
//!
//! The list is a file located on the `resources` folder and loaded once on first need.
//!
//! The list can be updated with `./mach update-pub-domains` from this source:
//! https://publicsuffix.org/list/
//!
//! This implementation is not strictly following the specification of the list. Wildcards are not
//! restricted to appear only in the leftmost position, but the current list has no such cases so
//! we don't need to make the code more complex for it. The `mach` update command makes sure that
//! those cases are not present.

use std::collections::HashSet;
use std::str::from_utf8;
use std::sync::Arc;
use util::resource_files::read_resource_file;

lazy_static! {
    static ref PUB_DOMAINS: Arc<HashSet<String>> = load_pub_domains();
}

fn load_pub_domains() -> Arc<HashSet<String>> {
    let content = read_resource_file("public_domains.txt")
                  .expect("Could not find public suffix list file");
    let domains = from_utf8(&content)
        .expect("Could not read suffix list file")
        .lines()
        .filter_map(|i| {
            let domain = i.trim();
            if domain == "" { return None };
            if domain.starts_with("//") { return None };
            Some(domain.to_owned())
        });

    Arc::new(domains.collect())
}

/// Match the given domain against a static list of known public domains
pub fn is_pub_domain(domain: &str) -> bool {
    let domain = domain.trim_left_matches(".");

    // Start by looking for a plain match
    if PUB_DOMAINS.contains(&domain.to_string()) {
        return true
    }

    // Then look for a wildcard match
    // To make things simpler, just look for the same domain with its leftmost part replaced by a
    // wildcard.
    match domain.find(".") {
        None => {
            // This is a domain with only one part, so there is no need to search for wildcards or
            // exceptions
            return false
        }
        Some(position) => {
            let wildcard_domain = "*".to_string() + domain.split_at(position).1;
            if PUB_DOMAINS.contains(&wildcard_domain) {
                // We have a wildcard match, search for an eventual exception
                let exception_domain = "!".to_string() + domain;
                return ! PUB_DOMAINS.contains(&exception_domain)
            } else {
                // No wildcard match -> this is not a public domain
                return false
            }
        }
    }
}

