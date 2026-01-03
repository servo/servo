/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use log::warn;
use net_traits::pub_domains::registered_domain_name;
use net_traits::{ResourceThreads, SiteDescriptor};
use rustc_hash::FxHashMap;
use servo_url::ServoUrl;
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::{OriginDescriptor, WebStorageType};

bitflags! {
    /// Identifies categories of site data associated with a site.
    ///
    /// This type is used by `SiteDataManager` to query, describe, and manage
    /// different kinds of data stored by the user agent for a given site.
    ///
    /// Additional storage categories (e.g. IndexedDB) may be added in the
    /// future.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StorageType: u8 {
        /// Corresponds to the HTTP cookies:
        /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Cookies>
        const Cookies = 1 << 0;

        /// Corresponds to the `localStorage` Web API:
        /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage>
        const Local   = 1 << 1;

        /// Corresponds to the `sessionStorage` Web API:
        /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/sessionStorage>
        const Session = 1 << 2;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SiteData {
    name: String,
    storage_types: StorageType,
}

impl SiteData {
    pub fn new(name: impl Into<String>, storage_types: StorageType) -> SiteData {
        SiteData {
            name: name.into(),
            storage_types,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn storage_types(&self) -> StorageType {
        self.storage_types
    }
}

/// Provides APIs for inspecting and managing site data.
///
/// `SiteDataManager` exposes information about data that is conceptually
/// associated with a site (equivalent to an eTLD+1), such as web exposed
/// storage mechanisms like `localStorage` and `sessionStorage`.
///
/// The manager can be used by embedders to list sites with stored data.
/// Support for site scoped management operations (e.g. clearing data for a
/// specific site) will be added in the future.
///
/// Note: Network layer state (such as the HTTP cache) is intentionally not
/// handled by `SiteDataManager`. That functionality lives in `NetworkManager`.
pub struct SiteDataManager {
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
    public_storage_threads: StorageThreads,
    private_storage_threads: StorageThreads,
}

impl SiteDataManager {
    pub(crate) fn new(
        public_resource_threads: ResourceThreads,
        private_resource_threads: ResourceThreads,
        public_storage_threads: StorageThreads,
        private_storage_threads: StorageThreads,
    ) -> Self {
        Self {
            public_resource_threads,
            private_resource_threads,
            public_storage_threads,
            private_storage_threads,
        }
    }

    /// Return a list of sites that have associated site data.
    ///
    /// The returned list is filtered by the provided `storage_types` bitflags.
    /// Each [`SiteData`] entry represents a site (equivalent to an eTLD+1)
    /// and indicates which kinds of storage data are present for it (e.g.
    /// localStorage, sessionStorage).
    ///
    /// The returned list is sorted by site name.
    ///
    /// Both public and private storage are included in the result.
    pub fn site_data(&self, storage_types: StorageType) -> Vec<SiteData> {
        let mut all_sites: FxHashMap<String, StorageType> = FxHashMap::default();

        let mut add_sites = |sites: Vec<SiteDescriptor>, storage_type: StorageType| {
            for site in sites {
                all_sites
                    .entry(site.name)
                    .and_modify(|storage_types| *storage_types |= storage_type)
                    .or_insert(storage_type);
            }
        };

        if storage_types.contains(StorageType::Cookies) {
            let public_cookies = self.public_resource_threads.cookies();
            add_sites(public_cookies, StorageType::Cookies);

            let private_cookies = self.private_resource_threads.cookies();
            add_sites(private_cookies, StorageType::Cookies);
        }

        let mut add_origins = |origins: Vec<OriginDescriptor>, storage_type: StorageType| {
            for origin in origins {
                let url =
                    ServoUrl::parse(&origin.name).expect("Should always be able to parse origins.");

                let Some(domain) = registered_domain_name(&url) else {
                    warn!("Failed to get a registered domain name for: {url}.");
                    continue;
                };
                let domain = domain.to_string();

                all_sites
                    .entry(domain)
                    .and_modify(|storage_types| *storage_types |= storage_type)
                    .or_insert(storage_type);
            }
        };

        if storage_types.contains(StorageType::Local) {
            let public_origins = self
                .public_storage_threads
                .webstorage_origins(WebStorageType::Local);
            add_origins(public_origins, StorageType::Local);

            let private_origins = self
                .private_storage_threads
                .webstorage_origins(WebStorageType::Local);
            add_origins(private_origins, StorageType::Local);
        }

        if storage_types.contains(StorageType::Session) {
            let public_origins = self
                .public_storage_threads
                .webstorage_origins(WebStorageType::Session);
            add_origins(public_origins, StorageType::Session);

            let private_origins = self
                .private_storage_threads
                .webstorage_origins(WebStorageType::Session);
            add_origins(private_origins, StorageType::Session);
        }

        let mut result: Vec<SiteData> = all_sites
            .into_iter()
            .map(|(name, storage_types)| SiteData::new(name, storage_types))
            .collect();

        result.sort_by_key(SiteData::name);

        result
    }

    /// Clear site data for the given sites.
    ///
    /// The clearing is restricted to the provided `storage_types` bitflags.
    /// Both public and private browsing data are affected.
    ///
    /// TODO: At present this method can only clear cookies and sessionStorage
    /// data for the specified sites. Support for localStorage will be added
    /// in a follow-up.
    pub fn clear_site_data(&self, sites: &[&str], storage_types: StorageType) {
        if storage_types.contains(StorageType::Cookies) {
            self.public_resource_threads.clear_cookies_for_sites(sites);
            self.private_resource_threads.clear_cookies_for_sites(sites);
        }

        if storage_types.contains(StorageType::Session) {
            self.public_storage_threads
                .clear_webstorage_for_sites(WebStorageType::Session, sites);
            self.private_storage_threads
                .clear_webstorage_for_sites(WebStorageType::Session, sites);
        }
    }

    pub fn clear_cookies(&self) {
        self.public_resource_threads.clear_cookies();
        self.private_resource_threads.clear_cookies();
    }
}
