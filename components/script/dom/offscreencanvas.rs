/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use dom::bindings::codegen::Bindings::OffscreenCanvasBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::OffscreenCanvasMethods;
use dom::webglrenderingcontext::{LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext};
use dom::offscreencanvasrenderingcontext2d::{OffscreenCanvasRenderingContext2D, LayoutOffscreenCanvasRenderingContext2DHelpers};
use dom::htmlelement::HTMLElement;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::bindings::cell::DomRefCell;
use dom::bindings::root::{Dom, DomRoot, LayoutDom};
use dom::bindings::inheritance::Castable;
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};
use dom::node::{Node, window_from_node};
use html5ever::{LocalName, Prefix};
use dom::bindings::root::Root;
use dom::document::Document;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};


use dom_struct::dom_struct;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum CanvasContext {
    Context2d(Dom<OffscreenCanvasRenderingContext2D>),
    WebGL(Dom<WebGLRenderingContext>),
}

pub trait LayoutOffscreenCanvasHelpers {
    fn data(&self) -> HTMLCanvasData;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
    fn get_height(&self) -> LengthOrPercentageOrAuto;
}

#[dom_struct]
pub struct OffscreenCanvas {
    htmlelement: HTMLElement,
    context: DomRefCell<Option<CanvasContext>>,
}

impl OffscreenCanvas {

    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> OffscreenCanvas {
        OffscreenCanvas {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            context: DomRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> DomRoot<OffscreenCanvas> {
        Node::reflect_node(Box::new(OffscreenCanvas::new_inherited(local_name, prefix, document)),
                           document,
                           OffscreenCanvasBinding::Wrap)
    }

    pub fn Constructor(width: u64,
                        height: u64) -> DomRoot<OffscreenCanvas> {
      //  let document = window.Document();
	//  let instance = canvas.Canvas();
        //OffscreenCanvas::new();
    }

}

impl OffscreenCanvasMethods for OffscreenCanvas {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_getter!(Width, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_setter!(SetWidth, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_getter!(Height, "height", DEFAULT_HEIGHT);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_setter!(SetHeight, "height", DEFAULT_HEIGHT);

   // #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext
 /*   unsafe fn GetContext(&self,
                  cx: *mut JSContext,
                  id: DOMString,
                  attributes: Vec<HandleValue>)
        -> Option<CanvasRenderingContext2DOrWebGLRenderingContext> {
        match &*id {
            "2d" => {
                self.get_or_init_2d_context()
                    .map(CanvasRenderingContext2DOrWebGLRenderingContext::CanvasRenderingContext2D)
            }
            "webgl"  => {
                self.get_or_init_webgl_context(cx, attributes.get(0).cloned())
                    .map(CanvasRenderingContext2DOrWebGLRenderingContext::WebGLRenderingContext)
            }
            _ => None
        }
    }
*/
}

impl LayoutOffscreenCanvasHelpers for LayoutDom<OffscreenCanvas> {
    #[allow(unsafe_code)]
    fn data(&self) -> HTMLCanvasData {
        unsafe {
            let canvas = &*self.unsafe_get();
            let source = match canvas.context.borrow_for_layout().as_ref() {
                Some(&CanvasContext::Context2d(ref context)) => {
                    HTMLCanvasDataSource::Image(Some(context.to_layout().get_ipc_renderer()))
                },
                Some(&CanvasContext::WebGL(ref context)) => {
                    context.to_layout().canvas_data_source()
                },
                None => {
                    HTMLCanvasDataSource::Image(None)
                }
            };

            let width_attr = canvas.upcast::<Element>().get_attr_for_layout(&ns!(), &local_name!("width"));
            let height_attr = canvas.upcast::<Element>().get_attr_for_layout(&ns!(), &local_name!("height"));
            HTMLCanvasData {
                source: source,
                width: width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint()),
                height: height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint()),
            }
        }
        }

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (&*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("width"))
                .map(AttrValue::as_uint_px_dimension)
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }

    #[allow(unsafe_code)]
    fn get_height(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (&*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("height"))
                .map(AttrValue::as_uint_px_dimension)
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}
