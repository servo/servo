/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use base::id::{OffscreenCanvasId, OffscreenCanvasIndex};
use constellation_traits::{BlobImpl, TransferableOffscreenCanvas};
use dom_struct::dom_struct;
use euclid::default::Size2D;
use js::rust::{HandleObject, HandleValue};
use pixels::{EncodedImageType, Snapshot};
use script_bindings::weakref::WeakRef;

use crate::canvas_context::{CanvasContext, OffscreenRenderingContext};
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{
    ImageEncodeOptions, OffscreenCanvasMethods,
    OffscreenRenderingContext as RootedOffscreenRenderingContext,
};
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::blob::Blob;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::imagebitmaprenderingcontext::ImageBitmapRenderingContext;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::dom::promise::Promise;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext};

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
}

impl OffscreenCanvas {
    pub(crate) fn new_inherited(
        width: u64,
        height: u64,
        placeholder: Option<WeakRef<HTMLCanvasElement>>,
    ) -> OffscreenCanvas {
        OffscreenCanvas {
            eventtarget: EventTarget::new_inherited(),
            width: Cell::new(width),
            height: Cell::new(height),
            context: DomRefCell::new(None),
            placeholder,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        width: u64,
        height: u64,
        placeholder: Option<WeakRef<HTMLCanvasElement>>,
        can_gc: CanGc,
    ) -> DomRoot<OffscreenCanvas> {
        reflect_dom_object_with_proto(
            Box::new(OffscreenCanvas::new_inherited(width, height, placeholder)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_size(&self) -> Size2D<u32> {
        Size2D::new(
            self.Width().try_into().unwrap_or(u32::MAX),
            self.Height().try_into().unwrap_or(u32::MAX),
        )
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(ref context) => context.origin_is_clean(),
            _ => true,
        }
    }

    pub(crate) fn context(&self) -> Option<Ref<OffscreenRenderingContext>> {
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
        can_gc: CanGc,
    ) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenRenderingContext::Context2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
                _ => None,
            };
        }
        let context = OffscreenCanvasRenderingContext2D::new(&self.global(), self, can_gc)?;
        *self.context.borrow_mut() = Some(OffscreenRenderingContext::Context2d(Dom::from_ref(
            &*context,
        )));
        Some(context)
    }

    /// <https://html.spec.whatwg.org/multipage/#offscreen-context-type-bitmaprenderer>
    pub(crate) fn get_or_init_bitmaprenderer_context(
        &self,
        can_gc: CanGc,
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
        let context = ImageBitmapRenderingContext::new(
            &self.global(),
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(DomRoot::from_ref(self)),
            can_gc,
        );

        // Step 2. Set this's context mode to bitmaprenderer.
        *self.context.borrow_mut() = Some(OffscreenRenderingContext::BitmapRenderer(
            Dom::from_ref(&*context),
        ));

        // Step 3. Return context.
        Some(context)
    }

    pub(crate) fn placeholder(&self) -> Option<DomRoot<HTMLCanvasElement>> {
        self.placeholder
            .as_ref()
            .and_then(|placeholder| placeholder.root())
    }
}

impl Transferable for OffscreenCanvas {
    type Index = OffscreenCanvasIndex;
    type Data = TransferableOffscreenCanvas;

    /// <https://html.spec.whatwg.org/multipage/#the-offscreencanvas-interface:transfer-steps>
    fn transfer(&self) -> Fallible<(OffscreenCanvasId, TransferableOffscreenCanvas)> {
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
            return Err(Error::InvalidState);
        }

        // TODO(#37882): Allow to transfer with a placeholder canvas element.
        if self.placeholder.is_some() {
            return Err(Error::InvalidState);
        }

        // Step 2. Set value's context mode to detached.
        *self.context.borrow_mut() = Some(OffscreenRenderingContext::Detached);

        // Step 3. Let width and height be the dimensions of value's bitmap.
        // Step 5. Unset value's bitmap.
        let width = self.width.replace(0);
        let height = self.height.replace(0);

        // TODO(#37918) Step 4. Let language and direction be the values of
        // value's inherited language and inherited direction.

        // Step 6. Set dataHolder.[[Width]] to width and dataHolder.[[Height]]
        // to height.

        // TODO(#37918) Step 7. Set dataHolder.[[Language]] to language and
        // dataHolder.[[Direction]] to direction.

        // TODO(#37882) Step 8. Set dataHolder.[[PlaceholderCanvas]] to be a
        // weak reference to value's placeholder canvas element, if value has
        // one, or null if it does not.
        let transferred = TransferableOffscreenCanvas { width, height };

        Ok((OffscreenCanvasId::new(), transferred))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-offscreencanvas-interface:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        _: OffscreenCanvasId,
        transferred: TransferableOffscreenCanvas,
    ) -> Result<DomRoot<Self>, ()> {
        // Step 1. Initialize value's bitmap to a rectangular array of
        // transparent black pixels with width given by dataHolder.[[Width]] and
        // height given by dataHolder.[[Height]].

        // TODO(#37918) Step 2. Set value's inherited language to
        // dataHolder.[[Language]] and its inherited direction to
        // dataHolder.[[Direction]].

        // TODO(#37882) Step 3. If dataHolder.[[PlaceholderCanvas]] is not null,
        // set value's placeholder canvas element to
        // dataHolder.[[PlaceholderCanvas]] (while maintaining the weak
        // reference semantics).
        Ok(OffscreenCanvas::new(
            owner,
            None,
            transferred.width,
            transferred.height,
            None,
            CanGc::note(),
        ))
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<OffscreenCanvasId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.offscreen_canvases,
            StructuredData::Writer(w) => &mut w.offscreen_canvases,
        }
    }
}

impl OffscreenCanvasMethods<crate::DomTypeHolder> for OffscreenCanvas {
    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        width: u64,
        height: u64,
    ) -> Fallible<DomRoot<OffscreenCanvas>> {
        Ok(OffscreenCanvas::new(
            global, proto, width, height, None, can_gc,
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext>
    fn GetContext(
        &self,
        _cx: JSContext,
        id: DOMString,
        _options: HandleValue,
        can_gc: CanGc,
    ) -> Fallible<Option<RootedOffscreenRenderingContext>> {
        // Step 3. Throw an "InvalidStateError" DOMException if the
        // OffscreenCanvas object's context mode is detached.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            return Err(Error::InvalidState);
        }

        match &*id {
            "2d" => Ok(self
                .get_or_init_2d_context(can_gc)
                .map(RootedOffscreenRenderingContext::OffscreenCanvasRenderingContext2D)),
            "bitmaprenderer" => Ok(self
                .get_or_init_bitmaprenderer_context(can_gc)
                .map(RootedOffscreenRenderingContext::ImageBitmapRenderingContext)),
            /*"webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options)
                .map(OffscreenRenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options)
                .map(OffscreenRenderingContext::WebGL2RenderingContext),*/
            _ => Err(Error::Type(String::from(
                "Unrecognized OffscreenCanvas context type",
            ))),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width>
    fn Width(&self) -> u64 {
        self.width.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width>
    fn SetWidth(&self, value: u64, can_gc: CanGc) {
        self.width.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }

        if let Some(canvas) = self.placeholder() {
            canvas.set_natural_width(value as _, can_gc)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height>
    fn Height(&self) -> u64 {
        self.height.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height>
    fn SetHeight(&self, value: u64, can_gc: CanGc) {
        self.height.set(value);

        if let Some(canvas_context) = self.context() {
            canvas_context.resize();
        }

        if let Some(canvas) = self.placeholder() {
            canvas.set_natural_height(value as _, can_gc)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-transfertoimagebitmap>
    fn TransferToImageBitmap(&self, can_gc: CanGc) -> Fallible<DomRoot<ImageBitmap>> {
        // Step 1. If the value of this OffscreenCanvas object's [[Detached]]
        // internal slot is set to true, then throw an "InvalidStateError"
        // DOMException.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            return Err(Error::InvalidState);
        }

        // Step 2. If this OffscreenCanvas object's context mode is set to none,
        // then throw an "InvalidStateError" DOMException.
        if self.context.borrow().is_none() {
            return Err(Error::InvalidState);
        }

        // Step 3. Let image be a newly created ImageBitmap object that
        // references the same underlying bitmap data as this OffscreenCanvas
        // object's bitmap.
        let Some(snapshot) = self.get_image_data() else {
            return Err(Error::InvalidState);
        };

        let image_bitmap = ImageBitmap::new(&self.global(), snapshot, can_gc);
        image_bitmap.set_origin_clean(self.origin_is_clean());

        // Step 4. Set this OffscreenCanvas object's bitmap to reference a newly
        // created bitmap of the same dimensions and color space as the previous
        // bitmap, and with its pixels initialized to transparent black, or
        // opaque black if the rendering context's alpha is false.
        if let Some(canvas_context) = self.context() {
            canvas_context.reset_bitmap();
        }

        // Step 5. Return image.
        Ok(image_bitmap)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-converttoblob>
    fn ConvertToBlob(&self, options: &ImageEncodeOptions, can_gc: CanGc) -> Rc<Promise> {
        // Step 5. Let result be a new promise object.
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 1. If the value of this's [[Detached]] internal slot is true,
        // then return a promise rejected with an "InvalidStateError"
        // DOMException.
        if let Some(OffscreenRenderingContext::Detached) = *self.context.borrow() {
            promise.reject_error(Error::InvalidState, can_gc);
            return promise;
        }

        // Step 2. If this's context mode is 2d and the rendering context's
        // output bitmap's origin-clean flag is set to false, then return a
        // promise rejected with a "SecurityError" DOMException.
        if !self.origin_is_clean() {
            promise.reject_error(Error::Security, can_gc);
            return promise;
        }

        // Step 3. If this's bitmap has no pixels (i.e., either its horizontal
        // dimension or its vertical dimension is zero), then return a promise
        // rejected with an "IndexSizeError" DOMException.
        if self.Width() == 0 || self.Height() == 0 {
            promise.reject_error(Error::IndexSize, can_gc);
            return promise;
        }

        // Step 4. Let bitmap be a copy of this's bitmap.
        let Some(mut snapshot) = self.get_image_data() else {
            promise.reject_error(Error::InvalidState, can_gc);
            return promise;
        };

        // Step 7. Run these steps in parallel:
        // Step 7.1. Let file be a serialization of bitmap as a file, with
        // options's type and quality if present.
        // Step 7.2. Queue a global task on the canvas blob serialization task
        // source given global to run these steps:
        let trusted_this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());

        let image_type = EncodedImageType::from(options.type_.to_string());
        let quality = options.quality;

        self.global()
            .task_manager()
            .canvas_blob_task_source()
            .queue(task!(convert_to_blob: move || {
                let this = trusted_this.root();
                let promise = trusted_promise.root();

                let mut encoded: Vec<u8> = vec![];

                if snapshot.encode_for_mime_type(&image_type, quality, &mut encoded).is_err() {
                    // Step 7.2.1. If file is null, then reject result with an
                    // "EncodingError" DOMException.
                    promise.reject_error(Error::Encoding, CanGc::note());
                    return;
                };

                // Step 7.2.2. Otherwise, resolve result with a new Blob object,
                // created in global's relevant realm, representing file.
                let blob_impl = BlobImpl::new_from_bytes(encoded, image_type.as_mime_type());
                let blob = Blob::new(&this.global(), blob_impl, CanGc::note());

                promise.resolve_native(&blob, CanGc::note());
            }));

        // Step 8. Return result.
        promise
    }
}
