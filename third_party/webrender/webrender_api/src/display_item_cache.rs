/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::display_item::*;
use crate::display_list::*;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CachedDisplayItem {
    item: DisplayItem,
    data: Vec<u8>,
}

impl CachedDisplayItem {
    pub fn display_item(&self) -> &DisplayItem {
        &self.item
    }

    pub fn data_as_item_range<T>(&self) -> ItemRange<T> {
        ItemRange::new(&self.data)
    }
}

impl MallocSizeOf for CachedDisplayItem {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.data.size_of(ops)
    }
}

impl From<DisplayItemRef<'_, '_>> for CachedDisplayItem {
    fn from(item_ref: DisplayItemRef) -> Self {
        let item = item_ref.item();

        match item {
            DisplayItem::Text(..) => CachedDisplayItem {
                item: *item,
                data: item_ref.glyphs().bytes().to_vec(),
            },
            _ => CachedDisplayItem {
                item: *item,
                data: Vec::new(),
            },
        }
    }
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
struct CacheEntry {
    items: Vec<CachedDisplayItem>,
    occupied: bool,
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct DisplayItemCache {
    entries: Vec<CacheEntry>,
}

impl DisplayItemCache {
    fn add_item(&mut self, key: ItemKey, item: CachedDisplayItem) {
        let mut entry = &mut self.entries[key as usize];
        entry.items.push(item);
        entry.occupied = true;
    }

    fn clear_entry(&mut self, key: ItemKey) {
        let mut entry = &mut self.entries[key as usize];
        entry.items.clear();
        entry.occupied = false;
    }

    fn grow_if_needed(&mut self, capacity: usize) {
        if capacity > self.entries.len() {
            self.entries.resize_with(capacity, || CacheEntry {
                items: Vec::new(),
                occupied: false,
            });
        }
    }

    pub fn get_items(&self, key: ItemKey) -> &[CachedDisplayItem] {
        let entry = &self.entries[key as usize];
        debug_assert!(entry.occupied);
        entry.items.as_slice()
    }

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn update(&mut self, display_list: &BuiltDisplayList) {
        self.grow_if_needed(display_list.cache_size());

        let mut iter = display_list.extra_data_iter();
        let mut current_key: Option<ItemKey> = None;
        loop {
            let item = match iter.next() {
                Some(item) => item,
                None => break,
            };

            if let DisplayItem::RetainedItems(key) = item.item() {
                current_key = Some(*key);
                self.clear_entry(*key);
                continue;
            }

            let key = current_key.expect("Missing RetainedItems marker");
            let cached_item = CachedDisplayItem::from(item);
            self.add_item(key, cached_item);
        }
    }
}
