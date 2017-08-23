/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_rand;
use servo_rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use url::{Host, Origin};
use url_serde;
use uuid::Uuid;

/// The origin of an URL
#[derive(Clone, Debug, Deserialize, Eq, HeapSizeOf, PartialEq, Serialize)]
pub enum ImmutableOrigin {
    /// A globally unique identifier
    Opaque(OpaqueOrigin),

    /// Consists of the URL's scheme, host and port
    Tuple(
        String,
        #[serde(deserialize_with = "url_serde::deserialize", serialize_with = "url_serde::serialize")]
        Host,
        u16,
    )
}

impl ImmutableOrigin {
    pub fn new(origin: Origin) -> ImmutableOrigin {
        match origin {
            Origin::Opaque(_) => ImmutableOrigin::new_opaque(),
            Origin::Tuple(scheme, host, port) => ImmutableOrigin::Tuple(scheme, host, port),
        }
    }

    pub fn same_origin(&self, other: &MutableOrigin) -> bool {
        self == other.immutable()
    }

    pub fn same_origin_domain(&self, other: &MutableOrigin) -> bool {
        !other.has_domain() && self == other.immutable()
    }

    /// Creates a new opaque origin that is only equal to itself.
    pub fn new_opaque() -> ImmutableOrigin {
        ImmutableOrigin::Opaque(OpaqueOrigin(servo_rand::thread_rng().gen()))
    }

    pub fn scheme(&self) -> Option<&str> {
        match *self {
            ImmutableOrigin::Opaque(_) => None,
            ImmutableOrigin::Tuple(ref scheme, _, _) => Some(&**scheme),
        }
    }

    pub fn host(&self) -> Option<&Host> {
        match *self {
            ImmutableOrigin::Opaque(_) => None,
            ImmutableOrigin::Tuple(_, ref host, _) => Some(host),
        }
    }

    pub fn port(&self) -> Option<u16> {
        match *self {
            ImmutableOrigin::Opaque(_) => None,
            ImmutableOrigin::Tuple(_, _, port) => Some(port),
        }
    }

    pub fn into_url_origin(self) -> Origin {
        match self {
            ImmutableOrigin::Opaque(_) => Origin::new_opaque(),
            ImmutableOrigin::Tuple(scheme, host, port) => Origin::Tuple(scheme, host, port),
        }
    }

    /// Return whether this origin is a (scheme, host, port) tuple
    /// (as opposed to an opaque origin).
    pub fn is_tuple(&self) -> bool {
        match *self {
            ImmutableOrigin::Opaque(..) => false,
            ImmutableOrigin::Tuple(..) => true,
        }
    }

    /// https://html.spec.whatwg.org/multipage/#ascii-serialisation-of-an-origin
    pub fn ascii_serialization(&self) -> String {
        self.clone().into_url_origin().ascii_serialization()
    }

    /// https://html.spec.whatwg.org/multipage/#unicode-serialisation-of-an-origin
    pub fn unicode_serialization(&self) -> String {
        self.clone().into_url_origin().unicode_serialization()
    }
}

/// Opaque identifier for URLs that have file or other schemes
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OpaqueOrigin(Uuid);

known_heap_size!(0, OpaqueOrigin);

/// A representation of an [origin](https://html.spec.whatwg.org/multipage/#origin-2).
#[derive(Clone, Debug)]
pub struct MutableOrigin(Rc<(ImmutableOrigin, RefCell<Option<Host>>)>);

known_heap_size!(0, MutableOrigin);

impl MutableOrigin {
    pub fn new(origin: ImmutableOrigin) -> MutableOrigin {
        MutableOrigin(Rc::new((origin, RefCell::new(None))))
    }

    pub fn immutable(&self) -> &ImmutableOrigin {
        &(self.0).0
    }

    pub fn is_tuple(&self) -> bool {
        self.immutable().is_tuple()
    }

    pub fn scheme(&self) -> Option<&str> {
        self.immutable().scheme()
    }

    pub fn host(&self) -> Option<&Host> {
        self.immutable().host()
    }

    pub fn port(&self) -> Option<u16> {
        self.immutable().port()
    }

    pub fn same_origin(&self, other: &MutableOrigin) -> bool {
        self.immutable() == other.immutable()
    }

    pub fn same_origin_domain(&self, other: &MutableOrigin) -> bool {
        if let Some(ref self_domain) = *(self.0).1.borrow() {
            if let Some(ref other_domain) = *(other.0).1.borrow() {
                self_domain == other_domain &&
                    self.immutable().scheme() == other.immutable().scheme()
            } else {
                false
            }
        } else {
            self.immutable().same_origin_domain(other)
        }
    }

    pub fn domain(&self) -> Option<Host> {
        (self.0).1.borrow().clone()
    }

    pub fn set_domain(&self, domain: Host) {
        *(self.0).1.borrow_mut() = Some(domain);
    }

    pub fn has_domain(&self) -> bool {
        (self.0).1.borrow().is_some()
    }

    pub fn effective_domain(&self) -> Option<Host> {
        self.immutable().host()
            .map(|host| self.domain().unwrap_or_else(|| host.clone()))
    }
}
