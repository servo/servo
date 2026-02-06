/*
 * Copyright © 2011 Intel Corporation
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

/* This file declares stripped-down versions of functions that
 * normally exist outside of the glsl folder, so that they can be used
 * when running the GLSL compiler standalone (for unit testing or
 * compiling builtins).
 */

#include "standalone_scaffolding.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "util/ralloc.h"
#include "util/strtod.h"
#include "main/mtypes.h"

void
_mesa_warning(struct gl_context *ctx, const char *fmt, ...)
{
    va_list vargs;
    (void) ctx;

    va_start(vargs, fmt);

    /* This output is not thread-safe, but that's good enough for the
     * standalone compiler.
     */
    fprintf(stderr, "Mesa warning: ");
    vfprintf(stderr, fmt, vargs);
    fprintf(stderr, "\n");

    va_end(vargs);
}

void
_mesa_problem(struct gl_context *ctx, const char *fmt, ...)
{
    va_list vargs;
    (void) ctx;

    va_start(vargs, fmt);

    /* This output is not thread-safe, but that's good enough for the
     * standalone compiler.
     */
    fprintf(stderr, "Mesa problem: ");
    vfprintf(stderr, fmt, vargs);
    fprintf(stderr, "\n");

    va_end(vargs);
}

void
_mesa_reference_shader_program_data(struct gl_context *ctx,
                                    struct gl_shader_program_data **ptr,
                                    struct gl_shader_program_data *data)
{
   (void) ctx;
   *ptr = data;
}

void
_mesa_reference_shader(struct gl_context *ctx, struct gl_shader **ptr,
                       struct gl_shader *sh)
{
   (void) ctx;
   *ptr = sh;
}

void
_mesa_reference_program_(struct gl_context *ctx, struct gl_program **ptr,
                         struct gl_program *prog)
{
   (void) ctx;
   *ptr = prog;
}

void
_mesa_shader_debug(struct gl_context *, GLenum, GLuint *,
                   const char *)
{
}

struct gl_shader *
_mesa_new_shader(GLuint name, gl_shader_stage stage)
{
   struct gl_shader *shader;

   assert(stage == MESA_SHADER_FRAGMENT || stage == MESA_SHADER_VERTEX);
   shader = rzalloc(NULL, struct gl_shader);
   if (shader) {
      shader->Stage = stage;
      shader->Name = name;
      shader->RefCount = 1;
   }
   return shader;
}

GLbitfield
_mesa_program_state_flags(UNUSED const gl_state_index16 state[STATE_LENGTH])
{
   return 0;
}

char *
_mesa_program_state_string(UNUSED const gl_state_index16 state[STATE_LENGTH])
{
   return NULL;
}

void
_mesa_delete_shader(struct gl_context *, struct gl_shader *sh)
{
   free((void *)sh->Source);
   free(sh->Label);
   ralloc_free(sh);
}

void
_mesa_delete_linked_shader(struct gl_context *,
                           struct gl_linked_shader *sh)
{
   ralloc_free(sh);
}

void
_mesa_clear_shader_program_data(struct gl_context *ctx,
                                struct gl_shader_program *shProg)
{
   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      if (shProg->_LinkedShaders[i] != NULL) {
         _mesa_delete_linked_shader(ctx, shProg->_LinkedShaders[i]);
         shProg->_LinkedShaders[i] = NULL;
      }
   }

   shProg->data->NumUniformStorage = 0;
   shProg->data->UniformStorage = NULL;
   shProg->NumUniformRemapTable = 0;
   shProg->UniformRemapTable = NULL;
   shProg->UniformHash = NULL;

   ralloc_free(shProg->data->InfoLog);
   shProg->data->InfoLog = ralloc_strdup(shProg->data, "");

   ralloc_free(shProg->data->UniformBlocks);
   shProg->data->UniformBlocks = NULL;
   shProg->data->NumUniformBlocks = 0;

   ralloc_free(shProg->data->ShaderStorageBlocks);
   shProg->data->ShaderStorageBlocks = NULL;
   shProg->data->NumShaderStorageBlocks = 0;

   ralloc_free(shProg->data->AtomicBuffers);
   shProg->data->AtomicBuffers = NULL;
   shProg->data->NumAtomicBuffers = 0;
}

void initialize_context_to_defaults(struct gl_context *ctx, gl_api api)
{
   memset(ctx, 0, sizeof(*ctx));

   ctx->API = api;

   ctx->Extensions.dummy_false = false;
   ctx->Extensions.dummy_true = true;
   ctx->Extensions.ARB_blend_func_extended = true;
   ctx->Extensions.ARB_compute_shader = true;
   ctx->Extensions.ARB_compute_variable_group_size = true;
   ctx->Extensions.ARB_conservative_depth = true;
   ctx->Extensions.ARB_draw_instanced = true;
   ctx->Extensions.ARB_ES2_compatibility = true;
   ctx->Extensions.ARB_ES3_compatibility = true;
   ctx->Extensions.ARB_explicit_attrib_location = true;
   ctx->Extensions.ARB_fragment_coord_conventions = true;
   ctx->Extensions.ARB_fragment_layer_viewport = true;
   ctx->Extensions.ARB_gpu_shader5 = true;
   ctx->Extensions.ARB_gpu_shader_fp64 = true;
   ctx->Extensions.ARB_gpu_shader_int64 = true;
   ctx->Extensions.ARB_sample_shading = true;
   ctx->Extensions.ARB_shader_bit_encoding = true;
   ctx->Extensions.ARB_shader_draw_parameters = true;
   ctx->Extensions.ARB_shader_stencil_export = true;
   ctx->Extensions.ARB_shader_storage_buffer_object = true;
   ctx->Extensions.ARB_shader_texture_lod = true;
   ctx->Extensions.ARB_shading_language_420pack = true;
   ctx->Extensions.ARB_shading_language_packing = true;
   ctx->Extensions.ARB_tessellation_shader = true;
   ctx->Extensions.ARB_texture_cube_map_array = true;
   ctx->Extensions.ARB_texture_gather = true;
   ctx->Extensions.ARB_texture_multisample = true;
   ctx->Extensions.ARB_texture_query_levels = true;
   ctx->Extensions.ARB_texture_query_lod = true;
   ctx->Extensions.ARB_uniform_buffer_object = true;
   ctx->Extensions.ARB_viewport_array = true;
   ctx->Extensions.ARB_cull_distance = true;
   ctx->Extensions.ARB_bindless_texture = true;

   ctx->Extensions.KHR_blend_equation_advanced = true;

   ctx->Extensions.OES_EGL_image_external = true;
   ctx->Extensions.OES_standard_derivatives = true;

   ctx->Extensions.EXT_gpu_shader4 = true;
   ctx->Extensions.EXT_shader_integer_mix = true;
   ctx->Extensions.EXT_texture_array = true;

   ctx->Extensions.MESA_shader_integer_functions = true;

   ctx->Extensions.NV_texture_rectangle = true;

   ctx->Const.GLSLVersion = 120;

   /* 1.20 minimums. */
   ctx->Const.MaxLights = 8;
   ctx->Const.MaxClipPlanes = 6;
   ctx->Const.MaxTextureUnits = 2;
   ctx->Const.MaxTextureCoordUnits = 2;
   ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 16;

   ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 512;
   ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 32;
   ctx->Const.MaxVarying = 8; /* == gl_MaxVaryingFloats / 4 */
   ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 0;
   ctx->Const.MaxCombinedTextureImageUnits = 2;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = 2;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 64;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents = 32;

   ctx->Const.MaxDrawBuffers = 1;
   ctx->Const.MaxComputeWorkGroupCount[0] = 65535;
   ctx->Const.MaxComputeWorkGroupCount[1] = 65535;
   ctx->Const.MaxComputeWorkGroupCount[2] = 65535;
   ctx->Const.MaxComputeWorkGroupSize[0] = 1024;
   ctx->Const.MaxComputeWorkGroupSize[1] = 1024;
   ctx->Const.MaxComputeWorkGroupSize[2] = 64;
   ctx->Const.MaxComputeWorkGroupInvocations = 1024;
   ctx->Const.MaxComputeVariableGroupSize[0] = 512;
   ctx->Const.MaxComputeVariableGroupSize[1] = 512;
   ctx->Const.MaxComputeVariableGroupSize[2] = 64;
   ctx->Const.MaxComputeVariableGroupInvocations = 512;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxTextureImageUnits = 16;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxUniformComponents = 1024;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxInputComponents = 0; /* not used */
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxOutputComponents = 0; /* not used */

   ctx->Const.MaxVertexStreams = 4;
   ctx->Const.MaxTransformFeedbackBuffers = 4;
   ctx->Const.MaxShaderStorageBufferBindings = 4;
   ctx->Const.MaxShaderStorageBlockSize = 4096;
   ctx->Const.Program[MESA_SHADER_VERTEX].MaxShaderStorageBlocks = 8;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxShaderStorageBlocks = 8;

   ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformBlocks = 12;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformBlocks = 12;

   /* Set up default shader compiler options. */
   struct gl_shader_compiler_options options;
   memset(&options, 0, sizeof(options));
   options.MaxUnrollIterations = 32;
   options.MaxIfDepth = UINT_MAX;

   for (int sh = 0; sh < MESA_SHADER_STAGES; ++sh)
      memcpy(&ctx->Const.ShaderCompilerOptions[sh], &options, sizeof(options));

   _mesa_locale_init();
}
