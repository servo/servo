/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A structure to collect information about the cascade.

#![deny(missing_docs)]

use dom::TNode;
use properties::{DeclaredValue, PropertyDeclaration};
use values::HasViewportPercentage;

/// A structure to collect information about the cascade.
///
/// This is useful to gather information about what an element is affected by,
/// and can be used in the future to track optimisations like when a
/// non-inherited property is explicitly inherited, in order to cut-off the
/// traversal.
pub struct CascadeInfo {
    /// Whether we've seen viewport units so far.
    pub saw_viewport_units: bool,
    /// Whether the cascade has been marked as finished. This is a debug-only
    /// flag to ensure `finish` is called, given it's optional to not pass a
    /// `CascadeInfo`.
    #[cfg(debug_assertions)]
    finished: bool,
}

impl CascadeInfo {
    /// Construct a new `CascadeInfo`.
    #[cfg(debug_assertions)]
    pub fn new() -> Self {
        CascadeInfo {
            saw_viewport_units: false,
            finished: false,
        }
    }

    /// Construct a new `CascadeInfo`.
    #[cfg(not(debug_assertions))]
    pub fn new() -> Self {
        CascadeInfo {
            saw_viewport_units: false,
        }
    }

    /// Called when a property is cascaded.
    ///
    /// NOTE: We can add a vast amount of information here.
    #[inline]
    pub fn on_cascade_property<T>(&mut self,
                                  _property_declaration: &PropertyDeclaration,
                                  value: &DeclaredValue<T>)
        where T: HasViewportPercentage,
    {
        // TODO: we can be smarter and keep a property bitfield to keep track of
        // the last applying rule.
        if value.has_viewport_percentage() {
            self.saw_viewport_units = true;
        }
    }

    #[cfg(debug_assertions)]
    fn mark_as_finished_if_appropriate(&mut self) {
        self.finished = true;
    }

    #[cfg(not(debug_assertions))]
    fn mark_as_finished_if_appropriate(&mut self) {}

    /// Called when the cascade is finished, in order to use the information
    /// we've collected.
    ///
    /// Currently used for styling to mark a node as needing restyling when the
    /// viewport size changes.
    #[allow(unsafe_code)]
    pub fn finish<N: TNode>(mut self, node: &N) {
        self.mark_as_finished_if_appropriate();

        if self.saw_viewport_units {
            unsafe {
                node.set_dirty_on_viewport_size_changed();
            }
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for CascadeInfo {
    fn drop(&mut self) {
        debug_assert!(self.finished,
                      "Didn't use the result of CascadeInfo, if you don't need \
                      it, consider passing None");
    }
}
