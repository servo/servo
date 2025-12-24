/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use net_traits::ResourceThreads;
use rustc_hash::FxHashMap;
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::{OriginDescriptor, WebStorageType};

bitflags! {
    /// Identifies categories of site data associated with a site.
    ///
    /// This type is used by `SiteDataManager` to query, describe, and manage
    /// different kinds of data stored by the user agent for a given site.
    ///
    /// Additional storage categories (e.g. cookies, IndexedDB) may be added in
    /// the future.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StorageType: u8 {
        /// Corresponds to the `localStorage` Web API:
        /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage>
        const Local   = 1 << 0;

        /// Corresponds to the `sessionStorage` Web API:
        /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/sessionStorage>
        const Session = 1 << 1;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SiteData {
    name: String,
    storage_types: StorageType,
}

impl SiteData {
    pub fn new(name: String, storage_types: StorageType) -> SiteData {
        SiteData {
            name,
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
/// associated with a site (currently equivalent to an origin), such as
/// web exposed storage mechanisms like `localStorage` and `sessionStorage`.
///
/// The manager can be used by embedders to list sites with stored data.
/// Support for site scoped management operations (e.g. clearing data for a
/// specific site) will be added in the future.
///
/// At this stage, sites correspond directly to origins. Future work may group
/// data at a higher level (e.g. by eTLD+1).
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
    /// Each [`SiteData`] entry represents a site (currently equivalent to an
    /// origin) and indicates which kinds of storage data are present for it
    /// (e.g. localStorage, sessionStorage).
    ///
    /// The returned list is sorted by site name.
    ///
    /// Both public and private storage are included in the result.
    ///
    /// Note: At this stage, sites correspond directly to origins. Future work
    /// may group data at a higher level (e.g. by eTLD+1).
    pub fn site_data(&self, storage_types: StorageType) -> Vec<SiteData> {
        let mut sites: FxHashMap<String, StorageType> = FxHashMap::default();

        let mut add_origins = |origins: Vec<OriginDescriptor>, storage_type: StorageType| {
            for origin in origins {
                sites
                    .entry(origin.name)
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

        let mut result: Vec<SiteData> = sites
            .into_iter()
            .map(|(name, storage_types)| SiteData::new(name, storage_types))
            .collect();

        result.sort_by_key(|a| a.name());

        result
    }

    pub fn clear_cookies(&self) {
        self.public_resource_threads.clear_cookies();
        self.private_resource_threads.clear_cookies();
    }
}
