/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ResourceThreads;

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
