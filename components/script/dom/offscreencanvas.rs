/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{OffscreenCanvasMethods, Wrap as OffscreenCanvasWrap, OffscreenRenderingContext};
use dom::bindings::cell::DomRefCell;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, Dom};
use dom::bindings::str::DOMString;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use dom_struct::dom_struct;
use euclid::Size2D;
use js::rust::HandleValue;
use js::jsapi::JSContext;
use ref_filter_map;
use std::cell::Ref;
use std::cell::Cell;

#[must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum OffscreenCanvasContext {
    OffscreenContext2d(Dom<OffscreenCanvasRenderingContext2D>),
    //WebGL(Dom<WebGLRenderingContext>),
    //WebGL2(Dom<WebGL2RenderingContext>),
}


#[dom_struct]
pub struct OffscreenCanvas{
    eventtarget: EventTarget,
    height: Cell<u64>,
    width: Cell<u64>,
    context: DomRefCell<Option<OffscreenCanvasContext>>,
    placeholder: Option<Dom<HTMLCanvasElement>>,
}

impl OffscreenCanvas{
    pub fn new_inherited(height: u64, width: u64, placeholder: Option<&HTMLCanvasElement>) -> OffscreenCanvas {
        OffscreenCanvas {
            eventtarget: EventTarget::new_inherited(),
            height: Cell::new(height),
            width: Cell::new(width),
            context: DomRefCell::new(None),
            placeholder: placeholder.map(Dom::from_ref),
        }
    }

    pub fn new(global: &GlobalScope, height: u64, width: u64, placeholder: Option<&HTMLCanvasElement>) -> DomRoot<OffscreenCanvas> {
        reflect_dom_object(Box::new(OffscreenCanvas::new_inherited(height,width,placeholder)), global, OffscreenCanvasWrap)
    }

    pub fn Constructor (global: &GlobalScope, height: u64, width: u64) -> Fallible<DomRoot<OffscreenCanvas>> {
        let offscreencanvas = OffscreenCanvas::new(global,height,width,None);
        Ok(offscreencanvas)
    }

    pub fn get_size(&self) -> Size2D<u64> {
        Size2D::new(self.Width(), self.Height())
    }

    pub fn context(&self) -> Option<Ref<OffscreenCanvasContext>> {
        ref_filter_map::ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    #[allow(unsafe_code)]
    fn get_or_init_2d_context(&self,cx: *mut JSContext) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenCanvasContext::OffscreenContext2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let size = self.get_size();
        let context = OffscreenCanvasRenderingContext2D::new(self.global(), self, size);
        *self.context.borrow_mut() = Some(OffscreenCanvasContext::OffscreenContext2d(Dom::from_ref(&*context)));
        Some(context)
    }

}


impl OffscreenCanvasMethods for OffscreenCanvas{

    #[allow(unsafe_code)]
    unsafe fn GetContext(
        &self,
        cx: *mut JSContext,
        id: DOMString,
        options: HandleValue,
    ) -> Option<OffscreenRenderingContext> {
        match &*id {
            "2d" => self
                .get_or_init_2d_context(cx)
                .map(OffscreenRenderingContext::OffscreenCanvasRenderingContext2D),
            /*"webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options)
                .map(OffscreenRenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options)
                .map(OffscreenRenderingContext::WebGL2RenderingContext),*/
            _ => None,
        }
    }

    fn Width(&self) -> u64 {
        return self.width.get();
    }

    fn SetWidth(&self, value: u64) -> () {
        self.width.set(value);
    }

    fn Height(&self) -> u64 {
        return self.height.get();
    }

    fn SetHeight(&self, value: u64) -> () {
        self.height.set(value);
    }
}
