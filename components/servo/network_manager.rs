/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::ResourceThreads;

pub struct NetworkManager {
    _public_resource_threads: ResourceThreads,
    _private_resource_threads: ResourceThreads,
}

// Placeholder for protocol handlers and related networking functionality.
impl NetworkManager {
    pub(crate) fn new(
        public_resource_threads: ResourceThreads,
        private_resource_threads: ResourceThreads,
    ) -> Self {
        Self {
            _public_resource_threads: public_resource_threads,
            _private_resource_threads: private_resource_threads,
        }
    }
}
