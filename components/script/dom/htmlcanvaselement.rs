/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{CanvasId, CanvasMsg, FromScriptMsg};
use canvas_traits::webgl::{GLContextAttributes, WebGLVersion};
use dom_struct::dom_struct;
use euclid::default::{Rect, Size2D};
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use ipc_channel::ipc::{self as ipcchan, IpcSharedMemory};
use js::error::throw_type_error;
use js::rust::{HandleObject, HandleValue};
use profile_traits::ipc;
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};
use script_traits::ScriptMsg;
use servo_media::streams::registry::MediaStreamId;
use servo_media::streams::MediaStreamType;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::{
    HTMLCanvasElementMethods, RenderingContext,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding::MediaStreamMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::canvasrenderingcontext2d::{
    CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers,
};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpucanvascontext::GPUCanvasContext;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::node::{window_from_node, Node};
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::script_runtime::JSContext;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[crown::unrooted_must_root_lint::must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum CanvasContext {
    Context2d(Dom<CanvasRenderingContext2D>),
    WebGL(Dom<WebGLRenderingContext>),
    WebGL2(Dom<WebGL2RenderingContext>),
    WebGPU(Dom<GPUCanvasContext>),
}

#[dom_struct]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: DomRefCell<Option<CanvasContext>>,
}

impl HTMLCanvasElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            context: DomRefCell::new(None),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLCanvasElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLCanvasElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn recreate_contexts(&self) {
        let size = self.get_size();
        if let Some(ref context) = *self.context.borrow() {
            match *context {
                CanvasContext::Context2d(ref context) => {
                    context.set_canvas_bitmap_dimensions(size.to_u64())
                },
                CanvasContext::WebGL(ref context) => context.recreate(size),
                CanvasContext::WebGL2(ref context) => context.recreate(size),
                CanvasContext::WebGPU(_) => unimplemented!(),
            }
        }
    }

    pub fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }

    pub fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => context.origin_is_clean(),
            _ => true,
        }
    }
}

pub trait LayoutCanvasRenderingContextHelpers {
    fn canvas_data_source(self) -> HTMLCanvasDataSource;
}

pub trait LayoutHTMLCanvasElementHelpers {
    fn data(self) -> HTMLCanvasData;
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_height(self) -> LengthOrPercentageOrAuto;
    fn get_canvas_id_for_layout(self) -> CanvasId;
}

impl LayoutHTMLCanvasElementHelpers for LayoutDom<'_, HTMLCanvasElement> {
    #[allow(unsafe_code)]
    fn data(self) -> HTMLCanvasData {
        let source = unsafe {
            match self.unsafe_get().context.borrow_for_layout().as_ref() {
                Some(CanvasContext::Context2d(context)) => {
                    HTMLCanvasDataSource::Image(Some(context.to_layout().get_ipc_renderer()))
                },
                Some(CanvasContext::WebGL(context)) => context.to_layout().canvas_data_source(),
                Some(CanvasContext::WebGL2(context)) => context.to_layout().canvas_data_source(),
                Some(CanvasContext::WebGPU(context)) => context.to_layout().canvas_data_source(),
                None => HTMLCanvasDataSource::Image(None),
            }
        };

        let width_attr = self
            .upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"));
        let height_attr = self
            .upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"));
        HTMLCanvasData {
            source,
            width: width_attr.map_or(DEFAULT_WIDTH, |val| val.as_uint()),
            height: height_attr.map_or(DEFAULT_HEIGHT, |val| val.as_uint()),
            canvas_id: self.get_canvas_id_for_layout(),
        }
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_uint_px_dimension)
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_uint_px_dimension)
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    #[allow(unsafe_code)]
    fn get_canvas_id_for_layout(self) -> CanvasId {
        let canvas = self.unsafe_get();
        unsafe {
            if let &Some(CanvasContext::Context2d(ref context)) = canvas.context.borrow_for_layout()
            {
                context.to_layout().get_canvas_id()
            } else {
                CanvasId(0)
            }
        }
    }
}

impl HTMLCanvasElement {
    pub fn context(&self) -> Option<Ref<CanvasContext>> {
        ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    fn get_or_init_2d_context(&self) -> Option<DomRoot<CanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let window = window_from_node(self);
        let size = self.get_size();
        let context = CanvasRenderingContext2D::new(window.upcast::<GlobalScope>(), self, size);
        *self.context.borrow_mut() = Some(CanvasContext::Context2d(Dom::from_ref(&*context)));
        Some(context)
    }

    fn get_or_init_webgl_context(
        &self,
        cx: JSContext,
        options: HandleValue,
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::WebGL(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let window = window_from_node(self);
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        let context = WebGLRenderingContext::new(&window, self, WebGLVersion::WebGL1, size, attrs)?;
        *self.context.borrow_mut() = Some(CanvasContext::WebGL(Dom::from_ref(&*context)));
        Some(context)
    }

    fn get_or_init_webgl2_context(
        &self,
        cx: JSContext,
        options: HandleValue,
    ) -> Option<DomRoot<WebGL2RenderingContext>> {
        if !WebGL2RenderingContext::is_webgl2_enabled(cx, self.global().reflector().get_jsobject())
        {
            return None;
        }
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::WebGL2(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let window = window_from_node(self);
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        let context = WebGL2RenderingContext::new(&window, self, size, attrs)?;
        *self.context.borrow_mut() = Some(CanvasContext::WebGL2(Dom::from_ref(&*context)));
        Some(context)
    }

    fn get_or_init_webgpu_context(&self) -> Option<DomRoot<GPUCanvasContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::WebGPU(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let (sender, receiver) = ipcchan::channel().unwrap();
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::GetWebGPUChan(sender));
        receiver
            .recv()
            .expect("Failed to get WebGPU channel")
            .map(|channel| {
                let window = window_from_node(self);
                let context = GPUCanvasContext::new(window.upcast::<GlobalScope>(), self, channel);
                *self.context.borrow_mut() = Some(CanvasContext::WebGPU(Dom::from_ref(&*context)));
                context
            })
    }

    /// Gets the base WebGLRenderingContext for WebGL or WebGL 2, if exists.
    pub fn get_base_webgl_context(&self) -> Option<DomRoot<WebGLRenderingContext>> {
        match *self.context.borrow() {
            Some(CanvasContext::WebGL(ref context)) => Some(DomRoot::from_ref(context)),
            Some(CanvasContext::WebGL2(ref context)) => Some(context.base_context()),
            _ => None,
        }
    }

    #[allow(unsafe_code)]
    fn get_gl_attributes(cx: JSContext, options: HandleValue) -> Option<GLContextAttributes> {
        unsafe {
            match WebGLContextAttributes::new(cx, options) {
                Ok(ConversionResult::Success(ref attrs)) => Some(From::from(attrs)),
                Ok(ConversionResult::Failure(ref error)) => {
                    throw_type_error(*cx, error);
                    None
                },
                _ => {
                    debug!("Unexpected error on conversion of WebGLContextAttributes");
                    None
                },
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.Height() != 0 && self.Width() != 0
    }

    pub fn fetch_all_data(&self) -> Option<(Option<IpcSharedMemory>, Size2D<u32>)> {
        let size = self.get_size();

        if size.width == 0 || size.height == 0 {
            return None;
        }

        let data = match self.context.borrow().as_ref() {
            Some(CanvasContext::Context2d(context)) => {
                let (sender, receiver) =
                    ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
                let msg = CanvasMsg::FromScript(
                    FromScriptMsg::SendPixels(sender),
                    context.get_canvas_id(),
                );
                context.get_ipc_renderer().send(msg).unwrap();

                Some(receiver.recv().unwrap())
            },
            Some(&CanvasContext::WebGL(_)) => {
                // TODO: add a method in WebGLRenderingContext to get the pixels.
                return None;
            },
            Some(&CanvasContext::WebGL2(_)) => {
                // TODO: add a method in WebGL2RenderingContext to get the pixels.
                return None;
            },
            Some(&CanvasContext::WebGPU(_)) => {
                // TODO: add a method in GPUCanvasContext to get the pixels.
                return None;
            },
            None => None,
        };

        Some((data, size))
    }
}

impl HTMLCanvasElementMethods for HTMLCanvasElement {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_getter!(Width, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_setter!(SetWidth, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_getter!(Height, "height", DEFAULT_HEIGHT);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_setter!(SetHeight, "height", DEFAULT_HEIGHT);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-getcontext
    fn GetContext(
        &self,
        cx: JSContext,
        id: DOMString,
        options: HandleValue,
    ) -> Option<RenderingContext> {
        match &*id {
            "2d" => self
                .get_or_init_2d_context()
                .map(RenderingContext::CanvasRenderingContext2D),
            "webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options)
                .map(RenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options)
                .map(RenderingContext::WebGL2RenderingContext),
            "webgpu" => self
                .get_or_init_webgpu_context()
                .map(RenderingContext::GPUCanvasContext),
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-todataurl
    fn ToDataURL(
        &self,
        _context: JSContext,
        _mime_type: Option<DOMString>,
        _quality: HandleValue,
    ) -> Fallible<USVString> {
        // Step 1.
        if !self.origin_is_clean() {
            return Err(Error::Security);
        }

        // Step 2.
        if self.Width() == 0 || self.Height() == 0 {
            return Ok(USVString("data:,".into()));
        }

        // Step 3.
        let file = match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => {
                context.get_rect(Rect::from_size(self.get_size()))
            },
            Some(CanvasContext::WebGL(ref context)) => {
                match context.get_image_data(self.get_size()) {
                    Some(data) => data,
                    None => return Ok(USVString("data:,".into())),
                }
            },
            Some(CanvasContext::WebGL2(ref context)) => {
                match context.base_context().get_image_data(self.get_size()) {
                    Some(data) => data,
                    None => return Ok(USVString("data:,".into())),
                }
            },
            //TODO: Add method get_image_data to GPUCanvasContext
            Some(CanvasContext::WebGPU(_)) => return Ok(USVString("data:,".into())),
            None => {
                // Each pixel is fully-transparent black.
                vec![0; (self.Width() * self.Height() * 4) as usize]
            },
        };

        // FIXME: Only handle image/png for now.
        let mut url = "data:image/png;base64,".to_owned();
        let mut encoder = base64::write::EncoderStringWriter::from_consumer(
            &mut url,
            &base64::engine::general_purpose::STANDARD,
        );
        // FIXME(nox): https://github.com/image-rs/image-png/issues/86
        // FIXME(nox): https://github.com/image-rs/image-png/issues/87
        PngEncoder::new(&mut encoder)
            .write_image(&file, self.Width(), self.Height(), ColorType::Rgba8)
            .unwrap();
        encoder.into_inner();
        Ok(USVString(url))
    }

    /// <https://w3c.github.io/mediacapture-fromelement/#dom-htmlcanvaselement-capturestream>
    fn CaptureStream(&self, _frame_request_rate: Option<Finite<f64>>) -> DomRoot<MediaStream> {
        let global = self.global();
        let stream = MediaStream::new(&global);
        let track = MediaStreamTrack::new(&global, MediaStreamId::new(), MediaStreamType::Video);
        stream.AddTrack(&track);
        stream
    }
}

impl VirtualMethods for HTMLCanvasElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => self.recreate_contexts(),
            _ => (),
        };
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("width") => AttrValue::from_u32(value.into(), DEFAULT_WIDTH),
            local_name!("height") => AttrValue::from_u32(value.into(), DEFAULT_HEIGHT),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
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

pub mod utils {
    use net_traits::image_cache::ImageResponse;
    use net_traits::request::CorsSettings;
    use servo_url::ServoUrl;

    use crate::dom::window::Window;

    pub fn request_image_from_cache(
        window: &Window,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
    ) -> ImageResponse {
        let image_cache = window.image_cache();
        let result = image_cache.get_image(
            url.clone(),
            window.origin().immutable().clone(),
            cors_setting,
        );

        match result {
            Some(image) => ImageResponse::Loaded(image, url),
            None => ImageResponse::None,
        }
    }
}
