/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use servo_arc::Arc as ServoArc;
use style::context::QuirksMode;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRule, StylesheetContents, UrlExtraData};
use stylo_atoms::Atom;

const MAX_LENGTH_OF_TEXT_INSERTED_INTO_TABLE: usize = 1024;
const UNIQUE_OWNED: usize = 2;

/// Using [`Atom`] as a cache key to avoid inefficient string content comparison. Although
/// the [`Atom`] is already based on reference counting, an extra [`Rc`] is introduced
/// to trace how many [`style::stylesheets::Stylesheet`]s of style elements are sharing
/// same [`StylesheetContents`], based on the following considerations:
/// * The reference count within [`Atom`] is dedicated to lifecycle management and is not
///   suitable for tracking the number of [`StylesheetContents`]s owners.
/// * The reference count within [`Atom`] is not publicly acessible.
#[derive(Clone, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct StylesheetContentsCacheKey {
    #[conditional_malloc_size_of]
    stylesheet_text: Rc<Atom>,
    base_url: Atom,
    #[ignore_malloc_size_of = "defined in style crate"]
    quirks_mode: QuirksMode,
}

impl StylesheetContentsCacheKey {
    fn new(stylesheet_text: &str, base_url: &str, quirks_mode: QuirksMode) -> Self {
        // The stylesheet text may be quite lengthy, exceeding hundreds of kilobytes.
        // Instead of directly inserting such a huge string into AtomicString table,
        // take its hash value and use that. (This is not a cryptographic hash, so a
        // page could cause collisions if it wanted to.)
        let contents_atom = if stylesheet_text.len() > MAX_LENGTH_OF_TEXT_INSERTED_INTO_TABLE {
            let mut hasher = DefaultHasher::new();
            stylesheet_text.hash(&mut hasher);
            Atom::from(hasher.finish().to_string().as_str())
        } else {
            Atom::from(stylesheet_text)
        };

        Self {
            stylesheet_text: Rc::new(contents_atom),
            base_url: Atom::from(base_url),
            quirks_mode,
        }
    }

    pub(crate) fn is_uniquely_owned(&self) -> bool {
        // The cache itself already holds one reference.
        Rc::strong_count(&self.stylesheet_text) <= UNIQUE_OWNED
    }
}

thread_local! {
    static STYLESHEETCONTENTS_CACHE: RefCell<HashMap<StylesheetContentsCacheKey, ServoArc<StylesheetContents>>> =
       RefCell::default();
}

pub(crate) struct StylesheetContentsCache;

impl StylesheetContentsCache {
    fn contents_can_be_cached(contents: &StylesheetContents, shared_lock: &SharedRwLock) -> bool {
        let guard = shared_lock.read();
        let rules = contents.rules(&guard);
        // The copy-on-write can not be performed when the modification happens on the
        // imported stylesheet, because it containing cssom has no owner dom node.
        !(rules.is_empty() || rules.iter().any(|rule| matches!(rule, CssRule::Import(_))))
    }

    pub(crate) fn get_or_insert_with(
        stylesheet_text: &str,
        shared_lock: &SharedRwLock,
        url_data: UrlExtraData,
        quirks_mode: QuirksMode,
        stylesheetcontents_create_callback: impl FnOnce() -> ServoArc<StylesheetContents>,
    ) -> (
        Option<StylesheetContentsCacheKey>,
        ServoArc<StylesheetContents>,
    ) {
        let cache_key =
            StylesheetContentsCacheKey::new(stylesheet_text, url_data.as_str(), quirks_mode);
        STYLESHEETCONTENTS_CACHE.with_borrow_mut(|stylesheetcontents_cache| {
            let entry = stylesheetcontents_cache.entry(cache_key);
            match entry {
                Entry::Occupied(occupied_entry) => {
                    // Use a copy of the cache key from `Entry` instead of the newly created one above
                    // to correctly update and track to owner count of `StylesheetContents`.
                    (
                        Some(occupied_entry.key().clone()),
                        occupied_entry.get().clone(),
                    )
                },
                Entry::Vacant(vacant_entry) => {
                    let contents = stylesheetcontents_create_callback();
                    if Self::contents_can_be_cached(&contents, shared_lock) {
                        let occupied_entry = vacant_entry.insert_entry(contents.clone());
                        // Use a copy of the cache key from `Entry` instead of the newly created one above
                        // to correctly update and track to owner count of `StylesheetContents`.
                        (Some(occupied_entry.key().clone()), contents)
                    } else {
                        (None, contents)
                    }
                },
            }
        })
    }

    pub(crate) fn remove(cache_key: StylesheetContentsCacheKey) {
        STYLESHEETCONTENTS_CACHE.with_borrow_mut(|stylesheetcontents_cache| {
            stylesheetcontents_cache.remove(&cache_key)
        });
    }
}
