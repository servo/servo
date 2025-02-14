/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use canvas_traits::canvas::{CanvasId, CanvasMsg, FromScriptMsg};
use canvas_traits::webgl::{GLContextAttributes, WebGLVersion};
use dom_struct::dom_struct;
use euclid::default::{Rect, Size2D};
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder};
use ipc_channel::ipc::IpcSharedMemory;
#[cfg(feature = "webgpu")]
use ipc_channel::ipc::{self as ipcchan};
use js::error::throw_type_error;
use js::rust::{HandleObject, HandleValue};
use profile_traits::ipc;
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};
use script_traits::serializable::BlobImpl;
#[cfg(feature = "webgpu")]
use script_traits::ScriptMsg;
use servo_media::streams::registry::MediaStreamId;
use servo_media::streams::MediaStreamType;
use style::attr::AttrValue;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::{
    BlobCallback, HTMLCanvasElementMethods, RenderingContext,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding::MediaStreamMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::module::ExceptionHandling;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::canvasrenderingcontext2d::{
    CanvasRenderingContext2D, LayoutCanvasRenderingContext2DHelpers,
};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
#[cfg(not(feature = "webgpu"))]
use crate::dom::gpucanvascontext::GPUCanvasContext;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::dom::values::UNSIGNED_LONG_MAX;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpucanvascontext::GPUCanvasContext;
use crate::script_runtime::{CanGc, JSContext};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

enum EncodedImageType {
    Png,
    Jpeg,
    Webp,
}

impl From<DOMString> for EncodedImageType {
    // From: https://html.spec.whatwg.org/multipage/#serialising-bitmaps-to-a-file
    // User agents must support PNG ("image/png"). User agents may support other types.
    // If the user agent does not support the requested type, then it must create the file using the PNG format.
    // Anything different than image/jpeg or image/webp is thus treated as PNG.
    fn from(mime_type: DOMString) -> Self {
        let mime = mime_type.to_string().to_lowercase();
        if mime == "image/jpeg" {
            Self::Jpeg
        } else if mime == "image/webp" {
            Self::Webp
        } else {
            Self::Png
        }
    }
}

impl EncodedImageType {
    fn as_mime_type(&self) -> String {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
        .to_owned()
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum CanvasContext {
    Placeholder(Dom<OffscreenCanvasRenderingContext2D>),
    Context2d(Dom<CanvasRenderingContext2D>),
    WebGL(Dom<WebGLRenderingContext>),
    WebGL2(Dom<WebGL2RenderingContext>),
    #[cfg(feature = "webgpu")]
    WebGPU(Dom<GPUCanvasContext>),
}

#[dom_struct]
pub(crate) struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: DomRefCell<Option<CanvasContext>>,
    // This id and hashmap are used to keep track of ongoing toBlob() calls.
    callback_id: Cell<u32>,
    #[ignore_malloc_size_of = "not implemented for webidl callbacks"]
    blob_callbacks: RefCell<HashMap<u32, Rc<BlobCallback>>>,
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
            callback_id: Cell::new(0),
            blob_callbacks: RefCell::new(HashMap::new()),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCanvasElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLCanvasElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
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
                #[cfg(feature = "webgpu")]
                CanvasContext::WebGPU(ref context) => context.resize(),
                CanvasContext::Placeholder(ref context) => {
                    context.set_canvas_bitmap_dimensions(size.to_u64())
                },
            }
        }
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => context.origin_is_clean(),
            _ => true,
        }
    }

    pub(crate) fn set_natural_width(&self, value: u32) {
        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_WIDTH
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("width"), value, CanGc::note());
    }

    pub(crate) fn set_natural_height(&self, value: u32) {
        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_HEIGHT
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("height"), value, CanGc::note());
    }
}

pub(crate) trait LayoutCanvasRenderingContextHelpers {
    fn canvas_data_source(self) -> HTMLCanvasDataSource;
}

pub(crate) trait LayoutHTMLCanvasElementHelpers {
    fn data(self) -> HTMLCanvasData;
    fn get_canvas_id_for_layout(self) -> CanvasId;
}

impl LayoutHTMLCanvasElementHelpers for LayoutDom<'_, HTMLCanvasElement> {
    #[allow(unsafe_code)]
    fn data(self) -> HTMLCanvasData {
        let source = unsafe {
            match self.unsafe_get().context.borrow_for_layout().as_ref() {
                Some(CanvasContext::Context2d(context)) => {
                    HTMLCanvasDataSource::Image(context.to_layout().get_ipc_renderer())
                },
                Some(CanvasContext::WebGL(context)) => context.to_layout().canvas_data_source(),
                Some(CanvasContext::WebGL2(context)) => context.to_layout().canvas_data_source(),
                #[cfg(feature = "webgpu")]
                Some(CanvasContext::WebGPU(context)) => context.to_layout().canvas_data_source(),
                Some(CanvasContext::Placeholder(_)) | None => HTMLCanvasDataSource::Empty,
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
    pub(crate) fn context(&self) -> Option<Ref<CanvasContext>> {
        ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    fn get_or_init_2d_context(&self) -> Option<DomRoot<CanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        let window = self.owner_window();
        let size = self.get_size();
        let context = CanvasRenderingContext2D::new(window.as_global_scope(), self, size);
        *self.context.borrow_mut() = Some(CanvasContext::Context2d(Dom::from_ref(&*context)));
        Some(context)
    }

    fn get_or_init_webgl_context(
        &self,
        cx: JSContext,
        options: HandleValue,
        can_gc: CanGc,
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::WebGL(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let window = self.owner_window();
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        let canvas = HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(DomRoot::from_ref(self));
        let context = WebGLRenderingContext::new(
            &window,
            &canvas,
            WebGLVersion::WebGL1,
            size,
            attrs,
            can_gc,
        )?;
        *self.context.borrow_mut() = Some(CanvasContext::WebGL(Dom::from_ref(&*context)));
        Some(context)
    }

    fn get_or_init_webgl2_context(
        &self,
        cx: JSContext,
        options: HandleValue,
        can_gc: CanGc,
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
        let window = self.owner_window();
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        let canvas = HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(DomRoot::from_ref(self));
        let context = WebGL2RenderingContext::new(&window, &canvas, size, attrs, can_gc)?;
        *self.context.borrow_mut() = Some(CanvasContext::WebGL2(Dom::from_ref(&*context)));
        Some(context)
    }

    #[cfg(not(feature = "webgpu"))]
    fn get_or_init_webgpu_context(&self) -> Option<DomRoot<GPUCanvasContext>> {
        None
    }

    #[cfg(feature = "webgpu")]
    fn get_or_init_webgpu_context(&self) -> Option<DomRoot<GPUCanvasContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                CanvasContext::WebGPU(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let (sender, receiver) = ipcchan::channel().unwrap();
        let global_scope = self.owner_global();
        let _ = global_scope
            .script_to_constellation_chan()
            .send(ScriptMsg::GetWebGPUChan(sender));
        receiver
            .recv()
            .expect("Failed to get WebGPU channel")
            .map(|channel| {
                let context = GPUCanvasContext::new(&global_scope, self, channel);
                *self.context.borrow_mut() = Some(CanvasContext::WebGPU(Dom::from_ref(&*context)));
                context
            })
    }

    /// Gets the base WebGLRenderingContext for WebGL or WebGL 2, if exists.
    pub(crate) fn get_base_webgl_context(&self) -> Option<DomRoot<WebGLRenderingContext>> {
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

    pub(crate) fn is_valid(&self) -> bool {
        self.Height() != 0 && self.Width() != 0
    }

    pub(crate) fn fetch_all_data(&self) -> Option<(Option<IpcSharedMemory>, Size2D<u32>)> {
        let size = self.get_size();

        if size.width == 0 || size.height == 0 {
            return None;
        }

        let data = match self.context.borrow().as_ref() {
            Some(CanvasContext::Context2d(context)) => Some(context.fetch_data()),
            Some(&CanvasContext::WebGL(_)) => {
                // TODO: add a method in WebGLRenderingContext to get the pixels.
                return None;
            },
            Some(&CanvasContext::WebGL2(_)) => {
                // TODO: add a method in WebGL2RenderingContext to get the pixels.
                return None;
            },
            #[cfg(feature = "webgpu")]
            Some(CanvasContext::WebGPU(context)) => Some(context.get_ipc_image()),
            Some(CanvasContext::Placeholder(context)) => {
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

        Some((data, size))
    }

    fn get_content(&self) -> Option<Vec<u8>> {
        match *self.context.borrow() {
            Some(CanvasContext::Context2d(ref context)) => {
                Some(context.get_rect(Rect::from_size(self.get_size())))
            },
            Some(CanvasContext::WebGL(ref context)) => context.get_image_data(self.get_size()),
            Some(CanvasContext::WebGL2(ref context)) => {
                context.base_context().get_image_data(self.get_size())
            },
            #[cfg(feature = "webgpu")]
            Some(CanvasContext::WebGPU(ref context)) => Some(context.get_image_data()),
            Some(CanvasContext::Placeholder(_)) | None => {
                // Each pixel is fully-transparent black.
                Some(vec![0; (self.Width() * self.Height() * 4) as usize])
            },
        }
    }

    fn maybe_quality(quality: HandleValue) -> Option<f64> {
        if quality.is_number() {
            Some(quality.to_number())
        } else {
            None
        }
    }

    fn encode_for_mime_type<W: std::io::Write>(
        &self,
        image_type: &EncodedImageType,
        quality: Option<f64>,
        bytes: &[u8],
        encoder: &mut W,
    ) {
        match image_type {
            EncodedImageType::Png => {
                // FIXME(nox): https://github.com/image-rs/image-png/issues/86
                // FIXME(nox): https://github.com/image-rs/image-png/issues/87
                PngEncoder::new(encoder)
                    .write_image(bytes, self.Width(), self.Height(), ColorType::Rgba8)
                    .unwrap();
            },
            EncodedImageType::Jpeg => {
                let jpeg_encoder = if let Some(quality) = quality {
                    // The specification allows quality to be in [0.0..1.0] but the JPEG encoder
                    // expects it to be in [1..100]
                    if (0.0..=1.0).contains(&quality) {
                        JpegEncoder::new_with_quality(
                            encoder,
                            (quality * 100.0).round().clamp(1.0, 100.0) as u8,
                        )
                    } else {
                        JpegEncoder::new(encoder)
                    }
                } else {
                    JpegEncoder::new(encoder)
                };

                jpeg_encoder
                    .write_image(bytes, self.Width(), self.Height(), ColorType::Rgba8)
                    .unwrap();
            },

            EncodedImageType::Webp => {
                // No quality support because of https://github.com/image-rs/image/issues/1984
                WebPEncoder::new_lossless(encoder)
                    .write_image(bytes, self.Width(), self.Height(), ColorType::Rgba8)
                    .unwrap();
            },
        }
    }
}

impl HTMLCanvasElementMethods<crate::DomTypeHolder> for HTMLCanvasElement {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_getter!(Width, "width", DEFAULT_WIDTH);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    // When setting the value of the width or height attribute, if the context mode of the canvas element
    // is set to placeholder, the user agent must throw an "InvalidStateError" DOMException and leave the
    // attribute's value unchanged.
    fn SetWidth(&self, value: u32) -> Fallible<()> {
        if let Some(CanvasContext::Placeholder(_)) = *self.context.borrow() {
            return Err(Error::InvalidState);
        }

        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_WIDTH
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("width"), value, CanGc::note());
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_getter!(Height, "height", DEFAULT_HEIGHT);

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    fn SetHeight(&self, value: u32) -> Fallible<()> {
        if let Some(CanvasContext::Placeholder(_)) = *self.context.borrow() {
            return Err(Error::InvalidState);
        }

        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_HEIGHT
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("height"), value, CanGc::note());
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-getcontext>
    fn GetContext(
        &self,
        cx: JSContext,
        id: DOMString,
        options: HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Option<RenderingContext>> {
        // Always throw an InvalidState exception when the canvas is in Placeholder mode (See table in the spec).
        if let Some(CanvasContext::Placeholder(_)) = *self.context.borrow() {
            return Err(Error::InvalidState);
        }

        Ok(match &*id {
            "2d" => self
                .get_or_init_2d_context()
                .map(RenderingContext::CanvasRenderingContext2D),
            "webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options, can_gc)
                .map(RenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options, can_gc)
                .map(RenderingContext::WebGL2RenderingContext),
            #[cfg(feature = "webgpu")]
            "webgpu" => self
                .get_or_init_webgpu_context()
                .map(RenderingContext::GPUCanvasContext),
            _ => None,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-todataurl>
    fn ToDataURL(
        &self,
        _context: JSContext,
        mime_type: DOMString,
        quality: HandleValue,
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
        let Some(file) = self.get_content() else {
            return Ok(USVString("data:,".into()));
        };

        let image_type = EncodedImageType::from(mime_type);
        let mut url = format!("data:{};base64,", image_type.as_mime_type());

        let mut encoder = base64::write::EncoderStringWriter::from_consumer(
            &mut url,
            &base64::engine::general_purpose::STANDARD,
        );

        self.encode_for_mime_type(
            &image_type,
            Self::maybe_quality(quality),
            &file,
            &mut encoder,
        );
        encoder.into_inner();
        Ok(USVString(url))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-toblob>
    fn ToBlob(
        &self,
        _cx: JSContext,
        callback: Rc<BlobCallback>,
        mime_type: DOMString,
        quality: HandleValue,
    ) -> Fallible<()> {
        // Step 1.
        // If this canvas element's bitmap's origin-clean flag is set to false, then throw a
        // "SecurityError" DOMException.
        if !self.origin_is_clean() {
            return Err(Error::Security);
        }

        // Step 2. and 3.
        // If this canvas element's bitmap has pixels (i.e., neither its horizontal dimension
        // nor its vertical dimension is zero),
        // then set result to a copy of this canvas element's bitmap.
        let result = if self.Width() == 0 || self.Height() == 0 {
            None
        } else {
            self.get_content()
        };

        let this = Trusted::new(self);
        let callback_id = self.callback_id.get().wrapping_add(1);
        self.callback_id.set(callback_id);

        self.blob_callbacks
            .borrow_mut()
            .insert(callback_id, callback);
        let quality = Self::maybe_quality(quality);
        let image_type = EncodedImageType::from(mime_type);
        self.global()
            .task_manager()
            .canvas_blob_task_source()
            .queue(task!(to_blob: move || {
                let this = this.root();
                let Some(callback) = &this.blob_callbacks.borrow_mut().remove(&callback_id) else {
                    return error!("Expected blob callback, but found none!");
                };

                if let Some(bytes) = result {
                    // Step 4.1
                    // If result is non-null, then set result to a serialization of result as a file with
                    // type and quality if given.
                    let mut encoded: Vec<u8> = vec![];

                    this.encode_for_mime_type(&image_type, quality, &bytes, &mut encoded);
                    let blob_impl = BlobImpl::new_from_bytes(encoded, image_type.as_mime_type());
                    // Step 4.2.1 & 4.2.2
                    // Set result to a new Blob object, created in the relevant realm of this canvas element
                    // Invoke callback with « result » and "report".
                    let blob = Blob::new(&this.global(), blob_impl, CanGc::note());
                    let _ = callback.Call__(Some(&blob), ExceptionHandling::Report);
                } else {
                    let _ = callback.Call__(None, ExceptionHandling::Report);
                }
            }));

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-transfercontroltooffscreen>
    fn TransferControlToOffscreen(&self) -> Fallible<DomRoot<OffscreenCanvas>> {
        if self.context.borrow().is_some() {
            // Step 1.
            // If this canvas element's context mode is not set to none, throw an "InvalidStateError" DOMException.
            return Err(Error::InvalidState);
        };

        // Step 2.
        // Let offscreenCanvas be a new OffscreenCanvas object with its width and height equal to the values of
        // the width and height content attributes of this canvas element.
        // Step 3.
        // Set the placeholder canvas element of offscreenCanvas to a weak reference to this canvas element.
        let offscreen_canvas = OffscreenCanvas::new(
            &self.global(),
            None,
            self.Width().into(),
            self.Height().into(),
            Some(&Dom::from_ref(self)),
            CanGc::note(),
        );
        // Step 4. Set this canvas element's context mode to placeholder.
        if let Some(ctx) = offscreen_canvas.get_or_init_2d_context() {
            *self.context.borrow_mut() = Some(CanvasContext::Placeholder(ctx.as_traced()));
        } else {
            return Err(Error::InvalidState);
        }

        // Step 5. Return offscreenCanvas.
        Ok(offscreen_canvas)
    }

    /// <https://w3c.github.io/mediacapture-fromelement/#dom-htmlcanvaselement-capturestream>
    fn CaptureStream(
        &self,
        _frame_request_rate: Option<Finite<f64>>,
        can_gc: CanGc,
    ) -> DomRoot<MediaStream> {
        let global = self.global();
        let stream = MediaStream::new(&global, can_gc);
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

pub(crate) mod utils {
    use net_traits::image_cache::ImageResponse;
    use net_traits::request::CorsSettings;
    use servo_url::ServoUrl;

    use crate::dom::window::Window;

    pub(crate) fn request_image_from_cache(
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
