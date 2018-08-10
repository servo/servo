/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{webgl_channel, WebGLCommand, WebGLError, WebGLRenderbufferId, WebGLResult};
use dom::bindings::codegen::Bindings::WebGL2RenderingContextBinding::WebGL2RenderingContextConstants as WebGl2Constants;
use dom::bindings::codegen::Bindings::WebGLRenderbufferBinding;
use dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::webglobject::WebGLObject;
use dom::webglrenderingcontext::{WebGLRenderingContext, is_gles};
use dom_struct::dom_struct;
use std::cell::Cell;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct WebGLRenderbuffer<TH: TypeHolderTrait> {
    webgl_object: WebGLObject<TH>,
    id: WebGLRenderbufferId,
    ever_bound: Cell<bool>,
    is_deleted: Cell<bool>,
    size: Cell<Option<(i32, i32)>>,
    internal_format: Cell<Option<u32>>,
}

impl<TH: TypeHolderTrait> WebGLRenderbuffer<TH> {
    fn new_inherited(context: &WebGLRenderingContext<TH>, id: WebGLRenderbufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            ever_bound: Cell::new(false),
            is_deleted: Cell::new(false),
            internal_format: Cell::new(None),
            size: Cell::new(None),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext<TH>) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateRenderbuffer(sender));
        receiver.recv().unwrap().map(|id| WebGLRenderbuffer::new(context, id))
    }

    pub fn new(context: &WebGLRenderingContext<TH>, id: WebGLRenderbufferId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLRenderbuffer::new_inherited(context, id)),
            &*context.global(),
            WebGLRenderbufferBinding::Wrap,
        )
    }
}


impl<TH: TypeHolderTrait> WebGLRenderbuffer<TH> {
    pub fn id(&self) -> WebGLRenderbufferId {
        self.id
    }

    pub fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    pub fn internal_format(&self) -> u32 {
        self.internal_format.get().unwrap_or(constants::RGBA4)
    }

    pub fn bind(&self, target: u32) {
        self.ever_bound.set(true);
        self.upcast::<WebGLObject<TH>>()
            .context()
            .send_command(WebGLCommand::BindRenderbuffer(target, Some(self.id)));
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.upcast::<WebGLObject<TH>>()
                .context()
                .send_command(WebGLCommand::DeleteRenderbuffer(self.id));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn ever_bound(&self) -> bool {
        self.ever_bound.get()
    }

    pub fn storage(&self, internal_format: u32, width: i32, height: i32) -> WebGLResult<()> {
        // Validate the internal_format, and save it for completeness
        // validation.
        let actual_format = match internal_format {
            constants::RGBA4 |
            constants::DEPTH_COMPONENT16 |
            constants::STENCIL_INDEX8 |
            // https://www.khronos.org/registry/webgl/specs/latest/1.0/#6.7
            constants::DEPTH_STENCIL => internal_format,
            constants::RGB5_A1 => {
                // 16-bit RGBA formats are not supported on desktop GL.
                if is_gles() {
                    constants::RGB5_A1
                } else {
                    WebGl2Constants::RGBA8
                }
            }
            constants::RGB565 => {
                // RGB565 is not supported on desktop GL.
                if is_gles() {
                    constants::RGB565
                } else {
                    WebGl2Constants::RGB8
                }
            }
            _ => return Err(WebGLError::InvalidEnum),
        };

        self.internal_format.set(Some(internal_format));

        // FIXME: Invalidate completeness after the call

        self.upcast::<WebGLObject<TH>>().context().send_command(
            WebGLCommand::RenderbufferStorage(
                constants::RENDERBUFFER,
                actual_format,
                width,
                height,
            )
        );

        self.size.set(Some((width, height)));

        Ok(())
    }
}
