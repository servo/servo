/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ResourceThreads;

pub struct CookieManager {
    public_resource_threads: ResourceThreads,
    private_resource_threads: ResourceThreads,
}

// TODO: Eventually rename to `NetworkManager` that could also cover protocol
// handlers and related networking functionality.
impl CookieManager {
    pub(crate) fn new(
        public_resource_threads: ResourceThreads,
        private_resource_threads: ResourceThreads,
    ) -> Self {
        Self {
            public_resource_threads,
            private_resource_threads,
        }
    }

    pub fn clear_cookies(&self) {
        self.public_resource_threads.clear_cookies();
        self.private_resource_threads.clear_cookies();
    }
}
