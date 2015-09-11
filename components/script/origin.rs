/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use url::Origin as UrlOrigin;
use url::Url;

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
