/*
 * Copyright © 2008, 2009 Intel Corporation
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
#include <inttypes.h> /* for PRIx64 macro */
#include <stdio.h>
#include <stdarg.h>
#include <string.h>
#include <assert.h>

#include "main/context.h"
#include "main/debug_output.h"
#include "main/formats.h"
#include "main/shaderobj.h"
#include "util/u_atomic.h" /* for p_atomic_cmpxchg */
#include "util/ralloc.h"
#include "util/disk_cache.h"
#include "util/mesa-sha1.h"
#include "ast.h"
#include "glsl_parser_extras.h"
#include "glsl_parser.h"
#include "ir_optimization.h"
#include "loop_analysis.h"
#include "builtin_functions.h"

/**
 * Format a short human-readable description of the given GLSL version.
 */
const char *
glsl_compute_version_string(void *mem_ctx, bool is_es, unsigned version)
{
   return ralloc_asprintf(mem_ctx, "GLSL%s %d.%02d", is_es ? " ES" : "",
                          version / 100, version % 100);
}


static const unsigned known_desktop_glsl_versions[] =
   { 110, 120, 130, 140, 150, 330, 400, 410, 420, 430, 440, 450, 460 };
static const unsigned known_desktop_gl_versions[] =
   {  20,  21,  30,  31,  32,  33,  40,  41,  42,  43,  44,  45, 46 };


_mesa_glsl_parse_state::_mesa_glsl_parse_state(struct gl_context *_ctx,
					       gl_shader_stage stage,
                                               void *mem_ctx)
   : ctx(_ctx), cs_input_local_size_specified(false), cs_input_local_size(),
     switch_state(), warnings_enabled(true)
{
   assert(stage < MESA_SHADER_STAGES);
   this->stage = stage;

   this->scanner = NULL;
   this->translation_unit.make_empty();
   this->symbols = new(mem_ctx) glsl_symbol_table;

   this->linalloc = linear_alloc_parent(this, 0);

   this->info_log = ralloc_strdup(mem_ctx, "");
   this->error = false;
   this->loop_nesting_ast = NULL;

   this->uses_builtin_functions = false;

   /* Set default language version and extensions */
   this->language_version = 110;
   this->forced_language_version = ctx->Const.ForceGLSLVersion;
   this->zero_init = ctx->Const.GLSLZeroInit;
   this->gl_version = 20;
   this->compat_shader = true;
   this->es_shader = false;
   this->had_version_string = false;
   this->ARB_texture_rectangle_enable = true;

   /* OpenGL ES 2.0 has different defaults from desktop GL. */
   if (ctx->API == API_OPENGLES2) {
      this->language_version = 100;
      this->es_shader = true;
      this->ARB_texture_rectangle_enable = false;
   }

   this->extensions = &ctx->Extensions;

   this->Const.MaxLights = ctx->Const.MaxLights;
   this->Const.MaxClipPlanes = ctx->Const.MaxClipPlanes;
   this->Const.MaxTextureUnits = ctx->Const.MaxTextureUnits;
   this->Const.MaxTextureCoords = ctx->Const.MaxTextureCoordUnits;
   this->Const.MaxVertexAttribs = ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs;
   this->Const.MaxVertexUniformComponents = ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents;
   this->Const.MaxVertexTextureImageUnits = ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits;
   this->Const.MaxCombinedTextureImageUnits = ctx->Const.MaxCombinedTextureImageUnits;
   this->Const.MaxTextureImageUnits = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits;
   this->Const.MaxFragmentUniformComponents = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents;
   this->Const.MinProgramTexelOffset = ctx->Const.MinProgramTexelOffset;
   this->Const.MaxProgramTexelOffset = ctx->Const.MaxProgramTexelOffset;

   this->Const.MaxDrawBuffers = ctx->Const.MaxDrawBuffers;

   this->Const.MaxDualSourceDrawBuffers = ctx->Const.MaxDualSourceDrawBuffers;

   /* 1.50 constants */
   this->Const.MaxVertexOutputComponents = ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents;
   this->Const.MaxGeometryInputComponents = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxInputComponents;
   this->Const.MaxGeometryOutputComponents = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxOutputComponents;
   this->Const.MaxGeometryShaderInvocations = ctx->Const.MaxGeometryShaderInvocations;
   this->Const.MaxFragmentInputComponents = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents;
   this->Const.MaxGeometryTextureImageUnits = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxTextureImageUnits;
   this->Const.MaxGeometryOutputVertices = ctx->Const.MaxGeometryOutputVertices;
   this->Const.MaxGeometryTotalOutputComponents = ctx->Const.MaxGeometryTotalOutputComponents;
   this->Const.MaxGeometryUniformComponents = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxUniformComponents;

   this->Const.MaxVertexAtomicCounters = ctx->Const.Program[MESA_SHADER_VERTEX].MaxAtomicCounters;
   this->Const.MaxTessControlAtomicCounters = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxAtomicCounters;
   this->Const.MaxTessEvaluationAtomicCounters = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxAtomicCounters;
   this->Const.MaxGeometryAtomicCounters = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxAtomicCounters;
   this->Const.MaxFragmentAtomicCounters = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxAtomicCounters;
   this->Const.MaxComputeAtomicCounters = ctx->Const.Program[MESA_SHADER_COMPUTE].MaxAtomicCounters;
   this->Const.MaxCombinedAtomicCounters = ctx->Const.MaxCombinedAtomicCounters;
   this->Const.MaxAtomicBufferBindings = ctx->Const.MaxAtomicBufferBindings;
   this->Const.MaxVertexAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAtomicBuffers;
   this->Const.MaxTessControlAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxAtomicBuffers;
   this->Const.MaxTessEvaluationAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxAtomicBuffers;
   this->Const.MaxGeometryAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxAtomicBuffers;
   this->Const.MaxFragmentAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxAtomicBuffers;
   this->Const.MaxComputeAtomicCounterBuffers =
      ctx->Const.Program[MESA_SHADER_COMPUTE].MaxAtomicBuffers;
   this->Const.MaxCombinedAtomicCounterBuffers =
      ctx->Const.MaxCombinedAtomicBuffers;
   this->Const.MaxAtomicCounterBufferSize =
      ctx->Const.MaxAtomicBufferSize;

   /* ARB_enhanced_layouts constants */
   this->Const.MaxTransformFeedbackBuffers = ctx->Const.MaxTransformFeedbackBuffers;
   this->Const.MaxTransformFeedbackInterleavedComponents = ctx->Const.MaxTransformFeedbackInterleavedComponents;

   /* Compute shader constants */
   for (unsigned i = 0; i < ARRAY_SIZE(this->Const.MaxComputeWorkGroupCount); i++)
      this->Const.MaxComputeWorkGroupCount[i] = ctx->Const.MaxComputeWorkGroupCount[i];
   for (unsigned i = 0; i < ARRAY_SIZE(this->Const.MaxComputeWorkGroupSize); i++)
      this->Const.MaxComputeWorkGroupSize[i] = ctx->Const.MaxComputeWorkGroupSize[i];

   this->Const.MaxComputeTextureImageUnits = ctx->Const.Program[MESA_SHADER_COMPUTE].MaxTextureImageUnits;
   this->Const.MaxComputeUniformComponents = ctx->Const.Program[MESA_SHADER_COMPUTE].MaxUniformComponents;

   this->Const.MaxImageUnits = ctx->Const.MaxImageUnits;
   this->Const.MaxCombinedShaderOutputResources = ctx->Const.MaxCombinedShaderOutputResources;
   this->Const.MaxImageSamples = ctx->Const.MaxImageSamples;
   this->Const.MaxVertexImageUniforms = ctx->Const.Program[MESA_SHADER_VERTEX].MaxImageUniforms;
   this->Const.MaxTessControlImageUniforms = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxImageUniforms;
   this->Const.MaxTessEvaluationImageUniforms = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxImageUniforms;
   this->Const.MaxGeometryImageUniforms = ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxImageUniforms;
   this->Const.MaxFragmentImageUniforms = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxImageUniforms;
   this->Const.MaxComputeImageUniforms = ctx->Const.Program[MESA_SHADER_COMPUTE].MaxImageUniforms;
   this->Const.MaxCombinedImageUniforms = ctx->Const.MaxCombinedImageUniforms;

   /* ARB_viewport_array */
   this->Const.MaxViewports = ctx->Const.MaxViewports;

   /* tessellation shader constants */
   this->Const.MaxPatchVertices = ctx->Const.MaxPatchVertices;
   this->Const.MaxTessGenLevel = ctx->Const.MaxTessGenLevel;
   this->Const.MaxTessControlInputComponents = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxInputComponents;
   this->Const.MaxTessControlOutputComponents = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxOutputComponents;
   this->Const.MaxTessControlTextureImageUnits = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxTextureImageUnits;
   this->Const.MaxTessEvaluationInputComponents = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxInputComponents;
   this->Const.MaxTessEvaluationOutputComponents = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxOutputComponents;
   this->Const.MaxTessEvaluationTextureImageUnits = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxTextureImageUnits;
   this->Const.MaxTessPatchComponents = ctx->Const.MaxTessPatchComponents;
   this->Const.MaxTessControlTotalOutputComponents = ctx->Const.MaxTessControlTotalOutputComponents;
   this->Const.MaxTessControlUniformComponents = ctx->Const.Program[MESA_SHADER_TESS_CTRL].MaxUniformComponents;
   this->Const.MaxTessEvaluationUniformComponents = ctx->Const.Program[MESA_SHADER_TESS_EVAL].MaxUniformComponents;

   /* GL 4.5 / OES_sample_variables */
   this->Const.MaxSamples = ctx->Const.MaxSamples;

   this->current_function = NULL;
   this->toplevel_ir = NULL;
   this->found_return = false;
   this->found_begin_interlock = false;
   this->found_end_interlock = false;
   this->all_invariant = false;
   this->user_structures = NULL;
   this->num_user_structures = 0;
   this->num_subroutines = 0;
   this->subroutines = NULL;
   this->num_subroutine_types = 0;
   this->subroutine_types = NULL;

   /* supported_versions should be large enough to support the known desktop
    * GLSL versions plus 4 GLES versions (ES 1.00, ES 3.00, ES 3.10, ES 3.20)
    */
   STATIC_ASSERT((ARRAY_SIZE(known_desktop_glsl_versions) + 4) ==
                 ARRAY_SIZE(this->supported_versions));

   /* Populate the list of supported GLSL versions */
   /* FINISHME: Once the OpenGL 3.0 'forward compatible' context or
    * the OpenGL 3.2 Core context is supported, this logic will need
    * change.  Older versions of GLSL are no longer supported
    * outside the compatibility contexts of 3.x.
    */
   this->num_supported_versions = 0;
   if (_mesa_is_desktop_gl(ctx)) {
      for (unsigned i = 0; i < ARRAY_SIZE(known_desktop_glsl_versions); i++) {
         if (known_desktop_glsl_versions[i] <= ctx->Const.GLSLVersion) {
            this->supported_versions[this->num_supported_versions].ver
               = known_desktop_glsl_versions[i];
            this->supported_versions[this->num_supported_versions].gl_ver
               = known_desktop_gl_versions[i];
            this->supported_versions[this->num_supported_versions].es = false;
            this->num_supported_versions++;
         }
      }
   }
   if (ctx->API == API_OPENGLES2 || ctx->Extensions.ARB_ES2_compatibility) {
      this->supported_versions[this->num_supported_versions].ver = 100;
      this->supported_versions[this->num_supported_versions].gl_ver = 20;
      this->supported_versions[this->num_supported_versions].es = true;
      this->num_supported_versions++;
   }
   if (_mesa_is_gles3(ctx) || ctx->Extensions.ARB_ES3_compatibility) {
      this->supported_versions[this->num_supported_versions].ver = 300;
      this->supported_versions[this->num_supported_versions].gl_ver = 30;
      this->supported_versions[this->num_supported_versions].es = true;
      this->num_supported_versions++;
   }
   if (_mesa_is_gles31(ctx) || ctx->Extensions.ARB_ES3_1_compatibility) {
      this->supported_versions[this->num_supported_versions].ver = 310;
      this->supported_versions[this->num_supported_versions].gl_ver = 31;
      this->supported_versions[this->num_supported_versions].es = true;
      this->num_supported_versions++;
   }
   if ((ctx->API == API_OPENGLES2 && ctx->Version >= 32) ||
       ctx->Extensions.ARB_ES3_2_compatibility) {
      this->supported_versions[this->num_supported_versions].ver = 320;
      this->supported_versions[this->num_supported_versions].gl_ver = 32;
      this->supported_versions[this->num_supported_versions].es = true;
      this->num_supported_versions++;
   }

   /* Create a string for use in error messages to tell the user which GLSL
    * versions are supported.
    */
   char *supported = ralloc_strdup(this, "");
   for (unsigned i = 0; i < this->num_supported_versions; i++) {
      unsigned ver = this->supported_versions[i].ver;
      const char *const prefix = (i == 0)
	 ? ""
	 : ((i == this->num_supported_versions - 1) ? ", and " : ", ");
      const char *const suffix = (this->supported_versions[i].es) ? " ES" : "";

      ralloc_asprintf_append(& supported, "%s%u.%02u%s",
			     prefix,
			     ver / 100, ver % 100,
			     suffix);
   }

   this->supported_version_string = supported;

   if (ctx->Const.ForceGLSLExtensionsWarn)
      _mesa_glsl_process_extension("all", NULL, "warn", NULL, this);

   this->default_uniform_qualifier = new(this) ast_type_qualifier();
   this->default_uniform_qualifier->flags.q.shared = 1;
   this->default_uniform_qualifier->flags.q.column_major = 1;

   this->default_shader_storage_qualifier = new(this) ast_type_qualifier();
   this->default_shader_storage_qualifier->flags.q.shared = 1;
   this->default_shader_storage_qualifier->flags.q.column_major = 1;

   this->fs_uses_gl_fragcoord = false;
   this->fs_redeclares_gl_fragcoord = false;
   this->fs_origin_upper_left = false;
   this->fs_pixel_center_integer = false;
   this->fs_redeclares_gl_fragcoord_with_no_layout_qualifiers = false;

   this->gs_input_prim_type_specified = false;
   this->tcs_output_vertices_specified = false;
   this->gs_input_size = 0;
   this->in_qualifier = new(this) ast_type_qualifier();
   this->out_qualifier = new(this) ast_type_qualifier();
   this->fs_early_fragment_tests = false;
   this->fs_inner_coverage = false;
   this->fs_post_depth_coverage = false;
   this->fs_pixel_interlock_ordered = false;
   this->fs_pixel_interlock_unordered = false;
   this->fs_sample_interlock_ordered = false;
   this->fs_sample_interlock_unordered = false;
   this->fs_blend_support = 0;
   memset(this->atomic_counter_offsets, 0,
          sizeof(this->atomic_counter_offsets));
   this->allow_extension_directive_midshader =
      ctx->Const.AllowGLSLExtensionDirectiveMidShader;
   this->allow_builtin_variable_redeclaration =
      ctx->Const.AllowGLSLBuiltinVariableRedeclaration;
   this->allow_layout_qualifier_on_function_parameter =
      ctx->Const.AllowLayoutQualifiersOnFunctionParameters;

   this->cs_input_local_size_variable_specified = false;

   /* ARB_bindless_texture */
   this->bindless_sampler_specified = false;
   this->bindless_image_specified = false;
   this->bound_sampler_specified = false;
   this->bound_image_specified = false;
}

/**
 * Determine whether the current GLSL version is sufficiently high to support
 * a certain feature, and generate an error message if it isn't.
 *
 * \param required_glsl_version and \c required_glsl_es_version are
 * interpreted as they are in _mesa_glsl_parse_state::is_version().
 *
 * \param locp is the parser location where the error should be reported.
 *
 * \param fmt (and additional arguments) constitute a printf-style error
 * message to report if the version check fails.  Information about the
 * current and required GLSL versions will be appended.  So, for example, if
 * the GLSL version being compiled is 1.20, and check_version(130, 300, locp,
 * "foo unsupported") is called, the error message will be "foo unsupported in
 * GLSL 1.20 (GLSL 1.30 or GLSL 3.00 ES required)".
 */
bool
_mesa_glsl_parse_state::check_version(unsigned required_glsl_version,
                                      unsigned required_glsl_es_version,
                                      YYLTYPE *locp, const char *fmt, ...)
{
   if (this->is_version(required_glsl_version, required_glsl_es_version))
      return true;

   va_list args;
   va_start(args, fmt);
   char *problem = ralloc_vasprintf(this, fmt, args);
   va_end(args);
   const char *glsl_version_string
      = glsl_compute_version_string(this, false, required_glsl_version);
   const char *glsl_es_version_string
      = glsl_compute_version_string(this, true, required_glsl_es_version);
   const char *requirement_string = "";
   if (required_glsl_version && required_glsl_es_version) {
      requirement_string = ralloc_asprintf(this, " (%s or %s required)",
                                           glsl_version_string,
                                           glsl_es_version_string);
   } else if (required_glsl_version) {
      requirement_string = ralloc_asprintf(this, " (%s required)",
                                           glsl_version_string);
   } else if (required_glsl_es_version) {
      requirement_string = ralloc_asprintf(this, " (%s required)",
                                           glsl_es_version_string);
   }
   _mesa_glsl_error(locp, this, "%s in %s%s",
                    problem, this->get_version_string(),
                    requirement_string);

   return false;
}

/**
 * Process a GLSL #version directive.
 *
 * \param version is the integer that follows the #version token.
 *
 * \param ident is a string identifier that follows the integer, if any is
 * present.  Otherwise NULL.
 */
void
_mesa_glsl_parse_state::process_version_directive(YYLTYPE *locp, int version,
                                                  const char *ident)
{
   bool es_token_present = false;
   bool compat_token_present = false;
   if (ident) {
      if (strcmp(ident, "es") == 0) {
         es_token_present = true;
      } else if (version >= 150) {
         if (strcmp(ident, "core") == 0) {
            /* Accept the token.  There's no need to record that this is
             * a core profile shader since that's the only profile we support.
             */
         } else if (strcmp(ident, "compatibility") == 0) {
            compat_token_present = true;

            if (this->ctx->API != API_OPENGL_COMPAT) {
               _mesa_glsl_error(locp, this,
                                "the compatibility profile is not supported");
            }
         } else {
            _mesa_glsl_error(locp, this,
                             "\"%s\" is not a valid shading language profile; "
                             "if present, it must be \"core\"", ident);
         }
      } else {
         _mesa_glsl_error(locp, this,
                          "illegal text following version number");
      }
   }

   this->es_shader = es_token_present;
   if (version == 100) {
      if (es_token_present) {
         _mesa_glsl_error(locp, this,
                          "GLSL 1.00 ES should be selected using "
                          "`#version 100'");
      } else {
         this->es_shader = true;
      }
   }

   if (this->es_shader) {
      this->ARB_texture_rectangle_enable = false;
   }

   if (this->forced_language_version)
      this->language_version = this->forced_language_version;
   else
      this->language_version = version;
   this->had_version_string = true;

   this->compat_shader = compat_token_present ||
                         (this->ctx->API == API_OPENGL_COMPAT &&
                          this->language_version == 140) ||
                         (!this->es_shader && this->language_version < 140);

   bool supported = false;
   for (unsigned i = 0; i < this->num_supported_versions; i++) {
      if (this->supported_versions[i].ver == this->language_version
          && this->supported_versions[i].es == this->es_shader) {
         this->gl_version = this->supported_versions[i].gl_ver;
         supported = true;
         break;
      }
   }

   if (!supported) {
      _mesa_glsl_error(locp, this, "%s is not supported. "
                       "Supported versions are: %s",
                       this->get_version_string(),
                       this->supported_version_string);

      /* On exit, the language_version must be set to a valid value.
       * Later calls to _mesa_glsl_initialize_types will misbehave if
       * the version is invalid.
       */
      switch (this->ctx->API) {
      case API_OPENGL_COMPAT:
      case API_OPENGL_CORE:
	 this->language_version = this->ctx->Const.GLSLVersion;
	 break;

      case API_OPENGLES:
	 assert(!"Should not get here.");
	 /* FALLTHROUGH */

      case API_OPENGLES2:
	 this->language_version = 100;
	 break;
      }
   }
}


/* This helper function will append the given message to the shader's
   info log and report it via GL_ARB_debug_output. Per that extension,
   'type' is one of the enum values classifying the message, and
   'id' is the implementation-defined ID of the given message. */
static void
_mesa_glsl_msg(const YYLTYPE *locp, _mesa_glsl_parse_state *state,
               GLenum type, const char *fmt, va_list ap)
{
   bool error = (type == MESA_DEBUG_TYPE_ERROR);
   GLuint msg_id = 0;

   assert(state->info_log != NULL);

   /* Get the offset that the new message will be written to. */
   int msg_offset = strlen(state->info_log);

   if (locp->path) {
      ralloc_asprintf_append(&state->info_log, "\"%s\"", locp->path);
   } else {
      ralloc_asprintf_append(&state->info_log, "%u", locp->source);
   }
   ralloc_asprintf_append(&state->info_log, ":%u(%u): %s: ",
                          locp->first_line, locp->first_column,
                          error ? "error" : "warning");

   ralloc_vasprintf_append(&state->info_log, fmt, ap);

   const char *const msg = &state->info_log[msg_offset];
   struct gl_context *ctx = state->ctx;

   /* Report the error via GL_ARB_debug_output. */
   _mesa_shader_debug(ctx, type, &msg_id, msg);

   ralloc_strcat(&state->info_log, "\n");
}

void
_mesa_glsl_error(YYLTYPE *locp, _mesa_glsl_parse_state *state,
		 const char *fmt, ...)
{
   va_list ap;

   state->error = true;

   va_start(ap, fmt);
   _mesa_glsl_msg(locp, state, MESA_DEBUG_TYPE_ERROR, fmt, ap);
   va_end(ap);
}


void
_mesa_glsl_warning(const YYLTYPE *locp, _mesa_glsl_parse_state *state,
		   const char *fmt, ...)
{
   if (state->warnings_enabled) {
      va_list ap;

      va_start(ap, fmt);
      _mesa_glsl_msg(locp, state, MESA_DEBUG_TYPE_OTHER, fmt, ap);
      va_end(ap);
   }
}


/**
 * Enum representing the possible behaviors that can be specified in
 * an #extension directive.
 */
enum ext_behavior {
   extension_disable,
   extension_enable,
   extension_require,
   extension_warn
};

/**
 * Element type for _mesa_glsl_supported_extensions
 */
struct _mesa_glsl_extension {
   /**
    * Name of the extension when referred to in a GLSL extension
    * statement
    */
   const char *name;

   /**
    * Whether this extension is a part of AEP
    */
   bool aep;

   /**
    * Predicate that checks whether the relevant extension is available for
    * this context.
    */
   bool (*available_pred)(const struct gl_context *,
                          gl_api api, uint8_t version);

   /**
    * Flag in the _mesa_glsl_parse_state struct that should be set
    * when this extension is enabled.
    *
    * See note in _mesa_glsl_extension::supported_flag about "pointer
    * to member" types.
    */
   bool _mesa_glsl_parse_state::* enable_flag;

   /**
    * Flag in the _mesa_glsl_parse_state struct that should be set
    * when the shader requests "warn" behavior for this extension.
    *
    * See note in _mesa_glsl_extension::supported_flag about "pointer
    * to member" types.
    */
   bool _mesa_glsl_parse_state::* warn_flag;


   bool compatible_with_state(const _mesa_glsl_parse_state *state,
                              gl_api api, uint8_t gl_version) const;
   void set_flags(_mesa_glsl_parse_state *state, ext_behavior behavior) const;
};

/** Checks if the context supports a user-facing extension */
#define EXT(name_str, driver_cap, ...) \
static UNUSED bool \
has_##name_str(const struct gl_context *ctx, gl_api api, uint8_t version) \
{ \
   return ctx->Extensions.driver_cap && (version >= \
          _mesa_extension_table[MESA_EXTENSION_##name_str].version[api]); \
}
#include "main/extensions_table.h"
#undef EXT

#define EXT(NAME)                                           \
   { "GL_" #NAME, false, has_##NAME,                        \
     &_mesa_glsl_parse_state::NAME##_enable,                \
     &_mesa_glsl_parse_state::NAME##_warn }

#define EXT_AEP(NAME)                                       \
   { "GL_" #NAME, true, has_##NAME,                         \
     &_mesa_glsl_parse_state::NAME##_enable,                \
     &_mesa_glsl_parse_state::NAME##_warn }

/**
 * Table of extensions that can be enabled/disabled within a shader,
 * and the conditions under which they are supported.
 */
static const _mesa_glsl_extension _mesa_glsl_supported_extensions[] = {
   /* ARB extensions go here, sorted alphabetically.
    */
   EXT(ARB_ES3_1_compatibility),
   EXT(ARB_ES3_2_compatibility),
   EXT(ARB_arrays_of_arrays),
   EXT(ARB_bindless_texture),
   EXT(ARB_compatibility),
   EXT(ARB_compute_shader),
   EXT(ARB_compute_variable_group_size),
   EXT(ARB_conservative_depth),
   EXT(ARB_cull_distance),
   EXT(ARB_derivative_control),
   EXT(ARB_draw_buffers),
   EXT(ARB_draw_instanced),
   EXT(ARB_enhanced_layouts),
   EXT(ARB_explicit_attrib_location),
   EXT(ARB_explicit_uniform_location),
   EXT(ARB_fragment_coord_conventions),
   EXT(ARB_fragment_layer_viewport),
   EXT(ARB_fragment_shader_interlock),
   EXT(ARB_gpu_shader5),
   EXT(ARB_gpu_shader_fp64),
   EXT(ARB_gpu_shader_int64),
   EXT(ARB_post_depth_coverage),
   EXT(ARB_sample_shading),
   EXT(ARB_separate_shader_objects),
   EXT(ARB_shader_atomic_counter_ops),
   EXT(ARB_shader_atomic_counters),
   EXT(ARB_shader_ballot),
   EXT(ARB_shader_bit_encoding),
   EXT(ARB_shader_clock),
   EXT(ARB_shader_draw_parameters),
   EXT(ARB_shader_group_vote),
   EXT(ARB_shader_image_load_store),
   EXT(ARB_shader_image_size),
   EXT(ARB_shader_precision),
   EXT(ARB_shader_stencil_export),
   EXT(ARB_shader_storage_buffer_object),
   EXT(ARB_shader_subroutine),
   EXT(ARB_shader_texture_image_samples),
   EXT(ARB_shader_texture_lod),
   EXT(ARB_shader_viewport_layer_array),
   EXT(ARB_shading_language_420pack),
   EXT(ARB_shading_language_include),
   EXT(ARB_shading_language_packing),
   EXT(ARB_tessellation_shader),
   EXT(ARB_texture_cube_map_array),
   EXT(ARB_texture_gather),
   EXT(ARB_texture_multisample),
   EXT(ARB_texture_query_levels),
   EXT(ARB_texture_query_lod),
   EXT(ARB_texture_rectangle),
   EXT(ARB_uniform_buffer_object),
   EXT(ARB_vertex_attrib_64bit),
   EXT(ARB_viewport_array),

   /* KHR extensions go here, sorted alphabetically.
    */
   EXT_AEP(KHR_blend_equation_advanced),

   /* OES extensions go here, sorted alphabetically.
    */
   EXT(OES_EGL_image_external),
   EXT(OES_EGL_image_external_essl3),
   EXT(OES_geometry_point_size),
   EXT(OES_geometry_shader),
   EXT(OES_gpu_shader5),
   EXT(OES_primitive_bounding_box),
   EXT_AEP(OES_sample_variables),
   EXT_AEP(OES_shader_image_atomic),
   EXT(OES_shader_io_blocks),
   EXT_AEP(OES_shader_multisample_interpolation),
   EXT(OES_standard_derivatives),
   EXT(OES_tessellation_point_size),
   EXT(OES_tessellation_shader),
   EXT(OES_texture_3D),
   EXT(OES_texture_buffer),
   EXT(OES_texture_cube_map_array),
   EXT_AEP(OES_texture_storage_multisample_2d_array),
   EXT(OES_viewport_array),

   /* All other extensions go here, sorted alphabetically.
    */
   EXT(AMD_conservative_depth),
   EXT(AMD_gpu_shader_int64),
   EXT(AMD_shader_stencil_export),
   EXT(AMD_shader_trinary_minmax),
   EXT(AMD_texture_texture4),
   EXT(AMD_vertex_shader_layer),
   EXT(AMD_vertex_shader_viewport_index),
   EXT(ANDROID_extension_pack_es31a),
   EXT(EXT_blend_func_extended),
   EXT(EXT_demote_to_helper_invocation),
   EXT(EXT_frag_depth),
   EXT(EXT_draw_buffers),
   EXT(EXT_draw_instanced),
   EXT(EXT_clip_cull_distance),
   EXT(EXT_geometry_point_size),
   EXT_AEP(EXT_geometry_shader),
   EXT(EXT_gpu_shader4),
   EXT_AEP(EXT_gpu_shader5),
   EXT_AEP(EXT_primitive_bounding_box),
   EXT(EXT_separate_shader_objects),
   EXT(EXT_shader_framebuffer_fetch),
   EXT(EXT_shader_framebuffer_fetch_non_coherent),
   EXT(EXT_shader_image_load_formatted),
   EXT(EXT_shader_image_load_store),
   EXT(EXT_shader_implicit_conversions),
   EXT(EXT_shader_integer_mix),
   EXT_AEP(EXT_shader_io_blocks),
   EXT(EXT_shader_samples_identical),
   EXT(EXT_tessellation_point_size),
   EXT_AEP(EXT_tessellation_shader),
   EXT(EXT_texture_array),
   EXT_AEP(EXT_texture_buffer),
   EXT_AEP(EXT_texture_cube_map_array),
   EXT(EXT_texture_query_lod),
   EXT(EXT_texture_shadow_lod),
   EXT(INTEL_conservative_rasterization),
   EXT(INTEL_shader_atomic_float_minmax),
   EXT(INTEL_shader_integer_functions2),
   EXT(MESA_shader_integer_functions),
   EXT(NV_compute_shader_derivatives),
   EXT(NV_fragment_shader_interlock),
   EXT(NV_image_formats),
   EXT(NV_shader_atomic_float),
   EXT(NV_viewport_array2),
};

#undef EXT


/**
 * Determine whether a given extension is compatible with the target,
 * API, and extension information in the current parser state.
 */
bool _mesa_glsl_extension::compatible_with_state(
      const _mesa_glsl_parse_state *state, gl_api api, uint8_t gl_version) const
{
   return this->available_pred(state->ctx, api, gl_version);
}

/**
 * Set the appropriate flags in the parser state to establish the
 * given behavior for this extension.
 */
void _mesa_glsl_extension::set_flags(_mesa_glsl_parse_state *state,
                                     ext_behavior behavior) const
{
   /* Note: the ->* operator indexes into state by the
    * offsets this->enable_flag and this->warn_flag.  See
    * _mesa_glsl_extension::supported_flag for more info.
    */
   state->*(this->enable_flag) = (behavior != extension_disable);
   state->*(this->warn_flag)   = (behavior == extension_warn);
}

/**
 * Find an extension by name in _mesa_glsl_supported_extensions.  If
 * the name is not found, return NULL.
 */
static const _mesa_glsl_extension *find_extension(const char *name)
{
   for (unsigned i = 0; i < ARRAY_SIZE(_mesa_glsl_supported_extensions); ++i) {
      if (strcmp(name, _mesa_glsl_supported_extensions[i].name) == 0) {
         return &_mesa_glsl_supported_extensions[i];
      }
   }
   return NULL;
}

bool
_mesa_glsl_process_extension(const char *name, YYLTYPE *name_locp,
			     const char *behavior_string, YYLTYPE *behavior_locp,
			     _mesa_glsl_parse_state *state)
{
   uint8_t gl_version = state->ctx->Extensions.Version;
   gl_api api = state->ctx->API;
   ext_behavior behavior;
   if (strcmp(behavior_string, "warn") == 0) {
      behavior = extension_warn;
   } else if (strcmp(behavior_string, "require") == 0) {
      behavior = extension_require;
   } else if (strcmp(behavior_string, "enable") == 0) {
      behavior = extension_enable;
   } else if (strcmp(behavior_string, "disable") == 0) {
      behavior = extension_disable;
   } else {
      _mesa_glsl_error(behavior_locp, state,
		       "unknown extension behavior `%s'",
		       behavior_string);
      return false;
   }

   /* If we're in a desktop context but with an ES shader, use an ES API enum
    * to verify extension availability.
    */
   if (state->es_shader && api != API_OPENGLES2)
      api = API_OPENGLES2;
   /* Use the language-version derived GL version to extension checks, unless
    * we're using meta, which sets the version to the max.
    */
   if (gl_version != 0xff)
      gl_version = state->gl_version;

   if (strcmp(name, "all") == 0) {
      if ((behavior == extension_enable) || (behavior == extension_require)) {
	 _mesa_glsl_error(name_locp, state, "cannot %s all extensions",
			  (behavior == extension_enable)
			  ? "enable" : "require");
	 return false;
      } else {
         for (unsigned i = 0;
              i < ARRAY_SIZE(_mesa_glsl_supported_extensions); ++i) {
            const _mesa_glsl_extension *extension
               = &_mesa_glsl_supported_extensions[i];
            if (extension->compatible_with_state(state, api, gl_version)) {
               _mesa_glsl_supported_extensions[i].set_flags(state, behavior);
            }
         }
      }
   } else {
      const _mesa_glsl_extension *extension = find_extension(name);
      if (extension && extension->compatible_with_state(state, api, gl_version)) {
         extension->set_flags(state, behavior);
         if (extension->available_pred == has_ANDROID_extension_pack_es31a) {
            for (unsigned i = 0;
                 i < ARRAY_SIZE(_mesa_glsl_supported_extensions); ++i) {
               const _mesa_glsl_extension *extension =
                  &_mesa_glsl_supported_extensions[i];

               if (!extension->aep)
                  continue;
               /* AEP should not be enabled if all of the sub-extensions can't
                * also be enabled. This is not the proper layer to do such
                * error-checking though.
                */
               assert(extension->compatible_with_state(state, api, gl_version));
               extension->set_flags(state, behavior);
            }
         }
      } else {
         static const char fmt[] = "extension `%s' unsupported in %s shader";

         if (behavior == extension_require) {
            _mesa_glsl_error(name_locp, state, fmt,
                             name, _mesa_shader_stage_to_string(state->stage));
            return false;
         } else {
            _mesa_glsl_warning(name_locp, state, fmt,
                               name, _mesa_shader_stage_to_string(state->stage));
         }
      }
   }

   return true;
}


/**
 * Recurses through <type> and <expr> if <expr> is an aggregate initializer
 * and sets <expr>'s <constructor_type> field to <type>. Gives later functions
 * (process_array_constructor, et al) sufficient information to do type
 * checking.
 *
 * Operates on assignments involving an aggregate initializer. E.g.,
 *
 * vec4 pos = {1.0, -1.0, 0.0, 1.0};
 *
 * or more ridiculously,
 *
 * struct S {
 *     vec4 v[2];
 * };
 *
 * struct {
 *     S a[2], b;
 *     int c;
 * } aggregate = {
 *     {
 *         {
 *             {
 *                 {1.0, 2.0, 3.0, 4.0}, // a[0].v[0]
 *                 {5.0, 6.0, 7.0, 8.0}  // a[0].v[1]
 *             } // a[0].v
 *         }, // a[0]
 *         {
 *             {
 *                 {1.0, 2.0, 3.0, 4.0}, // a[1].v[0]
 *                 {5.0, 6.0, 7.0, 8.0}  // a[1].v[1]
 *             } // a[1].v
 *         } // a[1]
 *     }, // a
 *     {
 *         {
 *             {1.0, 2.0, 3.0, 4.0}, // b.v[0]
 *             {5.0, 6.0, 7.0, 8.0}  // b.v[1]
 *         } // b.v
 *     }, // b
 *     4 // c
 * };
 *
 * This pass is necessary because the right-hand side of <type> e = { ... }
 * doesn't contain sufficient information to determine if the types match.
 */
void
_mesa_ast_set_aggregate_type(const glsl_type *type,
                             ast_expression *expr)
{
   ast_aggregate_initializer *ai = (ast_aggregate_initializer *)expr;
   ai->constructor_type = type;

   /* If the aggregate is an array, recursively set its elements' types. */
   if (type->is_array()) {
      /* Each array element has the type type->fields.array.
       *
       * E.g., if <type> if struct S[2] we want to set each element's type to
       * struct S.
       */
      for (exec_node *expr_node = ai->expressions.get_head_raw();
           !expr_node->is_tail_sentinel();
           expr_node = expr_node->next) {
         ast_expression *expr = exec_node_data(ast_expression, expr_node,
                                               link);

         if (expr->oper == ast_aggregate)
            _mesa_ast_set_aggregate_type(type->fields.array, expr);
      }

   /* If the aggregate is a struct, recursively set its fields' types. */
   } else if (type->is_struct()) {
      exec_node *expr_node = ai->expressions.get_head_raw();

      /* Iterate through the struct's fields. */
      for (unsigned i = 0; !expr_node->is_tail_sentinel() && i < type->length;
           i++, expr_node = expr_node->next) {
         ast_expression *expr = exec_node_data(ast_expression, expr_node,
                                               link);

         if (expr->oper == ast_aggregate) {
            _mesa_ast_set_aggregate_type(type->fields.structure[i].type, expr);
         }
      }
   /* If the aggregate is a matrix, set its columns' types. */
   } else if (type->is_matrix()) {
      for (exec_node *expr_node = ai->expressions.get_head_raw();
           !expr_node->is_tail_sentinel();
           expr_node = expr_node->next) {
         ast_expression *expr = exec_node_data(ast_expression, expr_node,
                                               link);

         if (expr->oper == ast_aggregate)
            _mesa_ast_set_aggregate_type(type->column_type(), expr);
      }
   }
}

void
_mesa_ast_process_interface_block(YYLTYPE *locp,
                                  _mesa_glsl_parse_state *state,
                                  ast_interface_block *const block,
                                  const struct ast_type_qualifier &q)
{
   if (q.flags.q.buffer) {
      if (!state->has_shader_storage_buffer_objects()) {
         _mesa_glsl_error(locp, state,
                          "#version 430 / GL_ARB_shader_storage_buffer_object "
                          "required for defining shader storage blocks");
      } else if (state->ARB_shader_storage_buffer_object_warn) {
         _mesa_glsl_warning(locp, state,
                            "#version 430 / GL_ARB_shader_storage_buffer_object "
                            "required for defining shader storage blocks");
      }
   } else if (q.flags.q.uniform) {
      if (!state->has_uniform_buffer_objects()) {
         _mesa_glsl_error(locp, state,
                          "#version 140 / GL_ARB_uniform_buffer_object "
                          "required for defining uniform blocks");
      } else if (state->ARB_uniform_buffer_object_warn) {
         _mesa_glsl_warning(locp, state,
                            "#version 140 / GL_ARB_uniform_buffer_object "
                            "required for defining uniform blocks");
      }
   } else {
      if (!state->has_shader_io_blocks()) {
         if (state->es_shader) {
            _mesa_glsl_error(locp, state,
                             "GL_OES_shader_io_blocks or #version 320 "
                             "required for using interface blocks");
         } else {
            _mesa_glsl_error(locp, state,
                             "#version 150 required for using "
                             "interface blocks");
         }
      }
   }

   /* From the GLSL 1.50.11 spec, section 4.3.7 ("Interface Blocks"):
    * "It is illegal to have an input block in a vertex shader
    *  or an output block in a fragment shader"
    */
   if ((state->stage == MESA_SHADER_VERTEX) && q.flags.q.in) {
      _mesa_glsl_error(locp, state,
                       "`in' interface block is not allowed for "
                       "a vertex shader");
   } else if ((state->stage == MESA_SHADER_FRAGMENT) && q.flags.q.out) {
      _mesa_glsl_error(locp, state,
                       "`out' interface block is not allowed for "
                       "a fragment shader");
   }

   /* Since block arrays require names, and both features are added in
    * the same language versions, we don't have to explicitly
    * version-check both things.
    */
   if (block->instance_name != NULL) {
      state->check_version(150, 300, locp, "interface blocks with "
                           "an instance name are not allowed");
   }

   ast_type_qualifier::bitset_t interface_type_mask;
   struct ast_type_qualifier temp_type_qualifier;

   /* Get a bitmask containing only the in/out/uniform/buffer
    * flags, allowing us to ignore other irrelevant flags like
    * interpolation qualifiers.
    */
   temp_type_qualifier.flags.i = 0;
   temp_type_qualifier.flags.q.uniform = true;
   temp_type_qualifier.flags.q.in = true;
   temp_type_qualifier.flags.q.out = true;
   temp_type_qualifier.flags.q.buffer = true;
   temp_type_qualifier.flags.q.patch = true;
   interface_type_mask = temp_type_qualifier.flags.i;

   /* Get the block's interface qualifier.  The interface_qualifier
    * production rule guarantees that only one bit will be set (and
    * it will be in/out/uniform).
    */
   ast_type_qualifier::bitset_t block_interface_qualifier = q.flags.i;

   block->default_layout.flags.i |= block_interface_qualifier;

   if (state->stage == MESA_SHADER_GEOMETRY &&
       state->has_explicit_attrib_stream() &&
       block->default_layout.flags.q.out) {
      /* Assign global layout's stream value. */
      block->default_layout.flags.q.stream = 1;
      block->default_layout.flags.q.explicit_stream = 0;
      block->default_layout.stream = state->out_qualifier->stream;
   }

   if (state->has_enhanced_layouts() && block->default_layout.flags.q.out) {
      /* Assign global layout's xfb_buffer value. */
      block->default_layout.flags.q.xfb_buffer = 1;
      block->default_layout.flags.q.explicit_xfb_buffer = 0;
      block->default_layout.xfb_buffer = state->out_qualifier->xfb_buffer;
   }

   foreach_list_typed (ast_declarator_list, member, link, &block->declarations) {
      ast_type_qualifier& qualifier = member->type->qualifier;
      if ((qualifier.flags.i & interface_type_mask) == 0) {
         /* GLSLangSpec.1.50.11, 4.3.7 (Interface Blocks):
          * "If no optional qualifier is used in a member declaration, the
          *  qualifier of the variable is just in, out, or uniform as declared
          *  by interface-qualifier."
          */
         qualifier.flags.i |= block_interface_qualifier;
      } else if ((qualifier.flags.i & interface_type_mask) !=
                 block_interface_qualifier) {
         /* GLSLangSpec.1.50.11, 4.3.7 (Interface Blocks):
          * "If optional qualifiers are used, they can include interpolation
          *  and storage qualifiers and they must declare an input, output,
          *  or uniform variable consistent with the interface qualifier of
          *  the block."
          */
         _mesa_glsl_error(locp, state,
                          "uniform/in/out qualifier on "
                          "interface block member does not match "
                          "the interface block");
      }

      if (!(q.flags.q.in || q.flags.q.out) && qualifier.flags.q.invariant)
         _mesa_glsl_error(locp, state,
                          "invariant qualifiers can be used only "
                          "in interface block members for shader "
                          "inputs or outputs");
   }
}

static void
_mesa_ast_type_qualifier_print(const struct ast_type_qualifier *q)
{
   if (q->is_subroutine_decl())
      printf("subroutine ");

   if (q->subroutine_list) {
      printf("subroutine (");
      q->subroutine_list->print();
      printf(")");
   }

   if (q->flags.q.constant)
      printf("const ");

   if (q->flags.q.invariant)
      printf("invariant ");

   if (q->flags.q.attribute)
      printf("attribute ");

   if (q->flags.q.varying)
      printf("varying ");

   if (q->flags.q.in && q->flags.q.out)
      printf("inout ");
   else {
      if (q->flags.q.in)
	 printf("in ");

      if (q->flags.q.out)
	 printf("out ");
   }

   if (q->flags.q.centroid)
      printf("centroid ");
   if (q->flags.q.sample)
      printf("sample ");
   if (q->flags.q.patch)
      printf("patch ");
   if (q->flags.q.uniform)
      printf("uniform ");
   if (q->flags.q.buffer)
      printf("buffer ");
   if (q->flags.q.smooth)
      printf("smooth ");
   if (q->flags.q.flat)
      printf("flat ");
   if (q->flags.q.noperspective)
      printf("noperspective ");
}


void
ast_node::print(void) const
{
   printf("unhandled node ");
}


ast_node::ast_node(void)
{
   this->location.source = 0;
   this->location.first_line = 0;
   this->location.first_column = 0;
   this->location.last_line = 0;
   this->location.last_column = 0;
}


static void
ast_opt_array_dimensions_print(const ast_array_specifier *array_specifier)
{
   if (array_specifier)
      array_specifier->print();
}


void
ast_compound_statement::print(void) const
{
   printf("{\n");

   foreach_list_typed(ast_node, ast, link, &this->statements) {
      ast->print();
   }

   printf("}\n");
}


ast_compound_statement::ast_compound_statement(int new_scope,
					       ast_node *statements)
{
   this->new_scope = new_scope;

   if (statements != NULL) {
      this->statements.push_degenerate_list_at_head(&statements->link);
   }
}


void
ast_expression::print(void) const
{
   switch (oper) {
   case ast_assign:
   case ast_mul_assign:
   case ast_div_assign:
   case ast_mod_assign:
   case ast_add_assign:
   case ast_sub_assign:
   case ast_ls_assign:
   case ast_rs_assign:
   case ast_and_assign:
   case ast_xor_assign:
   case ast_or_assign:
      subexpressions[0]->print();
      printf("%s ", operator_string(oper));
      subexpressions[1]->print();
      break;

   case ast_field_selection:
      subexpressions[0]->print();
      printf(". %s ", primary_expression.identifier);
      break;

   case ast_plus:
   case ast_neg:
   case ast_bit_not:
   case ast_logic_not:
   case ast_pre_inc:
   case ast_pre_dec:
      printf("%s ", operator_string(oper));
      subexpressions[0]->print();
      break;

   case ast_post_inc:
   case ast_post_dec:
      subexpressions[0]->print();
      printf("%s ", operator_string(oper));
      break;

   case ast_conditional:
      subexpressions[0]->print();
      printf("? ");
      subexpressions[1]->print();
      printf(": ");
      subexpressions[2]->print();
      break;

   case ast_array_index:
      subexpressions[0]->print();
      printf("[ ");
      subexpressions[1]->print();
      printf("] ");
      break;

   case ast_function_call: {
      subexpressions[0]->print();
      printf("( ");

      foreach_list_typed (ast_node, ast, link, &this->expressions) {
	 if (&ast->link != this->expressions.get_head())
	    printf(", ");

	 ast->print();
      }

      printf(") ");
      break;
   }

   case ast_identifier:
      printf("%s ", primary_expression.identifier);
      break;

   case ast_int_constant:
      printf("%d ", primary_expression.int_constant);
      break;

   case ast_uint_constant:
      printf("%u ", primary_expression.uint_constant);
      break;

   case ast_float_constant:
      printf("%f ", primary_expression.float_constant);
      break;

   case ast_double_constant:
      printf("%f ", primary_expression.double_constant);
      break;

   case ast_int64_constant:
      printf("%" PRId64 " ", primary_expression.int64_constant);
      break;

   case ast_uint64_constant:
      printf("%" PRIu64 " ", primary_expression.uint64_constant);
      break;

   case ast_bool_constant:
      printf("%s ",
	     primary_expression.bool_constant
	     ? "true" : "false");
      break;

   case ast_sequence: {
      printf("( ");
      foreach_list_typed (ast_node, ast, link, & this->expressions) {
	 if (&ast->link != this->expressions.get_head())
	    printf(", ");

	 ast->print();
      }
      printf(") ");
      break;
   }

   case ast_aggregate: {
      printf("{ ");
      foreach_list_typed (ast_node, ast, link, & this->expressions) {
	 if (&ast->link != this->expressions.get_head())
	    printf(", ");

	 ast->print();
      }
      printf("} ");
      break;
   }

   default:
      assert(0);
      break;
   }
}

ast_expression::ast_expression(int oper,
			       ast_expression *ex0,
			       ast_expression *ex1,
			       ast_expression *ex2) :
   primary_expression()
{
   this->oper = ast_operators(oper);
   this->subexpressions[0] = ex0;
   this->subexpressions[1] = ex1;
   this->subexpressions[2] = ex2;
   this->non_lvalue_description = NULL;
   this->is_lhs = false;
}


void
ast_expression_statement::print(void) const
{
   if (expression)
      expression->print();

   printf("; ");
}


ast_expression_statement::ast_expression_statement(ast_expression *ex) :
   expression(ex)
{
   /* empty */
}


void
ast_function::print(void) const
{
   return_type->print();
   printf(" %s (", identifier);

   foreach_list_typed(ast_node, ast, link, & this->parameters) {
      ast->print();
   }

   printf(")");
}


ast_function::ast_function(void)
   : return_type(NULL), identifier(NULL), is_definition(false),
     signature(NULL)
{
   /* empty */
}


void
ast_fully_specified_type::print(void) const
{
   _mesa_ast_type_qualifier_print(& qualifier);
   specifier->print();
}


void
ast_parameter_declarator::print(void) const
{
   type->print();
   if (identifier)
      printf("%s ", identifier);
   ast_opt_array_dimensions_print(array_specifier);
}


void
ast_function_definition::print(void) const
{
   prototype->print();
   body->print();
}


void
ast_declaration::print(void) const
{
   printf("%s ", identifier);
   ast_opt_array_dimensions_print(array_specifier);

   if (initializer) {
      printf("= ");
      initializer->print();
   }
}


ast_declaration::ast_declaration(const char *identifier,
				 ast_array_specifier *array_specifier,
				 ast_expression *initializer)
{
   this->identifier = identifier;
   this->array_specifier = array_specifier;
   this->initializer = initializer;
}


void
ast_declarator_list::print(void) const
{
   assert(type || invariant);

   if (type)
      type->print();
   else if (invariant)
      printf("invariant ");
   else
      printf("precise ");

   foreach_list_typed (ast_node, ast, link, & this->declarations) {
      if (&ast->link != this->declarations.get_head())
	 printf(", ");

      ast->print();
   }

   printf("; ");
}


ast_declarator_list::ast_declarator_list(ast_fully_specified_type *type)
{
   this->type = type;
   this->invariant = false;
   this->precise = false;
}

void
ast_jump_statement::print(void) const
{
   switch (mode) {
   case ast_continue:
      printf("continue; ");
      break;
   case ast_break:
      printf("break; ");
      break;
   case ast_return:
      printf("return ");
      if (opt_return_value)
	 opt_return_value->print();

      printf("; ");
      break;
   case ast_discard:
      printf("discard; ");
      break;
   }
}


ast_jump_statement::ast_jump_statement(int mode, ast_expression *return_value)
   : opt_return_value(NULL)
{
   this->mode = ast_jump_modes(mode);

   if (mode == ast_return)
      opt_return_value = return_value;
}


void
ast_demote_statement::print(void) const
{
   printf("demote; ");
}


void
ast_selection_statement::print(void) const
{
   printf("if ( ");
   condition->print();
   printf(") ");

   then_statement->print();

   if (else_statement) {
      printf("else ");
      else_statement->print();
   }
}


ast_selection_statement::ast_selection_statement(ast_expression *condition,
						 ast_node *then_statement,
						 ast_node *else_statement)
{
   this->condition = condition;
   this->then_statement = then_statement;
   this->else_statement = else_statement;
}


void
ast_switch_statement::print(void) const
{
   printf("switch ( ");
   test_expression->print();
   printf(") ");

   body->print();
}


ast_switch_statement::ast_switch_statement(ast_expression *test_expression,
					   ast_node *body)
{
   this->test_expression = test_expression;
   this->body = body;
}


void
ast_switch_body::print(void) const
{
   printf("{\n");
   if (stmts != NULL) {
      stmts->print();
   }
   printf("}\n");
}


ast_switch_body::ast_switch_body(ast_case_statement_list *stmts)
{
   this->stmts = stmts;
}


void ast_case_label::print(void) const
{
   if (test_value != NULL) {
      printf("case ");
      test_value->print();
      printf(": ");
   } else {
      printf("default: ");
   }
}


ast_case_label::ast_case_label(ast_expression *test_value)
{
   this->test_value = test_value;
}


void ast_case_label_list::print(void) const
{
   foreach_list_typed(ast_node, ast, link, & this->labels) {
      ast->print();
   }
   printf("\n");
}


ast_case_label_list::ast_case_label_list(void)
{
}


void ast_case_statement::print(void) const
{
   labels->print();
   foreach_list_typed(ast_node, ast, link, & this->stmts) {
      ast->print();
      printf("\n");
   }
}


ast_case_statement::ast_case_statement(ast_case_label_list *labels)
{
   this->labels = labels;
}


void ast_case_statement_list::print(void) const
{
   foreach_list_typed(ast_node, ast, link, & this->cases) {
      ast->print();
   }
}


ast_case_statement_list::ast_case_statement_list(void)
{
}


void
ast_iteration_statement::print(void) const
{
   switch (mode) {
   case ast_for:
      printf("for( ");
      if (init_statement)
	 init_statement->print();
      printf("; ");

      if (condition)
	 condition->print();
      printf("; ");

      if (rest_expression)
	 rest_expression->print();
      printf(") ");

      body->print();
      break;

   case ast_while:
      printf("while ( ");
      if (condition)
	 condition->print();
      printf(") ");
      body->print();
      break;

   case ast_do_while:
      printf("do ");
      body->print();
      printf("while ( ");
      if (condition)
	 condition->print();
      printf("); ");
      break;
   }
}


ast_iteration_statement::ast_iteration_statement(int mode,
						 ast_node *init,
						 ast_node *condition,
						 ast_expression *rest_expression,
						 ast_node *body)
{
   this->mode = ast_iteration_modes(mode);
   this->init_statement = init;
   this->condition = condition;
   this->rest_expression = rest_expression;
   this->body = body;
}


void
ast_struct_specifier::print(void) const
{
   printf("struct %s { ", name);
   foreach_list_typed(ast_node, ast, link, &this->declarations) {
      ast->print();
   }
   printf("} ");
}


ast_struct_specifier::ast_struct_specifier(const char *identifier,
					   ast_declarator_list *declarator_list)
   : name(identifier), layout(NULL), declarations(), is_declaration(true),
     type(NULL)
{
   this->declarations.push_degenerate_list_at_head(&declarator_list->link);
}

void ast_subroutine_list::print(void) const
{
   foreach_list_typed (ast_node, ast, link, & this->declarations) {
      if (&ast->link != this->declarations.get_head())
         printf(", ");
      ast->print();
   }
}

static void
set_shader_inout_layout(struct gl_shader *shader,
		     struct _mesa_glsl_parse_state *state)
{
   /* Should have been prevented by the parser. */
   if (shader->Stage != MESA_SHADER_GEOMETRY &&
       shader->Stage != MESA_SHADER_TESS_EVAL &&
       shader->Stage != MESA_SHADER_COMPUTE) {
      assert(!state->in_qualifier->flags.i);
   }

   if (shader->Stage != MESA_SHADER_COMPUTE) {
      /* Should have been prevented by the parser. */
      assert(!state->cs_input_local_size_specified);
      assert(!state->cs_input_local_size_variable_specified);
      assert(state->cs_derivative_group == DERIVATIVE_GROUP_NONE);
   }

   if (shader->Stage != MESA_SHADER_FRAGMENT) {
      /* Should have been prevented by the parser. */
      assert(!state->fs_uses_gl_fragcoord);
      assert(!state->fs_redeclares_gl_fragcoord);
      assert(!state->fs_pixel_center_integer);
      assert(!state->fs_origin_upper_left);
      assert(!state->fs_early_fragment_tests);
      assert(!state->fs_inner_coverage);
      assert(!state->fs_post_depth_coverage);
      assert(!state->fs_pixel_interlock_ordered);
      assert(!state->fs_pixel_interlock_unordered);
      assert(!state->fs_sample_interlock_ordered);
      assert(!state->fs_sample_interlock_unordered);
   }

   for (unsigned i = 0; i < MAX_FEEDBACK_BUFFERS; i++) {
      if (state->out_qualifier->out_xfb_stride[i]) {
         unsigned xfb_stride;
         if (state->out_qualifier->out_xfb_stride[i]->
                process_qualifier_constant(state, "xfb_stride", &xfb_stride,
                true)) {
            shader->TransformFeedbackBufferStride[i] = xfb_stride;
         }
      }
   }

   switch (shader->Stage) {
   case MESA_SHADER_TESS_CTRL:
      shader->info.TessCtrl.VerticesOut = 0;
      if (state->tcs_output_vertices_specified) {
         unsigned vertices;
         if (state->out_qualifier->vertices->
               process_qualifier_constant(state, "vertices", &vertices,
                                          false)) {

            YYLTYPE loc = state->out_qualifier->vertices->get_location();
            if (vertices > state->Const.MaxPatchVertices) {
               _mesa_glsl_error(&loc, state, "vertices (%d) exceeds "
                                "GL_MAX_PATCH_VERTICES", vertices);
            }
            shader->info.TessCtrl.VerticesOut = vertices;
         }
      }
      break;
   case MESA_SHADER_TESS_EVAL:
      shader->info.TessEval.PrimitiveMode = PRIM_UNKNOWN;
      if (state->in_qualifier->flags.q.prim_type)
         shader->info.TessEval.PrimitiveMode = state->in_qualifier->prim_type;

      shader->info.TessEval.Spacing = TESS_SPACING_UNSPECIFIED;
      if (state->in_qualifier->flags.q.vertex_spacing)
         shader->info.TessEval.Spacing = state->in_qualifier->vertex_spacing;

      shader->info.TessEval.VertexOrder = 0;
      if (state->in_qualifier->flags.q.ordering)
         shader->info.TessEval.VertexOrder = state->in_qualifier->ordering;

      shader->info.TessEval.PointMode = -1;
      if (state->in_qualifier->flags.q.point_mode)
         shader->info.TessEval.PointMode = state->in_qualifier->point_mode;
      break;
   case MESA_SHADER_GEOMETRY:
      shader->info.Geom.VerticesOut = -1;
      if (state->out_qualifier->flags.q.max_vertices) {
         unsigned qual_max_vertices;
         if (state->out_qualifier->max_vertices->
               process_qualifier_constant(state, "max_vertices",
                                          &qual_max_vertices, true)) {

            if (qual_max_vertices > state->Const.MaxGeometryOutputVertices) {
               YYLTYPE loc = state->out_qualifier->max_vertices->get_location();
               _mesa_glsl_error(&loc, state,
                                "maximum output vertices (%d) exceeds "
                                "GL_MAX_GEOMETRY_OUTPUT_VERTICES",
                                qual_max_vertices);
            }
            shader->info.Geom.VerticesOut = qual_max_vertices;
         }
      }

      if (state->gs_input_prim_type_specified) {
         shader->info.Geom.InputType = state->in_qualifier->prim_type;
      } else {
         shader->info.Geom.InputType = PRIM_UNKNOWN;
      }

      if (state->out_qualifier->flags.q.prim_type) {
         shader->info.Geom.OutputType = state->out_qualifier->prim_type;
      } else {
         shader->info.Geom.OutputType = PRIM_UNKNOWN;
      }

      shader->info.Geom.Invocations = 0;
      if (state->in_qualifier->flags.q.invocations) {
         unsigned invocations;
         if (state->in_qualifier->invocations->
               process_qualifier_constant(state, "invocations",
                                          &invocations, false)) {

            YYLTYPE loc = state->in_qualifier->invocations->get_location();
            if (invocations > state->Const.MaxGeometryShaderInvocations) {
               _mesa_glsl_error(&loc, state,
                                "invocations (%d) exceeds "
                                "GL_MAX_GEOMETRY_SHADER_INVOCATIONS",
                                invocations);
            }
            shader->info.Geom.Invocations = invocations;
         }
      }
      break;

   case MESA_SHADER_COMPUTE:
      if (state->cs_input_local_size_specified) {
         for (int i = 0; i < 3; i++)
            shader->info.Comp.LocalSize[i] = state->cs_input_local_size[i];
      } else {
         for (int i = 0; i < 3; i++)
            shader->info.Comp.LocalSize[i] = 0;
      }

      shader->info.Comp.LocalSizeVariable =
         state->cs_input_local_size_variable_specified;

      shader->info.Comp.DerivativeGroup = state->cs_derivative_group;

      if (state->NV_compute_shader_derivatives_enable) {
         /* We allow multiple cs_input_layout nodes, but do not store them in
          * a convenient place, so for now live with an empty location error.
          */
         YYLTYPE loc = {0};
         if (shader->info.Comp.DerivativeGroup == DERIVATIVE_GROUP_QUADS) {
            if (shader->info.Comp.LocalSize[0] % 2 != 0) {
               _mesa_glsl_error(&loc, state, "derivative_group_quadsNV must be used with a "
                                "local group size whose first dimension "
                                "is a multiple of 2\n");
            }
            if (shader->info.Comp.LocalSize[1] % 2 != 0) {
               _mesa_glsl_error(&loc, state, "derivative_group_quadsNV must be used with a "
                                "local group size whose second dimension "
                                "is a multiple of 2\n");
            }
         } else if (shader->info.Comp.DerivativeGroup == DERIVATIVE_GROUP_LINEAR) {
            if ((shader->info.Comp.LocalSize[0] *
                 shader->info.Comp.LocalSize[1] *
                 shader->info.Comp.LocalSize[2]) % 4 != 0) {
               _mesa_glsl_error(&loc, state, "derivative_group_linearNV must be used with a "
                            "local group size whose total number of invocations "
                            "is a multiple of 4\n");
            }
         }
      }

      break;

   case MESA_SHADER_FRAGMENT:
      shader->redeclares_gl_fragcoord = state->fs_redeclares_gl_fragcoord;
      shader->uses_gl_fragcoord = state->fs_uses_gl_fragcoord;
      shader->pixel_center_integer = state->fs_pixel_center_integer;
      shader->origin_upper_left = state->fs_origin_upper_left;
      shader->ARB_fragment_coord_conventions_enable =
         state->ARB_fragment_coord_conventions_enable;
      shader->EarlyFragmentTests = state->fs_early_fragment_tests;
      shader->InnerCoverage = state->fs_inner_coverage;
      shader->PostDepthCoverage = state->fs_post_depth_coverage;
      shader->PixelInterlockOrdered = state->fs_pixel_interlock_ordered;
      shader->PixelInterlockUnordered = state->fs_pixel_interlock_unordered;
      shader->SampleInterlockOrdered = state->fs_sample_interlock_ordered;
      shader->SampleInterlockUnordered = state->fs_sample_interlock_unordered;
      shader->BlendSupport = state->fs_blend_support;
      break;

   default:
      /* Nothing to do. */
      break;
   }

   shader->bindless_sampler = state->bindless_sampler_specified;
   shader->bindless_image = state->bindless_image_specified;
   shader->bound_sampler = state->bound_sampler_specified;
   shader->bound_image = state->bound_image_specified;
   shader->redeclares_gl_layer = state->redeclares_gl_layer;
   shader->layer_viewport_relative = state->layer_viewport_relative;
}

/* src can be NULL if only the symbols found in the exec_list should be
 * copied
 */
void
_mesa_glsl_copy_symbols_from_table(struct exec_list *shader_ir,
                                   struct glsl_symbol_table *src,
                                   struct glsl_symbol_table *dest)
{
   foreach_in_list (ir_instruction, ir, shader_ir) {
      switch (ir->ir_type) {
      case ir_type_function:
         dest->add_function((ir_function *) ir);
         break;
      case ir_type_variable: {
         ir_variable *const var = (ir_variable *) ir;

         if (var->data.mode != ir_var_temporary)
            dest->add_variable(var);
         break;
      }
      default:
         break;
      }
   }

   if (src != NULL) {
      /* Explicitly copy the gl_PerVertex interface definitions because these
       * are needed to check they are the same during the interstage link.
       * They can’t necessarily be found via the exec_list because the members
       * might not be referenced. The GL spec still requires that they match
       * in that case.
       */
      const glsl_type *iface =
         src->get_interface("gl_PerVertex", ir_var_shader_in);
      if (iface)
         dest->add_interface(iface->name, iface, ir_var_shader_in);

      iface = src->get_interface("gl_PerVertex", ir_var_shader_out);
      if (iface)
         dest->add_interface(iface->name, iface, ir_var_shader_out);
   }
}

extern "C" {

static void
assign_subroutine_indexes(struct _mesa_glsl_parse_state *state)
{
   int j, k;
   int index = 0;

   for (j = 0; j < state->num_subroutines; j++) {
      while (state->subroutines[j]->subroutine_index == -1) {
         for (k = 0; k < state->num_subroutines; k++) {
            if (state->subroutines[k]->subroutine_index == index)
               break;
            else if (k == state->num_subroutines - 1) {
               state->subroutines[j]->subroutine_index = index;
            }
         }
         index++;
      }
   }
}

void
add_builtin_defines(struct _mesa_glsl_parse_state *state,
                    void (*add_builtin_define)(struct glcpp_parser *, const char *, int),
                    struct glcpp_parser *data,
                    unsigned version,
                    bool es)
{
   unsigned gl_version = state->ctx->Extensions.Version;
   gl_api api = state->ctx->API;

   if (gl_version != 0xff) {
      unsigned i;
      for (i = 0; i < state->num_supported_versions; i++) {
         if (state->supported_versions[i].ver == version &&
             state->supported_versions[i].es == es) {
            gl_version = state->supported_versions[i].gl_ver;
            break;
         }
      }

      if (i == state->num_supported_versions)
         return;
   }

   if (es)
      api = API_OPENGLES2;

   for (unsigned i = 0;
        i < ARRAY_SIZE(_mesa_glsl_supported_extensions); ++i) {
      const _mesa_glsl_extension *extension
         = &_mesa_glsl_supported_extensions[i];
      if (extension->compatible_with_state(state, api, gl_version)) {
         add_builtin_define(data, extension->name, 1);
      }
   }
}

/* Implements parsing checks that we can't do during parsing */
static void
do_late_parsing_checks(struct _mesa_glsl_parse_state *state)
{
   if (state->stage == MESA_SHADER_COMPUTE && !state->has_compute_shader()) {
      YYLTYPE loc;
      memset(&loc, 0, sizeof(loc));
      _mesa_glsl_error(&loc, state, "Compute shaders require "
                       "GLSL 4.30 or GLSL ES 3.10");
   }
}

static void
opt_shader_and_create_symbol_table(struct gl_context *ctx,
                                   struct glsl_symbol_table *source_symbols,
                                   struct gl_shader *shader)
{
   assert(shader->CompileStatus != COMPILE_FAILURE &&
          !shader->ir->is_empty());

   struct gl_shader_compiler_options *options =
      &ctx->Const.ShaderCompilerOptions[shader->Stage];

   /* Do some optimization at compile time to reduce shader IR size
    * and reduce later work if the same shader is linked multiple times
    */
   if (ctx->Const.GLSLOptimizeConservatively) {
      /* Run it just once. */
      do_common_optimization(shader->ir, false, false, options,
                             ctx->Const.NativeIntegers);
   } else {
      /* Repeat it until it stops making changes. */
      while (do_common_optimization(shader->ir, false, false, options,
                                    ctx->Const.NativeIntegers))
         ;
   }

   validate_ir_tree(shader->ir);

   enum ir_variable_mode other;
   switch (shader->Stage) {
   case MESA_SHADER_VERTEX:
      other = ir_var_shader_in;
      break;
   case MESA_SHADER_FRAGMENT:
      other = ir_var_shader_out;
      break;
   default:
      /* Something invalid to ensure optimize_dead_builtin_uniforms
       * doesn't remove anything other than uniforms or constants.
       */
      other = ir_var_mode_count;
      break;
   }

   optimize_dead_builtin_variables(shader->ir, other);

   validate_ir_tree(shader->ir);

   /* Retain any live IR, but trash the rest. */
   reparent_ir(shader->ir, shader->ir);

   /* Destroy the symbol table.  Create a new symbol table that contains only
    * the variables and functions that still exist in the IR.  The symbol
    * table will be used later during linking.
    *
    * There must NOT be any freed objects still referenced by the symbol
    * table.  That could cause the linker to dereference freed memory.
    *
    * We don't have to worry about types or interface-types here because those
    * are fly-weights that are looked up by glsl_type.
    */
   _mesa_glsl_copy_symbols_from_table(shader->ir, source_symbols,
                                      shader->symbols);
}

static bool
can_skip_compile(struct gl_context *ctx, struct gl_shader *shader,
                 const char *source, bool force_recompile,
                 bool source_has_shader_include)
{
   if (!force_recompile) {
      if (ctx->Cache) {
         char buf[41];
         disk_cache_compute_key(ctx->Cache, source, strlen(source),
                                shader->sha1);
         if (disk_cache_has_key(ctx->Cache, shader->sha1)) {
            /* We've seen this shader before and know it compiles */
            if (ctx->_Shader->Flags & GLSL_CACHE_INFO) {
               _mesa_sha1_format(buf, shader->sha1);
               fprintf(stderr, "deferring compile of shader: %s\n", buf);
            }
            shader->CompileStatus = COMPILE_SKIPPED;

            free((void *)shader->FallbackSource);

            /* Copy pre-processed shader include to fallback source otherwise
             * we have no guarantee the shader include source tree has not
             * changed.
             */
            shader->FallbackSource = source_has_shader_include ?
               strdup(source) : NULL;
            return true;
         }
      }
   } else {
      /* We should only ever end up here if a re-compile has been forced by a
       * shader cache miss. In which case we can skip the compile if its
       * already been done by a previous fallback or the initial compile call.
       */
      if (shader->CompileStatus == COMPILE_SUCCESS)
         return true;
   }

   return false;
}

void
_mesa_glsl_compile_shader(struct gl_context *ctx, struct gl_shader *shader,
                          bool dump_ast, bool dump_hir, bool force_recompile)
{
   const char *source = force_recompile && shader->FallbackSource ?
      shader->FallbackSource : shader->Source;

   /* Note this will be true for shaders the have #include inside comments
    * however that should be rare enough not to worry about.
    */
   bool source_has_shader_include =
      strstr(source, "#include") == NULL ? false : true;

   /* If there was no shader include we can check the shader cache and skip
    * compilation before we run the preprocessor. We never skip compiling
    * shaders that use ARB_shading_language_include because we would need to
    * keep duplicate copies of the shader include source tree and paths.
    */
   if (!source_has_shader_include &&
       can_skip_compile(ctx, shader, source, force_recompile, false))
      return;

    struct _mesa_glsl_parse_state *state =
      new(shader) _mesa_glsl_parse_state(ctx, shader->Stage, shader);

   if (ctx->Const.GenerateTemporaryNames)
      (void) p_atomic_cmpxchg(&ir_variable::temporaries_allocate_names,
                              false, true);

   if (!source_has_shader_include || !force_recompile) {
      state->error = glcpp_preprocess(state, &source, &state->info_log,
                                      add_builtin_defines, state, ctx);
   }

   /* Now that we have run the preprocessor we can check the shader cache and
    * skip compilation if possible for those shaders that contained a shader
    * include.
    */
   if (source_has_shader_include &&
       can_skip_compile(ctx, shader, source, force_recompile, true))
      return;

   if (!state->error) {
     _mesa_glsl_lexer_ctor(state, source);
     _mesa_glsl_parse(state);
     _mesa_glsl_lexer_dtor(state);
     do_late_parsing_checks(state);
   }

   if (dump_ast) {
      foreach_list_typed(ast_node, ast, link, &state->translation_unit) {
         ast->print();
      }
      printf("\n\n");
   }

   ralloc_free(shader->ir);
   shader->ir = new(shader) exec_list;
   if (!state->error && !state->translation_unit.is_empty())
      _mesa_ast_to_hir(shader->ir, state);

   if (!state->error) {
      validate_ir_tree(shader->ir);

      /* Print out the unoptimized IR. */
      if (dump_hir) {
         _mesa_print_ir(stdout, shader->ir, state);
      }
   }

   if (shader->InfoLog)
      ralloc_free(shader->InfoLog);

   if (!state->error)
      set_shader_inout_layout(shader, state);

   shader->symbols = new(shader->ir) glsl_symbol_table;
   shader->CompileStatus = state->error ? COMPILE_FAILURE : COMPILE_SUCCESS;
   shader->InfoLog = state->info_log;
   shader->Version = state->language_version;
   shader->IsES = state->es_shader;

   struct gl_shader_compiler_options *options =
      &ctx->Const.ShaderCompilerOptions[shader->Stage];

   if (!state->error && !shader->ir->is_empty()) {
      if (options->LowerPrecision)
         lower_precision(shader->ir);
      lower_builtins(shader->ir);
      assign_subroutine_indexes(state);
      lower_subroutine(shader->ir, state);
      opt_shader_and_create_symbol_table(ctx, state->symbols, shader);
   }

   if (!force_recompile) {
      free((void *)shader->FallbackSource);

      /* Copy pre-processed shader include to fallback source otherwise we
       * have no guarantee the shader include source tree has not changed.
       */
      shader->FallbackSource = source_has_shader_include ?
         strdup(source) : NULL;
   }

   delete state->symbols;
   ralloc_free(state);

   if (ctx->Cache && shader->CompileStatus == COMPILE_SUCCESS) {
      char sha1_buf[41];
      disk_cache_put_key(ctx->Cache, shader->sha1);
      if (ctx->_Shader->Flags & GLSL_CACHE_INFO) {
         _mesa_sha1_format(sha1_buf, shader->sha1);
         fprintf(stderr, "marking shader: %s\n", sha1_buf);
      }
   }
}

} /* extern "C" */
/**
 * Do the set of common optimizations passes
 *
 * \param ir                          List of instructions to be optimized
 * \param linked                      Is the shader linked?  This enables
 *                                    optimizations passes that remove code at
 *                                    global scope and could cause linking to
 *                                    fail.
 * \param uniform_locations_assigned  Have locations already been assigned for
 *                                    uniforms?  This prevents the declarations
 *                                    of unused uniforms from being removed.
 *                                    The setting of this flag only matters if
 *                                    \c linked is \c true.
 * \param options                     The driver's preferred shader options.
 * \param native_integers             Selects optimizations that depend on the
 *                                    implementations supporting integers
 *                                    natively (as opposed to supporting
 *                                    integers in floating point registers).
 */
bool
do_common_optimization(exec_list *ir, bool linked,
		       bool uniform_locations_assigned,
                       const struct gl_shader_compiler_options *options,
                       bool native_integers)
{
   const bool debug = false;
   bool progress = false;

#define OPT(PASS, ...) do {                                             \
      if (debug) {                                                      \
         fprintf(stderr, "START GLSL optimization %s\n", #PASS);        \
         const bool opt_progress = PASS(__VA_ARGS__);                   \
         progress = opt_progress || progress;                           \
         if (opt_progress)                                              \
            _mesa_print_ir(stderr, ir, NULL);                           \
         fprintf(stderr, "GLSL optimization %s: %s progress\n",         \
                 #PASS, opt_progress ? "made" : "no");                  \
      } else {                                                          \
         progress = PASS(__VA_ARGS__) || progress;                      \
      }                                                                 \
   } while (false)

   OPT(lower_instructions, ir, SUB_TO_ADD_NEG);

   if (linked) {
      OPT(do_function_inlining, ir);
      OPT(do_dead_functions, ir);
      OPT(do_structure_splitting, ir);
   }
   propagate_invariance(ir);
   OPT(do_if_simplification, ir);
   OPT(opt_flatten_nested_if_blocks, ir);
   OPT(opt_conditional_discard, ir);
   OPT(do_copy_propagation_elements, ir);

   if (options->OptimizeForAOS && !linked)
      OPT(opt_flip_matrices, ir);

   if (linked && options->OptimizeForAOS) {
      OPT(do_vectorize, ir);
   }

   if (linked)
      OPT(do_dead_code, ir, uniform_locations_assigned);
   else
      OPT(do_dead_code_unlinked, ir);
   OPT(do_dead_code_local, ir);
   OPT(do_tree_grafting, ir);
   OPT(do_constant_propagation, ir);
   if (linked)
      OPT(do_constant_variable, ir);
   else
      OPT(do_constant_variable_unlinked, ir);
   OPT(do_constant_folding, ir);
   OPT(do_minmax_prune, ir);
   OPT(do_rebalance_tree, ir);
   OPT(do_algebraic, ir, native_integers, options);
   OPT(do_lower_jumps, ir, true, true, options->EmitNoMainReturn,
       options->EmitNoCont, options->EmitNoLoops);
   OPT(do_vec_index_to_swizzle, ir);
   OPT(lower_vector_insert, ir, false);
   OPT(optimize_swizzles, ir);

   /* Some drivers only call do_common_optimization() once rather than in a
    * loop, and split arrays causes each element of a constant array to
    * dereference is own copy of the entire array initilizer. This IR is not
    * something that can be generated manually in a shader and is not
    * accounted for by NIR optimisations, the result is an exponential slow
    * down in compilation speed as a constant arrays element count grows. To
    * avoid that here we make sure to always clean up the mess split arrays
    * causes to constant arrays.
    */
   bool array_split = optimize_split_arrays(ir, linked);
   if (array_split)
      do_constant_propagation(ir);
   progress |= array_split;

   OPT(optimize_redundant_jumps, ir);

   if (options->MaxUnrollIterations) {
      loop_state *ls = analyze_loop_variables(ir);
      if (ls->loop_found) {
         bool loop_progress = unroll_loops(ir, ls, options);
         while (loop_progress) {
            loop_progress = false;
            loop_progress |= do_constant_propagation(ir);
            loop_progress |= do_if_simplification(ir);

            /* Some drivers only call do_common_optimization() once rather
             * than in a loop. So we must call do_lower_jumps() after
             * unrolling a loop because for drivers that use LLVM validation
             * will fail if a jump is not the last instruction in the block.
             * For example the following will fail LLVM validation:
             *
             *   (loop (
             *      ...
             *   break
             *   (assign  (x) (var_ref v124)  (expression int + (var_ref v124)
             *      (constant int (1)) ) )
             *   ))
             */
            loop_progress |= do_lower_jumps(ir, true, true,
                                            options->EmitNoMainReturn,
                                            options->EmitNoCont,
                                            options->EmitNoLoops);
         }
         progress |= loop_progress;
      }
      delete ls;
   }

#undef OPT

   return progress;
}
