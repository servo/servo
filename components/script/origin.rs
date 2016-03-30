/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use url::Origin as UrlOrigin;
use url::{Url, Host};

/// A representation of an [origin](https://url.spec.whatwg.org/#origin)
#[derive(Clone, JSTraceable, HeapSizeOf)]
pub struct Origin {
    #[ignore_heap_size_of = "Rc<T> has unclear ownership semantics"]
    repr: Rc<OriginRepresentation>,
}

#[derive(Clone, JSTraceable)]
enum OriginRepresentation {
    Origin(UrlOrigin),
    Alias(Rc<OriginRepresentation>),
}

impl OriginRepresentation {
    fn is_scheme_host_port_tuple(&self) -> bool {
        match *self {
            OriginRepresentation::Origin(UrlOrigin::Tuple(..)) => true,
            OriginRepresentation::Origin(UrlOrigin::UID(..)) => false,
            OriginRepresentation::Alias(ref origin) => origin.is_scheme_host_port_tuple(),
        }
    }

    fn host(&self) -> Option<&Host> {
        match *self {
            OriginRepresentation::Origin(UrlOrigin::Tuple(_, ref host, _)) => Some(host),
            OriginRepresentation::Origin(UrlOrigin::UID(..)) => None,
            OriginRepresentation::Alias(ref origin) => origin.host(),
        }
    }
}

impl Origin {
    pub fn opaque_identifier() -> Origin {
        Origin {
            repr: Rc::new(OriginRepresentation::Origin(url!("file:///tmp").origin())),
        }
    }

    pub fn alias(&self) -> Origin {
        Origin {
            repr: Rc::new(OriginRepresentation::Alias(self.repr.clone())),
        }
    }

    pub fn new(url: &Url) -> Origin {
        Origin {
            repr: Rc::new(OriginRepresentation::Origin(url.origin())),
        }
    }

    pub fn is_scheme_host_port_tuple(&self) -> bool {
        self.repr.is_scheme_host_port_tuple()
    }

    pub fn host(&self) -> Option<&Host> {
        self.repr.host()
    }
}

trait Dealias {
    fn dealiased(&self) -> Self;
}

impl Dealias for Rc<OriginRepresentation> {
    fn dealiased(&self) -> Rc<OriginRepresentation> {
        match **self {
            OriginRepresentation::Alias(ref aliased) => aliased.dealiased(),
            _ => self.clone(),
        }
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Origin) -> bool {
        match (&*self.repr.dealiased(), &*other.repr.dealiased()) {
            (&OriginRepresentation::Origin(ref origin1),
             &OriginRepresentation::Origin(ref origin2)) => origin1 == origin2,
            _ => false,
        }
    }
}
