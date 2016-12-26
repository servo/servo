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

use servo_config::resource_files::read_resource_file;
use servo_url::ServoUrl;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::from_utf8;

#[derive(Clone,Debug)]
pub struct PubDomainRules {
    rules: HashSet<String>,
    wildcards: HashSet<String>,
    exceptions: HashSet<String>,
}

lazy_static! {
    static ref PUB_DOMAINS: PubDomainRules = load_pub_domains();
}

impl<'a> FromIterator<&'a str> for PubDomainRules {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=&'a str> {
        let mut result = PubDomainRules::new();
        for item in iter {
            if item.starts_with("!") {
                result.exceptions.insert(String::from(&item[1..]));
            } else if item.starts_with("*.") {
                result.wildcards.insert(String::from(&item[2..]));
            } else {
                result.rules.insert(String::from(item));
            }
        }
        result
    }
}

impl PubDomainRules {
    pub fn new() -> PubDomainRules {
        PubDomainRules {
            rules: HashSet::new(),
            wildcards: HashSet::new(),
            exceptions: HashSet::new(),
        }
    }
    pub fn parse(content: &str) -> PubDomainRules {
        content.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !s.starts_with("//"))
            .collect()
    }
    fn suffix_pair<'a>(&self, domain: &'a str) -> (&'a str, &'a str) {
        let domain = domain.trim_left_matches(".");
        let mut suffix = domain;
        let mut prev_suffix = domain;
        for (index, _) in domain.match_indices(".") {
            let next_suffix = &domain[index + 1..];
            if self.exceptions.contains(suffix) {
                return (next_suffix, suffix);
            } else if self.wildcards.contains(next_suffix) {
                return (suffix, prev_suffix);
            } else if self.rules.contains(suffix) {
                return (suffix, prev_suffix);
            } else {
                prev_suffix = suffix;
                suffix = next_suffix;
            }
        }
        return (suffix, prev_suffix);
    }
    pub fn public_suffix<'a>(&self, domain: &'a str) -> &'a str {
        let (public, _) = self.suffix_pair(domain);
        public
    }
    pub fn registrable_suffix<'a>(&self, domain: &'a str) -> &'a str {
        let (_, registrable) = self.suffix_pair(domain);
        registrable
    }
    pub fn is_public_suffix(&self, domain: &str) -> bool {
        // Speeded-up version of
        // domain != "" &&
        // self.public_suffix(domain) == domain.
        let domain = domain.trim_left_matches(".");
        match domain.find(".") {
            None => !domain.is_empty(),
            Some(index) => !self.exceptions.contains(domain) &&
                self.wildcards.contains(&domain[index + 1..]) ||
                self.rules.contains(domain),
        }
    }
    pub fn is_registrable_suffix(&self, domain: &str) -> bool {
        // Speeded-up version of
        // self.public_suffix(domain) != domain &&
        // self.registrable_suffix(domain) == domain.
        let domain = domain.trim_left_matches(".");
        match domain.find(".") {
            None => false,
            Some(index) => self.exceptions.contains(domain) ||
                !self.wildcards.contains(&domain[index + 1..]) &&
                !self.rules.contains(domain) &&
                self.is_public_suffix(&domain[index + 1..]),
        }
    }
}

fn load_pub_domains() -> PubDomainRules {
    let content = read_resource_file("public_domains.txt")
        .expect("Could not find public suffix list file");
    let content = from_utf8(&content)
        .expect("Could not read public suffix list file");
    PubDomainRules::parse(content)
}

pub fn pub_suffix(domain: &str) -> &str {
    PUB_DOMAINS.public_suffix(domain)
}

pub fn reg_suffix(domain: &str) -> &str {
    PUB_DOMAINS.registrable_suffix(domain)
}

pub fn is_pub_domain(domain: &str) -> bool {
    PUB_DOMAINS.is_public_suffix(domain)
}

pub fn is_reg_domain(domain: &str) -> bool {
    PUB_DOMAINS.is_registrable_suffix(domain)
}

/// The registered domain name (aka eTLD+1) for a URL.
/// Returns None if the URL has no host name.
/// Returns the registered suffix for the host name if it is a domain.
/// Leaves the host name alone if it is an IP address.
pub fn reg_host<'a>(url: &'a ServoUrl) -> Option<&'a str> {
    url.domain().map(reg_suffix).or(url.host_str())
}
