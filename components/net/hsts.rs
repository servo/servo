/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::IncludeSubdomains;
use net_traits::pub_domains::reg_suffix;
use rustc_serialize::json::decode;
use servo_config::resource_files::read_resource_file;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::from_utf8;
use time;
use url::Url;

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
            }

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
    pub entries_map: HashMap<String, Vec<HstsEntry>>,
}

impl HstsList {
    pub fn new() -> HstsList {
        HstsList { entries_map: HashMap::new() }
    }

    /// Create an `HstsList` from the bytes of a JSON preload file.
    pub fn from_preload(preload_content: &[u8]) -> Option<HstsList> {
        #[derive(RustcDecodable)]
        struct HstsEntries {
            entries: Vec<HstsEntry>,
        }

        let hsts_entries: Option<HstsEntries> = from_utf8(&preload_content)
            .ok()
            .and_then(|c| decode(c).ok());

        hsts_entries.map_or(None, |hsts_entries| {
            let mut hsts_list: HstsList = HstsList::new();

            for hsts_entry in hsts_entries.entries {
                hsts_list.push(hsts_entry);
            }

            return Some(hsts_list);
        })
    }

    pub fn from_servo_preload() -> HstsList {
        let file_bytes = read_resource_file("hsts_preload.json")
                            .expect("Could not find Servo HSTS preload file");
        HstsList::from_preload(&file_bytes)
            .expect("Servo HSTS preload file is invalid")
    }

    pub fn is_host_secure(&self, host: &str) -> bool {
        let base_domain = reg_suffix(host);
        self.entries_map.get(base_domain).map_or(false, |entries| {
            entries.iter().any(|e| {
                if e.include_subdomains {
                    e.matches_subdomain(host) || e.matches_domain(host)
                } else {
                    e.matches_domain(host)
                }
            })
        })
    }

    fn has_domain(&self, host: &str, base_domain: &str) -> bool {
        self.entries_map.get(base_domain).map_or(false, |entries| {
            entries.iter().any(|e| e.matches_domain(&host))
        })
    }

    fn has_subdomain(&self, host: &str, base_domain: &str) -> bool {
       self.entries_map.get(base_domain).map_or(false, |entries| {
            entries.iter().any(|e| e.matches_subdomain(host))
       })
    }

    pub fn push(&mut self, entry: HstsEntry) {
        let host = entry.host.clone();
        let base_domain = reg_suffix(&host);
        let have_domain = self.has_domain(&entry.host, base_domain);
        let have_subdomain = self.has_subdomain(&entry.host, base_domain);

        let entries = self.entries_map.entry(base_domain.to_owned()).or_insert(vec![]);
        if !have_domain && !have_subdomain {
            entries.push(entry);
        } else if !have_subdomain {
            for e in entries {
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
