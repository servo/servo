/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use euclid::default::Size2D;
#[cfg(feature = "webgl")]
use js::error::throw_type_error_safe;
use js::realm::CurrentRealm;
use js::rust::{HandleObject, HandleValue};
use pixels::{EncodedImageType, Snapshot};
use rustc_hash::FxHashMap;
use script_bindings::cell::{DomRefCell, Ref};
use script_bindings::inheritance::Castable;
#[cfg(feature = "webgl")]
use script_bindings::reflector::DomObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::weakref::WeakRef;
use servo_base::id::{OffscreenCanvasId, OffscreenCanvasIndex};
#[cfg(feature = "webgl")]
use servo_canvas_traits::webgl::{GLContextAttributes, WebGLVersion};
use servo_constellation_traits::{
    BlobImpl, ScriptToConstellationMessage, TransferableOffscreenCanvas,
    TransferablePlaceholderCanvas,
};

use crate::canvas_context::{CanvasContext, OffscreenRenderingContext};
#[cfg(feature = "webgl")]
use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{
    ImageEncodeOptions, OffscreenCanvasMethods,
    OffscreenRenderingContext as RootedOffscreenRenderingContext, OffscreenRenderingContextId,
};
#[cfg(feature = "webgl")]
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLContextAttributes;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas as RootedHTMLCanvasElementOrOffscreenCanvas;
#[cfg(feature = "webgl")]
use crate::dom::bindings::conversions::ConversionResult;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::blob::Blob;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::imagebitmaprenderingcontext::ImageBitmapRenderingContext;
use crate::dom::node::Node;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::dom::promise::Promise;
#[cfg(feature = "webgl")]
use crate::dom::types::WebGLRenderingContext;
use crate::dom::types::Window;
#[cfg(feature = "webgl")]
use crate::dom::webgl::webgl2renderingcontext::WebGL2RenderingContext;

/// <https://html.spec.whatwg.org/multipage/#offscreencanvas>
#[dom_struct]
pub(crate) struct OffscreenCanvas {
    eventtarget: EventTarget,
    width: Cell<u64>,
    height: Cell<u64>,

    /// Represents both the [bitmap] and the [context mode] of the canvas.
    ///
    /// [bitmap]: https://html.spec.whatwg.org/multipage/#offscreencanvas-bitmap
    /// [context mode]: https://html.spec.whatwg.org/multipage/#offscreencanvas-context-mode
    context: DomRefCell<Option<OffscreenRenderingContext>>,

    /// <https://html.spec.whatwg.org/multipage/#offscreencanvas-placeholder>
    placeholder: Option<WeakRef<HTMLCanvasElement>>,
    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-placeholder>
    #[no_trace]
    transferable_placeholder: Cell<Option<TransferablePlaceholderCanvas>>,
    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-inherited-lang>
    inherited_language: DomRefCell<DOMString>,
    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-inherited-direction>
    inherited_direction: DomRefCell<DOMString>,
    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-placeholder>
    placeholder_update_pending: Cell<bool>,
}

impl OffscreenCanvas {
    pub(crate) fn new_inherited(
        width: u64,
        height: u64,
        inherited_language: DOMString,
        inherited_direction: DOMString,
        placeholder: Option<WeakRef<HTMLCanvasElement>>,
        transferable_placeholder: Option<TransferablePlaceholderCanvas>,
    ) -> OffscreenCanvas {
        OffscreenCanvas {
            eventtarget: EventTarget::new_inherited(),
            width: Cell::new(width),
            height: Cell::new(height),
            context: DomRefCell::new(None),
            placeholder,
            transferable_placeholder: Cell::new(transferable_placeholder),
            inherited_language: DomRefCell::new(inherited_language),
            inherited_direction: DomRefCell::new(inherited_direction),
            placeholder_update_pending: Cell::new(false),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u64,
        height: u64,
        inherited_language: DOMString,
        inherited_direction: DOMString,
        placeholder: Option<WeakRef<HTMLCanvasElement>>,
        transferable_placeholder: Option<TransferablePlaceholderCanvas>,
    ) -> DomRoot<OffscreenCanvas> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(OffscreenCanvas::new_inherited(
                width,
                height,
                inherited_language,
                inherited_direction,
                placeholder,
                transferable_placeholder,
            )),
            global,
            proto,
        )
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(
            self.Width().try_into().unwrap_or(u32::MAX),
            self.Height().try_into().unwrap_or(u32::MAX),
        )
    }

    #[cfg(feature = "webgl")]
    fn get_gl_attributes(
        cx: &mut js::context::JSContext,
        options: HandleValue,
    ) -> Option<GLContextAttributes> {
        match WebGLContextAttributes::new(cx, options) {
            Ok(ConversionResult::Success(attrs)) => Some(attrs.convert()),
            Ok(ConversionResult::Failure(error)) => {
                throw_type_error_safe(cx, &error);
                None
            },
            _ => {
                debug!("Unexpected error on conversion of WebGLContextAttributes");
                None
            },
        }
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(ref context) => context.origin_is_clean(),
            _ => true,
        }
    }

    pub(crate) fn context(&self) -> Option<Ref<'_, OffscreenRenderingContext>> {
        Ref::filter_map(self.context.borrow(), |ctx| ctx.as_ref()).ok()
    }

    pub(crate) fn get_image_data(&self) -> Option<Snapshot> {
        match self.context.borrow().as_ref() {
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
                    Some(Snapshot::cleared(size))
                }
            },
        }
    }

    pub(crate) fn get_or_init_2d_context(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let context =
            OffscreenCanvasRenderingContext2D::new(cx, &self.global(), self, self.get_size())?;
        *self.context.safe_borrow_mut(cx.no_gc()) = Some(OffscreenRenderingContext::Context2d(
            Dom::from_ref(&*context),
        ));
        Some(context)
    }

    /// <https://html.spec.whatwg.org/multipage/#offscreen-context-type-bitmaprenderer>
    pub(crate) fn get_or_init_bitmaprenderer_context(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Option<DomRoot<ImageBitmapRenderingContext>> {
        // Return the same object as was returned the last time the method was
        // invoked with this same first argument.
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::BitmapRenderer(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        // Step 1. Let context be the result of running the
        // ImageBitmapRenderingContext creation algorithm given this and
        // options.
        let canvas =
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(DomRoot::from_ref(self));

        let context = ImageBitmapRenderingContext::new(cx, &self.global(), &canvas);

        // Step 2. Set this's context mode to bitmaprenderer.
        *self.context.safe_borrow_mut(cx.no_gc()) = Some(
            OffscreenRenderingContext::BitmapRenderer(Dom::from_ref(&*context)),
        );

        // Step 3. Return context.
        Some(context)
    }

    #[cfg(feature = "webgl")]
    // <https://html.spec.whatwg.org/multipage/#offscreen-context-type-webgl>
    pub(crate) fn get_or_init_webgl_context(
        &self,
        cx: &mut js::context::JSContext,
        options: HandleValue,
    ) -> Option<DomRoot<WebGLRenderingContext>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::WebGL(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        // 1. Let context be the result of following the instructions given in the
        // WebGL specifications' Context Creation sections.
        let canvas =
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(DomRoot::from_ref(self));
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        self.global()
            .downcast::<Window>()
            .and_then(|window| {
                WebGLRenderingContext::new(cx, window, &canvas, WebGLVersion::WebGL1, size, attrs)
            })
            .map(|context| {
                // Step 2. If context is null, then return null;
                // otherwise set this's context mode to webgl or webgl2.
                *self.context.safe_borrow_mut(cx.no_gc()) =
                    Some(OffscreenRenderingContext::WebGL(Dom::from_ref(&*context)));

                // Step 3. Return context.
                context
            })
    }

    #[cfg(feature = "webgl")]
    // <https://html.spec.whatwg.org/multipage/#offscreen-context-type-webgl>
    fn get_or_init_webgl2_context(
        &self,
        cx: &mut js::context::JSContext,
        options: HandleValue,
    ) -> Option<DomRoot<WebGL2RenderingContext>> {
        if !WebGL2RenderingContext::is_webgl2_enabled(cx, self.global().reflector().get_jsobject())
        {
            return None;
        }
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::WebGL2(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }

        // 1. Let context be the result of following the instructions given in the
        // WebGL specifications' Context Creation sections.
        let canvas =
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(DomRoot::from_ref(self));
        let size = self.get_size();
        let attrs = Self::get_gl_attributes(cx, options)?;
        self.global()
            .downcast::<Window>()
            .and_then(|window| WebGL2RenderingContext::new(cx, window, &canvas, size, attrs))
            .map(|context| {
                // Step 2. If context is null, then return null;
                // otherwise set this's context mode to webgl or webgl2.
                *self.context.safe_borrow_mut(cx.no_gc()) =
                    Some(OffscreenRenderingContext::WebGL2(Dom::from_ref(&*context)));

                // Step 3. Return context.
                context
            })
    }

    pub(crate) fn placeholder(&self) -> Option<DomRoot<HTMLCanvasElement>> {
        self.placeholder
            .as_ref()
            .and_then(|placeholder| placeholder.root())
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-placeholder>
    pub(crate) fn set_transferable_placeholder(&self, placeholder: TransferablePlaceholderCanvas) {
        self.transferable_placeholder.set(Some(placeholder));
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-inherited-lang>
    pub(crate) fn set_inherited_language(&self, language: DOMString) {
        *self.inherited_language.borrow_mut() = language;
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-inherited-direction>
    pub(crate) fn set_inherited_direction(&self, direction: DOMString) {
        *self.inherited_direction.borrow_mut() = direction;
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-placeholder>
    pub(crate) fn request_placeholder_update(&self) {
        // The bitmap of the OffscreenCanvas object is pushed to the placeholder canvas element as
        // part of the OffscreenCanvas's relevant agent's event loop's update the rendering steps.
        if self.transferable_placeholder.get().is_none() ||
            self.placeholder_update_pending.replace(true)
        {
            return;
        }
        self.global().request_offscreen_canvas_update(self);
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#offscreencanvas-placeholder>
    pub(crate) fn update_the_rendering(&self) {
        // The bitmap of the OffscreenCanvas object is pushed to the placeholder canvas element as
        // part of the OffscreenCanvas's relevant agent's event loop's update the rendering steps.
        self.placeholder_update_pending.set(false);
        let Some(placeholder) = self.transferable_placeholder.get() else {
            return;
        };
        let (bitmap, origin_clean) = match self.context.borrow().as_ref() {
            Some(OffscreenRenderingContext::Context2d(context)) => {
                context.update_rendering();
                (context.get_image_data(), context.origin_is_clean())
            },
            Some(OffscreenRenderingContext::BitmapRenderer(context)) => {
                (context.get_image_data(), context.origin_is_clean())
            },
            #[cfg(feature = "webgl")]
            Some(OffscreenRenderingContext::WebGL(context)) => {
                (context.get_image_data(), context.origin_is_clean())
            },
            #[cfg(feature = "webgl")]
            Some(OffscreenRenderingContext::WebGL2(context)) => {
                (context.get_image_data(), context.origin_is_clean())
            },
            Some(OffscreenRenderingContext::Detached) => return,
            None => (Some(Snapshot::cleared(self.get_size())), true),
        };
        if let Err(error) = self.global().script_to_constellation_chan().send(
            ScriptToConstellationMessage::UpdatePlaceholderCanvas(
                placeholder.pipeline_id,
                placeholder.id,
                placeholder.image_key,
                self.width.get(),
                self.height.get(),
                bitmap.map(|bitmap| bitmap.to_shared()),
                origin_clean,
            ),
        ) {
            warn!("Failed to send placeholder canvas update: {error}");
        }
    }
}

impl Transferable for OffscreenCanvas {
    type Index = OffscreenCanvasIndex;
    type Data = TransferableOffscreenCanvas;

    /// <https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface:transfer-steps>
    fn transfer(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Fallible<(OffscreenCanvasId, TransferableOffscreenCanvas)> {
        // <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
        // Step 5.2. If transferable has a [[Detached]] internal slot and
        // transferable.[[Detached]] is true, then throw a "DataCloneError"
        // DOMException.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            return Err(Error::DataClone(None));
        }

        // Step 1. If value's context mode is not equal to none, then throw an
        // "InvalidStateError" DOMException.
        if !self.context.borrow().is_none() {
            return Err(Error::InvalidState(None));
        }

        // Step 2. Set value's context mode to detached.
        *self.context.safe_borrow_mut(cx.no_gc()) = Some(OffscreenRenderingContext::Detached);

        // Step 3. Let width and height be the dimensions of value's bitmap.
        let width = self.width.get();
        let height = self.height.get();

        // Step 4. Let language and direction be the values of
        // value's inherited language and inherited direction.
        let inherited_language = self.inherited_language.borrow().to_string();
        let inherited_direction = self.inherited_direction.borrow().to_string();

        // Step 5. Unset value's bitmap.
        self.width.set(0);
        self.height.set(0);

        let transferred = TransferableOffscreenCanvas {
            // Step 6. Set dataHolder.[[Width]] to width and dataHolder.[[Height]] to height.
            width,
            height,
            // Step 7. Set dataHolder.[[Language]] to language and dataHolder.[[Direction]] to
            // direction.
            inherited_language,
            inherited_direction,
            // Step 8. Set dataHolder.[[PlaceholderCanvas]] to be a weak reference to value's
            // placeholder canvas element, if value has one, or null if it does not.
            placeholder: self.transferable_placeholder.take(),
        };

        Ok((OffscreenCanvasId::new(), transferred))
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface:transfer-receiving-steps>
    fn transfer_receive(
        cx: &mut js::context::JSContext,
        owner: &GlobalScope,
        _: OffscreenCanvasId,
        transferred: TransferableOffscreenCanvas,
    ) -> Result<DomRoot<Self>, ()> {
        // Step 1. Initialize value's bitmap to a rectangular array of transparent black pixels
        // with width given by dataHolder.[[Width]] and height given by dataHolder.[[Height]].
        // Step 2. Set value's inherited language to dataHolder.[[Language]] and its inherited
        // direction to dataHolder.[[Direction]].
        // Step 3. If dataHolder.[[PlaceholderCanvas]] is not null, set value's placeholder canvas
        // element to dataHolder.[[PlaceholderCanvas]] (while maintaining the weak reference
        // semantics).
        let canvas = OffscreenCanvas::new(
            cx,
            owner,
            None,
            transferred.width,
            transferred.height,
            transferred.inherited_language.into(),
            transferred.inherited_direction.into(),
            None,
            transferred.placeholder,
        );
        Ok(canvas)
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<OffscreenCanvasId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.offscreen_canvases,
            StructuredData::Writer(w) => &mut w.offscreen_canvases,
        }
    }
}

impl OffscreenCanvasMethods<crate::DomTypeHolder> for OffscreenCanvas {
    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas>
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u64,
        height: u64,
    ) -> Fallible<DomRoot<OffscreenCanvas>> {
        // Step 1. Initialize the bitmap of this to a rectangular array of transparent black pixels
        // of the dimensions specified by width and height.
        // Step 2. Initialize the width of this to width.
        // Step 3. Initialize the height of this to height.
        // Step 4. Set this's inherited language to explicitly unknown.
        // Step 5. Set this's inherited direction to "ltr".
        let canvas = OffscreenCanvas::new(
            cx,
            global,
            proto,
            width,
            height,
            DOMString::new(),
            "ltr".into(),
            None,
            None,
        );

        // Step 6. Let global be the relevant global object of this.
        // Step 7. If global is a Window object:
        if let Some(window) = global.downcast::<Window>() {
            // Step 7.1. Let element be the document element of global's associated Document.
            let element = window.Document().GetDocumentElement();

            // Step 7.2. If element is not null:
            if let Some(element) = element {
                // Step 7.2.1. Set the inherited language of this to element's language.
                canvas.set_inherited_language(
                    element
                        .upcast::<Node>()
                        .get_lang()
                        .unwrap_or_default()
                        .into(),
                );

                // Step 7.2.2. Set the inherited direction of this to element's directionality.
                canvas.set_inherited_direction(element.directionality().into());
            }
        }

        Ok(canvas)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext>
    fn GetContext(
        &self,
        cx: &mut js::context::JSContext,
        id: OffscreenRenderingContextId,
        options: HandleValue,
    ) -> Fallible<Option<RootedOffscreenRenderingContext>> {
        // Step 3. Throw an "InvalidStateError" DOMException if the
        // OffscreenCanvas object's context mode is detached.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            return Err(Error::InvalidState(None));
        }

        match id {
            OffscreenRenderingContextId::_2d => Ok(self
                .get_or_init_2d_context(cx)
                .map(RootedOffscreenRenderingContext::OffscreenCanvasRenderingContext2D)),
            OffscreenRenderingContextId::Bitmaprenderer => Ok(self
                .get_or_init_bitmaprenderer_context(cx)
                .map(RootedOffscreenRenderingContext::ImageBitmapRenderingContext)),
            #[cfg(feature = "webgl")]
            OffscreenRenderingContextId::Webgl => Ok(self
                .get_or_init_webgl_context(cx, options)
                .map(RootedOffscreenRenderingContext::WebGLRenderingContext)),
            #[cfg(feature = "webgl")]
            OffscreenRenderingContextId::Experimental_webgl => Ok(self
                .get_or_init_webgl_context(cx, options)
                .map(RootedOffscreenRenderingContext::WebGLRenderingContext)),
            #[cfg(feature = "webgl")]
            OffscreenRenderingContextId::Webgl2 => Ok(self
                .get_or_init_webgl2_context(cx, options)
                .map(RootedOffscreenRenderingContext::WebGL2RenderingContext)),
            #[cfg(feature = "webgl")]
            OffscreenRenderingContextId::Experimental_webgl2 => Ok(self
                .get_or_init_webgl2_context(cx, options)
                .map(RootedOffscreenRenderingContext::WebGL2RenderingContext)),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas-width>
    fn Width(&self) -> u64 {
        self.width.get()
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas-width>
    fn SetWidth(&self, _cx: &mut js::context::JSContext, value: u64) {
        self.width.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }
        // If an OffscreenCanvas object whose dimensions were changed has a placeholder canvas
        // element, then the placeholder canvas element's natural size will only be updated during
        // the OffscreenCanvas's relevant agent's event loop's update the rendering steps.
        self.request_placeholder_update();
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas-height>
    fn Height(&self) -> u64 {
        self.height.get()
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas-height>
    fn SetHeight(&self, _cx: &mut js::context::JSContext, value: u64) {
        self.height.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }
        // If an OffscreenCanvas object whose dimensions were changed has a placeholder canvas
        // element, then the placeholder canvas element's natural size will only be updated during
        // the OffscreenCanvas's relevant agent's event loop's update the rendering steps.
        self.request_placeholder_update();
    }

    /// <https://html.spec.whatwg.org/multipage/canvas.html#dom-offscreencanvas-transfertoimagebitmap>
    fn TransferToImageBitmap(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Fallible<DomRoot<ImageBitmap>> {
        // Step 1. If the value of this OffscreenCanvas object's [[Detached]]
        // internal slot is set to true, then throw an "InvalidStateError"
        // DOMException.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            return Err(Error::InvalidState(None));
        }

        // Step 2. If this OffscreenCanvas object's context mode is set to none,
        // then throw an "InvalidStateError" DOMException.
        if self.context.borrow().is_none() {
            return Err(Error::InvalidState(None));
        }

        // Step 3. Let image be a newly created ImageBitmap object that
        // references the same underlying bitmap data as this OffscreenCanvas
        // object's bitmap.
        let Some(snapshot) = self.get_image_data() else {
            return Err(Error::InvalidState(None));
        };

        let image_bitmap = ImageBitmap::new(cx, &self.global(), snapshot);
        image_bitmap.set_origin_clean(self.origin_is_clean());

        // Step 4. Set this OffscreenCanvas object's bitmap to reference a newly
        // created bitmap of the same dimensions and color space as the previous
        // bitmap, and with its pixels initialized to transparent black, or
        // opaque black if the rendering context's alpha is false.
        if let Some(canvas_context) = self.context() {
            canvas_context.reset_bitmap();
        }
        self.request_placeholder_update();

        // Step 5. Return image.
        Ok(image_bitmap)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-converttoblob>
    fn ConvertToBlob(
        &self,
        cx: &mut js::context::JSContext,
        options: &ImageEncodeOptions,
    ) -> Rc<Promise> {
        // Step 5. Let result be a new promise object.
        let mut realm = CurrentRealm::assert(cx);
        let promise = Promise::new_in_realm(&mut realm);

        // Step 1. If the value of this's [[Detached]] internal slot is true,
        // then return a promise rejected with an "InvalidStateError"
        // DOMException.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            promise.reject_error(cx, Error::InvalidState(None));
            return promise;
        }

        // Step 2. If this's context mode is 2d and the rendering context's
        // output bitmap's origin-clean flag is set to false, then return a
        // promise rejected with a "SecurityError" DOMException.
        if !self.origin_is_clean() {
            promise.reject_error(cx, Error::Security(None));
            return promise;
        }

        // Step 3. If this's bitmap has no pixels (i.e., either its horizontal
        // dimension or its vertical dimension is zero), then return a promise
        // rejected with an "IndexSizeError" DOMException.
        if self.Width() == 0 || self.Height() == 0 {
            promise.reject_error(cx, Error::IndexSize(None));
            return promise;
        }

        // Step 4. Let bitmap be a copy of this's bitmap.
        let Some(mut snapshot) = self.get_image_data() else {
            promise.reject_error(cx, Error::InvalidState(None));
            return promise;
        };

        // Step 7. Run these steps in parallel:
        // Step 7.1. Let file be a serialization of bitmap as a file, with
        // options's type and quality if present.
        // Step 7.2. Queue a global task on the canvas blob serialization task
        // source given global to run these steps:
        let trusted_this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());

        let image_type = EncodedImageType::from(&options.type_.str() as &str);
        let quality = options.quality;

        self.global()
            .task_manager()
            .canvas_blob_task_source()
            .queue(task!(convert_to_blob: move |cx| {
                let this = trusted_this.root();
                let promise = trusted_promise.root();

                let mut encoded: Vec<u8> = vec![];

                if snapshot.encode_for_mime_type(&image_type, quality, &mut encoded).is_err() {
                    // Step 7.2.1. If file is null, then reject result with an
                    // "EncodingError" DOMException.
                    promise.reject_error(cx, Error::Encoding(None));
                    return;
                };

                // Step 7.2.2. Otherwise, resolve result with a new Blob object,
                // created in global's relevant realm, representing file.
                let blob_impl = BlobImpl::new_from_bytes(encoded, image_type.as_mime_type().to_owned());
                let blob = Blob::new(cx, &this.global(), blob_impl);

                promise.resolve_native(cx, &blob);
            }));

        // Step 8. Return result.
        promise
    }
}
