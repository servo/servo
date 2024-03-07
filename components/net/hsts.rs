/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use embedder_traits::resources::{self, Resource};
use headers::{HeaderMapExt, StrictTransportSecurity};
use http::HeaderMap;
use log::info;
use net_traits::pub_domains::reg_suffix;
use net_traits::IncludeSubdomains;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use servo_url::{Host, ServoUrl};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HstsEntry {
    pub host: String,
    pub include_subdomains: bool,
    pub max_age: Option<u64>,
    pub timestamp: Option<u64>,
}

impl HstsEntry {
    pub fn new(
        host: String,
        subdomains: IncludeSubdomains,
        max_age: Option<u64>,
    ) -> Option<HstsEntry> {
        if host.parse::<Ipv4Addr>().is_ok() || host.parse::<Ipv6Addr>().is_ok() {
            None
        } else {
            Some(HstsEntry {
                host,
                include_subdomains: (subdomains == IncludeSubdomains::Included),
                max_age,
                timestamp: Some(time::get_time().sec as u64),
            })
        }
    }

    pub fn is_expired(&self) -> bool {
        match (self.max_age, self.timestamp) {
            (Some(max_age), Some(timestamp)) => {
                (time::get_time().sec as u64) - timestamp >= max_age
            },

            _ => false,
        }
    }

    fn matches_domain(&self, host: &str) -> bool {
        !self.is_expired() && self.host == host
    }

    fn matches_subdomain(&self, host: &str) -> bool {
        !self.is_expired() && host.ends_with(&format!(".{}", self.host))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HstsList {
    pub entries_map: HashMap<String, Vec<HstsEntry>>,
}

impl HstsList {
    /// Create an `HstsList` from the bytes of a JSON preload file.
    pub fn from_preload(preload_content: &str) -> Option<HstsList> {
        #[derive(Deserialize)]
        struct HstsEntries {
            entries: Vec<HstsEntry>,
        }

        let hsts_entries: Option<HstsEntries> = serde_json::from_str(preload_content).ok();

        hsts_entries.map(|hsts_entries| {
            let mut hsts_list: HstsList = HstsList::default();

            for hsts_entry in hsts_entries.entries {
                hsts_list.push(hsts_entry);
            }

            hsts_list
        })
    }

    pub fn from_servo_preload() -> HstsList {
        let list = resources::read_string(Resource::HstsPreloadList);
        HstsList::from_preload(&list).expect("Servo HSTS preload file is invalid")
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
            entries.iter().any(|e| e.matches_domain(host))
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

        let entries = self.entries_map.entry(base_domain.to_owned()).or_default();
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

    /// Step 2.9 of <https://fetch.spec.whatwg.org/#concept-main-fetch>.
    pub fn apply_hsts_rules(&self, url: &mut ServoUrl) {
        if url.scheme() != "http" && url.scheme() != "ws" {
            return;
        }

        let upgrade_scheme = if pref!(network.enforce_tls.enabled) {
            if (!pref!(network.enforce_tls.localhost) &&
                match url.host() {
                    Some(Host::Domain(domain)) => {
                        domain.ends_with(".localhost") || domain == "localhost"
                    },
                    Some(Host::Ipv4(ipv4)) => ipv4.is_loopback(),
                    Some(Host::Ipv6(ipv6)) => ipv6.is_loopback(),
                    _ => false,
                }) ||
                (!pref!(network.enforce_tls.onion) &&
                    url.domain()
                        .map_or(false, |domain| domain.ends_with(".onion")))
            {
                url.domain()
                    .map_or(false, |domain| self.is_host_secure(domain))
            } else {
                true
            }
        } else {
            url.domain()
                .map_or(false, |domain| self.is_host_secure(domain))
        };

        if upgrade_scheme {
            let upgraded_scheme = match url.scheme() {
                "ws" => "wss",
                _ => "https",
            };
            url.as_mut_url().set_scheme(upgraded_scheme).unwrap();
        }
    }

    pub fn update_hsts_list_from_response(&mut self, url: &ServoUrl, headers: &HeaderMap) {
        if url.scheme() != "https" && url.scheme() != "wss" {
            return;
        }

        if let Some(header) = headers.typed_get::<StrictTransportSecurity>() {
            if let Some(host) = url.domain() {
                let include_subdomains = if header.include_subdomains() {
                    IncludeSubdomains::Included
                } else {
                    IncludeSubdomains::NotIncluded
                };

                if let Some(entry) = HstsEntry::new(
                    host.to_owned(),
                    include_subdomains,
                    Some(header.max_age().as_secs()),
                ) {
                    info!("adding host {} to the strict transport security list", host);
                    info!("- max-age {}", header.max_age().as_secs());
                    if header.include_subdomains() {
                        info!("- includeSubdomains");
                    }

                    self.push(entry);
                }
            }
        }
    }
}
