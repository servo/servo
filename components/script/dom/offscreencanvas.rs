/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use dom::bindings::codegen::Bindings::OffscreenCanvasBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::OffscreenCanvasMethods;
use dom::bindings::cell::DomRefCell;
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, LayoutDom};
use dom::bindings::root::Root;
use dom::bindings::inheritance::Castable;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::offscreencanvasrenderingcontext2d::{OffscreenCanvasRenderingContext2D,LayoutOffscreenCanvasRenderingContext2DHelpers};
use dom::webglrenderingcontext::{LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

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
}

	#[dom_struct]
pub struct OffscreenCanvas {
    reflector_: Reflector,
    context: DomRefCell<Option<CanvasContext>>,
}

impl OffscreenCanvas {
    fn new_inherited(document: &Document) -> OffscreenCanvas {
        OffscreenCanvas {
            reflector_: Reflector::new(),
            context: DomRefCell::new(None),
        }
    }

    pub fn Constructor(global : &GlobalScope, width: u64,
                        height: u64) -> Result<DomRoot<OffscreenCanvas>, Error> {
                            unimplemented!()
    }
}

impl OffscreenCanvasMethods for OffscreenCanvas {
     fn Width(&self) -> u64 {
         let width: u64 = 300;
    	  width
     }
    fn SetHeight(&self, height: u64) -> (){

    }
    fn SetWidth(&self, width: u64) -> (){

    }
    fn Height(&self) -> u64{
        let height: u64 = 300;
         height
    }
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
            HTMLCanvasData{
                source: source,
                width: DEFAULT_WIDTH,
                height: DEFAULT_HEIGHT,
            }
        }
        }
 }
