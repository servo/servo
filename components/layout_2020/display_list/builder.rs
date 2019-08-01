/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Builds display lists from flows and fragments.
//!
//! Other browser engines sometimes call this "painting", but it is more accurately called display
//! list building, as the actual painting does not happen here—only deciding *what* we're going to
//! paint.

use crate::display_list::items::OpaqueNode;
use app_units::Au;
use euclid::default::Point2D;
use fnv::FnvHashMap;
use gfx::text::glyph::ByteIndex;
use gfx::text::TextRun;
use range::Range;
use std::sync::Arc;

pub struct IndexableTextItem {
    /// The placement of the text item on the plane.
    pub origin: Point2D<Au>,
    /// The text run.
    pub text_run: Arc<TextRun>,
    /// The range of text within the text run.
    pub range: Range<ByteIndex>,
    /// The position of the start of the baseline of this text.
    pub baseline_origin: Point2D<Au>,
}

#[derive(Default)]
pub struct IndexableText {
    inner: FnvHashMap<OpaqueNode, Vec<IndexableTextItem>>,
}

impl IndexableText {
    pub fn get(&self, node: OpaqueNode) -> Option<&[IndexableTextItem]> {
        self.inner.get(&node).map(|x| x.as_slice())
    }

    // Returns the text index within a node for the point of interest.
    pub fn text_index(&self, node: OpaqueNode, point_in_item: Point2D<Au>) -> Option<usize> {
        let item = self.inner.get(&node)?;
        // TODO(#20020): access all elements
        let point = point_in_item + item[0].origin.to_vector();
        let offset = point - item[0].baseline_origin;
        Some(
            item[0]
                .text_run
                .range_index_of_advance(&item[0].range, offset.x),
        )
    }
}
