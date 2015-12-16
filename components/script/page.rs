/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, Root};
use dom::document::Document;
use dom::window::Window;
use msg::constellation_msg::PipelineId;
use std::cell::Cell;
use std::rc::Rc;

/// Encapsulates a handle to a frame in a frame tree.
#[derive(JSTraceable, HeapSizeOf)]
#[allow(unrooted_must_root)] // FIXME(#6687) this is wrong
pub struct Page {
    /// Pipeline id associated with this page.
    id: PipelineId,

    /// The outermost frame containing the document and window.
    frame: DOMRefCell<Option<Frame>>,

    /// Indicates if reflow is required when reloading.
    needs_reflow: Cell<bool>,

    // Child Pages.
    pub children: DOMRefCell<Vec<Rc<Page>>>,
}

pub struct PageIterator {
    stack: Vec<Rc<Page>>,
}

pub trait IterablePage {
    fn iter(&self) -> PageIterator;
    fn find(&self, id: PipelineId) -> Option<Rc<Page>>;
}

impl IterablePage for Rc<Page> {
    fn iter(&self) -> PageIterator {
        PageIterator {
            stack: vec!(self.clone()),
        }
    }
    fn find(&self, id: PipelineId) -> Option<Rc<Page>> {
        if self.id == id {
            return Some(self.clone());
        }

        self.children.borrow()
                     .iter()
                     .filter_map(|p| p.find(id))
                     .next()
    }

}

impl Page {
    pub fn new(id: PipelineId) -> Page {
        Page {
            id: id,
            frame: DOMRefCell::new(None),
            needs_reflow: Cell::new(true),
            children: DOMRefCell::new(vec!()),
        }
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn window(&self) -> Root<Window> {
        Root::from_ref(&*self.frame.borrow().as_ref().unwrap().window)
    }

    pub fn document(&self) -> Root<Document> {
        Root::from_ref(&*self.frame.borrow().as_ref().unwrap().document)
    }

    // must handle root case separately
    pub fn remove(&self, id: PipelineId) -> Option<Rc<Page>> {
        let remove_idx = {
            self.children
                .borrow()
                .iter()
                .position(|page_tree| page_tree.id == id)
        };
        match remove_idx {
            Some(idx) => Some(self.children.borrow_mut().remove(idx)),
            None => {
                self.children
                    .borrow_mut()
                    .iter_mut()
                    .filter_map(|page_tree| page_tree.remove(id))
                    .next()
            }
        }
    }
}

impl Iterator for PageIterator {
    type Item = Rc<Page>;

    fn next(&mut self) -> Option<Rc<Page>> {
        let popped = self.stack.pop();
        if let Some(ref page) = popped {
            self.stack.extend(page.children.borrow().iter().cloned());
        }
        popped
    }
}

impl Page {
    pub fn set_reflow_status(&self, status: bool) -> bool {
        let old = self.needs_reflow.get();
        self.needs_reflow.set(status);
        old
    }

    #[allow(unrooted_must_root)]
    pub fn set_frame(&self, frame: Option<Frame>) {
        *self.frame.borrow_mut() = frame;
    }
}

/// Information for one frame in the browsing context.
#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct Frame {
    /// The document for this frame.
    pub document: JS<Document>,
    /// The window object for this frame.
    pub window: JS<Window>,
}
