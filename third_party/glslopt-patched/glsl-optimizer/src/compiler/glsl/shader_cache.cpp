/*
 * Copyright © 2014 Intel Corporation
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
 * \file shader_cache.cpp
 *
 * GLSL shader cache implementation
 *
 * This uses disk_cache.c to write out a serialization of various
 * state that's required in order to successfully load and use a
 * binary written out by a drivers backend, this state is referred to as
 * "metadata" throughout the implementation.
 *
 * The hash key for glsl metadata is a hash of the hashes of each GLSL
 * source string as well as some API settings that change the final program
 * such as SSO, attribute bindings, frag data bindings, etc.
 *
 * In order to avoid caching any actual IR we use the put_key/get_key support
 * in the disk_cache to put the SHA-1 hash for each successfully compiled
 * shader into the cache, and optimisticly return early from glCompileShader
 * (if the identical shader had been successfully compiled in the past),
 * in the hope that the final linked shader will be found in the cache.
 * If anything goes wrong (shader variant not found, backend cache item is
 * corrupt, etc) we will use a fallback path to compile and link the IR.
 */

#include "compiler/shader_info.h"
#include "glsl_symbol_table.h"
#include "glsl_parser_extras.h"
#include "ir.h"
#include "ir_optimization.h"
#include "ir_rvalue_visitor.h"
#include "ir_uniform.h"
#include "linker.h"
#include "link_varyings.h"
#include "program.h"
#include "serialize.h"
#include "shader_cache.h"
#include "util/mesa-sha1.h"
#include "string_to_uint_map.h"
#include "main/mtypes.h"

extern "C" {
#include "main/enums.h"
#include "main/shaderobj.h"
#include "program/program.h"
}

static void
compile_shaders(struct gl_context *ctx, struct gl_shader_program *prog) {
   for (unsigned i = 0; i < prog->NumShaders; i++) {
      _mesa_glsl_compile_shader(ctx, prog->Shaders[i], false, false, true);
   }
}

static void
create_binding_str(const char *key, unsigned value, void *closure)
{
   char **bindings_str = (char **) closure;
   ralloc_asprintf_append(bindings_str, "%s:%u,", key, value);
}

void
shader_cache_write_program_metadata(struct gl_context *ctx,
                                    struct gl_shader_program *prog)
{
   struct disk_cache *cache = ctx->Cache;
   if (!cache)
      return;

   /* Exit early when we are dealing with a ff shader with no source file to
    * generate a source from, or with a SPIR-V shader.
    *
    * TODO: In future we should use another method to generate a key for ff
    * programs, and SPIR-V shaders.
    */
   static const char zero[sizeof(prog->data->sha1)] = {0};
   if (memcmp(prog->data->sha1, zero, sizeof(prog->data->sha1)) == 0)
      return;

   struct blob metadata;
   blob_init(&metadata);

   if (ctx->Driver.ShaderCacheSerializeDriverBlob) {
      for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
         struct gl_linked_shader *sh = prog->_LinkedShaders[i];
         if (sh)
            ctx->Driver.ShaderCacheSerializeDriverBlob(ctx, sh->Program);
      }
   }

   serialize_glsl_program(&metadata, ctx, prog);

   struct cache_item_metadata cache_item_metadata;
   cache_item_metadata.type = CACHE_ITEM_TYPE_GLSL;
   cache_item_metadata.keys =
      (cache_key *) malloc(prog->NumShaders * sizeof(cache_key));
   cache_item_metadata.num_keys = prog->NumShaders;

   if (!cache_item_metadata.keys)
      goto fail;

   for (unsigned i = 0; i < prog->NumShaders; i++) {
      memcpy(cache_item_metadata.keys[i], prog->Shaders[i]->sha1,
             sizeof(cache_key));
   }

   disk_cache_put(cache, prog->data->sha1, metadata.data, metadata.size,
                  &cache_item_metadata);

   char sha1_buf[41];
   if (ctx->_Shader->Flags & GLSL_CACHE_INFO) {
      _mesa_sha1_format(sha1_buf, prog->data->sha1);
      fprintf(stderr, "putting program metadata in cache: %s\n", sha1_buf);
   }

fail:
   free(cache_item_metadata.keys);
   blob_finish(&metadata);
}

bool
shader_cache_read_program_metadata(struct gl_context *ctx,
                                   struct gl_shader_program *prog)
{
   /* Fixed function programs generated by Mesa, or SPIR-V shaders, are not
    * cached. So don't try to read metadata for them from the cache.
    */
   if (prog->Name == 0 || prog->data->spirv)
      return false;

   struct disk_cache *cache = ctx->Cache;
   if (!cache)
      return false;

   /* Include bindings when creating sha1. These bindings change the resulting
    * binary so they are just as important as the shader source.
    */
   char *buf = ralloc_strdup(NULL, "vb: ");
   prog->AttributeBindings->iterate(create_binding_str, &buf);
   ralloc_strcat(&buf, "fb: ");
   prog->FragDataBindings->iterate(create_binding_str, &buf);
   ralloc_strcat(&buf, "fbi: ");
   prog->FragDataIndexBindings->iterate(create_binding_str, &buf);
   ralloc_asprintf_append(&buf, "tf: %d ", prog->TransformFeedback.BufferMode);
   for (unsigned int i = 0; i < prog->TransformFeedback.NumVarying; i++) {
      ralloc_asprintf_append(&buf, "%s ",
                             prog->TransformFeedback.VaryingNames[i]);
   }

   /* SSO has an effect on the linked program so include this when generating
    * the sha also.
    */
   ralloc_asprintf_append(&buf, "sso: %s\n",
                          prog->SeparateShader ? "T" : "F");

   /* A shader might end up producing different output depending on the glsl
    * version supported by the compiler. For example a different path might be
    * taken by the preprocessor, so add the version to the hash input.
    */
   ralloc_asprintf_append(&buf, "api: %d glsl: %d fglsl: %d\n",
                          ctx->API, ctx->Const.GLSLVersion,
                          ctx->Const.ForceGLSLVersion);

   /* We run the preprocessor on shaders after hashing them, so we need to
    * add any extension override vars to the hash. If we don't do this the
    * preprocessor could result in different output and we could load the
    * wrong shader.
    */
   char *ext_override = getenv("MESA_EXTENSION_OVERRIDE");
   if (ext_override) {
      ralloc_asprintf_append(&buf, "ext:%s", ext_override);
   }

   /* DRI config options may also change the output from the compiler so
    * include them as an input to sha1 creation.
    */
   char sha1buf[41];
   _mesa_sha1_format(sha1buf, ctx->Const.dri_config_options_sha1);
   ralloc_strcat(&buf, sha1buf);

   for (unsigned i = 0; i < prog->NumShaders; i++) {
      struct gl_shader *sh = prog->Shaders[i];
      _mesa_sha1_format(sha1buf, sh->sha1);
      ralloc_asprintf_append(&buf, "%s: %s\n",
                             _mesa_shader_stage_to_abbrev(sh->Stage), sha1buf);
   }
   disk_cache_compute_key(cache, buf, strlen(buf), prog->data->sha1);
   ralloc_free(buf);

   size_t size;
   uint8_t *buffer = (uint8_t *) disk_cache_get(cache, prog->data->sha1,
                                                &size);
   if (buffer == NULL) {
      /* Cached program not found. We may have seen the individual shaders
       * before and skipped compiling but they may not have been used together
       * in this combination before. Fall back to linking shaders but first
       * re-compile the shaders.
       *
       * We could probably only compile the shaders which were skipped here
       * but we need to be careful because the source may also have been
       * changed since the last compile so for now we just recompile
       * everything.
       */
      compile_shaders(ctx, prog);
      return false;
   }

   if (ctx->_Shader->Flags & GLSL_CACHE_INFO) {
      _mesa_sha1_format(sha1buf, prog->data->sha1);
      fprintf(stderr, "loading shader program meta data from cache: %s\n",
              sha1buf);
   }

   struct blob_reader metadata;
   blob_reader_init(&metadata, buffer, size);

   bool deserialized = deserialize_glsl_program(&metadata, ctx, prog);

   if (!deserialized || metadata.current != metadata.end || metadata.overrun) {
      /* Something has gone wrong discard the item from the cache and rebuild
       * from source.
       */
      assert(!"Invalid GLSL shader disk cache item!");

      if (ctx->_Shader->Flags & GLSL_CACHE_INFO) {
         fprintf(stderr, "Error reading program from cache (invalid GLSL "
                 "cache item)\n");
      }

      disk_cache_remove(cache, prog->data->sha1);
      compile_shaders(ctx, prog);
      free(buffer);
      return false;
   }

   /* This is used to flag a shader retrieved from cache */
   prog->data->LinkStatus = LINKING_SKIPPED;

   free (buffer);

   return true;
}
