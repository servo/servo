/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::reflector::Reflector;
use dom::bindings::js::JS;
use dom::document::Document;
use msg::constellation_msg::PipelineId;
use std::cell::Cell;

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// The pipeline associated with this parse, unavailable if this parse
    /// does not correspond to a page load.
    pipeline: Option<PipelineId>,
    /// Input chunks received but not yet passed to the parser.
    pending_input: DOMRefCell<Vec<String>>,
    /// Whether to expect any further input from the associated network request.
    last_chunk_received: Cell<bool>,
}

impl ServoParser {
    pub fn new_inherited(
            document: &Document,
            pipeline: Option<PipelineId>,
            last_chunk_received: bool)
            -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
            pipeline: pipeline,
            pending_input: DOMRefCell::new(vec![]),
            last_chunk_received: Cell::new(last_chunk_received),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn pipeline(&self) -> Option<PipelineId> {
        self.pipeline
    }

    pub fn has_pending_input(&self) -> bool {
        !self.pending_input.borrow().is_empty()
    }

    pub fn push_input_chunk(&self, chunk: String) {
        self.pending_input.borrow_mut().push(chunk);
    }

    pub fn take_next_input_chunk(&self) -> Option<String> {
        let mut pending_input = self.pending_input.borrow_mut();
        if pending_input.is_empty() {
            None
        } else {
            Some(pending_input.remove(0))
        }
    }

    pub fn last_chunk_received(&self) -> bool {
        self.last_chunk_received.get()
    }

    pub fn mark_last_chunk_received(&self) {
        self.last_chunk_received.set(true)
    }
}
