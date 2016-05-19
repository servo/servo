/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::*;
use heapsize::HeapSizeOf;
use std::fmt::{self, Debug};

macro_rules! define_holder_gecko_arc {
    ($x:ident, $y:ident, $z:ident) => (
        #[derive(PartialEq)]
        pub struct $x {
            ptr: *mut $z,
        }

        impl $x {
            pub fn new(data: *mut $z) -> $x {
                assert!(!data.is_null());
                unsafe { concat_idents!(Gecko_AddRef, $y, ArbitraryThread)(data); }
                $x {
                    ptr: data
                }
            }

            pub fn as_raw(&self) -> *mut $z { self.ptr }
        }

        unsafe impl Send for $x {}
        unsafe impl Sync for $x {}

        impl Clone for $x {
            fn clone(&self) -> $x {
                $x::new(self.ptr)
            }

            fn clone_from(&mut self, source: &$x) {
                unsafe {
                    concat_idents!(Gecko_AddRef, $y, ArbitraryThread)(source.ptr);
                    concat_idents!(Gecko_Release, $y, ArbitraryThread)(self.ptr);
                }
                self.ptr = source.ptr;
            }
        }

        impl Drop for $x {
            fn drop(&mut self) {
                unsafe { concat_idents!(Gecko_Release, $y, ArbitraryThread)(self.ptr); }
            }
        }

        impl HeapSizeOf for $x {
            fn heap_size_of_children(&self) -> usize { 0 }
        }

        impl Debug for $x {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, stringify!($y))
            }
        }
    )
}

define_holder_gecko_arc!(GeckoArcPrincipal, Principal, ThreadSafePrincipalHolder);
define_holder_gecko_arc!(GeckoArcURI, URI, ThreadSafeURIHolder);
