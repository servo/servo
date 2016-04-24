/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::ComputedValues;
use selectors::parser::SelectorImpl;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::sync::atomic::AtomicIsize;

pub struct PrivateStyleData<Impl: SelectorImpl, ConcreteComputedValues: ComputedValues> {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ConcreteComputedValues>>,

    /// The results of CSS styling for each pseudo-element (if any).
    pub per_pseudo: HashMap<Impl::PseudoElement, Arc<ConcreteComputedValues>,
                            BuildHasherDefault<::fnv::FnvHasher>>,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl<Impl, ConcreteComputedValues> PrivateStyleData<Impl, ConcreteComputedValues>
    where Impl: SelectorImpl, ConcreteComputedValues: ComputedValues {
    pub fn new() -> PrivateStyleData<Impl, ConcreteComputedValues> {
        PrivateStyleData {
            style: None,
            per_pseudo: HashMap::with_hasher(Default::default()),
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

