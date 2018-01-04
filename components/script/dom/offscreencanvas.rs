/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::OffscreenCanvasBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::OffscreenCanvasMethods;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use dom::bindings::conversions::ConversionResult;
use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, LayoutDom};
use dom::bindings::root::Root;
use dom::document::Document;
use dom::element::{Element, RawLayoutElementHelpers};
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::offscreencanvasrenderingcontext2d::LayoutOffscreenCanvasRenderingContext2DHelpers;
use dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use dom::webglrenderingcontext::{LayoutCanvasWebGLRenderingContextHelpers, WebGLRenderingContext};
use dom_struct::dom_struct;
use euclid::Size2D;
use html5ever::{LocalName, Prefix};
use offscreen_gl_context::GLContextAttributes;
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
    width: u64,
    height: u64,
}

impl OffscreenCanvas {
    pub fn new_inherited(
		width: u64,
	    height: u64
	) -> OffscreenCanvas {

	OffscreenCanvas {
		reflector_: Reflector::new(),
	    context: DomRefCell::new(None),
        width: width,
        height: height,
		}
	}
    //#[allow(unrooted_must_root)]
	fn new(
	   global: &GlobalScope,
       width: u64,
	   height: u64
   ) -> DomRoot<OffscreenCanvas> {
		reflect_dom_object(Box::new(OffscreenCanvas::new_inherited(width, height)),
		global,
		OffscreenCanvasBinding::Wrap)
    }

    pub fn Constructor(
			global : &GlobalScope,
			width: u64,
			height: u64
	) -> Result<DomRoot<OffscreenCanvas>, Error> {
        Ok(OffscreenCanvas::new(global, width, height))
	}

    pub fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.Width() as i32, self.Height() as i32)
    }
}

impl OffscreenCanvasMethods for OffscreenCanvas {
    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn Width(&self) -> u64 {
        self.width
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn SetHeight(&self, height: u64) {
        self.height = height
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, width: u64) {
        self.width = width
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u64 {
        self.height
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext
    unsafe fn GetContext(&self,
                  cx: *mut JSContext,
                  id: DOMString,
                  attributes: Vec<HandleValue>)
        -> Option<OffscreenRenderingContext> {
        match &*id {
            "2d" => {
                self.get_or_init_2d_context()
                    .map(OffscreenRenderingContext::OffscreenCanvasRenderingContext2D)
            }
            "webgl" | "experimental-webgl" => {
                self.get_or_init_webgl_context(cx, attributes.get(0).cloned())
                    .map(OffscreenRenderingContext::WebGLRenderingContext)
            }
            _ => None
        }
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
            HTMLCanvasData {
                source: source,
                width: DEFAULT_WIDTH,
                height: DEFAULT_HEIGHT,
            }
        }
        }
}

impl OffscreenCanvas {
    pub fn get_or_init_2d_context(&self) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if self.context.borrow().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();
            let context = OffscreenCanvasRenderingContext2D::new(window.upcast::<GlobalScope>(), self, size);
            *self.context.borrow_mut() = Some(CanvasContext::Context2d(Dom::from_ref(&*context)));
        }

        match *self.context.borrow().as_ref().unwrap() {
            CanvasContext::Context2d(ref context) => Some(DomRoot::from_ref(&*context)),
            _   => None,
        }
    }

    pub fn get_or_init_webgl_context(
        &self,
        cx: *mut JSContext,
        attrs: Option<HandleValue>
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        if self.context.borrow().is_none() {
            let window = window_from_node(self);
            let size = self.get_size();
            let attrs = Self::get_gl_attributes(cx, attrs)?;
            let maybe_ctx = WebGLRenderingContext::new(&window, self, WebGLVersion::WebGL1, size, attrs);

            *self.context.borrow_mut() = maybe_ctx.map( |ctx| CanvasContext::WebGL(Dom::from_ref(&*ctx)));
        }

        if let Some(CanvasContext::WebGL(ref context)) = *self.context.borrow() {
            Some(DomRoot::from_ref(&*context))
        } else {
            None
        }
    }

    #[allow(unsafe_code)]
    fn get_gl_attributes(cx: *mut JSContext, attrs: Option<HandleValue>) -> Option<GLContextAttributes> {
        let webgl_attributes = match attrs {
            Some(attrs) => attrs,
            None => return Some(GLContextAttributes::default()),
        };
        match unsafe { WebGLContextAttributes::new(cx, webgl_attributes) } {
            Ok(ConversionResult::Success(ref attrs)) => Some(From::from(attrs)),
            Ok(ConversionResult::Failure(ref error)) => {
                unsafe { throw_type_error(cx, &error); }
                None
            }
            _ => {
                debug!("Unexpected error on conversion of WebGLContextAttributes");
                None
            }
        }
    }

    /// Gets the base WebGLRenderingContext for WebGL or WebGL 2, if exists.
    pub fn get_base_webgl_context(&self) -> Option<DomRoot<WebGLRenderingContext>> {
        match *self.context.borrow() {
            Some(CanvasContext::WebGL(ref context)) => Some(DomRoot::from_ref(&*context)),
            Some(CanvasContext::WebGL2(ref context)) => Some(context.base_context()),
            _ => None
        }
}

impl<'a> From<&'a WebGLContextAttributes> for GLContextAttributes {
    fn from(attrs: &'a WebGLContextAttributes) -> GLContextAttributes {
        GLContextAttributes {
            alpha: attrs.alpha,
            depth: attrs.depth,
            stencil: attrs.stencil,
            antialias: attrs.antialias,
            premultiplied_alpha: attrs.premultipliedAlpha,
            preserve_drawing_buffer: attrs.preserveDrawingBuffer,
        }
    }
}
