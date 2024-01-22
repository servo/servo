/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ResizeObserverSizeBinding::ResizeObserverSizeMethods;
use crate::dom::bindings::reflector::Reflector;

/// Non-DOM implementation backing `ResizeObserverSize`. 
#[derive(JSTraceable, MallocSizeOf)]
pub struct ResizeObserverSizeImpl {
    inline_size: f64,
    block_size: f64,
}

impl ResizeObserverSizeImpl {
    fn inline_size(&self) -> f64 {
        self.inline_size
    }
    fn block_size(&self) -> f64 {
        self.block_size
    }
}

/// https://drafts.csswg.org/resize-observer/#resizeobserversize
#[dom_struct]
pub struct ResizeObserverSize {
    reflector_: Reflector,
    size_impl: ResizeObserverSizeImpl
}

impl ResizeObserverSizeMethods for ResizeObserverSize {
    fn InlineSize(&self) -> f64 {
        self.size_impl.inline_size()
    }
    fn BlockSize(&self) -> f64 {
        self.size_impl.block_size()
    }
}
