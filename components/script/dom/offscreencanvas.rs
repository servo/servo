/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::default::Size2D;
use js::rust::{HandleObject, HandleValue};
use pixels::Snapshot;

use crate::canvas_context::{CanvasContext, OffscreenRenderingContext};
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{
    OffscreenCanvasMethods, OffscreenRenderingContext as RootedOffscreenRenderingContext,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::script_runtime::{CanGc, JSContext};

/// <https://html.spec.whatwg.org/multipage/#offscreencanvas>
#[dom_struct]
pub(crate) struct OffscreenCanvas {
    eventtarget: EventTarget,
    width: Cell<u64>,
    height: Cell<u64>,

    /// Represents both the [bitmap] and the [context mode] of the canvas.
    ///
    /// [bitmap]: https://html.spec.whatwg.org/multipage/#offscreencanvas-bitmap
    /// [context mode]: https://html.spec.whatwg.org/multipage/#offscreencanvas-context-mode
    context: DomRefCell<Option<OffscreenRenderingContext>>,

    /// <https://html.spec.whatwg.org/multipage/#offscreencanvas-placeholder>
    placeholder: Option<Dom<HTMLCanvasElement>>,
}

impl OffscreenCanvas {
    pub(crate) fn new_inherited(
        width: u64,
        height: u64,
        placeholder: Option<&HTMLCanvasElement>,
    ) -> OffscreenCanvas {
        OffscreenCanvas {
            eventtarget: EventTarget::new_inherited(),
            width: Cell::new(width),
            height: Cell::new(height),
            context: DomRefCell::new(None),
            placeholder: placeholder.map(Dom::from_ref),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u64,
        height: u64,
        placeholder: Option<&HTMLCanvasElement>,
        can_gc: CanGc,
    ) -> DomRoot<OffscreenCanvas> {
        reflect_dom_object_with_proto(
            Box::new(OffscreenCanvas::new_inherited(width, height, placeholder)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(
            self.Width().try_into().unwrap_or(u32::MAX),
            self.Height().try_into().unwrap_or(u32::MAX),
        )
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(ref context) => context.origin_is_clean(),
            _ => true,
        }
    }

    pub(crate) fn context(&self) -> Option<Ref<OffscreenRenderingContext>> {
        Ref::filter_map(self.context.borrow(), |ctx| ctx.as_ref()).ok()
    }

    pub(crate) fn get_image_data(&self) -> Option<Snapshot> {
        match self.context.borrow().as_ref() {
            Some(context) => context.get_image_data(),
            None => {
                let size = self.get_size();
                if size.is_empty() ||
                    pixels::compute_rgba8_byte_length_if_within_limit(
                        size.width as usize,
                        size.height as usize,
                    )
                    .is_none()
                {
                    None
                } else {
                    Some(Snapshot::cleared(size))
                }
            },
        }
    }

    pub(crate) fn get_or_init_2d_context(
        &self,
        can_gc: CanGc,
    ) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
            };
        }
        let context = OffscreenCanvasRenderingContext2D::new(&self.global(), self, can_gc);
        *self.context.borrow_mut() = Some(OffscreenRenderingContext::Context2d(Dom::from_ref(
            &*context,
        )));
        Some(context)
    }

    pub(crate) fn placeholder(&self) -> Option<&HTMLCanvasElement> {
        self.placeholder.as_deref()
    }
}

impl OffscreenCanvasMethods<crate::DomTypeHolder> for OffscreenCanvas {
    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        width: u64,
        height: u64,
    ) -> Fallible<DomRoot<OffscreenCanvas>> {
        let offscreencanvas = OffscreenCanvas::new(global, proto, width, height, None, can_gc);
        Ok(offscreencanvas)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext>
    fn GetContext(
        &self,
        _cx: JSContext,
        id: DOMString,
        _options: HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Option<RootedOffscreenRenderingContext>> {
        match &*id {
            "2d" => Ok(self
                .get_or_init_2d_context(can_gc)
                .map(RootedOffscreenRenderingContext::OffscreenCanvasRenderingContext2D)),
            /*"webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options)
                .map(OffscreenRenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options)
                .map(OffscreenRenderingContext::WebGL2RenderingContext),*/
            _ => Err(Error::Type(String::from(
                "Unrecognized OffscreenCanvas context type",
            ))),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width>
    fn Width(&self) -> u64 {
        self.width.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width>
    fn SetWidth(&self, value: u64, can_gc: CanGc) {
        self.width.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }

        if let Some(canvas) = &self.placeholder {
            canvas.set_natural_width(value as _, can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height>
    fn Height(&self) -> u64 {
        self.height.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height>
    fn SetHeight(&self, value: u64, can_gc: CanGc) {
        self.height.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }

        if let Some(canvas) = &self.placeholder {
            canvas.set_natural_height(value as _, can_gc);
        }
    }
}
