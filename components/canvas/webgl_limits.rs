/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::{GLLimits, WebGLVersion};
use sparkle::gl;
use sparkle::gl::GLenum;
use sparkle::gl::Gl;
use sparkle::gl::GlType;

pub trait GLLimitsDetect {
    fn detect(gl: &Gl, webgl_version: WebGLVersion) -> Self;
}

impl GLLimitsDetect for GLLimits {
    fn detect(gl: &Gl, webgl_version: WebGLVersion) -> GLLimits {
        let max_vertex_attribs = gl.get_integer(gl::MAX_VERTEX_ATTRIBS);
        let max_tex_size = gl.get_integer(gl::MAX_TEXTURE_SIZE);
        let max_cube_map_tex_size = gl.get_integer(gl::MAX_CUBE_MAP_TEXTURE_SIZE);
        let max_combined_texture_image_units = gl.get_integer(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS);
        let max_renderbuffer_size = gl.get_integer(gl::MAX_RENDERBUFFER_SIZE);
        let max_texture_image_units = gl.get_integer(gl::MAX_TEXTURE_IMAGE_UNITS);
        let max_vertex_texture_image_units = gl.get_integer(gl::MAX_VERTEX_TEXTURE_IMAGE_UNITS);

        // TODO: better value for this?
        let max_client_wait_timeout_webgl = std::time::Duration::new(1, 0);

        // Based on:
        // https://searchfox.org/mozilla-central/rev/5a744713370ec47969595e369fd5125f123e6d24/dom/canvas/WebGLContextValidate.cpp#523-558
        let (
            max_fragment_uniform_vectors,
            max_varying_vectors,
            max_vertex_uniform_vectors,
            max_vertex_output_vectors,
            max_fragment_input_vectors,
        );
        if gl.get_type() == GlType::Gles {
            max_fragment_uniform_vectors = gl.get_integer(gl::MAX_FRAGMENT_UNIFORM_VECTORS);
            max_varying_vectors = gl.get_integer(gl::MAX_VARYING_VECTORS);
            max_vertex_uniform_vectors = gl.get_integer(gl::MAX_VERTEX_UNIFORM_VECTORS);
            max_vertex_output_vectors = gl
                .try_get_integer(gl::MAX_VERTEX_OUTPUT_COMPONENTS)
                .map(|c| c / 4)
                .unwrap_or(max_varying_vectors);
            max_fragment_input_vectors = gl
                .try_get_integer(gl::MAX_FRAGMENT_INPUT_COMPONENTS)
                .map(|c| c / 4)
                .unwrap_or(max_vertex_output_vectors);
        } else {
            max_fragment_uniform_vectors = gl.get_integer(gl::MAX_FRAGMENT_UNIFORM_COMPONENTS) / 4;
            max_vertex_uniform_vectors = gl.get_integer(gl::MAX_VERTEX_UNIFORM_COMPONENTS) / 4;

            max_fragment_input_vectors = gl
                .try_get_integer(gl::MAX_FRAGMENT_INPUT_COMPONENTS)
                .or_else(|| gl.try_get_integer(gl::MAX_VARYING_COMPONENTS))
                .map(|c| c / 4)
                .unwrap_or_else(|| gl.get_integer(gl::MAX_VARYING_VECTORS));
            max_vertex_output_vectors = gl
                .try_get_integer(gl::MAX_VERTEX_OUTPUT_COMPONENTS)
                .map(|c| c / 4)
                .unwrap_or(max_fragment_input_vectors);
            max_varying_vectors = max_vertex_output_vectors
                .min(max_fragment_input_vectors)
                .max(4);
        };

        let (
            max_uniform_block_size,
            max_uniform_buffer_bindings,
            min_program_texel_offset,
            max_program_texel_offset,
            max_transform_feedback_separate_attribs,
            max_draw_buffers,
            max_color_attachments,
            max_combined_uniform_blocks,
            max_combined_vertex_uniform_components,
            max_combined_fragment_uniform_components,
            max_vertex_uniform_blocks,
            max_vertex_uniform_components,
            max_fragment_uniform_blocks,
            max_fragment_uniform_components,
            uniform_buffer_offset_alignment,
        );
        if webgl_version == WebGLVersion::WebGL2 {
            max_uniform_block_size = gl.get_integer(gl::MAX_UNIFORM_BLOCK_SIZE);
            max_uniform_buffer_bindings = gl.get_integer(gl::MAX_UNIFORM_BUFFER_BINDINGS);
            min_program_texel_offset = gl.get_integer(gl::MIN_PROGRAM_TEXEL_OFFSET);
            max_program_texel_offset = gl.get_integer(gl::MAX_PROGRAM_TEXEL_OFFSET);
            max_transform_feedback_separate_attribs =
                gl.get_integer(gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS);
            max_color_attachments = gl.get_integer(gl::MAX_COLOR_ATTACHMENTS);
            max_draw_buffers = gl
                .get_integer(gl::MAX_DRAW_BUFFERS)
                .min(max_color_attachments);
            max_combined_uniform_blocks = gl.get_integer(gl::MAX_COMBINED_UNIFORM_BLOCKS);
            max_combined_vertex_uniform_components =
                gl.get_integer(gl::MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS);
            max_combined_fragment_uniform_components =
                gl.get_integer(gl::MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS);
            max_vertex_uniform_blocks = gl.get_integer(gl::MAX_VERTEX_UNIFORM_BLOCKS);
            max_vertex_uniform_components = gl.get_integer(gl::MAX_VERTEX_UNIFORM_COMPONENTS);
            max_fragment_uniform_blocks = gl.get_integer(gl::MAX_FRAGMENT_UNIFORM_BLOCKS);
            max_fragment_uniform_components = gl.get_integer(gl::MAX_FRAGMENT_UNIFORM_COMPONENTS);
            uniform_buffer_offset_alignment = gl.get_integer(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT);
        } else {
            max_uniform_block_size = 0;
            max_uniform_buffer_bindings = 0;
            min_program_texel_offset = 0;
            max_program_texel_offset = 0;
            max_transform_feedback_separate_attribs = 0;
            max_color_attachments = 1;
            max_draw_buffers = 1;
            max_combined_uniform_blocks = 0;
            max_combined_vertex_uniform_components = 0;
            max_combined_fragment_uniform_components = 0;
            max_vertex_uniform_blocks = 0;
            max_vertex_uniform_components = 0;
            max_fragment_uniform_blocks = 0;
            max_fragment_uniform_components = 0;
            uniform_buffer_offset_alignment = 0;
        }

        GLLimits {
            max_vertex_attribs,
            max_tex_size,
            max_cube_map_tex_size,
            max_combined_texture_image_units,
            max_fragment_uniform_vectors,
            max_renderbuffer_size,
            max_texture_image_units,
            max_varying_vectors,
            max_vertex_texture_image_units,
            max_vertex_uniform_vectors,
            max_client_wait_timeout_webgl,
            max_transform_feedback_separate_attribs,
            max_vertex_output_vectors,
            max_fragment_input_vectors,
            max_uniform_buffer_bindings,
            min_program_texel_offset,
            max_program_texel_offset,
            max_color_attachments,
            max_draw_buffers,
            max_uniform_block_size,
            max_combined_uniform_blocks,
            max_combined_vertex_uniform_components,
            max_combined_fragment_uniform_components,
            max_vertex_uniform_blocks,
            max_vertex_uniform_components,
            max_fragment_uniform_blocks,
            max_fragment_uniform_components,
            uniform_buffer_offset_alignment,
        }
    }
}

trait GLExt {
    fn try_get_integer(self, parameter: GLenum) -> Option<u32>;
    fn get_integer(self, parameter: GLenum) -> u32;
}

impl<'a> GLExt for &'a Gl {
    #[allow(unsafe_code)]
    fn try_get_integer(self, parameter: GLenum) -> Option<u32> {
        let mut value = [0];
        unsafe {
            self.get_integer_v(parameter, &mut value);
        }
        if self.get_error() != gl::NO_ERROR {
            None
        } else {
            Some(value[0] as u32)
        }
    }

    fn get_integer(self, parameter: GLenum) -> u32 {
        self.try_get_integer(parameter).unwrap()
    }
}
