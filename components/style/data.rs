/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::ComputedValues;
use std::sync::Arc;
use std::sync::atomic::AtomicIsize;

pub struct PrivateStyleData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `before` pseudo-element, if any.
    pub before_style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `after` pseudo-element, if any.
    pub after_style: Option<Arc<ComputedValues>>,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl PrivateStyleData {
    pub fn new() -> PrivateStyleData {
        PrivateStyleData {
            style: None,
            before_style: None,
            after_style: None,
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

