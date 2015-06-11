/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use geom::size::Size2D;
use core::nonzero::NonZero;
use gleam::gl;
use gleam::gl::types::{GLsizei};

use util::task::spawn_named;

use std::borrow::ToOwned;
use std::slice::bytes::copy_memory;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;
use layers::platform::surface::NativeSurface;
use offscreen_gl_context::{GLContext, GLContextAttributes, ColorAttachmentType};

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    original_context_size: Size2D<i32>,
    gl_context: GLContext,
}

// This allows trying to create the PaintTask
// before creating the thread
unsafe impl Send for WebGLPaintTask {}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>, attrs: GLContextAttributes) -> Result<WebGLPaintTask, &'static str> {
        let context = try!(
            GLContext::create_offscreen_with_color_attachment(
                size, attrs, ColorAttachmentType::TextureWithSurface));

        // NOTE: As of right now this is always equal to the size parameter,
        // but this doesn't have to be true. Firefox after failing with
        // the requested size, tries with the nearest powers of two, for example.
        let real_size = context.borrow_draw_buffer().unwrap().size();

        Ok(WebGLPaintTask {
            size: real_size,
            original_context_size: real_size,
            gl_context: context
        })
    }

    pub fn handle_webgl_message(&self, message: CanvasWebGLMsg) {
        match message {
            CanvasWebGLMsg::GetContextAttributes(sender) => self.get_context_attributes(sender),
            CanvasWebGLMsg::AttachShader(program_id, shader_id) => self.attach_shader(program_id, shader_id),
            CanvasWebGLMsg::BufferData(buffer_type, data, usage) => self.buffer_data(buffer_type, data, usage),
            CanvasWebGLMsg::Clear(mask) => self.clear(mask),
            CanvasWebGLMsg::ClearColor(r, g, b, a) => self.clear_color(r, g, b, a),
            CanvasWebGLMsg::DrawArrays(mode, first, count) => self.draw_arrays(mode, first, count),
            CanvasWebGLMsg::EnableVertexAttribArray(attrib_id) => self.enable_vertex_attrib_array(attrib_id),
            CanvasWebGLMsg::GetAttribLocation(program_id, name, chan) =>
                self.get_attrib_location(program_id, name, chan),
            CanvasWebGLMsg::GetShaderInfoLog(shader_id, chan) => self.get_shader_info_log(shader_id, chan),
            CanvasWebGLMsg::GetShaderParameter(shader_id, param_id, chan) =>
                self.get_shader_parameter(shader_id, param_id, chan),
            CanvasWebGLMsg::GetUniformLocation(program_id, name, chan) =>
                self.get_uniform_location(program_id, name, chan),
            CanvasWebGLMsg::CompileShader(shader_id) => self.compile_shader(shader_id),
            CanvasWebGLMsg::CreateBuffer(chan) => self.create_buffer(chan),
            CanvasWebGLMsg::CreateFramebuffer(chan) => self.create_framebuffer(chan),
            CanvasWebGLMsg::CreateRenderbuffer(chan) => self.create_renderbuffer(chan),
            CanvasWebGLMsg::CreateTexture(chan) => self.create_texture(chan),
            CanvasWebGLMsg::CreateProgram(chan) => self.create_program(chan),
            CanvasWebGLMsg::CreateShader(shader_type, chan) => self.create_shader(shader_type, chan),
            CanvasWebGLMsg::DeleteBuffer(id) => self.delete_buffer(id),
            CanvasWebGLMsg::DeleteFramebuffer(id) => self.delete_framebuffer(id),
            CanvasWebGLMsg::DeleteRenderbuffer(id) => self.delete_renderbuffer(id),
            CanvasWebGLMsg::DeleteTexture(id) => self.delete_texture(id),
            CanvasWebGLMsg::DeleteProgram(id) => self.delete_program(id),
            CanvasWebGLMsg::DeleteShader(id) => self.delete_shader(id),
            CanvasWebGLMsg::BindBuffer(target, id) => self.bind_buffer(target, id),
            CanvasWebGLMsg::BindFramebuffer(target, id) => self.bind_framebuffer(target, id),
            CanvasWebGLMsg::BindRenderbuffer(target, id) => self.bind_renderbuffer(target, id),
            CanvasWebGLMsg::BindTexture(target, id) => self.bind_texture(target, id),
            CanvasWebGLMsg::LinkProgram(program_id) => self.link_program(program_id),
            CanvasWebGLMsg::ShaderSource(shader_id, source) => self.shader_source(shader_id, source),
            CanvasWebGLMsg::Uniform4fv(uniform_id, data) => self.uniform_4fv(uniform_id, data),
            CanvasWebGLMsg::UseProgram(program_id) => self.use_program(program_id),
            CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) =>
                self.vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset),
            CanvasWebGLMsg::Viewport(x, y, width, height) => self.viewport(x, y, width, height),
            CanvasWebGLMsg::DrawingBufferWidth(sender) => self.send_drawing_buffer_width(sender),
            CanvasWebGLMsg::DrawingBufferHeight(sender) => self.send_drawing_buffer_height(sender),
        }
    }

    pub fn start(size: Size2D<i32>, attrs: GLContextAttributes) -> Result<Sender<CanvasMsg>, &'static str> {
        let (chan, port) = channel::<CanvasMsg>();
        let mut painter = try!(WebGLPaintTask::new(size, attrs));
        spawn_named("WebGLTask".to_owned(), move || {
            painter.init();
            loop {
                match port.recv().unwrap() {
                    CanvasMsg::WebGL(message) => painter.handle_webgl_message(message),
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            CanvasCommonMsg::SendPixelContents(chan) =>
                                painter.send_pixel_contents(chan),
                            CanvasCommonMsg::SendNativeSurface(chan) =>
                                painter.send_native_surface(chan),
                            // TODO(ecoal95): handle error nicely
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size).unwrap(),
                        }
                    },
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLTask"),
                }
            }
        });

        Ok(chan)
    }

    fn get_context_attributes(&self, sender: Sender<GLContextAttributes>) {
        sender.send(*self.gl_context.borrow_attributes()).unwrap()
    }

    fn send_drawing_buffer_width(&self, sender: Sender<i32>) {
        sender.send(self.size.width).unwrap()
    }

    fn send_drawing_buffer_height(&self, sender: Sender<i32>) {
        sender.send(self.size.height).unwrap()
    }

    fn attach_shader(&self, program_id: u32, shader_id: u32) {
        gl::attach_shader(program_id, shader_id);
    }

    fn buffer_data(&self, buffer_type: u32, data: Vec<f32>, usage: u32) {
        gl::buffer_data(buffer_type, &data, usage);
    }

    fn clear(&self, mask: u32) {
        gl::clear(mask);
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl::clear_color(r, g, b, a);
    }

    fn create_buffer(&self, chan: Sender<Option<NonZero<u32>>>) {
        let buffer = gl::gen_buffers(1)[0];
        let buffer = if buffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(buffer) })
        };
        chan.send(buffer).unwrap();
    }

    fn create_framebuffer(&self, chan: Sender<Option<NonZero<u32>>>) {
        let framebuffer = gl::gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(framebuffer) })
        };
        chan.send(framebuffer).unwrap();
    }

    fn create_renderbuffer(&self, chan: Sender<Option<NonZero<u32>>>) {
        let renderbuffer = gl::gen_renderbuffers(1)[0];
        let renderbuffer = if renderbuffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(renderbuffer) })
        };
        chan.send(renderbuffer).unwrap();
    }

    fn create_texture(&self, chan: Sender<Option<NonZero<u32>>>) {
        let texture = gl::gen_framebuffers(1)[0];
        let texture = if texture == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(texture) })
        };
        chan.send(texture).unwrap();
    }

    fn create_program(&self, chan: Sender<Option<NonZero<u32>>>) {
        let program = gl::create_program();
        let program = if program == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(program) })
        };
        chan.send(program).unwrap();
    }

    fn create_shader(&self, shader_type: u32, chan: Sender<Option<NonZero<u32>>>) {
        let shader = gl::create_shader(shader_type);
        let shader = if shader == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(shader) })
        };
        chan.send(shader).unwrap();
    }

    #[inline]
    fn delete_buffer(&self, id: u32) {
        gl::delete_buffers(&[id]);
    }

    #[inline]
    fn delete_renderbuffer(&self, id: u32) {
        gl::delete_renderbuffers(&[id]);
    }

    #[inline]
    fn delete_framebuffer(&self, id: u32) {
        gl::delete_framebuffers(&[id]);
    }

    #[inline]
    fn delete_texture(&self, id: u32) {
        gl::delete_textures(&[id]);
    }

    #[inline]
    fn delete_program(&self, id: u32) {
        gl::delete_program(id);
    }

    #[inline]
    fn delete_shader(&self, id: u32) {
        gl::delete_shader(id);
    }

    #[inline]
    fn bind_buffer(&self, target: u32, id: u32) {
        gl::bind_buffer(target, id);
    }

    #[inline]
    fn bind_framebuffer(&self, target: u32, id: u32) {
        gl::bind_framebuffer(target, id);
    }

    #[inline]
    fn bind_renderbuffer(&self, target: u32, id: u32) {
        gl::bind_renderbuffer(target, id);
    }

    #[inline]
    fn bind_texture(&self, target: u32, id: u32) {
        gl::bind_texture(target, id);
    }

    // TODO(ecoal95): This is not spec-compliant, we must check
    // the version of GLSL used. This functionality should probably
    // be in the WebGLShader object
    fn compile_shader(&self, shader_id: u32) {
        gl::compile_shader(shader_id);
    }

    fn draw_arrays(&self, mode: u32, first: i32, count: i32) {
        gl::draw_arrays(mode, first, count);
    }

    fn enable_vertex_attrib_array(&self, attrib_id: u32) {
        gl::enable_vertex_attrib_array(attrib_id);
    }

    fn get_attrib_location(&self, program_id: u32, name: String, chan: Sender<i32> ) {
        let attrib_location = gl::get_attrib_location(program_id, &name);
        chan.send(attrib_location).unwrap();
    }

    fn get_shader_info_log(&self, shader_id: u32, chan: Sender<String>) {
        let info = gl::get_shader_info_log(shader_id);
        chan.send(info).unwrap();
    }

    fn get_shader_parameter(&self, shader_id: u32, param_id: u32, chan: Sender<i32>) {
        let parameter = gl::get_shader_iv(shader_id, param_id);
        chan.send(parameter as i32).unwrap();
    }

    fn get_uniform_location(&self, program_id: u32, name: String, chan: Sender<Option<i32>>) {
        let location = gl::get_uniform_location(program_id, &name);
        let location = if location == -1 {
            None
        } else {
            Some(location)
        };

        chan.send(location).unwrap();
    }

    fn link_program(&self, program_id: u32) {
        gl::link_program(program_id);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
        // FIXME(#5652, dmarcos) Instead of a readback strategy we have
        // to layerize the canvas
        let width = self.size.width as usize;
        let height = self.size.height as usize;
        let mut pixels = gl::read_pixels(0, 0,
                                         self.size.width as gl::GLsizei,
                                         self.size.height as gl::GLsizei,
                                         gl::RGBA, gl::UNSIGNED_BYTE);
        // flip image vertically (texture is upside down)
        let orig_pixels = pixels.clone();
        let stride = width * 4;
        for y in 0..height {
            let dst_start = y * stride;
            let src_start = (height - y - 1) * stride;
            let src_slice = &orig_pixels[src_start .. src_start + stride];
            copy_memory(&src_slice[..stride], &mut pixels[dst_start .. dst_start + stride]);
        }

        // rgba -> bgra
        byte_swap(&mut pixels);
        chan.send(pixels).unwrap();
    }

    fn send_native_surface(&self, _: Sender<NativeSurface>) {
        // FIXME(ecoal95): We need to make a clone of the surface in order to
        // implement this
        unimplemented!()
    }

    fn shader_source(&self, shader_id: u32, source: String) {
        gl::shader_source(shader_id, &[source.as_bytes()]);
    }

    fn uniform_4fv(&self, uniform_id: i32, data: Vec<f32>) {
        gl::uniform_4f(uniform_id, data[0], data[1], data[2], data[3]);
    }

    fn use_program(&self, program_id: u32) {
        gl::use_program(program_id);
    }

    fn vertex_attrib_pointer_f32(&self, attrib_id: u32, size: i32,
                              normalized: bool, stride: i32, offset: i64) {
        gl::vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset as u32);
    }

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        gl::viewport(x, y, width, height);
    }

    fn recreate(&mut self, size: Size2D<i32>) -> Result<(), &'static str> {
        if size.width > self.original_context_size.width ||
           size.height > self.original_context_size.height {
            try!(self.gl_context.resize(size));
            self.size = self.gl_context.borrow_draw_buffer().unwrap().size();
        } else {
            self.size = size;
            unsafe { gl::Scissor(0, 0, size.width, size.height); }
        }
        Ok(())
    }

    fn init(&mut self) {
        self.gl_context.make_current().unwrap();
    }
}
