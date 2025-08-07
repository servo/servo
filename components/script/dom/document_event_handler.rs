/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use embedder_traits::Cursor;
use euclid::Point2D;
use style_traits::CSSPixel;

use crate::dom::bindings::root::MutNullableDom;
use crate::dom::types::Element;

/// The [`DocumentEventHandler`] is a structure responsible for handling events for
/// the [`crate::Document`] and storing data related to event handling. It exists to
/// decrease the size of the [`crate::Document`] structure.
#[derive(Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentEventHandler {
    /// The element that is currently hovered by the cursor.
    pub(crate) current_hover_target: MutNullableDom<Element>,
    /// The most recent mouse movement point, used for processing `mouseleave` events.
    #[no_trace]
    pub(crate) most_recent_mousemove_point: Point2D<f32, CSSPixel>,
    /// The currently set [`Cursor`] or `None` if the `Document` isn't being hovered
    /// by the cursor.
    #[no_trace]
    pub(crate) current_cursor: Cell<Option<Cursor>>,
}
