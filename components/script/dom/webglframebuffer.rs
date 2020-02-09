/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGLFramebufferBinding;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::WebGLTexture;
use crate::dom::xrsession::XRSession;
use canvas_traits::webgl::{webgl_channel, WebGLError, WebGLResult};
use canvas_traits::webgl::{WebGLCommand, WebGLFramebufferBindingRequest};
use canvas_traits::webgl::{WebGLFramebufferId, WebGLOpaqueFramebufferId};
use dom_struct::dom_struct;
use euclid::Size2D;
use std::cell::Cell;
use webxr_api::SwapChainId as WebXRSwapChainId;
use webxr_api::Viewport;

pub enum CompleteForRendering {
    Complete,
    Incomplete,
    MissingColorAttachment,
}

#[unrooted_must_root_lint::must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
enum WebGLFramebufferAttachment {
    Renderbuffer(Dom<WebGLRenderbuffer>),
    Texture {
        texture: Dom<WebGLTexture>,
        level: i32,
    },
}

impl WebGLFramebufferAttachment {
    fn needs_initialization(&self) -> bool {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref r) => !r.is_initialized(),
            WebGLFramebufferAttachment::Texture { .. } => false,
        }
    }

    fn mark_initialized(&self) {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref r) => r.mark_initialized(),
            WebGLFramebufferAttachment::Texture { .. } => (),
        }
    }

    fn root(&self) -> WebGLFramebufferAttachmentRoot {
        match *self {
            WebGLFramebufferAttachment::Renderbuffer(ref rb) => {
                WebGLFramebufferAttachmentRoot::Renderbuffer(DomRoot::from_ref(&rb))
            },
            WebGLFramebufferAttachment::Texture { ref texture, .. } => {
                WebGLFramebufferAttachmentRoot::Texture(DomRoot::from_ref(&texture))
            },
        }
    }

    fn detach(&self) {
        match self {
            WebGLFramebufferAttachment::Renderbuffer(rb) => rb.detach_from_framebuffer(),
            WebGLFramebufferAttachment::Texture { ref texture, .. } => {
                texture.detach_from_framebuffer()
            },
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum WebGLFramebufferAttachmentRoot {
    Renderbuffer(DomRoot<WebGLRenderbuffer>),
    Texture(DomRoot<WebGLTexture>),
}

#[dom_struct]
pub struct WebGLFramebuffer {
    webgl_object: WebGLObject,
    id: WebGLFramebufferId,
    /// target can only be gl::FRAMEBUFFER at the moment
    target: Cell<Option<u32>>,
    is_deleted: Cell<bool>,
    size: Cell<Option<(i32, i32)>>,
    status: Cell<u32>,
    // The attachment points for textures and renderbuffers on this
    // FBO.
    color: DomRefCell<Option<WebGLFramebufferAttachment>>,
    depth: DomRefCell<Option<WebGLFramebufferAttachment>>,
    stencil: DomRefCell<Option<WebGLFramebufferAttachment>>,
    depthstencil: DomRefCell<Option<WebGLFramebufferAttachment>>,
    is_initialized: Cell<bool>,
    // Framebuffers for XR keep a reference to the XR session.
    // https://github.com/immersive-web/webxr/issues/856
    xr_session: MutNullableDom<XRSession>,
}

impl WebGLFramebuffer {
    fn new_inherited(context: &WebGLRenderingContext, id: WebGLFramebufferId) -> Self {
        Self {
            webgl_object: WebGLObject::new_inherited(context),
            id: id,
            target: Cell::new(None),
            is_deleted: Cell::new(false),
            size: Cell::new(None),
            status: Cell::new(constants::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT),
            color: DomRefCell::new(None),
            depth: DomRefCell::new(None),
            stencil: DomRefCell::new(None),
            depthstencil: DomRefCell::new(None),
            is_initialized: Cell::new(false),
            xr_session: Default::default(),
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateFramebuffer(sender));
        let id = receiver.recv().unwrap()?;
        let framebuffer = WebGLFramebuffer::new(context, WebGLFramebufferId::Transparent(id));
        Some(framebuffer)
    }

    // TODO: depth, stencil and alpha
    // https://github.com/servo/servo/issues/24498
    pub fn maybe_new_webxr(
        session: &XRSession,
        context: &WebGLRenderingContext,
        size: Size2D<i32, Viewport>,
    ) -> Option<(WebXRSwapChainId, DomRoot<Self>)> {
        let (sender, receiver) = webgl_channel().unwrap();
        let _ = context
            .webgl_sender()
            .send_create_webxr_swap_chain(size.to_untyped(), sender);
        let swap_chain_id = receiver.recv().unwrap()?;
        let framebuffer_id =
            WebGLFramebufferId::Opaque(WebGLOpaqueFramebufferId::WebXR(swap_chain_id));
        let framebuffer = WebGLFramebuffer::new(context, framebuffer_id);
        framebuffer.size.set(Some((size.width, size.height)));
        framebuffer.status.set(constants::FRAMEBUFFER_COMPLETE);
        framebuffer.xr_session.set(Some(session));
        Some((swap_chain_id, framebuffer))
    }

    pub fn new(context: &WebGLRenderingContext, id: WebGLFramebufferId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(WebGLFramebuffer::new_inherited(context, id)),
            &*context.global(),
            WebGLFramebufferBinding::Wrap,
        )
    }
}

impl WebGLFramebuffer {
    pub fn id(&self) -> WebGLFramebufferId {
        self.id
    }

    fn is_in_xr_session(&self) -> bool {
        self.xr_session.get().is_some()
    }

    pub fn validate_transparent(&self) -> WebGLResult<()> {
        if self.is_in_xr_session() {
            Err(WebGLError::InvalidOperation)
        } else {
            Ok(())
        }
    }

    pub fn bind(&self, target: u32) {
        if !self.is_in_xr_session() {
            // Update the framebuffer status on binding.  It may have
            // changed if its attachments were resized or deleted while
            // we've been unbound.
            self.update_status();
        }

        self.target.set(Some(target));
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindFramebuffer(
                target,
                WebGLFramebufferBindingRequest::Explicit(self.id),
            ));
    }

    pub fn delete(&self, fallible: bool) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            let context = self.upcast::<WebGLObject>().context();
            let cmd = WebGLCommand::DeleteFramebuffer(self.id);
            if fallible {
                context.send_command_ignored(cmd);
            } else {
                context.send_command(cmd);
            }
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    pub fn update_status(&self) {
        let c = self.color.borrow();
        let z = self.depth.borrow();
        let s = self.stencil.borrow();
        let zs = self.depthstencil.borrow();
        let has_c = c.is_some();
        let has_z = z.is_some();
        let has_s = s.is_some();
        let has_zs = zs.is_some();
        let attachments = [&*c, &*z, &*s, &*zs];
        let attachment_constraints = [
            &[
                constants::RGBA4,
                constants::RGB5_A1,
                constants::RGB565,
                constants::RGBA,
                constants::RGB,
            ][..],
            &[constants::DEPTH_COMPONENT16][..],
            &[constants::STENCIL_INDEX8][..],
            &[constants::DEPTH_STENCIL][..],
        ];

        // From the WebGL spec, 6.6 ("Framebuffer Object Attachments"):
        //
        //    "In the WebGL API, it is an error to concurrently attach
        //     renderbuffers to the following combinations of
        //     attachment points:
        //
        //     DEPTH_ATTACHMENT + DEPTH_STENCIL_ATTACHMENT
        //     STENCIL_ATTACHMENT + DEPTH_STENCIL_ATTACHMENT
        //     DEPTH_ATTACHMENT + STENCIL_ATTACHMENT
        //
        //     If any of the constraints above are violated, then:
        //
        //     checkFramebufferStatus must return FRAMEBUFFER_UNSUPPORTED."
        if (has_zs && (has_z || has_s)) || (has_z && has_s) {
            self.status.set(constants::FRAMEBUFFER_UNSUPPORTED);
            return;
        }

        let mut fb_size = None;
        for (attachment, constraints) in attachments.iter().zip(&attachment_constraints) {
            // Get the size of this attachment.
            let (format, size) = match **attachment {
                Some(WebGLFramebufferAttachment::Renderbuffer(ref att_rb)) => {
                    (Some(att_rb.internal_format()), att_rb.size())
                },
                Some(WebGLFramebufferAttachment::Texture {
                    texture: ref att_tex,
                    level,
                }) => match att_tex.image_info_at_face(0, level as u32) {
                    Some(info) => (
                        Some(info.internal_format().as_gl_constant()),
                        Some((info.width() as i32, info.height() as i32)),
                    ),
                    None => {
                        self.status
                            .set(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT);
                        return;
                    },
                },
                None => (None, None),
            };

            // Make sure that, if we've found any other attachment,
            // that the size matches.
            if size.is_some() {
                if fb_size.is_some() && size != fb_size {
                    self.status
                        .set(constants::FRAMEBUFFER_INCOMPLETE_DIMENSIONS);
                    return;
                } else {
                    fb_size = size;
                }
            }

            if let Some(format) = format {
                if constraints.iter().all(|c| *c != format) {
                    self.status
                        .set(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT);
                    return;
                }
            }
        }
        self.size.set(fb_size);

        if has_c || has_z || has_zs || has_s {
            if self.size.get().map_or(false, |(w, h)| w != 0 && h != 0) {
                self.status.set(constants::FRAMEBUFFER_COMPLETE);
            } else {
                self.status
                    .set(constants::FRAMEBUFFER_INCOMPLETE_ATTACHMENT);
            }
        } else {
            self.status
                .set(constants::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT);
        }
    }

    pub fn check_status(&self) -> u32 {
        // For opaque framebuffers, check to see if the XR session is currently processing an rAF
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        if let Some(xr_session) = self.xr_session.get() {
            if xr_session.is_outside_raf() {
                constants::FRAMEBUFFER_UNSUPPORTED
            } else {
                constants::FRAMEBUFFER_COMPLETE
            }
        } else {
            self.status.get()
        }
    }

    pub fn check_status_for_rendering(&self) -> CompleteForRendering {
        let result = self.check_status();
        if result != constants::FRAMEBUFFER_COMPLETE {
            return CompleteForRendering::Incomplete;
        }

        // XR framebuffers are complete inside an rAF
        // https://github.com/immersive-web/webxr/issues/854
        if self.xr_session.get().is_some() {
            return CompleteForRendering::Complete;
        }

        if self.color.borrow().is_none() {
            return CompleteForRendering::MissingColorAttachment;
        }

        if !self.is_initialized.get() {
            let attachments = [
                (&self.color, constants::COLOR_BUFFER_BIT),
                (&self.depth, constants::DEPTH_BUFFER_BIT),
                (&self.stencil, constants::STENCIL_BUFFER_BIT),
                (
                    &self.depthstencil,
                    constants::DEPTH_BUFFER_BIT | constants::STENCIL_BUFFER_BIT,
                ),
            ];
            let mut clear_bits = 0;
            for &(attachment, bits) in &attachments {
                if let Some(ref att) = *attachment.borrow() {
                    if att.needs_initialization() {
                        att.mark_initialized();
                        clear_bits |= bits;
                    }
                }
            }
            self.upcast::<WebGLObject>()
                .context()
                .initialize_framebuffer(clear_bits);
            self.is_initialized.set(true);
        }

        CompleteForRendering::Complete
    }

    pub fn renderbuffer(&self, attachment: u32, rb: Option<&WebGLRenderbuffer>) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let binding = self
            .attachment_binding(attachment)
            .ok_or(WebGLError::InvalidEnum)?;

        let rb_id = match rb {
            Some(rb) => {
                if !rb.ever_bound() {
                    return Err(WebGLError::InvalidOperation);
                }
                *binding.borrow_mut() =
                    Some(WebGLFramebufferAttachment::Renderbuffer(Dom::from_ref(rb)));
                rb.attach_to_framebuffer(self);
                Some(rb.id())
            },

            _ => None,
        };

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::FramebufferRenderbuffer(
                constants::FRAMEBUFFER,
                attachment,
                constants::RENDERBUFFER,
                rb_id,
            ));

        if rb.is_none() {
            self.detach_binding(binding, attachment)?;
        }

        self.update_status();
        self.is_initialized.set(false);
        Ok(())
    }

    fn detach_binding(
        &self,
        binding: &DomRefCell<Option<WebGLFramebufferAttachment>>,
        attachment: u32,
    ) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        if let Some(att) = &*binding.borrow() {
            att.detach();
        }
        *binding.borrow_mut() = None;
        if INTERESTING_ATTACHMENT_POINTS.contains(&attachment) {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    fn attachment_binding(
        &self,
        attachment: u32,
    ) -> Option<&DomRefCell<Option<WebGLFramebufferAttachment>>> {
        match attachment {
            constants::COLOR_ATTACHMENT0 => Some(&self.color),
            constants::DEPTH_ATTACHMENT => Some(&self.depth),
            constants::STENCIL_ATTACHMENT => Some(&self.stencil),
            constants::DEPTH_STENCIL_ATTACHMENT => Some(&self.depthstencil),
            _ => None,
        }
    }

    fn reattach_depth_stencil(&self) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let reattach = |attachment: &WebGLFramebufferAttachment, attachment_point| {
            let context = self.upcast::<WebGLObject>().context();
            match *attachment {
                WebGLFramebufferAttachment::Renderbuffer(ref rb) => {
                    rb.attach_to_framebuffer(self);
                    context.send_command(WebGLCommand::FramebufferRenderbuffer(
                        constants::FRAMEBUFFER,
                        attachment_point,
                        constants::RENDERBUFFER,
                        Some(rb.id()),
                    ));
                },
                WebGLFramebufferAttachment::Texture { ref texture, level } => {
                    texture.attach_to_framebuffer(self);
                    context.send_command(WebGLCommand::FramebufferTexture2D(
                        constants::FRAMEBUFFER,
                        attachment_point,
                        texture.target().expect("missing texture target"),
                        Some(texture.id()),
                        level,
                    ));
                },
            }
        };

        // Since the DEPTH_STENCIL attachment causes both the DEPTH and STENCIL
        // attachments to be overwritten, we need to ensure that we reattach
        // the DEPTH and STENCIL attachments when any of those attachments
        // is cleared.
        if let Some(ref depth) = *self.depth.borrow() {
            reattach(depth, constants::DEPTH_ATTACHMENT);
        }
        if let Some(ref stencil) = *self.stencil.borrow() {
            reattach(stencil, constants::STENCIL_ATTACHMENT);
        }
        if let Some(ref depth_stencil) = *self.depthstencil.borrow() {
            reattach(depth_stencil, constants::DEPTH_STENCIL_ATTACHMENT);
        }
        Ok(())
    }

    pub fn attachment(&self, attachment: u32) -> Option<WebGLFramebufferAttachmentRoot> {
        let binding = self.attachment_binding(attachment)?;
        binding
            .borrow()
            .as_ref()
            .map(WebGLFramebufferAttachment::root)
    }

    pub fn texture2d(
        &self,
        attachment: u32,
        textarget: u32,
        texture: Option<&WebGLTexture>,
        level: i32,
    ) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let binding = self
            .attachment_binding(attachment)
            .ok_or(WebGLError::InvalidEnum)?;

        let tex_id = match texture {
            // Note, from the GLES 2.0.25 spec, page 113:
            //      "If texture is zero, then textarget and level are ignored."
            Some(texture) => {
                // From the GLES 2.0.25 spec, page 113:
                //
                //     "level specifies the mipmap level of the texture image
                //      to be attached to the framebuffer and must be
                //      0. Otherwise, INVALID_VALUE is generated."
                if level != 0 {
                    return Err(WebGLError::InvalidValue);
                }

                //     "If texture is not zero, then texture must either
                //      name an existing texture object with an target of
                //      textarget, or texture must name an existing cube
                //      map texture and textarget must be one of:
                //      TEXTURE_CUBE_MAP_POSITIVE_X,
                //      TEXTURE_CUBE_MAP_POSITIVE_Y,
                //      TEXTURE_CUBE_MAP_POSITIVE_Z,
                //      TEXTURE_CUBE_MAP_NEGATIVE_X,
                //      TEXTURE_CUBE_MAP_NEGATIVE_Y, or
                //      TEXTURE_CUBE_MAP_NEGATIVE_Z. Otherwise,
                //      INVALID_OPERATION is generated."
                let is_cube = match textarget {
                    constants::TEXTURE_2D => false,

                    constants::TEXTURE_CUBE_MAP_POSITIVE_X => true,
                    constants::TEXTURE_CUBE_MAP_POSITIVE_Y => true,
                    constants::TEXTURE_CUBE_MAP_POSITIVE_Z => true,
                    constants::TEXTURE_CUBE_MAP_NEGATIVE_X => true,
                    constants::TEXTURE_CUBE_MAP_NEGATIVE_Y => true,
                    constants::TEXTURE_CUBE_MAP_NEGATIVE_Z => true,

                    _ => return Err(WebGLError::InvalidEnum),
                };

                match texture.target() {
                    Some(constants::TEXTURE_CUBE_MAP) if is_cube => {},
                    Some(_) if !is_cube => {},
                    _ => return Err(WebGLError::InvalidOperation),
                }

                *binding.borrow_mut() = Some(WebGLFramebufferAttachment::Texture {
                    texture: Dom::from_ref(texture),
                    level: level,
                });
                texture.attach_to_framebuffer(self);

                Some(texture.id())
            },

            _ => None,
        };

        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::FramebufferTexture2D(
                constants::FRAMEBUFFER,
                attachment,
                textarget,
                tex_id,
                level,
            ));

        if texture.is_none() {
            self.detach_binding(binding, attachment)?;
        }

        self.update_status();
        self.is_initialized.set(false);
        Ok(())
    }

    fn with_matching_renderbuffers<F>(&self, rb: &WebGLRenderbuffer, mut closure: F)
    where
        F: FnMut(&DomRefCell<Option<WebGLFramebufferAttachment>>, u32),
    {
        let attachments = [
            (&self.color, constants::COLOR_ATTACHMENT0),
            (&self.depth, constants::DEPTH_ATTACHMENT),
            (&self.stencil, constants::STENCIL_ATTACHMENT),
            (&self.depthstencil, constants::DEPTH_STENCIL_ATTACHMENT),
        ];

        for (attachment, name) in &attachments {
            let matched = {
                match *attachment.borrow() {
                    Some(WebGLFramebufferAttachment::Renderbuffer(ref att_rb))
                        if rb.id() == att_rb.id() =>
                    {
                        true
                    },
                    _ => false,
                }
            };

            if matched {
                closure(attachment, *name);
            }
        }
    }

    fn with_matching_textures<F>(&self, texture: &WebGLTexture, mut closure: F)
    where
        F: FnMut(&DomRefCell<Option<WebGLFramebufferAttachment>>, u32),
    {
        let attachments = [
            (&self.color, constants::COLOR_ATTACHMENT0),
            (&self.depth, constants::DEPTH_ATTACHMENT),
            (&self.stencil, constants::STENCIL_ATTACHMENT),
            (&self.depthstencil, constants::DEPTH_STENCIL_ATTACHMENT),
        ];

        for (attachment, name) in &attachments {
            let matched = {
                match *attachment.borrow() {
                    Some(WebGLFramebufferAttachment::Texture {
                        texture: ref att_texture,
                        ..
                    }) if texture.id() == att_texture.id() => true,
                    _ => false,
                }
            };

            if matched {
                closure(attachment, *name);
            }
        }
    }

    pub fn detach_renderbuffer(&self, rb: &WebGLRenderbuffer) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let mut depth_or_stencil_updated = false;
        self.with_matching_renderbuffers(rb, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            if let Some(att) = &*att.borrow() {
                att.detach();
            }
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    pub fn detach_texture(&self, texture: &WebGLTexture) -> WebGLResult<()> {
        // Opaque framebuffers cannot have their attachments changed
        // https://immersive-web.github.io/webxr/#opaque-framebuffer
        self.validate_transparent()?;

        let mut depth_or_stencil_updated = false;
        self.with_matching_textures(texture, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            if let Some(att) = &*att.borrow() {
                att.detach();
            }
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil()?;
        }
        Ok(())
    }

    pub fn invalidate_renderbuffer(&self, rb: &WebGLRenderbuffer) {
        self.with_matching_renderbuffers(rb, |_att, _| {
            self.is_initialized.set(false);
            self.update_status();
        });
    }

    pub fn invalidate_texture(&self, texture: &WebGLTexture) {
        self.with_matching_textures(texture, |_att, _name| {
            self.update_status();
        });
    }

    pub fn target(&self) -> Option<u32> {
        self.target.get()
    }
}

impl Drop for WebGLFramebuffer {
    fn drop(&mut self) {
        let _ = self.delete(true);
    }
}

static INTERESTING_ATTACHMENT_POINTS: &[u32] = &[
    constants::DEPTH_ATTACHMENT,
    constants::STENCIL_ATTACHMENT,
    constants::DEPTH_STENCIL_ATTACHMENT,
];
