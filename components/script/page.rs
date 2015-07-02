/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, Root};
use dom::document::{Document, DocumentHelpers};
use dom::node::NodeHelpers;
use dom::window::Window;

use msg::constellation_msg::PipelineId;
use std::cell::Cell;
use std::rc::Rc;
use url::Url;

/// Encapsulates a handle to a frame in a frame tree.
#[derive(JSTraceable)]
pub struct Page {
    /// Pipeline id associated with this page.
    id: PipelineId,

    /// The outermost frame containing the document and window.
    frame: DOMRefCell<Option<Frame>>,

    /// Cached copy of the most recent url loaded by the script, after all redirections.
    /// TODO(tkuehn): this currently does not follow any particular caching policy
    /// and simply caches pages forever (!).
    url: Url,

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
        if self.id == id { return Some(self.clone()); }
        for page in self.children.borrow().iter() {
            let found = page.find(id);
            if found.is_some() { return found; }
        }
        None
    }

}

impl Page {
    pub fn new(id: PipelineId, url: Url) -> Page {
        Page {
            id: id,
            frame: DOMRefCell::new(None),
            url: url,
            needs_reflow: Cell::new(true),
            children: DOMRefCell::new(vec!()),
        }
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn window(&self) -> Root<Window> {
        self.frame.borrow().as_ref().unwrap().window.root()
    }

    pub fn document(&self) -> Root<Document> {
        self.frame.borrow().as_ref().unwrap().document.root()
    }

    // must handle root case separately
    pub fn remove(&self, id: PipelineId) -> Option<Rc<Page>> {
        let remove_idx = {
            self.children
                .borrow_mut()
                .iter_mut()
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
        match self.stack.pop() {
            Some(next) => {
                for child in next.children.borrow().iter() {
                    self.stack.push(child.clone());
                }
                Some(next)
            },
            None => None,
        }
    }
}

impl Page {
    pub fn set_reflow_status(&self, status: bool) -> bool {
        let old = self.needs_reflow.get();
        self.needs_reflow.set(status);
        old
    }

    pub fn set_frame(&self, frame: Option<Frame>) {
        *self.frame.borrow_mut() = frame;
    }
}

/// Information for one frame in the browsing context.
#[derive(JSTraceable)]
#[must_root]
pub struct Frame {
    /// The document for this frame.
    pub document: JS<Document>,
    /// The window object for this frame.
    pub window: JS<Window>,
}
