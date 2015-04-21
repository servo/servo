/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas::webgl_paint_task::WebGLPaintTask;
use canvas::canvas_msg::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::{HTMLCanvasElement};
use geom::size::Size2D;
use std::sync::mpsc::{Sender};

#[dom_struct]
pub struct WebGLRenderingContext {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
}

impl WebGLRenderingContext {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
                     -> Result<WebGLRenderingContext, &'static str> {
        let chan = try!(WebGLPaintTask::start(size));

        Ok(WebGLRenderingContext {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: chan,
            canvas: JS::from_rooted(canvas),
        })
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
               -> Option<Temporary<WebGLRenderingContext>> {
        match WebGLRenderingContext::new_inherited(global, canvas, size) {
            Ok(ctx) => Some(reflect_dom_object(box ctx, global,
                                               WebGLRenderingContextBinding::Wrap)),
            Err(msg) => {
                error!("Couldn't create WebGLRenderingContext: {}", msg);
                None
            }
        }
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }
}

#[unsafe_destructor]
impl Drop for WebGLRenderingContext {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

impl<'a> WebGLRenderingContextMethods for JSRef<'a, WebGLRenderingContext> {
    fn Clear(self, mask: u32) -> () {
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::Clear(mask))).unwrap()
    }

    fn ClearColor(self, red: f32, green: f32, blue: f32, alpha: f32) -> (){
        self.renderer.send(CanvasMsg::WebGL(CanvasWebGLMsg::ClearColor(red, green, blue, alpha))).unwrap()
    }
}

pub trait LayoutCanvasWebGLRenderingContextHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasWebGLRenderingContextHelpers for LayoutJS<WebGLRenderingContext> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}
