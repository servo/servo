/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use net_traits::ResourceThreads;
use rustc_hash::FxHashMap;
use storage_traits::StorageThreads;
use storage_traits::webstorage_thread::{OriginDescriptor, StorageType};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StorageTypes: u32 {
        const LOCAL_STORAGE   = 1 << 0;
        const SESSION_STORAGE = 1 << 1;

        const ALL =
                   Self::LOCAL_STORAGE.bits() |
                   Self::SESSION_STORAGE.bits();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SiteData {
    name: String,
    storage_types: StorageTypes,
}

impl SiteData {
    pub fn new(name: String, storage_types: StorageTypes) -> SiteData {
        SiteData {
            name,
            storage_types,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn storage_types(&self) -> StorageTypes {
        self.storage_types
    }
}

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

    pub fn list_sites(&self, storage_types: StorageTypes) -> Vec<SiteData> {
        let mut builder = SiteDataBuilder::new();

        if storage_types.contains(StorageTypes::LOCAL_STORAGE) {
            let public_origins = self
                .public_storage_threads
                .list_webstorage_origins(StorageType::Local);
            builder.add_origins(public_origins, StorageTypes::LOCAL_STORAGE);

            let private_origins = self
                .private_storage_threads
                .list_webstorage_origins(StorageType::Local);
            builder.add_origins(private_origins, StorageTypes::LOCAL_STORAGE);
        }

        if storage_types.contains(StorageTypes::SESSION_STORAGE) {
            let public_origins = self
                .public_storage_threads
                .list_webstorage_origins(StorageType::Session);
            builder.add_origins(public_origins, StorageTypes::SESSION_STORAGE);

            let private_origins = self
                .private_storage_threads
                .list_webstorage_origins(StorageType::Session);
            builder.add_origins(private_origins, StorageTypes::SESSION_STORAGE);
        }

        builder.build()
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

struct SiteDataBuilder {
    sites: FxHashMap<String, StorageTypes>,
}

impl SiteDataBuilder {
    fn new() -> Self {
        SiteDataBuilder {
            sites: FxHashMap::default(),
        }
    }

    fn add_origins(&mut self, origins: Vec<OriginDescriptor>, storage_type: StorageTypes) {
        for origin in origins {
            self.sites
                .entry(origin.name)
                .and_modify(|types| *types |= storage_type)
                .or_insert(storage_type);
        }
    }

    fn build(self) -> Vec<SiteData> {
        self.sites
            .into_iter()
            .map(|(name, storage_types)| SiteData::new(name, storage_types))
            .collect()
    }
}
