/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::LazyLock;
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use embedder_traits::resources::{self, Resource};
use headers::{HeaderMapExt, StrictTransportSecurity};
use http::HeaderMap;
use log::{debug, error, info};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::IncludeSubdomains;
use net_traits::pub_domains::reg_suffix;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use servo_url::{Host, ServoUrl};

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct HstsEntry {
    pub host: String,
    pub include_subdomains: bool,
    // TODO(sebsebmc): This is wrong, CrossProcessInstant epoch is system boot, persisting this to disk and
    // loading it after reboot results in incorrect results
    pub expires_at: Option<CrossProcessInstant>,
}

impl HstsEntry {
    pub fn new(
        host: String,
        subdomains: IncludeSubdomains,
        max_age: Option<Duration>,
    ) -> Option<HstsEntry> {
        let expires_at =
            max_age.map(|duration| CrossProcessInstant::now() + duration.try_into().unwrap());
        if host.parse::<Ipv4Addr>().is_ok() || host.parse::<Ipv6Addr>().is_ok() {
            None
        } else {
            Some(HstsEntry {
                host,
                include_subdomains: (subdomains == IncludeSubdomains::Included),
                expires_at,
            })
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(timestamp) => CrossProcessInstant::now() > timestamp,
            _ => false,
        }
    }

    fn matches_domain(&self, host: &str) -> bool {
        self.host == host
    }

    fn matches_subdomain(&self, host: &str) -> bool {
        host.ends_with(&format!(".{}", self.host))
    }
}

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct HstsList {
    // Map from base domains to a list of entries that are subdomains of base domain
    pub entries_map: HashMap<String, Vec<HstsEntry>>,
}

/// Represents the portion of the HSTS list that comes from the preload list
/// it is split out to allow sharing between the private and public http state
/// as well as potentially swpaping out the underlying type to something immutable
/// and more efficient like FSTs or DAFSA/DAWGs.
#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct HstsPreloadList {
    pub entries_map: HashMap<String, Vec<HstsEntry>>,
}

pub static PRELOAD_LIST_ENTRIES: LazyLock<HstsPreloadList> =
    LazyLock::new(HstsPreloadList::from_servo_preload);

impl HstsPreloadList {
    /// Create an `HstsList` from the bytes of a JSON preload file.
    pub fn from_preload(preload_content: &str) -> Option<HstsPreloadList> {
        #[derive(Deserialize)]
        struct HstsEntries {
            entries: Vec<HstsEntry>,
        }

        let hsts_entries: Option<HstsEntries> = serde_json::from_str(preload_content).ok();

        hsts_entries.map(|hsts_entries| {
            let mut hsts_list: HstsPreloadList = HstsPreloadList::default();

            for hsts_entry in hsts_entries.entries {
                hsts_list.push(hsts_entry);
            }

            hsts_list
        })
    }

    pub fn from_servo_preload() -> HstsPreloadList {
        debug!("Intializing HSTS Preload list");
        let list = resources::read_string(Resource::HstsPreloadList);
        HstsPreloadList::from_preload(&list).unwrap_or_else(|| {
            error!("HSTS preload file is invalid. Setting HSTS list to default values");
            HstsPreloadList {
                entries_map: Default::default(),
            }
        })
    }

    pub fn is_host_secure(&self, host: &str) -> bool {
        let base_domain = reg_suffix(host);
        self.entries_map.get(base_domain).is_some_and(|entries| {
            // No need to check for expiration in the preload list
            entries.iter().any(|e| {
                if e.include_subdomains {
                    e.matches_subdomain(host) || e.matches_domain(host)
                } else {
                    e.matches_domain(host)
                }
            })
        })
    }

    pub fn has_domain(&self, host: &str, base_domain: &str) -> bool {
        self.entries_map
            .get(base_domain)
            .is_some_and(|entries| entries.iter().any(|e| e.matches_domain(host)))
    }

    pub fn has_subdomain(&self, host: &str, base_domain: &str) -> bool {
        self.entries_map.get(base_domain).is_some_and(|entries| {
            entries
                .iter()
                .any(|e| e.include_subdomains && e.matches_subdomain(host))
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
                    // TODO(sebsebmc): We could shrink the the HSTS preload memory use further by using a type
                    // that doesn't store an expiry since all preload entries should be "forever"
                    e.expires_at = entry.expires_at;
                }
            }
        }
    }
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

    pub fn is_host_secure(&self, host: &str) -> bool {
        debug!("HSTS: is {host} secure?");
        if PRELOAD_LIST_ENTRIES.is_host_secure(host) {
            info!("{host} is in the preload list");
            return true;
        }

        let base_domain = reg_suffix(host);
        self.entries_map.get(base_domain).is_some_and(|entries| {
            entries.iter().filter(|e| !e.is_expired()).any(|e| {
                if e.include_subdomains {
                    e.matches_subdomain(host) || e.matches_domain(host)
                } else {
                    e.matches_domain(host)
                }
            })
        })
    }

    fn has_domain(&self, host: &str, base_domain: &str) -> bool {
        self.entries_map
            .get(base_domain)
            .is_some_and(|entries| entries.iter().any(|e| e.matches_domain(host)))
    }

    fn has_subdomain(&self, host: &str, base_domain: &str) -> bool {
        self.entries_map.get(base_domain).is_some_and(|entries| {
            entries
                .iter()
                .any(|e| e.include_subdomains && e.matches_subdomain(host))
        })
    }

    pub fn push(&mut self, entry: HstsEntry) {
        let host = entry.host.clone();
        let base_domain = reg_suffix(&host);
        {
            // Remove expired entries, this seems to be as good a time as any and helps avoid a
            // pathologic case where we would keep adding short lived HSTS entries and having to
            // linear scan through increasing numbers of expired entries
            let entries = self.entries_map.entry(base_domain.to_owned()).or_default();
            entries.retain(|e| !e.is_expired());
        }
        let have_domain = self.has_domain(&entry.host, base_domain);
        let have_subdomain = self.has_subdomain(&entry.host, base_domain);

        let entries = self.entries_map.entry(base_domain.to_owned()).or_default();
        if !have_domain && !have_subdomain {
            entries.push(entry);
        } else if !have_subdomain {
            for e in entries {
                if e.matches_domain(&entry.host) {
                    e.include_subdomains = entry.include_subdomains;
                    e.expires_at = entry.expires_at;
                }
            }
        }
    }

    /// Step 2.9 of <https://fetch.spec.whatwg.org/#concept-main-fetch>.
    pub fn apply_hsts_rules(&self, url: &mut ServoUrl) {
        if url.scheme() != "http" && url.scheme() != "ws" {
            return;
        }

        let upgrade_scheme = if pref!(network_enforce_tls_enabled) {
            if (!pref!(network_enforce_tls_localhost) &&
                match url.host() {
                    Some(Host::Domain(domain)) => {
                        domain.ends_with(".localhost") || domain == "localhost"
                    },
                    Some(Host::Ipv4(ipv4)) => ipv4.is_loopback(),
                    Some(Host::Ipv6(ipv6)) => ipv6.is_loopback(),
                    _ => false,
                }) ||
                (!pref!(network_enforce_tls_onion) &&
                    url.domain()
                        .is_some_and(|domain| domain.ends_with(".onion")))
            {
                url.domain()
                    .is_some_and(|domain| self.is_host_secure(domain))
            } else {
                true
            }
        } else {
            url.domain()
                .is_some_and(|domain| self.is_host_secure(domain))
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

                if let Some(entry) =
                    HstsEntry::new(host.to_owned(), include_subdomains, Some(header.max_age()))
                {
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
