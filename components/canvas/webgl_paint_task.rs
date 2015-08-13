/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg, FromLayoutMsg, FromPaintMsg};
use canvas_traits::{WebGLShaderParameter, WebGLFramebufferBindingRequest};
use core::nonzero::NonZero;
use euclid::size::Size2D;
use gleam::gl;
use gleam::gl::types::{GLsizei};

use util::task::spawn_named;

use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use ipc_channel::router::ROUTER;
use layers::platform::surface::NativeSurface;
use offscreen_gl_context::{GLContext, GLContextAttributes, ColorAttachmentType};
use std::borrow::ToOwned;
use std::slice::bytes::copy_memory;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;

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

    /// In this function the gl commands are called.
    /// Those messages that just involve a gl call have the call inlined,
    /// processing of messages that require extra work are moved to functions
    ///
    /// NB: Not gl-related validations (names, lengths, accepted parameters...) are
    /// done in the corresponding DOM interfaces
    pub fn handle_webgl_message(&self, message: CanvasWebGLMsg) {
        match message {
            CanvasWebGLMsg::GetContextAttributes(sender) =>
                self.get_context_attributes(sender),
            CanvasWebGLMsg::ActiveTexture(target) =>
                gl::active_texture(target),
            CanvasWebGLMsg::BlendColor(r, g, b, a) =>
                gl::blend_color(r, g, b, a),
            CanvasWebGLMsg::BlendEquation(mode) =>
                gl::blend_equation(mode),
            CanvasWebGLMsg::BlendEquationSeparate(mode_rgb, mode_alpha) =>
                gl::blend_equation_separate(mode_rgb, mode_alpha),
            CanvasWebGLMsg::BlendFunc(src, dest) =>
                gl::blend_func(src, dest),
            CanvasWebGLMsg::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha) =>
                gl::blend_func_separate(src_rgb, dest_rgb, src_alpha, dest_alpha),
            CanvasWebGLMsg::AttachShader(program_id, shader_id) =>
                gl::attach_shader(program_id, shader_id),
            CanvasWebGLMsg::BufferData(buffer_type, data, usage) =>
                gl::buffer_data(buffer_type, &data, usage),
            CanvasWebGLMsg::Clear(mask) =>
                gl::clear(mask),
            CanvasWebGLMsg::ClearColor(r, g, b, a) =>
                gl::clear_color(r, g, b, a),
            CanvasWebGLMsg::ClearDepth(depth) =>
                gl::clear_depth(depth),
            CanvasWebGLMsg::ClearStencil(stencil) =>
                gl::clear_stencil(stencil),
            CanvasWebGLMsg::ColorMask(r, g, b, a) =>
                gl::color_mask(r, g, b, a),
            CanvasWebGLMsg::CullFace(mode) =>
                gl::cull_face(mode),
            CanvasWebGLMsg::DepthFunc(func) =>
                gl::depth_func(func),
            CanvasWebGLMsg::DepthMask(flag) =>
                gl::depth_mask(flag),
            CanvasWebGLMsg::DepthRange(near, far) =>
                gl::depth_range(near, far),
            CanvasWebGLMsg::Disable(cap) =>
                gl::disable(cap),
            CanvasWebGLMsg::Enable(cap) =>
                gl::enable(cap),
            CanvasWebGLMsg::FrontFace(mode) =>
                gl::front_face(mode),
            CanvasWebGLMsg::DrawArrays(mode, first, count) =>
                gl::draw_arrays(mode, first, count),
            CanvasWebGLMsg::Hint(name, val) =>
                gl::hint(name, val),
            CanvasWebGLMsg::LineWidth(width) =>
                gl::line_width(width),
            CanvasWebGLMsg::PixelStorei(name, val) =>
                gl::pixel_store_i(name, val),
            CanvasWebGLMsg::PolygonOffset(factor, units) =>
                gl::polygon_offset(factor, units),
            CanvasWebGLMsg::EnableVertexAttribArray(attrib_id) =>
                gl::enable_vertex_attrib_array(attrib_id),
            CanvasWebGLMsg::GetAttribLocation(program_id, name, chan) =>
                self.get_attrib_location(program_id, name, chan),
            CanvasWebGLMsg::GetShaderInfoLog(shader_id, chan) =>
                self.get_shader_info_log(shader_id, chan),
            CanvasWebGLMsg::GetShaderParameter(shader_id, param_id, chan) =>
                self.get_shader_parameter(shader_id, param_id, chan),
            CanvasWebGLMsg::GetUniformLocation(program_id, name, chan) =>
                self.get_uniform_location(program_id, name, chan),
            CanvasWebGLMsg::CompileShader(shader_id) =>
                self.compile_shader(shader_id),
            CanvasWebGLMsg::CreateBuffer(chan) =>
                self.create_buffer(chan),
            CanvasWebGLMsg::CreateFramebuffer(chan) =>
                self.create_framebuffer(chan),
            CanvasWebGLMsg::CreateRenderbuffer(chan) =>
                self.create_renderbuffer(chan),
            CanvasWebGLMsg::CreateTexture(chan) =>
                self.create_texture(chan),
            CanvasWebGLMsg::CreateProgram(chan) =>
                self.create_program(chan),
            CanvasWebGLMsg::CreateShader(shader_type, chan) =>
                self.create_shader(shader_type, chan),
            CanvasWebGLMsg::DeleteBuffer(id) =>
                gl::delete_buffers(&[id]),
            CanvasWebGLMsg::DeleteFramebuffer(id) =>
                gl::delete_framebuffers(&[id]),
            CanvasWebGLMsg::DeleteRenderbuffer(id) =>
                gl::delete_renderbuffers(&[id]),
            CanvasWebGLMsg::DeleteTexture(id) =>
                gl::delete_textures(&[id]),
            CanvasWebGLMsg::DeleteProgram(id) =>
                gl::delete_program(id),
            CanvasWebGLMsg::DeleteShader(id) =>
                gl::delete_shader(id),
            CanvasWebGLMsg::BindBuffer(target, id) =>
                gl::bind_buffer(target, id),
            CanvasWebGLMsg::BindFramebuffer(target, request) =>
                self.bind_framebuffer(target, request),
            CanvasWebGLMsg::BindRenderbuffer(target, id) =>
                gl::bind_renderbuffer(target, id),
            CanvasWebGLMsg::BindTexture(target, id) =>
                gl::bind_texture(target, id),
            CanvasWebGLMsg::LinkProgram(program_id) =>
                gl::link_program(program_id),
            CanvasWebGLMsg::ShaderSource(shader_id, source) =>
                gl::shader_source(shader_id, &[source.as_bytes()]),
            CanvasWebGLMsg::Uniform4fv(uniform_id, data) =>
                gl::uniform_4f(uniform_id, data[0], data[1], data[2], data[3]),
            CanvasWebGLMsg::UseProgram(program_id) =>
                gl::use_program(program_id),
            CanvasWebGLMsg::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) =>
                gl::vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset as u32),
            CanvasWebGLMsg::Viewport(x, y, width, height) =>
                gl::viewport(x, y, width, height),
            CanvasWebGLMsg::TexImage2D(target, level, internal, width, height, format, data_type, data) =>
                gl::tex_image_2d(target, level, internal, width, height, /*border*/0, format, data_type, Some(&data)),
            CanvasWebGLMsg::TexParameteri(target, name, value) =>
                gl::tex_parameter_i(target, name, value),
            CanvasWebGLMsg::TexParameterf(target, name, value) =>
                gl::tex_parameter_f(target, name, value),
            CanvasWebGLMsg::DrawingBufferWidth(sender) =>
                self.send_drawing_buffer_width(sender),
            CanvasWebGLMsg::DrawingBufferHeight(sender) =>
                self.send_drawing_buffer_height(sender),
        }
    }

    /// Creates a new `WebGLPaintTask` and returns the out-of-process sender and the in-process
    /// sender for it.
    pub fn start(size: Size2D<i32>, attrs: GLContextAttributes)
                 -> Result<(IpcSender<CanvasMsg>, Sender<CanvasMsg>), &'static str> {
        let (out_of_process_chan, out_of_process_port) = ipc::channel::<CanvasMsg>().unwrap();
        let (in_process_chan, in_process_port) = channel();
        ROUTER.route_ipc_receiver_to_mpsc_sender(out_of_process_port, in_process_chan.clone());
        let mut painter = try!(WebGLPaintTask::new(size, attrs));
        spawn_named("WebGLTask".to_owned(), move || {
            painter.init();
            loop {
                match in_process_port.recv().unwrap() {
                    CanvasMsg::WebGL(message) => painter.handle_webgl_message(message),
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            // TODO(ecoal95): handle error nicely
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size).unwrap(),
                        }
                    },
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendPixelContents(chan) =>
                                painter.send_pixel_contents(chan),
                        }
                    }
                    CanvasMsg::FromPaint(message) => {
                        match message {
                            FromPaintMsg::SendNativeSurface(chan) =>
                                painter.send_native_surface(chan),
                        }
                    }
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLTask"),
                }
            }
        });

        Ok((out_of_process_chan, in_process_chan))
    }

    #[inline]
    fn get_context_attributes(&self, sender: IpcSender<GLContextAttributes>) {
        sender.send(*self.gl_context.borrow_attributes()).unwrap()
    }

    #[inline]
    fn send_drawing_buffer_width(&self, sender: IpcSender<i32>) {
        sender.send(self.size.width).unwrap()
    }

    #[inline]
    fn send_drawing_buffer_height(&self, sender: IpcSender<i32>) {
        sender.send(self.size.height).unwrap()
    }

    fn create_buffer(&self, chan: IpcSender<Option<NonZero<u32>>>) {
        let buffer = gl::gen_buffers(1)[0];
        let buffer = if buffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(buffer) })
        };
        chan.send(buffer).unwrap();
    }

    fn create_framebuffer(&self, chan: IpcSender<Option<NonZero<u32>>>) {
        let framebuffer = gl::gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(framebuffer) })
        };
        chan.send(framebuffer).unwrap();
    }

    fn create_renderbuffer(&self, chan: IpcSender<Option<NonZero<u32>>>) {
        let renderbuffer = gl::gen_renderbuffers(1)[0];
        let renderbuffer = if renderbuffer == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(renderbuffer) })
        };
        chan.send(renderbuffer).unwrap();
    }

    fn create_texture(&self, chan: IpcSender<Option<NonZero<u32>>>) {
        let texture = gl::gen_framebuffers(1)[0];
        let texture = if texture == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(texture) })
        };
        chan.send(texture).unwrap();
    }

    fn create_program(&self, chan: IpcSender<Option<NonZero<u32>>>) {
        let program = gl::create_program();
        let program = if program == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(program) })
        };
        chan.send(program).unwrap();
    }

    fn create_shader(&self, shader_type: u32, chan: IpcSender<Option<NonZero<u32>>>) {
        let shader = gl::create_shader(shader_type);
        let shader = if shader == 0 {
            None
        } else {
            Some(unsafe { NonZero::new(shader) })
        };
        chan.send(shader).unwrap();
    }

    #[inline]
    fn bind_framebuffer(&self, target: u32, request: WebGLFramebufferBindingRequest) {
        let id = match request {
            WebGLFramebufferBindingRequest::Explicit(id) => id,
            WebGLFramebufferBindingRequest::Default =>
                self.gl_context.borrow_draw_buffer().unwrap().get_framebuffer(),
        };

        gl::bind_framebuffer(target, id);
    }

    // TODO(ecoal95): This is not spec-compliant, we must check
    // the version of GLSL used. This functionality should probably
    // be in the WebGLShader object
    #[inline]
    fn compile_shader(&self, shader_id: u32) {
        gl::compile_shader(shader_id);
    }

    fn get_attrib_location(&self, program_id: u32, name: String, chan: IpcSender<Option<i32>> ) {
        let attrib_location = gl::get_attrib_location(program_id, &name);

        let attrib_location = if attrib_location == -1 {
            None
        } else {
            Some(attrib_location)
        };

        chan.send(attrib_location).unwrap();
    }

    fn get_shader_info_log(&self, shader_id: u32, chan: IpcSender<Option<String>>) {
        // TODO(ecoal95): Right now we always return a value, we should
        // check for gl errors and return None there
        let info = gl::get_shader_info_log(shader_id);
        chan.send(Some(info)).unwrap();
    }

    fn get_shader_parameter(&self,
                            shader_id: u32,
                            param_id: u32,
                            chan: IpcSender<WebGLShaderParameter>) {
        let result = match param_id {
            gl::SHADER_TYPE =>
                WebGLShaderParameter::Int(gl::get_shader_iv(shader_id, param_id)),
            gl::DELETE_STATUS | gl::COMPILE_STATUS =>
                WebGLShaderParameter::Bool(gl::get_shader_iv(shader_id, param_id) != 0),
            _ => panic!("Unexpected shader parameter type"),
        };

        chan.send(result).unwrap();
    }

    fn get_uniform_location(&self, program_id: u32, name: String, chan: IpcSender<Option<i32>>) {
        let location = gl::get_uniform_location(program_id, &name);
        let location = if location == -1 {
            None
        } else {
            Some(location)
        };

        chan.send(location).unwrap();
    }

    fn send_pixel_contents(&mut self, chan: IpcSender<IpcSharedMemory>) {
        // FIXME(#5652, dmarcos) Instead of a readback strategy we have
        // to layerize the canvas.
        // TODO(pcwalton): We'd save a copy if we had an `IpcSharedMemoryBuilder` abstraction that
        // allowed you to mutate in-place before freezing the object for sending.
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
        chan.send(IpcSharedMemory::from_bytes(&pixels[..])).unwrap();
    }

    fn send_native_surface(&self, _: Sender<NativeSurface>) {
        // FIXME(ecoal95): We need to make a clone of the surface in order to
        // implement this
        unimplemented!()
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
