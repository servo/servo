/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ResourceThreads;
use storage_traits::StorageThreads;

pub struct SiteDataManager {
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
    _public_storage_threads: StorageThreads,
    _private_storage_threads: StorageThreads,
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
            _public_storage_threads: public_storage_threads,
            _private_storage_threads: private_storage_threads,
        }
    }

    pub fn clear_cookies(&self) {
        self.public_resource_threads.clear_cookies();
        self.private_resource_threads.clear_cookies();
    }

    pub fn clear_cache(&self) {
        self.public_resource_threads.clear_cache();
        self.private_resource_threads.clear_cache();
    }
}
