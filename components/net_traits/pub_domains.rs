/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of public domain matching.
//!
//! The list is a file located on the `resources` folder and loaded once on first need.
//!
//! The list can be updated with `./mach update-pub-domains` from this source:
//! <https://publicsuffix.org/list/>
//!
//! This implementation is not strictly following the specification of the list. Wildcards are not
//! restricted to appear only in the leftmost position, but the current list has no such cases so
//! we don't need to make the code more complex for it. The `mach` update command makes sure that
//! those cases are not present.

use servo_url::{ChromeReader, Host, ImmutableOrigin, ServoUrl};
use std::collections::HashSet;
use std::str::from_utf8;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct PubDomainRules {
    rules: HashSet<String>,
    wildcards: HashSet<String>,
    exceptions: HashSet<String>,
}

lazy_static! {
    static ref PUB_DOMAINS: RwLock<PubDomainRules> = RwLock::new(PubDomainRules::new());
}

pub fn init<T>(chrome_reader: &T) where T: ChromeReader {
    let url = ServoUrl::parse("chrome://resources/public_domains.txt").unwrap();
    let mut content = chrome_reader.resolve(&url).expect("Could not find public suffix list file");
    let mut bytes = vec![];
    content.read_to_end(&mut bytes).expect("Can't read public_domains.txt");
    let content = from_utf8(&bytes).expect("Could not read public suffix list file");
    PUB_DOMAINS.write().unwrap().parse(content);
}

impl PubDomainRules {
    pub fn new() -> PubDomainRules {
        PubDomainRules {
            rules: HashSet::new(),
            wildcards: HashSet::new(),
            exceptions: HashSet::new(),
        }
    }
    pub fn parse(&mut self, content: &str) {
        content.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !s.starts_with("//"))
            .for_each(|line| {
                if line.starts_with("!") {
                    self.exceptions.insert(String::from(&line[1..]));
                } else if line.starts_with("*.") {
                    self.wildcards.insert(String::from(&line[2..]));
                } else {
                    self.rules.insert(String::from(line));
                }
            });
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
            Some(index) => {
                !self.exceptions.contains(domain) && self.wildcards.contains(&domain[index + 1..]) ||
                self.rules.contains(domain)
            },
        }
    }
    pub fn is_registrable_suffix(&self, domain: &str) -> bool {
        // Speeded-up version of
        // self.public_suffix(domain) != domain &&
        // self.registrable_suffix(domain) == domain.
        let domain = domain.trim_left_matches(".");
        match domain.find(".") {
            None => false,
            Some(index) => {
                self.exceptions.contains(domain) ||
                !self.wildcards.contains(&domain[index + 1..]) && !self.rules.contains(domain) &&
                self.is_public_suffix(&domain[index + 1..])
            },
        }
    }
}

pub fn pub_suffix(domain: &str) -> &str {
    PUB_DOMAINS.read().unwrap().public_suffix(domain)
}

pub fn reg_suffix(domain: &str) -> &str {
    PUB_DOMAINS.read().unwrap().registrable_suffix(domain)
}

pub fn is_pub_domain(domain: &str) -> bool {
    PUB_DOMAINS.read().unwrap().is_public_suffix(domain)
}

pub fn is_reg_domain(domain: &str) -> bool {
    PUB_DOMAINS.read().unwrap().is_registrable_suffix(domain)
}

/// The registered domain name (aka eTLD+1) for a URL.
/// Returns None if the URL has no host name.
/// Returns the registered suffix for the host name if it is a domain.
/// Leaves the host name alone if it is an IP address.
pub fn reg_host(url: &ServoUrl) -> Option<Host> {
    match url.origin() {
        ImmutableOrigin::Tuple(_, Host::Domain(domain), _) => Some(Host::Domain(String::from(reg_suffix(&*domain)))),
        ImmutableOrigin::Tuple(_, ip, _) => Some(ip),
        ImmutableOrigin::Opaque(_) => None,
    }
}
