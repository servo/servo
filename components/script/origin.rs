/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use url::{OpaqueOrigin, Origin as UrlOrigin};
use url::{Url, Host};

/// A representation of an [origin](https://html.spec.whatwg.org/multipage/#origin-2).
#[derive(Clone, HeapSizeOf)]
pub struct Origin {
    #[ignore_heap_size_of = "Rc<T> has unclear ownership semantics"]
    inner: Rc<InnerOrigin>,
}

// We can't use RefCell inside JSTraceable, but Origin doesn't contain JS values and
// DOMRefCell makes it much harder to write unit tests (due to setting up required TLS).
no_jsmanaged_fields!(Origin);

/// An wrapper to encapsulate mutation of an origin in order to support aliasing.
struct InnerOrigin {
    repr: RefCell<OriginRepresentation>,
}

impl InnerOrigin {
    fn set(&self, origin: Origin) {
        let dealiased = origin.inner.dealiased();
        let repr = dealiased.repr.borrow().clone();
        *self.repr.borrow_mut() = repr;
    }

    fn is_scheme_host_port_tuple(&self) -> bool {
        self.repr.borrow().is_scheme_host_port_tuple()
    }

    fn host(&self) -> Option<Host> {
        self.repr.borrow().host().clone()
    }
}

/// The representation of the different types of origins.
#[derive(Clone)]
enum OriginRepresentation {
    /// An origin defined by the [URL specification](https://url.spec.whatwg.org/#concept-url-origin)
    Origin(UrlOrigin),
    /// A transparent alias to an existing origin.
    Alias(Rc<InnerOrigin>),
}

impl OriginRepresentation {
    fn is_scheme_host_port_tuple(&self) -> bool {
        match *self {
            OriginRepresentation::Origin(UrlOrigin::Tuple(..)) => true,
            OriginRepresentation::Origin(UrlOrigin::UID(..)) => false,
            OriginRepresentation::Alias(ref origin) => origin.is_scheme_host_port_tuple(),
        }
    }

    fn host(&self) -> Option<Host> {
        match *self {
            OriginRepresentation::Origin(UrlOrigin::Tuple(_, ref host, _)) => Some(host.clone()),
            OriginRepresentation::Origin(UrlOrigin::UID(..)) => None,
            OriginRepresentation::Alias(ref origin) => origin.host(),
        }
    }
}

impl Origin {
    #[allow(dead_code)]
    /// Set this origin to another de-aliased origin.
    pub fn set(&self, origin: Origin) {
        self.inner.set(origin);
    }

    /// Create a new origin comprising a unique, opaque identifier.
    pub fn opaque_identifier() -> Origin {
        let opaque = UrlOrigin::UID(OpaqueOrigin::new());
        Origin {
            inner: Rc::new(InnerOrigin {
                repr: RefCell::new(OriginRepresentation::Origin(opaque)),
            }),
        }
    }

    /// Create a new origin that aliases the callee.
    pub fn alias(&self) -> Origin {
        Origin {
            inner: Rc::new(InnerOrigin {
                repr: RefCell::new(OriginRepresentation::Alias(self.inner.clone())),
            }),
        }
    }

    /// Create a new origin for the given URL.
    pub fn new(url: &Url) -> Origin {
        Origin {
            inner: Rc::new(InnerOrigin {
                repr: RefCell::new(OriginRepresentation::Origin(url.origin())),
            }),
        }
    }

    /// Does this (possibly dealiased) origin represent a host/scheme/port tuple?
    pub fn is_scheme_host_port_tuple(&self) -> bool {
        self.inner.is_scheme_host_port_tuple()
    }

    /// Return the host associated with this origin.
    pub fn host(&self) -> Option<Host> {
        self.inner.host()
    }
}

trait Dealias {
    fn dealiased(&self) -> Self;
}

impl Dealias for Rc<InnerOrigin> {
    fn dealiased(&self) -> Rc<InnerOrigin> {
        match *self.repr.borrow() {
            OriginRepresentation::Alias(ref aliased) => aliased.dealiased(),
            _ => self.clone(),
        }
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Origin) -> bool {
        let first = self.inner.dealiased();
        let second = other.inner.dealiased();
        let first = first.repr.borrow();
        let second = second.repr.borrow();
        match (&*first, &*second) {
            (&OriginRepresentation::Origin(ref origin1),
             &OriginRepresentation::Origin(ref origin2)) => origin1 == origin2,
            _ => unreachable!(),
        }
    }
}
