/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use properties::ComputedValues;
use selector_impl::PseudoElement;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;

pub type PseudoStyles = HashMap<PseudoElement, Arc<ComputedValues>,
                                BuildHasherDefault<::fnv::FnvHasher>>;
pub struct PersistentStyleData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for each pseudo-element (if any).
    pub per_pseudo: PseudoStyles,
}

impl PersistentStyleData {
    pub fn new() -> Self {
        PersistentStyleData {
            style: None,
            per_pseudo: HashMap::with_hasher(Default::default()),
        }
    }
}

