/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use servo_svg::element::SvgNode;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;

#[derive(MallocSizeOf)]
pub(crate) struct SvgSubtreeCache {
    entries: HashMap<OpaqueNode, SvgSubtreeCacheEntry>,
}

impl Default for SvgSubtreeCache {
    fn default() -> Self {
        SvgSubtreeCache {
            entries: HashMap::new(),
        }
    }
}

#[derive(MallocSizeOf)]
struct SvgSubtreeCacheEntry {
    dom_version: u64,
    #[conditional_malloc_size_of]
    computed_style: Option<ServoArc<ComputedValues>>,
    #[conditional_malloc_size_of]
    subtree: Arc<SvgNode>,
    // TODO: Add (viewport, currentColor, cross-ref for <use>).
}

/// Why a cache lookup missed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheMiss {
    /// Not cached yet.
    NoEntry,
    /// Node or descendant was mutated.
    SubtreeDirty,
    /// Node's computed style changed.
    ComputedStyle,
    // TODO: Add (Viewport, CurrentColor and ReferenceTarget)
}

impl SvgSubtreeCache {
    /// Return a cached subtree if one is fresh for the given node, or the reason it is stale.
    pub(crate) fn lookup(
        &self,
        node: OpaqueNode,
        dom_version: u64,
        computed_style: Option<&ServoArc<ComputedValues>>,
    ) -> Result<Arc<SvgNode>, CacheMiss> {
        let Some(entry) = self.entries.get(&node) else {
            return Err(CacheMiss::NoEntry);
        };
        if entry.dom_version != dom_version {
            return Err(CacheMiss::SubtreeDirty);
        }
        if !same_computed_style(&entry.computed_style, computed_style) {
            return Err(CacheMiss::ComputedStyle);
        }
        // TODO: check (viewport, currentColor, cross-ref for <use>).
        Ok(Arc::clone(&entry.subtree))
    }

    /// Store a freshly built subtree.
    pub(crate) fn store(
        &mut self,
        node: OpaqueNode,
        dom_version: u64,
        computed_style: Option<ServoArc<ComputedValues>>,
        subtree: Arc<SvgNode>,
    ) {
        self.entries.insert(
            node,
            SvgSubtreeCacheEntry {
                dom_version,
                computed_style,
                subtree,
            },
        );
    }
}

fn same_computed_style<T>(cached: &Option<ServoArc<T>>, current: Option<&ServoArc<T>>) -> bool {
    match (cached, current) {
        (None, None) => true,
        (Some(cached), Some(current)) => ServoArc::ptr_eq(cached, current),
        _ => false,
    }
}
