/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::GLLimits;
use sparkle::gl;
use sparkle::gl::GLenum;
use sparkle::gl::Gl;

pub trait GLLimitsDetect {
    fn detect(gl: &Gl) -> Self;
}

impl GLLimitsDetect for GLLimits {
    fn detect(gl: &Gl) -> GLLimits {
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
        let (max_fragment_uniform_vectors, max_varying_vectors, max_vertex_uniform_vectors);
        match gl.try_get_integer(gl::MAX_FRAGMENT_UNIFORM_VECTORS) {
            Some(max_vectors) => {
                max_fragment_uniform_vectors = max_vectors;
                max_varying_vectors = gl.get_integer(gl::MAX_VARYING_VECTORS);
                max_vertex_uniform_vectors = gl.get_integer(gl::MAX_VERTEX_UNIFORM_VECTORS);
            },
            None => {
                let max_fragment_uniform_components =
                    gl.get_integer(gl::MAX_FRAGMENT_UNIFORM_COMPONENTS);
                let max_vertex_uniform_components =
                    gl.get_integer(gl::MAX_VERTEX_UNIFORM_COMPONENTS);

                let max_vertex_output_components = gl
                    .try_get_integer(gl::MAX_VERTEX_OUTPUT_COMPONENTS)
                    .unwrap_or(0);
                let max_fragment_input_components = gl
                    .try_get_integer(gl::MAX_FRAGMENT_INPUT_COMPONENTS)
                    .unwrap_or(0);
                let max_varying_components = max_vertex_output_components
                    .min(max_fragment_input_components)
                    .max(16);

                max_fragment_uniform_vectors = max_fragment_uniform_components / 4;
                max_varying_vectors = max_varying_components / 4;
                max_vertex_uniform_vectors = max_vertex_uniform_components / 4;
            },
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
