/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ref_filter_map::ref_filter_map;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use url::Origin as UrlOrigin;
use url::{Url, Host};

/// A representation of an [origin](https://html.spec.whatwg.org/multipage/#origin-2).
#[derive(HeapSizeOf)]
pub struct Origin {
    #[ignore_heap_size_of = "Rc<T> has unclear ownership semantics"]
    inner: Rc<RefCell<UrlOrigin>>,
}

// We can't use RefCell inside JSTraceable, but Origin doesn't contain JS values and
// DOMRefCell makes it much harder to write unit tests (due to setting up required TLS).
no_jsmanaged_fields!(Origin);

impl Origin {
    /// Create a new origin comprising a unique, opaque identifier.
    pub fn opaque_identifier() -> Origin {
        Origin {
            inner: Rc::new(RefCell::new(UrlOrigin::new_opaque())),
        }
    }

    /// Create a new origin for the given URL.
    pub fn new(url: &Url) -> Origin {
        Origin {
            inner: Rc::new(RefCell::new(url.origin())),
        }
    }

    pub fn set(&self, origin: UrlOrigin) {
        *self.inner.borrow_mut() = origin;
    }

    /// Does this origin represent a host/scheme/port tuple?
    pub fn is_scheme_host_port_tuple(&self) -> bool {
        self.inner.borrow().is_tuple()
    }

    /// Return the host associated with this origin.
    pub fn host(&self) -> Option<Ref<Host<String>>> {
        ref_filter_map(self.inner.borrow(), |origin| match *origin {
            UrlOrigin::Tuple(_, ref host, _) => Some(host),
            UrlOrigin::Opaque(..) => None,
        })
    }

    /// https://html.spec.whatwg.org/multipage/#same-origin
    pub fn same_origin(&self, other: &Origin) -> bool {
        *self.inner.borrow() == *other.inner.borrow()
    }

    pub fn copy(&self) -> Origin {
        Origin {
            inner: Rc::new(RefCell::new(self.inner.borrow().clone())),
        }
    }

    pub fn alias(&self) -> Origin {
        Origin {
            inner: self.inner.clone(),
        }
    }
}
