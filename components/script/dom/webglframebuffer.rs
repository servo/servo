/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.khronos.org/registry/webgl/specs/latest/1.0/webgl.idl
use canvas_traits::webgl::{WebGLCommand, WebGLFramebufferBindingRequest, WebGLFramebufferId};
use canvas_traits::webgl::{WebGLResult, WebGLError, webgl_channel};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGLFramebufferBinding;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomObject, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::webglobject::WebGLObject;
use crate::dom::webglrenderbuffer::WebGLRenderbuffer;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::webgltexture::WebGLTexture;
use dom_struct::dom_struct;
use std::cell::Cell;

pub enum CompleteForRendering {
    Complete,
    Incomplete,
    MissingColorAttachment,
}

#[must_root]
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
        }
    }

    pub fn maybe_new(context: &WebGLRenderingContext) -> Option<DomRoot<Self>> {
        let (sender, receiver) = webgl_channel().unwrap();
        context.send_command(WebGLCommand::CreateFramebuffer(sender));
        receiver
            .recv()
            .unwrap()
            .map(|id| WebGLFramebuffer::new(context, id))
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

    pub fn bind(&self, target: u32) {
        // Update the framebuffer status on binding.  It may have
        // changed if its attachments were resized or deleted while
        // we've been unbound.
        self.update_status();

        self.target.set(Some(target));
        self.upcast::<WebGLObject>()
            .context()
            .send_command(WebGLCommand::BindFramebuffer(
                target,
                WebGLFramebufferBindingRequest::Explicit(self.id),
            ));
    }

    pub fn delete(&self) {
        if !self.is_deleted.get() {
            self.is_deleted.set(true);
            self.upcast::<WebGLObject>()
                .context()
                .send_command(WebGLCommand::DeleteFramebuffer(self.id));
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted.get()
    }

    pub fn size(&self) -> Option<(i32, i32)> {
        self.size.get()
    }

    fn update_status(&self) {
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
                }) => {
                    let info = att_tex.image_info_at_face(0, level as u32);
                    (
                        info.internal_format().map(|t| t.as_gl_constant()),
                        Some((info.width() as i32, info.height() as i32)),
                    )
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
        return self.status.get();
    }

    pub fn check_status_for_rendering(&self) -> CompleteForRendering {
        let result = self.check_status();
        if result != constants::FRAMEBUFFER_COMPLETE {
            return CompleteForRendering::Incomplete;
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
            self.detach_binding(binding, attachment);
        }

        self.update_status();
        self.is_initialized.set(false);
        Ok(())
    }

    fn detach_binding(
        &self,
        binding: &DomRefCell<Option<WebGLFramebufferAttachment>>,
        attachment: u32,
    ) {
        *binding.borrow_mut() = None;
        if INTERESTING_ATTACHMENT_POINTS.contains(&attachment) {
            self.reattach_depth_stencil();
        }
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

    fn reattach_depth_stencil(&self) {
        let reattach = |attachment: &WebGLFramebufferAttachment, attachment_point| {
            let context = self.upcast::<WebGLObject>().context();
            match *attachment {
                WebGLFramebufferAttachment::Renderbuffer(ref rb) => {
                    context.send_command(WebGLCommand::FramebufferRenderbuffer(
                        constants::FRAMEBUFFER,
                        attachment_point,
                        constants::RENDERBUFFER,
                        Some(rb.id()),
                    ));
                },
                WebGLFramebufferAttachment::Texture { ref texture, level } => {
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
            self.detach_binding(binding, attachment);
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
                    })
                        if texture.id() == att_texture.id() =>
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

    pub fn detach_renderbuffer(&self, rb: &WebGLRenderbuffer) {
        let mut depth_or_stencil_updated = false;
        self.with_matching_renderbuffers(rb, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil();
        }
    }

    pub fn detach_texture(&self, texture: &WebGLTexture) {
        let mut depth_or_stencil_updated = false;
        self.with_matching_textures(texture, |att, name| {
            depth_or_stencil_updated |= INTERESTING_ATTACHMENT_POINTS.contains(&name);
            *att.borrow_mut() = None;
            self.update_status();
        });

        if depth_or_stencil_updated {
            self.reattach_depth_stencil();
        }
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
        self.delete();
    }
}

static INTERESTING_ATTACHMENT_POINTS: &[u32] = &[
    constants::DEPTH_ATTACHMENT,
    constants::STENCIL_ATTACHMENT,
    constants::DEPTH_STENCIL_ATTACHMENT,
];
