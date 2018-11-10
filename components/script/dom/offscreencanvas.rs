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
use js::rust::HandleValue;
use js::jsapi::JSContext;
use dom::node::{Node, window_from_node};


pub enum OffscreenCanvasContext {
Context2d(Dom<OffscreenCanvasRenderingContext2D>),
//WebGL(Dom<WebGLRenderingContext>),
//WebGL2(Dom<WebGL2RenderingContext>),
}

#[dom_struct]
pub struct OffscreenCanvas{
height: u64,
width: u64,
context: DomRefCell<Option<OffscreenCanvasContext>>,
placeholder: Option<Dom<HTMLCanvasElement>>,
}

impl OffscreenCanvas{
pub fn new_inherited(height: u64, width: u64, placeholder: Option<Dom<HTMLCanvasElement>>) -> OffscreenCanvas {
OffscreenCanvas {
reflector_: Reflector::new(),
height: height,
width: width,
context: DomRefCell::new(None),
placeholder: placeholder,
}
}

pub fn new(global: &GlobalScope, height: u64, width: u64, placeholder: Option<Dom<HTMLCanvasElement>>) -> DomRoot<OffscreenCanvas> {
reflect_dom_object(Box::new(OffscreenCanvas::new_inherited(height,width,placeholder)), global, OffscreenCanvasWrap)
}

pub fn Constructor (global: &GlobalScope, height: u64, width: u64) -> Fallible<DomRoot<OffscreenCanvas>> {
//step 1
let offscreencanvas = OffscreenCanvas::new(global,height,width,None);
//step 2

if(offscreencanvas.context.is_some()){
return Err(Error::InvalidState);
}

//offscreencanvas.height = height;
//offscreencanvas.width = width;

offscreencanvas.placeholder = ptr::null();

//step 3
Ok(offscreencanvas)
}

pub fn context(&self) -> Option<Ref<OffscreenCanvasContext>> {
ref_filter_map::ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
}

fn get_or_init_2d_context(&self) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
if let Some(ctx) = self.context() {
return match *ctx {
OffscreenCanvasContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
_ => None,
};
}
let window = window_from_node(self);
let size = self.get_size();
let context = OffscreenCanvasRenderingContext2D::new(window.upcast::<GlobalScope>(), self, size);
*self.context.borrow_mut() = Some(OffscreenCanvasContext::Context2d(Dom::from_ref(&*context)));
Some(context)
}

}


impl OffscreenCanvasMethods for OffscreenCanvas{
#[allow(unsafe_code)]
unsafe fn GetContext(&self,cx: *mut JSContext, contextID: DOMString, options: HandleValue) -> Option<UnionTypes::OffscreenCanvasRenderingContext2DOrWebGLRenderingContextOrWebGL2RenderingContext> {

if(!options.is_object())
{
options = ptr::null();
}



if(self.context.is_none())
{
if(contextID == "2d")
{
self.get_or_init_2d_context();
}
}

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
