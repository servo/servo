/*
 * Copyright © 2013 Intel Corporation
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

/**
 * \file builtin_types.cpp
 *
 * The glsl_type class has static members to represent all the built-in types
 * (such as the glsl_type::_float_type flyweight) as well as convenience pointer
 * accessors (such as glsl_type::float_type).  Those global variables are
 * declared and initialized in this file.
 *
 * This also contains _mesa_glsl_initialize_types(), a function which populates
 * a symbol table with the available built-in types for a particular language
 * version and set of enabled extensions.
 */

#include "compiler/glsl_types.h"
#include "glsl_parser_extras.h"
#include "util/macros.h"
#include "main/mtypes.h"

/**
 * Declarations of type flyweights (glsl_type::_foo_type) and
 * convenience pointers (glsl_type::foo_type).
 * @{
 */
#define DECL_TYPE(NAME, ...)

#define STRUCT_TYPE(NAME)                                       \
   const glsl_type glsl_type::_struct_##NAME##_type =           \
      glsl_type(NAME##_fields, ARRAY_SIZE(NAME##_fields), #NAME); \
   const glsl_type *const glsl_type::struct_##NAME##_type =     \
      &glsl_type::_struct_##NAME##_type;

static const struct glsl_struct_field gl_DepthRangeParameters_fields[] = {
   glsl_struct_field(glsl_type::float_type, GLSL_PRECISION_HIGH, "near"),
   glsl_struct_field(glsl_type::float_type, GLSL_PRECISION_HIGH, "far"),
   glsl_struct_field(glsl_type::float_type, GLSL_PRECISION_HIGH, "diff"),
};

static const struct glsl_struct_field gl_PointParameters_fields[] = {
   glsl_struct_field(glsl_type::float_type, "size"),
   glsl_struct_field(glsl_type::float_type, "sizeMin"),
   glsl_struct_field(glsl_type::float_type, "sizeMax"),
   glsl_struct_field(glsl_type::float_type, "fadeThresholdSize"),
   glsl_struct_field(glsl_type::float_type, "distanceConstantAttenuation"),
   glsl_struct_field(glsl_type::float_type, "distanceLinearAttenuation"),
   glsl_struct_field(glsl_type::float_type, "distanceQuadraticAttenuation"),
};

static const struct glsl_struct_field gl_MaterialParameters_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "emission"),
   glsl_struct_field(glsl_type::vec4_type, "ambient"),
   glsl_struct_field(glsl_type::vec4_type, "diffuse"),
   glsl_struct_field(glsl_type::vec4_type, "specular"),
   glsl_struct_field(glsl_type::float_type, "shininess"),
};

static const struct glsl_struct_field gl_LightSourceParameters_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "ambient"),
   glsl_struct_field(glsl_type::vec4_type, "diffuse"),
   glsl_struct_field(glsl_type::vec4_type, "specular"),
   glsl_struct_field(glsl_type::vec4_type, "position"),
   glsl_struct_field(glsl_type::vec4_type, "halfVector"),
   glsl_struct_field(glsl_type::vec3_type, "spotDirection"),
   glsl_struct_field(glsl_type::float_type, "spotExponent"),
   glsl_struct_field(glsl_type::float_type, "spotCutoff"),
   glsl_struct_field(glsl_type::float_type, "spotCosCutoff"),
   glsl_struct_field(glsl_type::float_type, "constantAttenuation"),
   glsl_struct_field(glsl_type::float_type, "linearAttenuation"),
   glsl_struct_field(glsl_type::float_type, "quadraticAttenuation"),
};

static const struct glsl_struct_field gl_LightModelParameters_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "ambient"),
};

static const struct glsl_struct_field gl_LightModelProducts_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "sceneColor"),
};

static const struct glsl_struct_field gl_LightProducts_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "ambient"),
   glsl_struct_field(glsl_type::vec4_type, "diffuse"),
   glsl_struct_field(glsl_type::vec4_type, "specular"),
};

static const struct glsl_struct_field gl_FogParameters_fields[] = {
   glsl_struct_field(glsl_type::vec4_type, "color"),
   glsl_struct_field(glsl_type::float_type, "density"),
   glsl_struct_field(glsl_type::float_type, "start"),
   glsl_struct_field(glsl_type::float_type, "end"),
   glsl_struct_field(glsl_type::float_type, "scale"),
};

#include "compiler/builtin_type_macros.h"
/** @} */

/**
 * Code to populate a symbol table with the built-in types available in a
 * particular shading language version.  The table below contains tags every
 * type with the GLSL/GLSL ES versions where it was introduced.
 *
 * @{
 */
#define T(TYPE, MIN_GL, MIN_ES) \
   { glsl_type::TYPE##_type, MIN_GL, MIN_ES },

static const struct builtin_type_versions {
   const glsl_type *const type;
   int min_gl;
   int min_es;
} builtin_type_versions[] = {
   T(void,                            110, 100)
   T(bool,                            110, 100)
   T(bvec2,                           110, 100)
   T(bvec3,                           110, 100)
   T(bvec4,                           110, 100)
   T(int,                             110, 100)
   T(ivec2,                           110, 100)
   T(ivec3,                           110, 100)
   T(ivec4,                           110, 100)
   T(uint,                            130, 300)
   T(uvec2,                           130, 300)
   T(uvec3,                           130, 300)
   T(uvec4,                           130, 300)
   T(float,                           110, 100)
   T(vec2,                            110, 100)
   T(vec3,                            110, 100)
   T(vec4,                            110, 100)
   T(mat2,                            110, 100)
   T(mat3,                            110, 100)
   T(mat4,                            110, 100)
   T(mat2x3,                          120, 300)
   T(mat2x4,                          120, 300)
   T(mat3x2,                          120, 300)
   T(mat3x4,                          120, 300)
   T(mat4x2,                          120, 300)
   T(mat4x3,                          120, 300)

   T(double,                          400, 999)
   T(dvec2,                           400, 999)
   T(dvec3,                           400, 999)
   T(dvec4,                           400, 999)
   T(dmat2,                           400, 999)
   T(dmat3,                           400, 999)
   T(dmat4,                           400, 999)
   T(dmat2x3,                         400, 999)
   T(dmat2x4,                         400, 999)
   T(dmat3x2,                         400, 999)
   T(dmat3x4,                         400, 999)
   T(dmat4x2,                         400, 999)
   T(dmat4x3,                         400, 999)

   T(sampler1D,                       110, 999)
   T(sampler2D,                       110, 100)
   T(sampler3D,                       110, 300)
   T(samplerCube,                     110, 100)
   T(sampler1DArray,                  130, 999)
   T(sampler2DArray,                  130, 300)
   T(samplerCubeArray,                400, 320)
   T(sampler2DRect,                   140, 999)
   T(samplerBuffer,                   140, 320)
   T(sampler2DMS,                     150, 310)
   T(sampler2DMSArray,                150, 320)

   T(isampler1D,                      130, 999)
   T(isampler2D,                      130, 300)
   T(isampler3D,                      130, 300)
   T(isamplerCube,                    130, 300)
   T(isampler1DArray,                 130, 999)
   T(isampler2DArray,                 130, 300)
   T(isamplerCubeArray,               400, 320)
   T(isampler2DRect,                  140, 999)
   T(isamplerBuffer,                  140, 320)
   T(isampler2DMS,                    150, 310)
   T(isampler2DMSArray,               150, 320)

   T(usampler1D,                      130, 999)
   T(usampler2D,                      130, 300)
   T(usampler3D,                      130, 300)
   T(usamplerCube,                    130, 300)
   T(usampler1DArray,                 130, 999)
   T(usampler2DArray,                 130, 300)
   T(usamplerCubeArray,               400, 320)
   T(usampler2DRect,                  140, 999)
   T(usamplerBuffer,                  140, 320)
   T(usampler2DMS,                    150, 310)
   T(usampler2DMSArray,               150, 320)

   T(sampler1DShadow,                 110, 999)
   T(sampler2DShadow,                 110, 300)
   T(samplerCubeShadow,               130, 300)
   T(sampler1DArrayShadow,            130, 999)
   T(sampler2DArrayShadow,            130, 300)
   T(samplerCubeArrayShadow,          400, 320)
   T(sampler2DRectShadow,             140, 999)

   T(struct_gl_DepthRangeParameters,  110, 100)

   T(image1D,                         420, 999)
   T(image2D,                         420, 310)
   T(image3D,                         420, 310)
   T(image2DRect,                     420, 999)
   T(imageCube,                       420, 310)
   T(imageBuffer,                     420, 320)
   T(image1DArray,                    420, 999)
   T(image2DArray,                    420, 310)
   T(imageCubeArray,                  420, 320)
   T(image2DMS,                       420, 999)
   T(image2DMSArray,                  420, 999)
   T(iimage1D,                        420, 999)
   T(iimage2D,                        420, 310)
   T(iimage3D,                        420, 310)
   T(iimage2DRect,                    420, 999)
   T(iimageCube,                      420, 310)
   T(iimageBuffer,                    420, 320)
   T(iimage1DArray,                   420, 999)
   T(iimage2DArray,                   420, 310)
   T(iimageCubeArray,                 420, 320)
   T(iimage2DMS,                      420, 999)
   T(iimage2DMSArray,                 420, 999)
   T(uimage1D,                        420, 999)
   T(uimage2D,                        420, 310)
   T(uimage3D,                        420, 310)
   T(uimage2DRect,                    420, 999)
   T(uimageCube,                      420, 310)
   T(uimageBuffer,                    420, 320)
   T(uimage1DArray,                   420, 999)
   T(uimage2DArray,                   420, 310)
   T(uimageCubeArray,                 420, 320)
   T(uimage2DMS,                      420, 999)
   T(uimage2DMSArray,                 420, 999)

   T(atomic_uint,                     420, 310)
};

static const glsl_type *const deprecated_types[] = {
   glsl_type::struct_gl_PointParameters_type,
   glsl_type::struct_gl_MaterialParameters_type,
   glsl_type::struct_gl_LightSourceParameters_type,
   glsl_type::struct_gl_LightModelParameters_type,
   glsl_type::struct_gl_LightModelProducts_type,
   glsl_type::struct_gl_LightProducts_type,
   glsl_type::struct_gl_FogParameters_type,
};

static inline void
add_type(glsl_symbol_table *symbols, const glsl_type *const type)
{
   symbols->add_type(type->name, type);
}

/**
 * Populate the symbol table with available built-in types.
 */
void
_mesa_glsl_initialize_types(struct _mesa_glsl_parse_state *state)
{
   struct glsl_symbol_table *symbols = state->symbols;

   for (unsigned i = 0; i < ARRAY_SIZE(builtin_type_versions); i++) {
      const struct builtin_type_versions *const t = &builtin_type_versions[i];
      if (state->is_version(t->min_gl, t->min_es)) {
         add_type(symbols, t->type);
      }
   }

   /* Add deprecated structure types.  While these were deprecated in 1.30,
    * they're still present.  We've removed them in 1.40+ (OpenGL 3.1+).
    */
   if (state->compat_shader || state->ARB_compatibility_enable) {
      for (unsigned i = 0; i < ARRAY_SIZE(deprecated_types); i++) {
         add_type(symbols, deprecated_types[i]);
      }
   }

   /* Add types for enabled extensions.  They may have already been added
    * by the version-based loop, but attempting to add them a second time
    * is harmless.
    */
   if (state->ARB_texture_cube_map_array_enable ||
       state->EXT_texture_cube_map_array_enable ||
       state->OES_texture_cube_map_array_enable) {
      add_type(symbols, glsl_type::samplerCubeArray_type);
      add_type(symbols, glsl_type::samplerCubeArrayShadow_type);
      add_type(symbols, glsl_type::isamplerCubeArray_type);
      add_type(symbols, glsl_type::usamplerCubeArray_type);
   }

   if (state->ARB_texture_multisample_enable) {
      add_type(symbols, glsl_type::sampler2DMS_type);
      add_type(symbols, glsl_type::isampler2DMS_type);
      add_type(symbols, glsl_type::usampler2DMS_type);
   }
   if (state->ARB_texture_multisample_enable ||
       state->OES_texture_storage_multisample_2d_array_enable) {
      add_type(symbols, glsl_type::sampler2DMSArray_type);
      add_type(symbols, glsl_type::isampler2DMSArray_type);
      add_type(symbols, glsl_type::usampler2DMSArray_type);
   }

   if (state->ARB_texture_rectangle_enable) {
      add_type(symbols, glsl_type::sampler2DRect_type);
      add_type(symbols, glsl_type::sampler2DRectShadow_type);
   }

   if (state->EXT_gpu_shader4_enable) {
      add_type(symbols, glsl_type::uint_type);
      add_type(symbols, glsl_type::uvec2_type);
      add_type(symbols, glsl_type::uvec3_type);
      add_type(symbols, glsl_type::uvec4_type);

      add_type(symbols, glsl_type::samplerCubeShadow_type);

      if (state->ctx->Extensions.EXT_texture_array) {
         add_type(symbols, glsl_type::sampler1DArray_type);
         add_type(symbols, glsl_type::sampler2DArray_type);
         add_type(symbols, glsl_type::sampler1DArrayShadow_type);
         add_type(symbols, glsl_type::sampler2DArrayShadow_type);
      }
      if (state->ctx->Extensions.EXT_texture_buffer_object) {
         add_type(symbols, glsl_type::samplerBuffer_type);
      }

      if (state->ctx->Extensions.EXT_texture_integer) {
         add_type(symbols, glsl_type::isampler1D_type);
         add_type(symbols, glsl_type::isampler2D_type);
         add_type(symbols, glsl_type::isampler3D_type);
         add_type(symbols, glsl_type::isamplerCube_type);

         add_type(symbols, glsl_type::usampler1D_type);
         add_type(symbols, glsl_type::usampler2D_type);
         add_type(symbols, glsl_type::usampler3D_type);
         add_type(symbols, glsl_type::usamplerCube_type);

         if (state->ctx->Extensions.NV_texture_rectangle) {
            add_type(symbols, glsl_type::isampler2DRect_type);
            add_type(symbols, glsl_type::usampler2DRect_type);
         }
         if (state->ctx->Extensions.EXT_texture_array) {
            add_type(symbols, glsl_type::isampler1DArray_type);
            add_type(symbols, glsl_type::isampler2DArray_type);
            add_type(symbols, glsl_type::usampler1DArray_type);
            add_type(symbols, glsl_type::usampler2DArray_type);
         }
         if (state->ctx->Extensions.EXT_texture_buffer_object) {
            add_type(symbols, glsl_type::isamplerBuffer_type);
            add_type(symbols, glsl_type::usamplerBuffer_type);
         }
      }
   }

   if (state->EXT_texture_array_enable) {
      add_type(symbols, glsl_type::sampler1DArray_type);
      add_type(symbols, glsl_type::sampler2DArray_type);
      add_type(symbols, glsl_type::sampler1DArrayShadow_type);
      add_type(symbols, glsl_type::sampler2DArrayShadow_type);
   }

   if (state->OES_EGL_image_external_enable ||
       state->OES_EGL_image_external_essl3_enable) {
      add_type(symbols, glsl_type::samplerExternalOES_type);
   }

   if (state->OES_texture_3D_enable) {
      add_type(symbols, glsl_type::sampler3D_type);
   }

   if (state->ARB_shader_image_load_store_enable ||
       state->EXT_texture_cube_map_array_enable ||
       state->OES_texture_cube_map_array_enable) {
      add_type(symbols, glsl_type::imageCubeArray_type);
      add_type(symbols, glsl_type::iimageCubeArray_type);
      add_type(symbols, glsl_type::uimageCubeArray_type);
   }

   if (state->ARB_shader_image_load_store_enable) {
      add_type(symbols, glsl_type::image1D_type);
      add_type(symbols, glsl_type::image2D_type);
      add_type(symbols, glsl_type::image3D_type);
      add_type(symbols, glsl_type::image2DRect_type);
      add_type(symbols, glsl_type::imageCube_type);
      add_type(symbols, glsl_type::imageBuffer_type);
      add_type(symbols, glsl_type::image1DArray_type);
      add_type(symbols, glsl_type::image2DArray_type);
      add_type(symbols, glsl_type::image2DMS_type);
      add_type(symbols, glsl_type::image2DMSArray_type);
      add_type(symbols, glsl_type::iimage1D_type);
      add_type(symbols, glsl_type::iimage2D_type);
      add_type(symbols, glsl_type::iimage3D_type);
      add_type(symbols, glsl_type::iimage2DRect_type);
      add_type(symbols, glsl_type::iimageCube_type);
      add_type(symbols, glsl_type::iimageBuffer_type);
      add_type(symbols, glsl_type::iimage1DArray_type);
      add_type(symbols, glsl_type::iimage2DArray_type);
      add_type(symbols, glsl_type::iimage2DMS_type);
      add_type(symbols, glsl_type::iimage2DMSArray_type);
      add_type(symbols, glsl_type::uimage1D_type);
      add_type(symbols, glsl_type::uimage2D_type);
      add_type(symbols, glsl_type::uimage3D_type);
      add_type(symbols, glsl_type::uimage2DRect_type);
      add_type(symbols, glsl_type::uimageCube_type);
      add_type(symbols, glsl_type::uimageBuffer_type);
      add_type(symbols, glsl_type::uimage1DArray_type);
      add_type(symbols, glsl_type::uimage2DArray_type);
      add_type(symbols, glsl_type::uimage2DMS_type);
      add_type(symbols, glsl_type::uimage2DMSArray_type);
   }

   if (state->EXT_texture_buffer_enable || state->OES_texture_buffer_enable) {
      add_type(symbols, glsl_type::samplerBuffer_type);
      add_type(symbols, glsl_type::isamplerBuffer_type);
      add_type(symbols, glsl_type::usamplerBuffer_type);

      add_type(symbols, glsl_type::imageBuffer_type);
      add_type(symbols, glsl_type::iimageBuffer_type);
      add_type(symbols, glsl_type::uimageBuffer_type);
   }

   if (state->has_atomic_counters()) {
      add_type(symbols, glsl_type::atomic_uint_type);
   }

   if (state->ARB_gpu_shader_fp64_enable) {
      add_type(symbols, glsl_type::double_type);
      add_type(symbols, glsl_type::dvec2_type);
      add_type(symbols, glsl_type::dvec3_type);
      add_type(symbols, glsl_type::dvec4_type);
      add_type(symbols, glsl_type::dmat2_type);
      add_type(symbols, glsl_type::dmat3_type);
      add_type(symbols, glsl_type::dmat4_type);
      add_type(symbols, glsl_type::dmat2x3_type);
      add_type(symbols, glsl_type::dmat2x4_type);
      add_type(symbols, glsl_type::dmat3x2_type);
      add_type(symbols, glsl_type::dmat3x4_type);
      add_type(symbols, glsl_type::dmat4x2_type);
      add_type(symbols, glsl_type::dmat4x3_type);
   }

   if (state->ARB_gpu_shader_int64_enable ||
       state->AMD_gpu_shader_int64_enable) {
      add_type(symbols, glsl_type::int64_t_type);
      add_type(symbols, glsl_type::i64vec2_type);
      add_type(symbols, glsl_type::i64vec3_type);
      add_type(symbols, glsl_type::i64vec4_type);

      add_type(symbols, glsl_type::uint64_t_type);
      add_type(symbols, glsl_type::u64vec2_type);
      add_type(symbols, glsl_type::u64vec3_type);
      add_type(symbols, glsl_type::u64vec4_type);
   }
}
/** @} */
