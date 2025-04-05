/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ResizeObserverSizeBinding::ResizeObserverSizeMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// Non-DOM implementation backing `ResizeObserverSize`.
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct ResizeObserverSizeImpl {
    inline_size: f64,
    block_size: f64,
}

impl ResizeObserverSizeImpl {
    pub(crate) fn new(inline_size: f64, block_size: f64) -> ResizeObserverSizeImpl {
        ResizeObserverSizeImpl {
            inline_size,
            block_size,
        }
    }

    pub(crate) fn inline_size(&self) -> f64 {
        self.inline_size
    }

    pub(crate) fn block_size(&self) -> f64 {
        self.block_size
    }
}

/// <https://drafts.csswg.org/resize-observer/#resizeobserversize>
#[dom_struct]
pub(crate) struct ResizeObserverSize {
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

    pub(crate) fn new(
        window: &Window,
        size_impl: ResizeObserverSizeImpl,
        can_gc: CanGc,
    ) -> DomRoot<ResizeObserverSize> {
        let observer_size = Box::new(ResizeObserverSize::new_inherited(size_impl));
        reflect_dom_object_with_proto(observer_size, window, None, can_gc)
    }
}

impl ResizeObserverSizeMethods<crate::DomTypeHolder> for ResizeObserverSize {
    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserversize-inlinesize>
    fn InlineSize(&self) -> f64 {
        self.size_impl.inline_size()
    }

    /// <https://drafts.csswg.org/resize-observer/#dom-resizeobserversize-blocksize>
    fn BlockSize(&self) -> f64 {
        self.size_impl.block_size()
    }
}
