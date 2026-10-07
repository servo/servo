/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use js::context::NoGC;
use script_bindings::root::DomRoot;
use servo_base::text::Utf32CodeUnitsOrNodeOffset;

use crate::dom::abstractrange::BoundaryPoint;
use crate::dom::bindings::root::Dom;
use crate::dom::comparator::compare_dom_positions;
use crate::dom::node::Node;
use crate::dom::range::Range;
use crate::dom::traversal::FlatTreeForSelectionNoGcTraversal;

/// A rooted selection boundary. This is similar to `SelectionBoundary`, but is rooted.
/// This means it is appropriate to use in variables stored on the stack.
#[derive(Clone, PartialEq)]
pub(crate) struct RootedSelectionBoundary {
    pub container: DomRoot<Node>,
    pub offset: u32,
}

impl RootedSelectionBoundary {
    pub(crate) fn start_of(node: &Node) -> Self {
        Self {
            container: DomRoot::from_ref(node),
            offset: 0,
        }
    }

    pub(crate) fn end_of(node: &Node) -> Self {
        Self {
            container: DomRoot::from_ref(node),
            offset: node.len(),
        }
    }

    pub(crate) fn new_with_utf16_offset(container: DomRoot<Node>, offset: u32) -> Self {
        Self { container, offset }
    }

    pub(crate) fn new_with_utf32_offset(
        container: DomRoot<Node>,
        utf32_offset: Utf32CodeUnitsOrNodeOffset,
    ) -> Self {
        let offset = container.to_sibling_or_utf16_offset(utf32_offset);
        Self { container, offset }
    }

    pub(crate) fn utf32_offset(&self) -> Utf32CodeUnitsOrNodeOffset {
        self.container.to_sibling_or_utf32_offset(self.offset)
    }

    pub(crate) fn compare_dom_positions(&self, no_gc: &NoGC, other: &Self) -> Option<Ordering> {
        compare_dom_positions::<FlatTreeForSelectionNoGcTraversal>(
            no_gc,
            &self.container,
            self.offset,
            &other.container,
            other.offset,
        )
        .0
    }
}

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
        *self.container == *boundary_point.node().get() && self.offset == boundary_point.offset().0
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
}

impl From<&Range> for SelectionRange {
    fn from(range: &Range) -> Self {
        Self::new(
            SelectionBoundary::new(&range.start_container(), range.start_offset()),
            SelectionBoundary::new(&range.end_container(), range.end_offset()),
        )
    }
}
