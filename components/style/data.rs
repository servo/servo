/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::ComputedValues;
use selectors::parser::SelectorImpl;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::AtomicIsize;

pub struct PrivateStyleData<Impl: SelectorImpl>
    where Impl::PseudoElement: Eq + Hash {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for each pseudo-element (if any).
    pub per_pseudo: HashMap<Impl::PseudoElement, Option<Arc<ComputedValues>>>,
}

impl<Impl: SelectorImpl> PrivateStyleData<Impl>
    where Impl::PseudoElement: Eq + Hash {
    pub fn new() -> PrivateStyleData<Impl> {
        PrivateStyleData {
            style: None,
            per_pseudo: HashMap::new(),
            parallel: DomParallelInfo::new(),
        }
    }
}

/// Information that we need stored in each DOM node.
#[derive(HeapSizeOf)]
pub struct DomParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicIsize,
}

impl DomParallelInfo {
    pub fn new() -> DomParallelInfo {
        DomParallelInfo {
            children_count: AtomicIsize::new(0),
        }
    }
}

