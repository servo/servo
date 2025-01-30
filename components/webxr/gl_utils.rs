/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::SurfmanGL;
use glow as gl;
use glow::Context as Gl;
use glow::HasContext;
use std::collections::HashMap;
use std::num::NonZero;
use surfman::Device as SurfmanDevice;
use webxr_api::ContextId;
use webxr_api::GLContexts;
use webxr_api::LayerId;

pub(crate) fn framebuffer(framebuffer: u32) -> Option<gl::NativeFramebuffer> {
    NonZero::new(framebuffer).map(gl::NativeFramebuffer)
}

// A utility to clear a color texture and optional depth/stencil texture
pub(crate) struct GlClearer {
    fbos: HashMap<
        (
            LayerId,
            Option<gl::NativeTexture>,
            Option<gl::NativeTexture>,
        ),
        Option<gl::NativeFramebuffer>,
    >,
    should_reverse_winding: bool,
}

impl GlClearer {
    pub(crate) fn new(should_reverse_winding: bool) -> GlClearer {
        let fbos = HashMap::new();
        GlClearer {
            fbos,
            should_reverse_winding,
        }
    }

    fn fbo(
        &mut self,
        gl: &Gl,
        layer_id: LayerId,
        color: Option<gl::NativeTexture>,
        color_target: u32,
        depth_stencil: Option<gl::NativeTexture>,
    ) -> Option<gl::NativeFramebuffer> {
        let should_reverse_winding = self.should_reverse_winding;
        *self
            .fbos
            .entry((layer_id, color, depth_stencil))
            .or_insert_with(|| {
                // Save the current GL state
                let mut bound_fbos = [0, 0];
                unsafe {
                    gl.get_parameter_i32_slice(gl::DRAW_FRAMEBUFFER_BINDING, &mut bound_fbos[0..]);
                    gl.get_parameter_i32_slice(gl::READ_FRAMEBUFFER_BINDING, &mut bound_fbos[1..]);

                    // Generate and set attachments of a new FBO
                    let fbo = gl.create_framebuffer().ok();

                    gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
                    gl.framebuffer_texture_2d(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        color_target,
                        color,
                        0,
                    );
                    gl.framebuffer_texture_2d(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_STENCIL_ATTACHMENT,
                        gl::TEXTURE_2D,
                        depth_stencil,
                        0,
                    );

                    // Necessary if using an OpenXR runtime that does not support mutable FOV,
                    // as flipping the projection matrix necessitates reversing the winding order.
                    if should_reverse_winding {
                        gl.front_face(gl::CW);
                    }

                    // Restore the GL state
                    gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, framebuffer(bound_fbos[0] as _));
                    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, framebuffer(bound_fbos[1] as _));
                    debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

                    fbo
                }
            })
    }

    pub(crate) fn clear(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        layer_id: LayerId,
        color: Option<glow::NativeTexture>,
        color_target: u32,
        depth_stencil: Option<glow::NativeTexture>,
    ) {
        let gl = match contexts.bindings(device, context_id) {
            None => return,
            Some(gl) => gl,
        };
        let fbo = self.fbo(gl, layer_id, color, color_target, depth_stencil);
        unsafe {
            // Save the current GL state
            let mut bound_fbos = [0, 0];
            let mut clear_color = [0., 0., 0., 0.];
            let mut clear_depth = [0.];
            let mut clear_stencil = [0];
            let color_mask;
            let depth_mask;
            let mut stencil_mask = [0];
            let scissor_enabled = gl.is_enabled(gl::SCISSOR_TEST);
            let rasterizer_enabled = gl.is_enabled(gl::RASTERIZER_DISCARD);

            gl.get_parameter_i32_slice(gl::DRAW_FRAMEBUFFER_BINDING, &mut bound_fbos[0..]);
            gl.get_parameter_i32_slice(gl::READ_FRAMEBUFFER_BINDING, &mut bound_fbos[1..]);
            gl.get_parameter_f32_slice(gl::COLOR_CLEAR_VALUE, &mut clear_color[..]);
            gl.get_parameter_f32_slice(gl::DEPTH_CLEAR_VALUE, &mut clear_depth[..]);
            gl.get_parameter_i32_slice(gl::STENCIL_CLEAR_VALUE, &mut clear_stencil[..]);
            depth_mask = gl.get_parameter_bool(gl::DEPTH_WRITEMASK);
            gl.get_parameter_i32_slice(gl::STENCIL_WRITEMASK, &mut stencil_mask[..]);
            color_mask = gl.get_parameter_bool_array::<4>(gl::COLOR_WRITEMASK);

            // Clear it
            gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
            gl.clear_color(0., 0., 0., 1.);
            gl.clear_depth(1.);
            gl.clear_stencil(0);
            gl.disable(gl::SCISSOR_TEST);
            gl.disable(gl::RASTERIZER_DISCARD);
            gl.depth_mask(true);
            gl.stencil_mask(0xFFFFFFFF);
            gl.color_mask(true, true, true, true);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

            // Restore the GL state
            gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, framebuffer(bound_fbos[0] as _));
            gl.bind_framebuffer(gl::READ_FRAMEBUFFER, framebuffer(bound_fbos[1] as _));
            gl.clear_color(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
            gl.color_mask(color_mask[0], color_mask[1], color_mask[2], color_mask[3]);
            gl.clear_depth(clear_depth[0] as f64);
            gl.clear_stencil(clear_stencil[0]);
            gl.depth_mask(depth_mask);
            gl.stencil_mask(stencil_mask[0] as _);
            if scissor_enabled {
                gl.enable(gl::SCISSOR_TEST);
            }
            if rasterizer_enabled {
                gl.enable(gl::RASTERIZER_DISCARD);
            }
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
        }
    }

    pub(crate) fn destroy_layer(
        &mut self,
        device: &mut SurfmanDevice,
        contexts: &mut dyn GLContexts<SurfmanGL>,
        context_id: ContextId,
        layer_id: LayerId,
    ) {
        let gl = match contexts.bindings(device, context_id) {
            None => return,
            Some(gl) => gl,
        };
        self.fbos.retain(|&(other_id, _, _), &mut fbo| {
            if layer_id != other_id {
                true
            } else {
                if let Some(fbo) = fbo {
                    unsafe { gl.delete_framebuffer(fbo) };
                }
                false
            }
        })
    }
}
