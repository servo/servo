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

#include "glsl_parser_extras.h"
#include "ir.h"
#include "ir_uniform.h"
#include "linker.h"
#include "main/errors.h"
#include "main/macros.h"
#include "main/mtypes.h"

namespace {
   /*
    * Atomic counter uniform as seen by the program.
    */
   struct active_atomic_counter_uniform {
      unsigned uniform_loc;
      ir_variable *var;
   };

   /*
    * Atomic counter buffer referenced by the program.  There is a one
    * to one correspondence between these and the objects that can be
    * queried using glGetActiveAtomicCounterBufferiv().
    */
   struct active_atomic_buffer {
      active_atomic_buffer()
         : uniforms(0), num_uniforms(0), stage_counter_references(), size(0)
      {}

      ~active_atomic_buffer()
      {
         free(uniforms);
      }

      void push_back(unsigned uniform_loc, ir_variable *var)
      {
         active_atomic_counter_uniform *new_uniforms;

         new_uniforms = (active_atomic_counter_uniform *)
            realloc(uniforms, sizeof(active_atomic_counter_uniform) *
                    (num_uniforms + 1));

         if (new_uniforms == NULL) {
            _mesa_error_no_memory(__func__);
            return;
         }

         uniforms = new_uniforms;
         uniforms[num_uniforms].uniform_loc = uniform_loc;
         uniforms[num_uniforms].var = var;
         num_uniforms++;
      }

      active_atomic_counter_uniform *uniforms;
      unsigned num_uniforms;
      unsigned stage_counter_references[MESA_SHADER_STAGES];
      unsigned size;
   };

   int
   cmp_actives(const void *a, const void *b)
   {
      const active_atomic_counter_uniform *const first = (active_atomic_counter_uniform *) a;
      const active_atomic_counter_uniform *const second = (active_atomic_counter_uniform *) b;

      return int(first->var->data.offset) - int(second->var->data.offset);
   }

   bool
   check_atomic_counters_overlap(const ir_variable *x, const ir_variable *y)
   {
      return ((x->data.offset >= y->data.offset &&
               x->data.offset < y->data.offset + y->type->atomic_size()) ||
              (y->data.offset >= x->data.offset &&
               y->data.offset < x->data.offset + x->type->atomic_size()));
   }

   void
   process_atomic_variable(const glsl_type *t, struct gl_shader_program *prog,
                           unsigned *uniform_loc, ir_variable *var,
                           active_atomic_buffer *const buffers,
                           unsigned *num_buffers, int *offset,
                           const unsigned shader_stage)
   {
      /* FIXME: Arrays of arrays get counted separately. For example:
       * x1[3][3][2] = 9 uniforms, 18 atomic counters
       * x2[3][2]    = 3 uniforms, 6 atomic counters
       * x3[2]       = 1 uniform, 2 atomic counters
       *
       * However this code marks all the counters as active even when they
       * might not be used.
       */
      if (t->is_array() && t->fields.array->is_array()) {
         for (unsigned i = 0; i < t->length; i++) {
            process_atomic_variable(t->fields.array, prog, uniform_loc,
                                    var, buffers, num_buffers, offset,
                                    shader_stage);
         }
      } else {
         active_atomic_buffer *buf = &buffers[var->data.binding];
         gl_uniform_storage *const storage =
            &prog->data->UniformStorage[*uniform_loc];

         /* If this is the first time the buffer is used, increment
          * the counter of buffers used.
          */
         if (buf->size == 0)
            (*num_buffers)++;

         buf->push_back(*uniform_loc, var);

         /* When checking for atomic counters we should count every member in
          * an array as an atomic counter reference.
          */
         if (t->is_array())
            buf->stage_counter_references[shader_stage] += t->length;
         else
            buf->stage_counter_references[shader_stage]++;
         buf->size = MAX2(buf->size, *offset + t->atomic_size());

         storage->offset = *offset;
         *offset += t->atomic_size();

         (*uniform_loc)++;
      }
   }

   active_atomic_buffer *
   find_active_atomic_counters(struct gl_context *ctx,
                               struct gl_shader_program *prog,
                               unsigned *num_buffers)
   {
      active_atomic_buffer *const buffers =
         new active_atomic_buffer[ctx->Const.MaxAtomicBufferBindings];

      *num_buffers = 0;

      for (unsigned i = 0; i < MESA_SHADER_STAGES; ++i) {
         struct gl_linked_shader *sh = prog->_LinkedShaders[i];
         if (sh == NULL)
            continue;

         foreach_in_list(ir_instruction, node, sh->ir) {
            ir_variable *var = node->as_variable();

            if (var && var->type->contains_atomic()) {
               int offset = var->data.offset;
               unsigned uniform_loc = var->data.location;
               process_atomic_variable(var->type, prog, &uniform_loc,
                                       var, buffers, num_buffers, &offset, i);
            }
         }
      }

      for (unsigned i = 0; i < ctx->Const.MaxAtomicBufferBindings; i++) {
         if (buffers[i].size == 0)
            continue;

         qsort(buffers[i].uniforms, buffers[i].num_uniforms,
               sizeof(active_atomic_counter_uniform),
               cmp_actives);

         for (unsigned j = 1; j < buffers[i].num_uniforms; j++) {
            /* If an overlapping counter found, it must be a reference to the
             * same counter from a different shader stage.
             */
            if (check_atomic_counters_overlap(buffers[i].uniforms[j-1].var,
                                              buffers[i].uniforms[j].var)
                && strcmp(buffers[i].uniforms[j-1].var->name,
                          buffers[i].uniforms[j].var->name) != 0) {
               linker_error(prog, "Atomic counter %s declared at offset %d "
                            "which is already in use.",
                            buffers[i].uniforms[j].var->name,
                            buffers[i].uniforms[j].var->data.offset);
            }
         }
      }
      return buffers;
   }
}

void
link_assign_atomic_counter_resources(struct gl_context *ctx,
                                     struct gl_shader_program *prog)
{
   unsigned num_buffers;
   unsigned num_atomic_buffers[MESA_SHADER_STAGES] = {};
   active_atomic_buffer *abs =
      find_active_atomic_counters(ctx, prog, &num_buffers);

   prog->data->AtomicBuffers = rzalloc_array(prog->data, gl_active_atomic_buffer,
                                             num_buffers);
   prog->data->NumAtomicBuffers = num_buffers;

   unsigned i = 0;
   for (unsigned binding = 0;
        binding < ctx->Const.MaxAtomicBufferBindings;
        binding++) {

      /* If the binding was not used, skip.
       */
      if (abs[binding].size == 0)
         continue;

      active_atomic_buffer &ab = abs[binding];
      gl_active_atomic_buffer &mab = prog->data->AtomicBuffers[i];

      /* Assign buffer-specific fields. */
      mab.Binding = binding;
      mab.MinimumSize = ab.size;
      mab.Uniforms = rzalloc_array(prog->data->AtomicBuffers, GLuint,
                                   ab.num_uniforms);
      mab.NumUniforms = ab.num_uniforms;

      /* Assign counter-specific fields. */
      for (unsigned j = 0; j < ab.num_uniforms; j++) {
         ir_variable *const var = ab.uniforms[j].var;
         gl_uniform_storage *const storage =
            &prog->data->UniformStorage[ab.uniforms[j].uniform_loc];

         mab.Uniforms[j] = ab.uniforms[j].uniform_loc;
         if (!var->data.explicit_binding)
            var->data.binding = i;

         storage->atomic_buffer_index = i;
         storage->offset = var->data.offset;
         storage->array_stride = (var->type->is_array() ?
                                  var->type->without_array()->atomic_size() : 0);
         if (!var->type->is_matrix())
            storage->matrix_stride = 0;
      }

      /* Assign stage-specific fields. */
      for (unsigned j = 0; j < MESA_SHADER_STAGES; ++j) {
         if (ab.stage_counter_references[j]) {
            mab.StageReferences[j] = GL_TRUE;
            num_atomic_buffers[j]++;
         } else {
            mab.StageReferences[j] = GL_FALSE;
         }
      }

      i++;
   }

   /* Store a list pointers to atomic buffers per stage and store the index
    * to the intra-stage buffer list in uniform storage.
    */
   for (unsigned j = 0; j < MESA_SHADER_STAGES; ++j) {
      if (prog->_LinkedShaders[j] && num_atomic_buffers[j] > 0) {
         struct gl_program *gl_prog = prog->_LinkedShaders[j]->Program;
         gl_prog->info.num_abos = num_atomic_buffers[j];
         gl_prog->sh.AtomicBuffers =
            rzalloc_array(gl_prog, gl_active_atomic_buffer *,
                          num_atomic_buffers[j]);

         unsigned intra_stage_idx = 0;
         for (unsigned i = 0; i < num_buffers; i++) {
            struct gl_active_atomic_buffer *atomic_buffer =
               &prog->data->AtomicBuffers[i];
            if (atomic_buffer->StageReferences[j]) {
               gl_prog->sh.AtomicBuffers[intra_stage_idx] = atomic_buffer;

               for (unsigned u = 0; u < atomic_buffer->NumUniforms; u++) {
                  prog->data->UniformStorage[atomic_buffer->Uniforms[u]].opaque[j].index =
                     intra_stage_idx;
                  prog->data->UniformStorage[atomic_buffer->Uniforms[u]].opaque[j].active =
                     true;
               }

               intra_stage_idx++;
            }
         }
      }
   }

   delete [] abs;
   assert(i == num_buffers);
}

void
link_check_atomic_counter_resources(struct gl_context *ctx,
                                    struct gl_shader_program *prog)
{
   unsigned num_buffers;
   active_atomic_buffer *const abs =
      find_active_atomic_counters(ctx, prog, &num_buffers);
   unsigned atomic_counters[MESA_SHADER_STAGES] = {};
   unsigned atomic_buffers[MESA_SHADER_STAGES] = {};
   unsigned total_atomic_counters = 0;
   unsigned total_atomic_buffers = 0;

   /* Sum the required resources.  Note that this counts buffers and
    * counters referenced by several shader stages multiple times
    * against the combined limit -- That's the behavior the spec
    * requires.
    */
   for (unsigned i = 0; i < ctx->Const.MaxAtomicBufferBindings; i++) {
      if (abs[i].size == 0)
         continue;

      for (unsigned j = 0; j < MESA_SHADER_STAGES; ++j) {
         const unsigned n = abs[i].stage_counter_references[j];

         if (n) {
            atomic_counters[j] += n;
            total_atomic_counters += n;
            atomic_buffers[j]++;
            total_atomic_buffers++;
         }
      }
   }

   /* Check that they are within the supported limits. */
   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      if (atomic_counters[i] > ctx->Const.Program[i].MaxAtomicCounters)
         linker_error(prog, "Too many %s shader atomic counters",
                      _mesa_shader_stage_to_string(i));

      if (atomic_buffers[i] > ctx->Const.Program[i].MaxAtomicBuffers)
         linker_error(prog, "Too many %s shader atomic counter buffers",
                      _mesa_shader_stage_to_string(i));
   }

   if (total_atomic_counters > ctx->Const.MaxCombinedAtomicCounters)
      linker_error(prog, "Too many combined atomic counters");

   if (total_atomic_buffers > ctx->Const.MaxCombinedAtomicBuffers)
      linker_error(prog, "Too many combined atomic buffers");

   delete [] abs;
}
