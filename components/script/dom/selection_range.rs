/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::abstractrange::BoundaryPoint;
use crate::dom::bindings::root::Dom;
use crate::dom::node::Node;
use crate::dom::range::Range;

/// A selection boundary. This is similar to `BoundaryPoint`, but supports
/// positions in the composed tree.
#[derive(Clone, JSTraceable, PartialEq, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct SelectionBoundary {
    pub container: Dom<Node>,
    pub offset: u32,
}

impl SelectionBoundary {
    pub(crate) fn new(container: &Node, offset: u32) -> Self {
        Self {
            container: Dom::from_ref(container),
            offset,
        }
    }
}

impl PartialEq<BoundaryPoint> for SelectionBoundary {
    fn eq(&self, boundary_point: &BoundaryPoint) -> bool {
        *self.container == *boundary_point.node().get() &&
            self.offset as usize == boundary_point.offset().0
    }
}

#[derive(JSTraceable, PartialEq, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct SelectionRange {
    pub start: SelectionBoundary,
    pub end: SelectionBoundary,
}

impl SelectionRange {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(start: SelectionBoundary, end: SelectionBoundary) -> Self {
        Self { start, end }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn collapsed_at(at: SelectionBoundary) -> Self {
        Self {
            start: at.clone(),
            end: at,
        }
    }

    pub(crate) fn collapsed(&self) -> bool {
        self.start == self.end
    }

    pub(crate) fn start_and_end_are_in_document_tree(&self) -> bool {
        self.start.container.is_in_a_document_tree() && self.end.container.is_in_a_document_tree()
    }
}

impl From<&Range> for SelectionRange {
    fn from(range: &Range) -> Self {
        Self::new(
            SelectionBoundary::new(&range.start_container(), range.start_offset()),
            SelectionBoundary::new(&range.end_container(), range.end_offset()),
        )
    }
}
