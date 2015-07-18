/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use regex::Regex;
use rustc_serialize::json::{decode};
use time;
use url::Url;

use std::str::{from_utf8};

use net_traits::LoadData;
use util::resource_files::read_resource_file;

static IPV4_REGEX: Regex = regex!(
    r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$"
);
static IPV6_REGEX: Regex = regex!(r"^([a-fA-F0-9]{0,4}[:]?){1,8}(/\d{1,3})?$");

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct HSTSEntry {
    pub host: String,
    pub include_subdomains: bool,
    pub max_age: Option<u64>,
    pub timestamp: Option<u64>
}

#[derive(PartialEq, Copy, Clone)]
pub enum Subdomains {
    Included,
    NotIncluded
}

impl HSTSEntry {
    pub fn new(host: String, subdomains: Subdomains, max_age: Option<u64>) -> Option<HSTSEntry> {
        if IPV4_REGEX.is_match(&host) || IPV6_REGEX.is_match(&host) {
            None
        } else {
            Some(HSTSEntry {
                host: host,
                include_subdomains: (subdomains == Subdomains::Included),
                max_age: max_age,
                timestamp: Some(time::get_time().sec as u64)
            })
        }
    }

    pub fn is_expired(&self) -> bool {
        match (self.max_age, self.timestamp) {
            (Some(max_age), Some(timestamp)) => {
                (time::get_time().sec as u64) - timestamp >= max_age
            },

            _ => false
        }
    }

    fn matches_domain(&self, host: &str) -> bool {
        !self.is_expired() && self.host == host
    }

    fn matches_subdomain(&self, host: &str) -> bool {
        !self.is_expired() && host.ends_with(&format!(".{}", self.host))
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct HSTSList {
    pub entries: Vec<HSTSEntry>
}

impl HSTSList {
    pub fn new_from_preload(preload_content: &str) -> Option<HSTSList> {
        decode(preload_content).ok()
    }

    pub fn is_host_secure(&self, host: &str) -> bool {
        // TODO - Should this be faster than O(n)? The HSTS list is only a few
        // hundred or maybe thousand entries...
        //
        // Could optimise by searching for exact matches first (via a map or
        // something), then checking for subdomains.
        self.entries.iter().any(|e| {
            if e.include_subdomains {
                e.matches_subdomain(host) || e.matches_domain(host)
            } else {
                e.matches_domain(host)
            }
        })
    }

    fn has_domain(&self, host: &str) -> bool {
        self.entries.iter().any(|e| {
            e.matches_domain(&host)
        })
    }

    fn has_subdomain(&self, host: &str) -> bool {
        self.entries.iter().any(|e| {
            e.matches_subdomain(host)
        })
    }

    pub fn push(&mut self, entry: HSTSEntry) {
        let have_domain = self.has_domain(&entry.host);
        let have_subdomain = self.has_subdomain(&entry.host);

        if !have_domain && !have_subdomain {
            self.entries.push(entry);
        } else if !have_subdomain {
            for e in &mut self.entries {
                if e.matches_domain(&entry.host) {
                    e.include_subdomains = entry.include_subdomains;
                    e.max_age = entry.max_age;
                }
            }
        }
    }
}

pub fn preload_hsts_domains() -> Option<HSTSList> {
    read_resource_file(&["hsts_preload.json"]).ok().and_then(|bytes| {
        from_utf8(&bytes).ok().and_then(|hsts_preload_content| {
            HSTSList::new_from_preload(hsts_preload_content)
        })
    })
}

pub fn secure_load_data(load_data: &LoadData) -> LoadData {
    if &*load_data.url.scheme == "http" {
        let mut secure_load_data = load_data.clone();
        let mut secure_url = load_data.url.clone();
        secure_url.scheme = "https".to_string();
        // The Url struct parses the port for a known scheme only once.
        // Updating the scheme doesn't update the port internally, resulting in
        // HTTPS connections attempted on port 80. Serialising and re-parsing
        // the Url is a hack to get around this.
        secure_load_data.url = Url::parse(&secure_url.serialize()).unwrap();

        secure_load_data
    } else {
        load_data.clone()
    }
}

