/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::net::IpAddr;
use std::rc::Rc;

use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use url::{Host, Origin};
use uuid::Uuid;

/// The origin of an URL
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum ImmutableOrigin {
    /// A globally unique identifier
    Opaque(OpaqueOrigin),

    /// Consists of the URL's scheme, host and port
    Tuple(String, Host, u16),
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
        ImmutableOrigin::Opaque(OpaqueOrigin::Opaque(servo_rand::random_uuid()))
    }

    // For use in mixed security context tests because data: URL workers inherit contexts
    pub fn new_opaque_data_url_worker() -> ImmutableOrigin {
        ImmutableOrigin::Opaque(OpaqueOrigin::SecureWorkerFromDataUrl(
            servo_rand::random_uuid(),
        ))
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

    /// <https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy>
    pub fn is_potentially_trustworthy(&self) -> bool {
        // 1. If origin is an opaque origin return "Not Trustworthy"
        if matches!(self, ImmutableOrigin::Opaque(_)) {
            return false;
        }

        if let ImmutableOrigin::Tuple(scheme, host, _) = self {
            // 3. If origin’s scheme is either "https" or "wss", return "Potentially Trustworthy"
            if scheme == "https" || scheme == "wss" {
                return true;
            }
            // 6. If origin’s scheme is "file", return "Potentially Trustworthy".
            if scheme == "file" {
                return true;
            }

            // 4. If origin’s host matches one of the CIDR notations 127.0.0.0/8 or ::1/128,
            // return "Potentially Trustworthy".
            if let Ok(ip_addr) = host.to_string().parse::<IpAddr>() {
                return ip_addr.is_loopback();
            }
            // 5. If the user agent conforms to the name resolution rules in
            // [let-localhost-be-localhost] and one of the following is true:
            // * origin’s host is "localhost" or "localhost."
            // * origin’s host ends with ".localhost" or ".localhost."
            // then return "Potentially Trustworthy".
            if let Host::Domain(domain) = host {
                if domain == "localhost" || domain.ends_with(".localhost") {
                    return true;
                }
            }
        }
        // 9. Return "Not Trustworthy".
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#ascii-serialisation-of-an-origin>
    pub fn ascii_serialization(&self) -> String {
        self.clone().into_url_origin().ascii_serialization()
    }
}

/// Opaque identifier for URLs that have file or other schemes
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OpaqueOrigin {
    Opaque(Uuid),
    // Workers created from `data:` urls will have opaque origins but need to be treated
    // as inheriting the secure context they were created in. This tracks that the origin
    // was created in such a context
    SecureWorkerFromDataUrl(Uuid),
}
malloc_size_of_is_0!(OpaqueOrigin);

/// A representation of an [origin](https://html.spec.whatwg.org/multipage/#origin-2).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MutableOrigin(Rc<(ImmutableOrigin, RefCell<Option<Host>>)>);

malloc_size_of_is_0!(MutableOrigin);

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
        self.immutable()
            .host()
            .map(|host| self.domain().unwrap_or_else(|| host.clone()))
    }
}
