/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ResizeObserverSizeBinding::ResizeObserverSizeMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;

/// Non-DOM implementation backing `ResizeObserverSize`.
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub struct ResizeObserverSizeImpl {
    inline_size: f64,
    block_size: f64,
}

impl ResizeObserverSizeImpl {
    pub fn new(inline_size: f64, block_size: f64) -> ResizeObserverSizeImpl {
        ResizeObserverSizeImpl {
            inline_size,
            block_size,
        }
    }

    pub fn inline_size(&self) -> f64 {
        self.inline_size
    }

    pub fn block_size(&self) -> f64 {
        self.block_size
    }
}

/// <https://drafts.csswg.org/resize-observer/#resizeobserversize>
#[dom_struct]
pub struct ResizeObserverSize {
    reflector_: Reflector,
    size_impl: ResizeObserverSizeImpl,
}

impl ResizeObserverSize {
    fn new_inherited(size_impl: ResizeObserverSizeImpl) -> ResizeObserverSize {
        ResizeObserverSize {
            reflector_: Reflector::new(),
            size_impl,
        }
    }

    pub fn new(window: &Window, size_impl: ResizeObserverSizeImpl) -> DomRoot<ResizeObserverSize> {
        let observer_size = Box::new(ResizeObserverSize::new_inherited(size_impl));
        reflect_dom_object_with_proto(observer_size, window, None)
    }
}

impl ResizeObserverSizeMethods for ResizeObserverSize {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserversize-inlinesize>
    fn InlineSize(&self) -> f64 {
        self.size_impl.inline_size()
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserversize-blocksize>
    fn BlockSize(&self) -> f64 {
        self.size_impl.block_size()
    }
}
