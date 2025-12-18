/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use net_traits::ResourceThreads;

#[derive(Clone, Debug, PartialEq)]
pub struct CacheEntry {
    key: String,
}

impl CacheEntry {
    pub fn new(key: String) -> CacheEntry {
        CacheEntry { key }
    }

    pub fn key(&self) -> String {
        self.key.clone()
    }
}

/// Provides APIs for managing network-related state.
///
/// `NetworkManager` is responsible for data owned by the networking layer,
/// such as the HTTP cache. This data is not considered site data and is
/// therefore intentionally separate from `SiteDataManager`.
pub struct NetworkManager {
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
}

impl NetworkManager {
    pub(crate) fn new(
        public_resource_threads: ResourceThreads,
        private_resource_threads: ResourceThreads,
    ) -> Self {
        Self {
            public_resource_threads,
            private_resource_threads,
        }
    }

    /// Returns cache entries currently stored in the HTTP cache.
    ///
    /// The returned list contains one [`CacheEntry`] per unique cache key
    /// (URL) for which the networking layer currently maintains cached
    /// responses.
    ///
    /// Both public and private browsing contexts are included in the result.
    ///
    /// Note: The networking layer currently only implements an in-memory HTTP
    /// cache. Support for an on-disk cache is under development.
    pub fn cache_entries(&self) -> Vec<CacheEntry> {
        let mut entries: HashSet<String> = HashSet::default();

        let public_entries = self.public_resource_threads.cache_entries();
        for public_entry in public_entries {
            entries.insert(public_entry.key);
        }

        let private_entries = self.private_resource_threads.cache_entries();
        for private_entry in private_entries {
            entries.insert(private_entry.key);
        }

        entries.into_iter().map(CacheEntry::new).collect()
    }

    /// Clears the network (HTTP) cache.
    ///
    /// This removes all cached network responses maintained by the networking
    /// layer for both public and private browsing contexts.
    ///
    /// Note: The networking layer currently only implements an in-memory HTTP
    /// cache. Support for an on-disk cache is under development.
    pub fn clear_cache(&self) {
        self.public_resource_threads.clear_cache();
        self.private_resource_threads.clear_cache();
    }
}
