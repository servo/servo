/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{OffscreenCanvasMethods, Wrap as OffscreenCanvasWrap};
 use dom::bindings::codegen::Bindings::OffscreenCanvasBinding;
 use dom::bindings::codegen::UnionTypes;
 use dom::bindings::error::{Error, Fallible};
 use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
 use std::ptr;
 use dom::bindings::root::{DomRoot, Dom};
 use std::cell::Ref;
 use dom::bindings::str::DOMString;
 use dom::globalscope::GlobalScope;
 use dom::htmlcanvaselement::{CanvasContext, HTMLCanvasElement};
 use dom_struct::dom_struct;
 use dom::bindings::cell::DomRefCell;
 use ref_filter_map;
 use dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
 use dom::webglrenderingcontext::WebGLRenderingContext;
 use dom::webgl2renderingcontext::WebGL2RenderingContext;
 use js::rust::HandleValue;
 use js::jsapi::JSContext;
 use dom::bindings::trace::RootedTraceableBox;
 use dom::node::{Node, window_from_node};
 use dom::bindings::codegen::UnionTypes::OffscreenCanvasRenderingContext2DOrWebGLRenderingContextOrWebGL2RenderingContext;
 use dom::eventtarget::EventTarget;

#[derive(JSTraceable, MallocSizeOf)]
 pub enum OffscreenRenderingContext {
     Context2D(Dom<OffscreenCanvasRenderingContext2D>),
     WebGL(Dom<WebGLRenderingContext>),
     WebGL2(Dom<WebGL2RenderingContext>),
 }

 #[dom_struct]
 pub struct OffscreenCanvas{
     eventtarget: EventTarget,
     height: u64,
     width: u64,
     context: DomRefCell<Option<OffscreenRenderingContext>>,
     placeholder: Option<Dom<HTMLCanvasElement>>,
 }

 impl OffscreenCanvas{
     pub fn new_inherited(height: u64, width: u64, placeholder: Option<&HTMLCanvasElement>) -> OffscreenCanvas {
         OffscreenCanvas {
             eventtarget: EventTarget::new_inherited(),
             height: height,
             width: width,
             context: DomRefCell::new(None),
             placeholder: placeholder.map(Dom::from_ref),
         }
     }

     pub fn new(
         global: &GlobalScope,
          height: u64,
           width: u64,
            placeholder: Option<&HTMLCanvasElement>
        ) -> DomRoot<OffscreenCanvas> {
         reflect_dom_object(Box::new
             (OffscreenCanvas::new_inherited(height,width,placeholder)), global, OffscreenCanvasWrap)
     }

     pub fn Constructor (global: &GlobalScope, height: u64, width: u64) -> Fallible<DomRoot<OffscreenCanvas>> {
         //step 1
         let offscreencanvas = OffscreenCanvas::new(global,height,width,None);
         //step 2
         if offscreencanvas.context.borrow().is_some() {
             return Err(Error::InvalidState);
         }

         offscreencanvas.height = height;
         offscreencanvas.width = width;

         offscreencanvas.placeholder = None;

         //step 3
         Ok(offscreencanvas)
     }

     pub fn context(&self) -> Option<Ref<OffscreenRenderingContext>> {
         ref_filter_map::ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
     }

     fn get_or_init_2d_context(&self) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
         if let Some(ctx) = self.context() {
             return match *ctx {
                 OffscreenRenderingContext::Context2D(ref ctx) => Some(DomRoot::from_ref(ctx)),
                 _ => None,
             };
         }
         //let window = window_from_node(self);
         //let size = self.get_size();
         let context = OffscreenCanvasRenderingContext2D::new(self);
         *self.context.borrow_mut() = Some(OffscreenRenderingContext::Context2D(Dom::from_ref(&*context)));
         Some(context)
     }

 }


 impl OffscreenCanvasMethods for OffscreenCanvas{
     #[allow(unsafe_code)]
     unsafe fn GetContext(&self,cx: *mut JSContext, contextID: DOMString, options: HandleValue) -> Option<OffscreenCanvasRenderingContext2DOrWebGLRenderingContextOrWebGL2RenderingContext> {

         //let options =
         /*if !options.is_object() {
             options = HandleValue::null();
         }*/




             if contextID == "2d"
             {
                 self.get_or_init_2d_context();
             }


        /* match &*contextID {
             "2d" => self.get_or_init_2d_context(),
             "webgl" | "experimental-webgl" => self
                 .map(CanvasRenderingContext::WebGLRenderingContext),
             "webgl2" | "experimental-webgl2" => self
                 .map(CanvasRenderingContext::WebGL2RenderingContext)
             _ => None,
         } */

     }



     fn Width(&self) -> u64 {
         return self.width;
     }
     fn SetWidth(&self, value: u64) -> () {
         self.width = value;
     }
     fn Height(&self) -> u64 {
         return self.height;
     }
     fn SetHeight(&self, value: u64) -> () {
         self.height = value;
     }
 }
