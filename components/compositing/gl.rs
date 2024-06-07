/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use gleam::gl::{self, Gl};
use image::RgbImage;
use log::trace;
use servo_geometry::FramebufferUintLength;

pub struct RenderTargetInfo {
    gl: Rc<dyn Gl>,
    framebuffer_ids: Vec<gl::GLuint>,
    renderbuffer_ids: Vec<gl::GLuint>,
    texture_ids: Vec<gl::GLuint>,
}

impl RenderTargetInfo {
    pub fn new(
        gl: Rc<dyn Gl>,
        width: FramebufferUintLength,
        height: FramebufferUintLength,
    ) -> Self {
        let framebuffer_ids = gl.gen_framebuffers(1);
        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);
        trace!("Configuring fbo {}", framebuffer_ids[0]);

        let texture_ids = gl.gen_textures(1);
        gl.bind_texture(gl::TEXTURE_2D, texture_ids[0]);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as gl::GLint,
            width.get() as gl::GLsizei,
            height.get() as gl::GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as gl::GLint,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as gl::GLint,
        );

        gl.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture_ids[0],
            0,
        );

        gl.bind_texture(gl::TEXTURE_2D, 0);

        let renderbuffer_ids = gl.gen_renderbuffers(1);
        let depth_rb = renderbuffer_ids[0];
        gl.bind_renderbuffer(gl::RENDERBUFFER, depth_rb);
        gl.renderbuffer_storage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            width.get() as gl::GLsizei,
            height.get() as gl::GLsizei,
        );
        gl.framebuffer_renderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_rb,
        );

        Self {
            gl,
            framebuffer_ids,
            renderbuffer_ids,
            texture_ids,
        }
    }

    pub fn framebuffer_id(&self) -> gl::GLuint {
        *self.framebuffer_ids.first().expect("Guaranteed by new")
    }

    pub fn bind(&self) {
        trace!("Binding FBO {}", self.framebuffer_id());
        self.gl
            .bind_framebuffer(gl::FRAMEBUFFER, self.framebuffer_id());
    }

    pub fn unbind(&self) {
        trace!("Unbinding FBO {}", self.framebuffer_id());
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
    }

    pub fn read_back_from_gpu(
        self,
        x: i32,
        y: i32,
        width: FramebufferUintLength,
        height: FramebufferUintLength,
    ) -> RgbImage {
        let width = width.get() as usize;
        let height = height.get() as usize;
        // For some reason, OSMesa fails to render on the 3rd
        // attempt in headless mode, under some conditions.
        // I think this can only be some kind of synchronization
        // bug in OSMesa, but explicitly un-binding any vertex
        // array here seems to work around that bug.
        // See https://github.com/servo/servo/issues/18606.
        self.gl.bind_vertex_array(0);

        let mut pixels = self.gl.read_pixels(
            x,
            y,
            width as gl::GLsizei,
            height as gl::GLsizei,
            gl::RGB,
            gl::UNSIGNED_BYTE,
        );

        // flip image vertically (texture is upside down)
        let orig_pixels = pixels.clone();
        let stride = width * 3;
        for y in 0..height {
            let dst_start = y * stride;
            let src_start = (height - y - 1) * stride;
            let src_slice = &orig_pixels[src_start..src_start + stride];
            pixels[dst_start..dst_start + stride].clone_from_slice(&src_slice[..stride]);
        }

        RgbImage::from_raw(width as u32, height as u32, pixels).expect("Flipping image failed!")
    }
}

impl Drop for RenderTargetInfo {
    fn drop(&mut self) {
        trace!("Dropping FBO {}", self.framebuffer_id());
        self.unbind();
        self.gl.delete_textures(&self.texture_ids);
        self.gl.delete_renderbuffers(&self.renderbuffer_ids);
        self.gl.delete_framebuffers(&self.framebuffer_ids);
    }
}
