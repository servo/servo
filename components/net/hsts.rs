/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::IncludeSubdomains;
use rustc_serialize::json::decode;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::from_utf8;
use time;
use url::Url;
use util::resource_files::read_resource_file;

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct HstsEntry {
    pub host: String,
    pub include_subdomains: bool,
    pub max_age: Option<u64>,
    pub timestamp: Option<u64>
}

impl HstsEntry {
    pub fn new(host: String, subdomains: IncludeSubdomains, max_age: Option<u64>) -> Option<HstsEntry> {
        if host.parse::<Ipv4Addr>().is_ok() || host.parse::<Ipv6Addr>().is_ok() {
            None
        } else {
            Some(HstsEntry {
                host: host,
                include_subdomains: (subdomains == IncludeSubdomains::Included),
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

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct HstsList {
    pub entries: Vec<HstsEntry>
}

impl HstsList {
    pub fn new() -> HstsList {
        HstsList {
            entries: vec![]
        }
    }

    /// Create an `HstsList` from the bytes of a JSON preload file.
    pub fn from_preload(preload_content: &[u8]) -> Option<HstsList> {
        from_utf8(&preload_content)
            .ok()
            .and_then(|c| decode(c).ok())
    }

    pub fn from_servo_preload() -> HstsList {
        let file_bytes = read_resource_file("hsts_preload.json")
                            .expect("Could not find Servo HSTS preload file");
        HstsList::from_preload(&file_bytes)
            .expect("Servo HSTS preload file is invalid")
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

    pub fn push(&mut self, entry: HstsEntry) {
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

pub fn secure_url(url: &Url) -> Url {
    if url.scheme() == "http" {
        let mut secure_url = url.clone();
        secure_url.set_scheme("https").unwrap();
        // .set_port(Some(443)) would set the port to None,
        // and should only be done when it was already None.
        secure_url
    } else {
        url.clone()
    }
}
