/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_url::ServoUrl;
use std::sync::Arc;
use url::Host;
use url::Origin as UrlOrigin;

/// A representation of an [origin](https://html.spec.whatwg.org/multipage/#origin-2).
#[derive(HeapSizeOf, JSTraceable)]
pub struct Origin {
    #[ignore_heap_size_of = "Arc<T> has unclear ownership semantics"]
    inner: Arc<UrlOrigin>,
}

impl Origin {
    /// Create a new origin comprising a unique, opaque identifier.
    pub fn opaque_identifier() -> Origin {
        Origin {
            inner: Arc::new(UrlOrigin::new_opaque()),
        }
    }

    /// Create a new origin for the given URL.
    pub fn new(url: &ServoUrl) -> Origin {
        Origin {
            inner: Arc::new(url.origin()),
        }
    }

    /// Does this origin represent a host/scheme/port tuple?
    pub fn is_scheme_host_port_tuple(&self) -> bool {
        self.inner.is_tuple()
    }

    /// Return the host associated with this origin.
    pub fn host(&self) -> Option<&Host<String>> {
        match *self.inner {
            UrlOrigin::Tuple(_, ref host, _) => Some(host),
            UrlOrigin::Opaque(..) => None,
        }
    }

    /// https://html.spec.whatwg.org/multipage/#same-origin
    pub fn same_origin(&self, other: &Origin) -> bool {
        self.inner == other.inner
    }

    pub fn copy(&self) -> Origin {
        Origin {
            inner: Arc::new((*self.inner).clone()),
        }
    }

    pub fn alias(&self) -> Origin {
        Origin {
            inner: self.inner.clone(),
        }
    }
}
