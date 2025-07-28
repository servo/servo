/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use canvas_traits::webgl::{GLContextAttributes, WebGLVersion};
use constellation_traits::BlobImpl;
#[cfg(feature = "webgpu")]
use constellation_traits::ScriptToConstellationMessage;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use html5ever::{LocalName, Prefix, local_name, ns};
#[cfg(feature = "webgpu")]
use ipc_channel::ipc::{self as ipcchan};
use js::error::throw_type_error;
use js::rust::{HandleObject, HandleValue};
use layout_api::HTMLCanvasData;
use pixels::{EncodedImageType, Snapshot};
use script_bindings::weakref::WeakRef;
use servo_media::streams::MediaStreamType;
use servo_media::streams::registry::MediaStreamId;
use style::attr::AttrValue;

use super::node::NodeDamage;
pub(crate) use crate::canvas_context::*;
use crate::conversions::Convert;
use crate::dom::attr::Attr;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::{
    BlobCallback, HTMLCanvasElementMethods, RenderingContext as RootedRenderingContext,
};
use crate::dom::bindings::codegen::Bindings::MediaStreamBinding::MediaStreamMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, ToLayout};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
#[cfg(not(feature = "webgpu"))]
use crate::dom::gpucanvascontext::GPUCanvasContext;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::imagebitmaprenderingcontext::ImageBitmapRenderingContext;
use crate::dom::mediastream::MediaStream;
use crate::dom::mediastreamtrack::MediaStreamTrack;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::values::UNSIGNED_LONG_MAX;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::webgl2renderingcontext::WebGL2RenderingContext;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::gpucanvascontext::GPUCanvasContext;
use crate::script_runtime::{CanGc, JSContext};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

/// <https://html.spec.whatwg.org/multipage/#htmlcanvaselement>
#[dom_struct]
pub(crate) struct HTMLCanvasElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#concept-canvas-context-mode>
    context_mode: DomRefCell<Option<RenderingContext>>,

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
            context_mode: DomRefCell::new(None),
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

    fn recreate_contexts_after_resize(&self) {
        if let Some(ref context) = *self.context_mode.borrow() {
            context.resize()
        }
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(self.Width(), self.Height())
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context_mode.borrow() {
            Some(ref context) => context.origin_is_clean(),
            _ => true,
        }
    }

    pub(crate) fn mark_as_dirty(&self) {
        if let Some(ref context) = *self.context_mode.borrow() {
            context.mark_as_dirty()
        }
    }

    pub(crate) fn set_natural_width(&self, value: u32, can_gc: CanGc) {
        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_WIDTH
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("width"), value, can_gc);
    }

    pub(crate) fn set_natural_height(&self, value: u32, can_gc: CanGc) {
        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_HEIGHT
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("height"), value, can_gc);
    }
}

impl LayoutHTMLCanvasElementHelpers for LayoutDom<'_, HTMLCanvasElement> {
    #[allow(unsafe_code)]
    fn data(self) -> HTMLCanvasData {
        let source = unsafe {
            match self.unsafe_get().context_mode.borrow_for_layout().as_ref() {
                Some(RenderingContext::Context2d(context)) => {
                    context.to_layout().canvas_data_source()
                },
                Some(RenderingContext::BitmapRenderer(context)) => {
                    context.to_layout().canvas_data_source()
                },
                Some(RenderingContext::WebGL(context)) => context.to_layout().canvas_data_source(),
                Some(RenderingContext::WebGL2(context)) => context.to_layout().canvas_data_source(),
                #[cfg(feature = "webgpu")]
                Some(RenderingContext::WebGPU(context)) => context.to_layout().canvas_data_source(),
                Some(RenderingContext::Placeholder(_)) | None => None,
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
        }
    }
}

impl HTMLCanvasElement {
    pub(crate) fn context(&self) -> Option<Ref<RenderingContext>> {
        Ref::filter_map(self.context_mode.borrow(), |ctx| ctx.as_ref()).ok()
    }

    fn get_or_init_2d_context(&self, can_gc: CanGc) -> Option<DomRoot<CanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                RenderingContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        let window = self.owner_window();
        let size = self.get_size();
        let context = CanvasRenderingContext2D::new(window.as_global_scope(), self, size, can_gc)?;
        *self.context_mode.borrow_mut() =
            Some(RenderingContext::Context2d(Dom::from_ref(&*context)));
        Some(context)
    }

    /// <https://html.spec.whatwg.org/multipage/#canvas-context-bitmaprenderer>
    fn get_or_init_bitmaprenderer_context(
        &self,
        can_gc: CanGc,
    ) -> Option<DomRoot<ImageBitmapRenderingContext>> {
        // Return the same object as was returned the last time the method was
        // invoked with this same first argument.
        if let Some(ctx) = self.context() {
            return match *ctx {
                RenderingContext::BitmapRenderer(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        // Step 1. Let context be the result of running the
        // ImageBitmapRenderingContext creation algorithm given this and
        // options.
        let context = ImageBitmapRenderingContext::new(
            &self.owner_global(),
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(DomRoot::from_ref(self)),
            can_gc,
        );

        // Step 2. Set this's context mode to bitmaprenderer.
        *self.context_mode.borrow_mut() =
            Some(RenderingContext::BitmapRenderer(Dom::from_ref(&*context)));

        // Step 3. Return context.
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
                RenderingContext::WebGL(ref ctx) => Some(DomRoot::from_ref(ctx)),
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
        *self.context_mode.borrow_mut() = Some(RenderingContext::WebGL(Dom::from_ref(&*context)));
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
                RenderingContext::WebGL2(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let window = self.owner_window();
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        let canvas = HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(DomRoot::from_ref(self));
        let context = WebGL2RenderingContext::new(&window, &canvas, size, attrs, can_gc)?;
        *self.context_mode.borrow_mut() = Some(RenderingContext::WebGL2(Dom::from_ref(&*context)));
        Some(context)
    }

    #[cfg(not(feature = "webgpu"))]
    fn get_or_init_webgpu_context(&self) -> Option<DomRoot<GPUCanvasContext>> {
        None
    }

    #[cfg(feature = "webgpu")]
    fn get_or_init_webgpu_context(&self, can_gc: CanGc) -> Option<DomRoot<GPUCanvasContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                RenderingContext::WebGPU(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let (sender, receiver) = ipcchan::channel().unwrap();
        let global_scope = self.owner_global();
        let _ = global_scope
            .script_to_constellation_chan()
            .send(ScriptToConstellationMessage::GetWebGPUChan(sender));
        receiver
            .recv()
            .expect("Failed to get WebGPU channel")
            .map(|channel| {
                let context = GPUCanvasContext::new(&global_scope, self, channel, can_gc);
                *self.context_mode.borrow_mut() =
                    Some(RenderingContext::WebGPU(Dom::from_ref(&*context)));
                context
            })
    }

    /// Gets the base WebGLRenderingContext for WebGL or WebGL 2, if exists.
    pub(crate) fn get_base_webgl_context(&self) -> Option<DomRoot<WebGLRenderingContext>> {
        match *self.context_mode.borrow() {
            Some(RenderingContext::WebGL(ref context)) => Some(DomRoot::from_ref(context)),
            Some(RenderingContext::WebGL2(ref context)) => Some(context.base_context()),
            _ => None,
        }
    }

    #[allow(unsafe_code)]
    fn get_gl_attributes(cx: JSContext, options: HandleValue) -> Option<GLContextAttributes> {
        unsafe {
            match WebGLContextAttributes::new(cx, options) {
                Ok(ConversionResult::Success(attrs)) => Some(attrs.convert()),
                Ok(ConversionResult::Failure(error)) => {
                    throw_type_error(*cx, &error);
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

    pub(crate) fn get_image_data(&self) -> Option<Snapshot> {
        match self.context_mode.borrow().as_ref() {
            Some(context) => context.get_image_data(),
            None => {
                let size = self.get_size();
                if size.is_empty() ||
                    pixels::compute_rgba8_byte_length_if_within_limit(
                        size.width as usize,
                        size.height as usize,
                    )
                    .is_none()
                {
                    None
                } else {
                    Some(Snapshot::cleared(size.cast()))
                }
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
}

impl HTMLCanvasElementMethods<crate::DomTypeHolder> for HTMLCanvasElement {
    // https://html.spec.whatwg.org/multipage/#dom-canvas-width
    make_uint_getter!(Width, "width", DEFAULT_WIDTH);

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-width>
    fn SetWidth(&self, value: u32, can_gc: CanGc) -> Fallible<()> {
        // > When setting the value of the width or height attribute, if the context mode of the canvas element
        // > is set to placeholder, the user agent must throw an "InvalidStateError" DOMException and leave the
        // > attribute's value unchanged.
        if let Some(RenderingContext::Placeholder(_)) = *self.context_mode.borrow() {
            return Err(Error::InvalidState);
        }

        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_WIDTH
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("width"), value, can_gc);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-canvas-height
    make_uint_getter!(Height, "height", DEFAULT_HEIGHT);

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-height>
    fn SetHeight(&self, value: u32, can_gc: CanGc) -> Fallible<()> {
        // > When setting the value of the width or height attribute, if the context mode of the canvas element
        // > is set to placeholder, the user agent must throw an "InvalidStateError" DOMException and leave the
        // > attribute's value unchanged.
        if let Some(RenderingContext::Placeholder(_)) = *self.context_mode.borrow() {
            return Err(Error::InvalidState);
        }

        let value = if value > UNSIGNED_LONG_MAX {
            DEFAULT_HEIGHT
        } else {
            value
        };
        let element = self.upcast::<Element>();
        element.set_uint_attribute(&html5ever::local_name!("height"), value, can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-getcontext>
    fn GetContext(
        &self,
        cx: JSContext,
        id: DOMString,
        options: HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Option<RootedRenderingContext>> {
        // Always throw an InvalidState exception when the canvas is in Placeholder mode (See table in the spec).
        if let Some(RenderingContext::Placeholder(_)) = *self.context_mode.borrow() {
            return Err(Error::InvalidState);
        }

        Ok(match &*id {
            "2d" => self
                .get_or_init_2d_context(can_gc)
                .map(RootedRenderingContext::CanvasRenderingContext2D),
            "bitmaprenderer" => self
                .get_or_init_bitmaprenderer_context(can_gc)
                .map(RootedRenderingContext::ImageBitmapRenderingContext),
            "webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options, can_gc)
                .map(RootedRenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options, can_gc)
                .map(RootedRenderingContext::WebGL2RenderingContext),
            #[cfg(feature = "webgpu")]
            "webgpu" => self
                .get_or_init_webgpu_context(can_gc)
                .map(RootedRenderingContext::GPUCanvasContext),
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
        // Step 1: If this canvas element's bitmap's origin-clean flag is set to false,
        // then throw a "SecurityError" DOMException.
        if !self.origin_is_clean() {
            return Err(Error::Security);
        }

        // Step 2: If this canvas element's bitmap has no pixels (i.e. either its
        // horizontal dimension or its vertical dimension is zero), then return the string
        // "data:,". (This is the shortest data: URL; it represents the empty string in a
        // text/plain resource.)
        if self.Width() == 0 || self.Height() == 0 {
            return Ok(USVString("data:,".into()));
        }

        // Step 3: Let file be a serialization of this canvas element's bitmap as a file,
        // passing type and quality if given.
        let Some(mut snapshot) = self.get_image_data() else {
            return Ok(USVString("data:,".into()));
        };

        let image_type = EncodedImageType::from(mime_type.to_string());

        let mut url = format!("data:{};base64,", image_type.as_mime_type());

        let mut encoder = base64::write::EncoderStringWriter::from_consumer(
            &mut url,
            &base64::engine::general_purpose::STANDARD,
        );

        if snapshot
            .encode_for_mime_type(&image_type, Self::maybe_quality(quality), &mut encoder)
            .is_err()
        {
            // Step 4. If file is null, then return "data:,".
            return Ok(USVString("data:,".into()));
        }

        // Step 5. Return a data: URL representing file. [RFC2397]
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

        // Step 2. Let result be null.
        // Step 3. If this canvas element's bitmap has pixels (i.e., neither its horizontal dimension
        // nor its vertical dimension is zero),
        // then set result to a copy of this canvas element's bitmap.
        let result = if self.Width() == 0 || self.Height() == 0 {
            None
        } else {
            self.get_image_data()
        };

        let this = Trusted::new(self);
        let callback_id = self.callback_id.get().wrapping_add(1);
        self.callback_id.set(callback_id);

        self.blob_callbacks
            .borrow_mut()
            .insert(callback_id, callback);
        let quality = Self::maybe_quality(quality);
        let image_type = EncodedImageType::from(mime_type.to_string());

        self.global()
            .task_manager()
            .canvas_blob_task_source()
            .queue(task!(to_blob: move || {
                let this = this.root();
                let Some(callback) = &this.blob_callbacks.borrow_mut().remove(&callback_id) else {
                    return error!("Expected blob callback, but found none!");
                };

                let Some(mut snapshot) = result else {
                    let _ = callback.Call__(None, ExceptionHandling::Report, CanGc::note());
                    return;
                };

                // Step 4.1: If result is non-null, then set result to a serialization of
                // result as a file with type and quality if given.
                // Step 4.2: Queue an element task on the canvas blob serialization task
                // source given the canvas element to run these steps:
                let mut encoded: Vec<u8> = vec![];
                let blob_impl;
                let blob;
                let result = match snapshot.encode_for_mime_type(&image_type, quality, &mut encoded) {
                   Ok(..) => {
                       // Step 4.2.1: If result is non-null, then set result to a new Blob
                       // object, created in the relevant realm of this canvas element,
                       // representing result. [FILEAPI]
                       blob_impl = BlobImpl::new_from_bytes(encoded, image_type.as_mime_type());
                       blob = Blob::new(&this.global(), blob_impl, CanGc::note());
                       Some(&*blob)
                   }
                   Err(..) => None,
                };

                // Step 4.2.2: Invoke callback with « result » and "report".
                let _ = callback.Call__(result, ExceptionHandling::Report, CanGc::note());
            }));

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-canvas-transfercontroltooffscreen>
    fn TransferControlToOffscreen(&self, can_gc: CanGc) -> Fallible<DomRoot<OffscreenCanvas>> {
        if self.context_mode.borrow().is_some() {
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
            Some(WeakRef::new(self)),
            can_gc,
        );

        // Step 4. Set this canvas element's context mode to placeholder.
        *self.context_mode.borrow_mut() =
            Some(RenderingContext::Placeholder(offscreen_canvas.as_traced()));

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
        let track = MediaStreamTrack::new(
            &global,
            MediaStreamId::new(),
            MediaStreamType::Video,
            can_gc,
        );
        stream.AddTrack(&track);
        stream
    }
}

impl VirtualMethods for HTMLCanvasElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => {
                self.recreate_contexts_after_resize();
                self.upcast::<Node>().dirty(NodeDamage::Other);
            },
            _ => {},
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

impl Convert<GLContextAttributes> for WebGLContextAttributes {
    fn convert(self) -> GLContextAttributes {
        GLContextAttributes {
            alpha: self.alpha,
            depth: self.depth,
            stencil: self.stencil,
            antialias: self.antialias,
            premultiplied_alpha: self.premultipliedAlpha,
            preserve_drawing_buffer: self.preserveDrawingBuffer,
        }
    }
}
