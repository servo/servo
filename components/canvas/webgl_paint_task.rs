/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_msg::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use geom::size::Size2D;

use gleam::gl;
use gleam::gl::types::{GLsizei};

use util::task::spawn_named;

use std::borrow::ToOwned;
use std::slice::bytes::copy_memory;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;
use offscreen_gl_context::{GLContext, GLContextAttributes};

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    original_context_size: Size2D<i32>,
    gl_context: GLContext,
}

// This allows trying to create the PaintTask
// before creating the thread
unsafe impl Send for WebGLPaintTask {}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>) -> Result<WebGLPaintTask, &'static str> {
        // TODO(ecoal95): Get the GLContextAttributes from the `GetContext` call
        let context = try!(GLContext::create_offscreen(size, GLContextAttributes::default()));
        Ok(WebGLPaintTask {
            size: size,
            original_context_size: size,
            gl_context: context
        })
    }

    pub fn start(size: Size2D<i32>) -> Result<Sender<CanvasMsg>, &'static str> {
        let (chan, port) = channel::<CanvasMsg>();
        let mut painter = try!(WebGLPaintTask::new(size));
        spawn_named("WebGLTask".to_owned(), move || {
            painter.init();
            loop {
                match port.recv().unwrap() {
                    CanvasMsg::WebGL(message) => {
                        match message {
                            CanvasWebGLMsg::AttachShader(program_id, shader_id) => painter.attach_shader(program_id, shader_id),
                            CanvasWebGLMsg::BindBuffer(buffer_type, buffer_id) => painter.bind_buffer(buffer_type, buffer_id),
                            CanvasWebGLMsg::BufferData(buffer_type, data, usage) => painter.buffer_data(buffer_type, data, usage),
                            CanvasWebGLMsg::Clear(mask) => painter.clear(mask),
                            CanvasWebGLMsg::ClearColor(r, g, b, a) => painter.clear_color(r, g, b, a),
                            CanvasWebGLMsg::CreateBuffer(chan) => painter.create_buffer(chan),
                            CanvasWebGLMsg::DrawArrays(mode, first, count) => painter.draw_arrays(mode, first, count),
                            CanvasWebGLMsg::EnableVertexAttribArray(attrib_id) => painter.enable_vertex_attrib_array(attrib_id),
                            CanvasWebGLMsg::GetAttribLocation(program_id, name, chan) => painter.get_attrib_location(program_id, name, chan),
                            CanvasWebGLMsg::GetShaderInfoLog(shader_id, chan) => painter.get_shader_info_log(shader_id, chan),
                            CanvasWebGLMsg::GetShaderParameter(shader_id, param_id, chan) => painter.get_shader_parameter(shader_id, param_id, chan),
                            CanvasWebGLMsg::GetUniformLocation(program_id, name, chan) => painter.get_uniform_location(program_id, name, chan),
                            CanvasWebGLMsg::CompileShader(shader_id) => painter.compile_shader(shader_id),
                            CanvasWebGLMsg::CreateProgram(chan) => painter.create_program(chan),
                            CanvasWebGLMsg::CreateShader(shader_type, chan) => painter.create_shader(shader_type, chan),
                            CanvasWebGLMsg::LinkProgram(program_id) => painter.link_program(program_id),
                            CanvasWebGLMsg::ShaderSource(shader_id, source) => painter.shader_source(shader_id, source),
                            CanvasWebGLMsg::Uniform4fv(uniform_id, data) => painter.uniform_4fv(uniform_id, data),
                            CanvasWebGLMsg::UseProgram(program_id) => painter.use_program(program_id),
                            CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) => {
                                painter.vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset);
                            },
                            CanvasWebGLMsg::Viewport(x, y, width, height) => painter.viewport(x, y, width, height),
                        }
                    },
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            CanvasCommonMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
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

    fn attach_shader(&self, program_id: u32, shader_id: u32) {
        gl::attach_shader(program_id, shader_id);
    }

    fn bind_buffer(&self, buffer_type: u32, buffer_id: u32) {
        gl::bind_buffer(buffer_type, buffer_id);
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

    fn create_buffer(&self, chan: Sender<u32>) {
        let buffers = gl::gen_buffers(1);
        chan.send(buffers[0]).unwrap();
    }

    fn compile_shader(&self, shader_id: u32) {
        gl::compile_shader(shader_id);
    }

    fn create_program(&self, chan: Sender<u32>) {
        let program = gl::create_program();
        chan.send(program).unwrap();
    }

    fn create_shader(&self, shader_type: u32, chan: Sender<u32>) {
        let shader = gl::create_shader(shader_type);
        chan.send(shader).unwrap();
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

    fn get_uniform_location(&self, program_id: u32, name: String, chan: Sender<u32>) {
        let uniform_location = gl::get_uniform_location(program_id, &name);
        chan.send(uniform_location as u32).unwrap();
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

    fn shader_source(&self, shader_id: u32, source_lines: Vec<String>) {
        let mut lines: Vec<&[u8]> = source_lines.iter().map(|line| line.as_bytes()).collect();
        gl::shader_source(shader_id, &mut lines);
    }

    fn uniform_4fv(&self, uniform_id: u32, data: Vec<f32>) {
        gl::uniform_4f(uniform_id as i32, data[0], data[1], data[2], data[3]);
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
            self.size = size;
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
