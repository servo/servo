/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use canvas_traits::canvas::{CanvasMsg, FromScriptMsg};
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSharedMemory;
use js::rust::{HandleObject, HandleValue};
use profile_traits::ipc;

use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{
    OffscreenCanvasMethods, OffscreenRenderingContext,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::script_runtime::{CanGc, JSContext};

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum OffscreenCanvasContext {
    OffscreenContext2d(Dom<OffscreenCanvasRenderingContext2D>),
    //WebGL(Dom<WebGLRenderingContext>),
    //WebGL2(Dom<WebGL2RenderingContext>),
}

#[dom_struct]
pub(crate) struct OffscreenCanvas {
    eventtarget: EventTarget,
    width: Cell<u64>,
    height: Cell<u64>,
    context: DomRefCell<Option<OffscreenCanvasContext>>,
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

    pub(crate) fn get_size(&self) -> Size2D<u64> {
        Size2D::new(self.Width(), self.Height())
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(OffscreenCanvasContext::OffscreenContext2d(ref context)) => {
                context.origin_is_clean()
            },
            _ => true,
        }
    }

    pub(crate) fn context(&self) -> Option<Ref<OffscreenCanvasContext>> {
        ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    pub(crate) fn fetch_all_data(&self) -> Option<(Option<IpcSharedMemory>, Size2D<u32>)> {
        let size = self.get_size();

        if size.width == 0 || size.height == 0 {
            return None;
        }

        let data = match self.context.borrow().as_ref() {
            Some(OffscreenCanvasContext::OffscreenContext2d(context)) => {
                let (sender, receiver) =
                    ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
                let msg = CanvasMsg::FromScript(
                    FromScriptMsg::SendPixels(sender),
                    context.get_canvas_id(),
                );
                context.get_ipc_renderer().send(msg).unwrap();

                Some(receiver.recv().unwrap())
            },
            None => None,
        };

        Some((data, size.to_u32()))
    }

    pub(crate) fn get_or_init_2d_context(
        &self,
    ) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenCanvasContext::OffscreenContext2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
            };
        }
        let context = OffscreenCanvasRenderingContext2D::new(
            &self.global(),
            self,
            self.placeholder.as_deref(),
        );
        *self.context.borrow_mut() = Some(OffscreenCanvasContext::OffscreenContext2d(
            Dom::from_ref(&*context),
        ));
        Some(context)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.Width() != 0 && self.Height() != 0
    }
}

impl OffscreenCanvasMethods<crate::DomTypeHolder> for OffscreenCanvas {
    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas
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

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext
    fn GetContext(
        &self,
        _cx: JSContext,
        id: DOMString,
        _options: HandleValue,
    ) -> Fallible<Option<OffscreenRenderingContext>> {
        match &*id {
            "2d" => Ok(self
                .get_or_init_2d_context()
                .map(OffscreenRenderingContext::OffscreenCanvasRenderingContext2D)),
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

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width
    fn Width(&self) -> u64 {
        self.width.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width
    fn SetWidth(&self, value: u64) {
        self.width.set(value);

        if let Some(canvas_context) = self.context() {
            match &*canvas_context {
                OffscreenCanvasContext::OffscreenContext2d(rendering_context) => {
                    rendering_context.set_canvas_bitmap_dimensions(self.get_size());
                },
            }
        }

        if let Some(canvas) = &self.placeholder {
            canvas.set_natural_width(value as _);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height
    fn Height(&self) -> u64 {
        self.height.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height
    fn SetHeight(&self, value: u64) {
        self.height.set(value);

        if let Some(canvas_context) = self.context() {
            match &*canvas_context {
                OffscreenCanvasContext::OffscreenContext2d(rendering_context) => {
                    rendering_context.set_canvas_bitmap_dimensions(self.get_size());
                },
            }
        }

        if let Some(canvas) = &self.placeholder {
            canvas.set_natural_height(value as _);
        }
    }
}
