/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use properties::ComputedValues;
use rule_tree::StrongRuleNode;
use selector_impl::PseudoElement;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::sync::atomic::AtomicIsize;

pub struct PrivateStyleData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,

    /// The rule node associated with this node.
    pub rule_node: Option<StrongRuleNode>,

    /// The results of CSS styling for each pseudo-element (if any), with the
    /// corresponding rule node if appropriate.
    pub per_pseudo: HashMap<PseudoElement,
                            (Arc<ComputedValues>, StrongRuleNode),
                            BuildHasherDefault<::fnv::FnvHasher>>,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl PrivateStyleData {
    pub fn new() -> Self {
        PrivateStyleData {
            style: None,
            rule_node: None,
            per_pseudo: HashMap::with_hasher(Default::default()),
            parallel: DomParallelInfo::new(),
        }
    }
}

/// Information that we need stored in each DOM node.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct DomParallelInfo {
    /// The number of children that still need work done.
    pub children_to_process: AtomicIsize,
}

impl DomParallelInfo {
    pub fn new() -> DomParallelInfo {
        DomParallelInfo {
            children_to_process: AtomicIsize::new(0),
        }
    }
}

