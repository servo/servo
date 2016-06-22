/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::*;
use heapsize::HeapSizeOf;
use std::fmt::{self, Debug};

// Defines an Arc-like type that manages a refcounted Gecko object stored
// in a ThreadSafeFooHolder smart pointer.  Used in tandem with the
// NS_DECL_HOLDER_FFI_REFCOUNTING-defined types and functions in Gecko.
macro_rules! define_holder_arc {
    ($arc_type:ident, $name:ident, $holder_type:ident, $addref: ident, $release: ident) => (
        #[derive(PartialEq)]
        pub struct $arc_type {
            ptr: *mut $holder_type,
        }

        impl $arc_type {
            pub fn new(data: *mut $holder_type) -> $arc_type {
                debug_assert!(!data.is_null());
                unsafe { $addref(data); }
                $arc_type {
                    ptr: data
                }
            }

            pub fn as_raw(&self) -> *mut $holder_type { self.ptr }
        }

        unsafe impl Send for $arc_type {}
        unsafe impl Sync for $arc_type {}

        impl Clone for $arc_type {
            fn clone(&self) -> $arc_type {
                $arc_type::new(self.ptr)
            }
        }

        impl Drop for $arc_type {
            fn drop(&mut self) {
                unsafe { $release(self.ptr); }
            }
        }

        impl HeapSizeOf for $arc_type {
            fn heap_size_of_children(&self) -> usize { 0 }
        }

        impl Debug for $arc_type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, stringify!($name))
            }
        }
    )
}

define_holder_arc!(GeckoArcPrincipal, Principal, ThreadSafePrincipalHolder,
                   Gecko_AddRefPrincipalArbitraryThread, Gecko_ReleasePrincipalArbitraryThread);
define_holder_arc!(GeckoArcURI, URI, ThreadSafeURIHolder,
                   Gecko_AddRefURIArbitraryThread, Gecko_ReleaseURIArbitraryThread);
