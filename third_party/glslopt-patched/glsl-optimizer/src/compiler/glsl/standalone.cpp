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
#include <getopt.h>

/** @file standalone.cpp
 *
 * Standalone compiler helper lib.  Used by standalone glsl_compiler and
 * also available to drivers to implement their own standalone compiler
 * with driver backend.
 */

#include "ast.h"
#include "glsl_parser_extras.h"
#include "ir_optimization.h"
#include "program.h"
#include "loop_analysis.h"
#include "standalone_scaffolding.h"
#include "standalone.h"
#include "string_to_uint_map.h"
#include "util/set.h"
#include "linker.h"
#include "glsl_parser_extras.h"
#include "ir_builder_print_visitor.h"
#include "builtin_functions.h"
#include "opt_add_neg_to_sub.h"
#include "main/mtypes.h"
#include "program/program.h"

class dead_variable_visitor : public ir_hierarchical_visitor {
public:
   dead_variable_visitor()
   {
      variables = _mesa_pointer_set_create(NULL);
   }

   virtual ~dead_variable_visitor()
   {
      _mesa_set_destroy(variables, NULL);
   }

   virtual ir_visitor_status visit(ir_variable *ir)
   {
      /* If the variable is auto or temp, add it to the set of variables that
       * are candidates for removal.
       */
      if (ir->data.mode != ir_var_auto && ir->data.mode != ir_var_temporary)
         return visit_continue;

      _mesa_set_add(variables, ir);

      return visit_continue;
   }

   virtual ir_visitor_status visit(ir_dereference_variable *ir)
   {
      struct set_entry *entry = _mesa_set_search(variables, ir->var);

      /* If a variable is dereferenced at all, remove it from the set of
       * variables that are candidates for removal.
       */
      if (entry != NULL)
         _mesa_set_remove(variables, entry);

      return visit_continue;
   }

   void remove_dead_variables()
   {
      set_foreach(variables, entry) {
         ir_variable *ir = (ir_variable *) entry->key;

         assert(ir->ir_type == ir_type_variable);
         ir->remove();
      }
   }

private:
   set *variables;
};

static void
init_gl_program(struct gl_program *prog, bool is_arb_asm, gl_shader_stage stage)
{
   prog->RefCount = 1;
   prog->Format = GL_PROGRAM_FORMAT_ASCII_ARB;
   prog->is_arb_asm = is_arb_asm;
   prog->info.stage = stage;
}

static struct gl_program *
new_program(UNUSED struct gl_context *ctx, gl_shader_stage stage,
            UNUSED GLuint id, bool is_arb_asm)
{
   struct gl_program *prog = rzalloc(NULL, struct gl_program);
   init_gl_program(prog, is_arb_asm, stage);
   return prog;
}

static const struct standalone_options *options;

static void
initialize_context(struct gl_context *ctx, gl_api api)
{
   initialize_context_to_defaults(ctx, api);
   _mesa_glsl_builtin_functions_init_or_ref();

   /* The standalone compiler needs to claim support for almost
    * everything in order to compile the built-in functions.
    */
   ctx->Const.GLSLVersion = options->glsl_version;
   ctx->Extensions.ARB_ES3_compatibility = true;
   ctx->Extensions.ARB_ES3_1_compatibility = true;
   ctx->Extensions.ARB_ES3_2_compatibility = true;
   ctx->Const.MaxComputeWorkGroupCount[0] = 65535;
   ctx->Const.MaxComputeWorkGroupCount[1] = 65535;
   ctx->Const.MaxComputeWorkGroupCount[2] = 65535;
   ctx->Const.MaxComputeWorkGroupSize[0] = 1024;
   ctx->Const.MaxComputeWorkGroupSize[1] = 1024;
   ctx->Const.MaxComputeWorkGroupSize[2] = 64;
   ctx->Const.MaxComputeWorkGroupInvocations = 1024;
   ctx->Const.MaxComputeSharedMemorySize = 32768;
   ctx->Const.MaxComputeVariableGroupSize[0] = 512;
   ctx->Const.MaxComputeVariableGroupSize[1] = 512;
   ctx->Const.MaxComputeVariableGroupSize[2] = 64;
   ctx->Const.MaxComputeVariableGroupInvocations = 512;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxTextureImageUnits = 16;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxUniformComponents = 1024;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxCombinedUniformComponents = 1024;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxInputComponents = 0; /* not used */
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxOutputComponents = 0; /* not used */
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxAtomicBuffers = 8;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxAtomicCounters = 8;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxImageUniforms = 8;
   ctx->Const.Program[MESA_SHADER_COMPUTE].MaxUniformBlocks = 12;

   switch (ctx->Const.GLSLVersion) {
   case 100:
      ctx->Const.MaxClipPlanes = 0;
      ctx->Const.MaxCombinedTextureImageUnits = 8;
      ctx->Const.MaxDrawBuffers = 2;
      ctx->Const.MinProgramTexelOffset = 0;
      ctx->Const.MaxProgramTexelOffset = 0;
      ctx->Const.MaxLights = 0;
      ctx->Const.MaxTextureCoordUnits = 0;
      ctx->Const.MaxTextureUnits = 8;

      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 8;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 0;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 128 * 4;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxCombinedUniformComponents = 128 * 4;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxInputComponents = 0; /* not used */
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 32;

      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits =
         ctx->Const.MaxCombinedTextureImageUnits;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 16 * 4;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxCombinedUniformComponents = 16 * 4;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents =
         ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxOutputComponents = 0; /* not used */

      ctx->Const.MaxVarying = ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents / 4;
      break;
   case 110:
   case 120:
      ctx->Const.MaxClipPlanes = 6;
      ctx->Const.MaxCombinedTextureImageUnits = 2;
      ctx->Const.MaxDrawBuffers = 1;
      ctx->Const.MinProgramTexelOffset = 0;
      ctx->Const.MaxProgramTexelOffset = 0;
      ctx->Const.MaxLights = 8;
      ctx->Const.MaxTextureCoordUnits = 2;
      ctx->Const.MaxTextureUnits = 2;

      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 0;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 512;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxCombinedUniformComponents = 512;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxInputComponents = 0; /* not used */
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 32;

      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits =
         ctx->Const.MaxCombinedTextureImageUnits;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 64;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxCombinedUniformComponents = 64;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents =
         ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxOutputComponents = 0; /* not used */

      ctx->Const.MaxVarying = ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents / 4;
      break;
   case 130:
   case 140:
      ctx->Const.MaxClipPlanes = 8;
      ctx->Const.MaxCombinedTextureImageUnits = 16;
      ctx->Const.MaxDrawBuffers = 8;
      ctx->Const.MinProgramTexelOffset = -8;
      ctx->Const.MaxProgramTexelOffset = 7;
      ctx->Const.MaxLights = 8;
      ctx->Const.MaxTextureCoordUnits = 8;
      ctx->Const.MaxTextureUnits = 2;
      ctx->Const.MaxUniformBufferBindings = 84;
      ctx->Const.MaxVertexStreams = 4;
      ctx->Const.MaxTransformFeedbackBuffers = 4;

      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxInputComponents = 0; /* not used */
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 64;

      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents =
         ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxOutputComponents = 0; /* not used */

      ctx->Const.MaxVarying = ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents / 4;
      break;
   case 150:
   case 330:
   case 400:
   case 410:
   case 420:
   case 430:
   case 440:
   case 450:
   case 460:
      ctx->Const.MaxClipPlanes = 8;
      ctx->Const.MaxDrawBuffers = 8;
      ctx->Const.MinProgramTexelOffset = -8;
      ctx->Const.MaxProgramTexelOffset = 7;
      ctx->Const.MaxLights = 8;
      ctx->Const.MaxTextureCoordUnits = 8;
      ctx->Const.MaxTextureUnits = 2;
      ctx->Const.MaxUniformBufferBindings = 84;
      ctx->Const.MaxVertexStreams = 4;
      ctx->Const.MaxTransformFeedbackBuffers = 4;
      ctx->Const.MaxShaderStorageBufferBindings = 4;
      ctx->Const.MaxShaderStorageBlockSize = 4096;
      ctx->Const.MaxAtomicBufferBindings = 4;

      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxInputComponents = 0; /* not used */
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 64;

      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxInputComponents =
         ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents;
      ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxOutputComponents = 128;

      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents =
         ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxOutputComponents;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxOutputComponents = 0; /* not used */

      ctx->Const.MaxCombinedTextureImageUnits =
         ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits
         + ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxTextureImageUnits
         + ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits;

      ctx->Const.MaxGeometryOutputVertices = 256;
      ctx->Const.MaxGeometryTotalOutputComponents = 1024;

      ctx->Const.MaxVarying = 60 / 4;
      break;
   case 300:
      ctx->Const.MaxClipPlanes = 8;
      ctx->Const.MaxCombinedTextureImageUnits = 32;
      ctx->Const.MaxDrawBuffers = 4;
      ctx->Const.MinProgramTexelOffset = -8;
      ctx->Const.MaxProgramTexelOffset = 7;
      ctx->Const.MaxLights = 0;
      ctx->Const.MaxTextureCoordUnits = 0;
      ctx->Const.MaxTextureUnits = 0;
      ctx->Const.MaxUniformBufferBindings = 84;
      ctx->Const.MaxVertexStreams = 4;
      ctx->Const.MaxTransformFeedbackBuffers = 4;

      ctx->Const.Program[MESA_SHADER_VERTEX].MaxAttribs = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxCombinedUniformComponents = 1024;
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxInputComponents = 0; /* not used */
      ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents = 16 * 4;

      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = 16;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxUniformComponents = 224;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxCombinedUniformComponents = 224;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents = 15 * 4;
      ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxOutputComponents = 0; /* not used */

      ctx->Const.MaxVarying = ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents / 4;
      break;
   }

   ctx->Const.GenerateTemporaryNames = true;
   ctx->Const.MaxPatchVertices = 32;

   /* GL_ARB_explicit_uniform_location, GL_MAX_UNIFORM_LOCATIONS */
   ctx->Const.MaxUserAssignableUniformLocations =
      4 * MESA_SHADER_STAGES * MAX_UNIFORMS;

   ctx->Driver.NewProgram = new_program;
}

/* Returned string will have 'ctx' as its ralloc owner. */
static char *
load_text_file(void *ctx, const char *file_name)
{
   char *text = NULL;
   size_t size;
   size_t total_read = 0;
   FILE *fp = fopen(file_name, "rb");

   if (!fp) {
      return NULL;
   }

   fseek(fp, 0L, SEEK_END);
   size = ftell(fp);
   fseek(fp, 0L, SEEK_SET);

   text = (char *) ralloc_size(ctx, size + 1);
   if (text != NULL) {
      do {
         size_t bytes = fread(text + total_read,
               1, size - total_read, fp);
         if (bytes < size - total_read) {
            free(text);
            text = NULL;
            goto error;
         }

         if (bytes == 0) {
            break;
         }

         total_read += bytes;
      } while (total_read < size);

      text[total_read] = '\0';
      error:;
   }

   fclose(fp);

   return text;
}

static void
compile_shader(struct gl_context *ctx, struct gl_shader *shader)
{
   struct _mesa_glsl_parse_state *state =
      new(shader) _mesa_glsl_parse_state(ctx, shader->Stage, shader);

   _mesa_glsl_compile_shader(ctx, shader, options->dump_ast,
                             options->dump_hir, true);

   /* Print out the resulting IR */
   if (!state->error && options->dump_lir) {
      _mesa_print_ir(stdout, shader->ir, state);
   }

   return;
}

extern "C" struct gl_shader_program *
standalone_compile_shader(const struct standalone_options *_options,
      unsigned num_files, char* const* files, struct gl_context *ctx)
{
   int status = EXIT_SUCCESS;
   bool glsl_es = false;

   options = _options;

   switch (options->glsl_version) {
   case 100:
   case 300:
      glsl_es = true;
      break;
   case 110:
   case 120:
   case 130:
   case 140:
   case 150:
   case 330:
   case 400:
   case 410:
   case 420:
   case 430:
   case 440:
   case 450:
   case 460:
      glsl_es = false;
      break;
   default:
      fprintf(stderr, "Unrecognized GLSL version `%d'\n", options->glsl_version);
      return NULL;
   }

   if (glsl_es) {
      initialize_context(ctx, API_OPENGLES2);
   } else {
      initialize_context(ctx, options->glsl_version > 130 ? API_OPENGL_CORE : API_OPENGL_COMPAT);
   }

   if (options->lower_precision) {
      for (unsigned i = MESA_SHADER_VERTEX; i <= MESA_SHADER_FRAGMENT; i++) {
         struct gl_shader_compiler_options *options =
            &ctx->Const.ShaderCompilerOptions[i];
         options->LowerPrecision = true;
      }
   }

   struct gl_shader_program *whole_program;

   whole_program = rzalloc (NULL, struct gl_shader_program);
   assert(whole_program != NULL);
   whole_program->data = rzalloc(whole_program, struct gl_shader_program_data);
   assert(whole_program->data != NULL);
   whole_program->data->InfoLog = ralloc_strdup(whole_program->data, "");

   /* Created just to avoid segmentation faults */
   whole_program->AttributeBindings = new string_to_uint_map;
   whole_program->FragDataBindings = new string_to_uint_map;
   whole_program->FragDataIndexBindings = new string_to_uint_map;

   for (unsigned i = 0; i < num_files; i++) {
      whole_program->Shaders =
            reralloc(whole_program, whole_program->Shaders,
                  struct gl_shader *, whole_program->NumShaders + 1);
      assert(whole_program->Shaders != NULL);

      struct gl_shader *shader = rzalloc(whole_program, gl_shader);

      whole_program->Shaders[whole_program->NumShaders] = shader;
      whole_program->NumShaders++;

      const unsigned len = strlen(files[i]);
      if (len < 6)
         goto fail;

      const char *const ext = & files[i][len - 5];
      /* TODO add support to read a .shader_test */
      if (strncmp(".vert", ext, 5) == 0 || strncmp(".glsl", ext, 5) == 0)
	 shader->Type = GL_VERTEX_SHADER;
      else if (strncmp(".tesc", ext, 5) == 0)
	 shader->Type = GL_TESS_CONTROL_SHADER;
      else if (strncmp(".tese", ext, 5) == 0)
	 shader->Type = GL_TESS_EVALUATION_SHADER;
      else if (strncmp(".geom", ext, 5) == 0)
	 shader->Type = GL_GEOMETRY_SHADER;
      else if (strncmp(".frag", ext, 5) == 0)
	 shader->Type = GL_FRAGMENT_SHADER;
      else if (strncmp(".comp", ext, 5) == 0)
         shader->Type = GL_COMPUTE_SHADER;
      else
         goto fail;
      shader->Stage = _mesa_shader_enum_to_shader_stage(shader->Type);

      shader->Source = load_text_file(whole_program, files[i]);
      if (shader->Source == NULL) {
         printf("File \"%s\" does not exist.\n", files[i]);
         exit(EXIT_FAILURE);
      }

      compile_shader(ctx, shader);

      if (strlen(shader->InfoLog) > 0) {
         if (!options->just_log)
            printf("Info log for %s:\n", files[i]);

         printf("%s", shader->InfoLog);
         if (!options->just_log)
            printf("\n");
      }

      if (!shader->CompileStatus) {
         status = EXIT_FAILURE;
         break;
      }
   }

   if (status == EXIT_SUCCESS) {
      _mesa_clear_shader_program_data(ctx, whole_program);

      if (options->do_link)  {
         link_shaders(ctx, whole_program);
      } else {
         const gl_shader_stage stage = whole_program->Shaders[0]->Stage;

         whole_program->data->LinkStatus = LINKING_SUCCESS;
         whole_program->_LinkedShaders[stage] =
            link_intrastage_shaders(whole_program /* mem_ctx */,
                                    ctx,
                                    whole_program,
                                    whole_program->Shaders,
                                    1,
                                    true);

         /* Par-linking can fail, for example, if there are undefined external
          * references.
          */
         if (whole_program->_LinkedShaders[stage] != NULL) {
            assert(whole_program->data->LinkStatus);

            struct gl_shader_compiler_options *const compiler_options =
               &ctx->Const.ShaderCompilerOptions[stage];

            exec_list *const ir =
               whole_program->_LinkedShaders[stage]->ir;

            bool progress;
            do {
               progress = do_function_inlining(ir);

               progress = do_common_optimization(ir,
                                                 false,
                                                 false,
                                                 compiler_options,
                                                 true)
                  && progress;
            } while(progress);
         }
      }

      status = (whole_program->data->LinkStatus) ? EXIT_SUCCESS : EXIT_FAILURE;

      if (strlen(whole_program->data->InfoLog) > 0) {
         printf("\n");
         if (!options->just_log)
            printf("Info log for linking:\n");
         printf("%s", whole_program->data->InfoLog);
         if (!options->just_log)
            printf("\n");
      }

      for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
         struct gl_linked_shader *shader = whole_program->_LinkedShaders[i];

         if (!shader)
            continue;

         add_neg_to_sub_visitor v;
         visit_list_elements(&v, shader->ir);

         dead_variable_visitor dv;
         visit_list_elements(&dv, shader->ir);
         dv.remove_dead_variables();
      }

      if (options->dump_builder) {
         for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
            struct gl_linked_shader *shader = whole_program->_LinkedShaders[i];

            if (!shader)
               continue;

            _mesa_print_builder_for_ir(stdout, shader->ir);
         }
      }
   }

   return whole_program;

fail:
   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      if (whole_program->_LinkedShaders[i])
         ralloc_free(whole_program->_LinkedShaders[i]->Program);
   }

   ralloc_free(whole_program);
   return NULL;
}

extern "C" void
standalone_compiler_cleanup(struct gl_shader_program *whole_program)
{
   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      if (whole_program->_LinkedShaders[i])
         ralloc_free(whole_program->_LinkedShaders[i]->Program);
   }

   delete whole_program->AttributeBindings;
   delete whole_program->FragDataBindings;
   delete whole_program->FragDataIndexBindings;

   ralloc_free(whole_program);
   _mesa_glsl_builtin_functions_decref();
}
