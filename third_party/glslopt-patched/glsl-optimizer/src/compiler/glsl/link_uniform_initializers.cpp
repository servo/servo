/*
 * Copyright © 2012 Intel Corporation
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

#include "ir.h"
#include "linker.h"
#include "ir_uniform.h"
#include "string_to_uint_map.h"
#include "main/mtypes.h"

/* These functions are put in a "private" namespace instead of being marked
 * static so that the unit tests can access them.  See
 * http://code.google.com/p/googletest/wiki/AdvancedGuide#Testing_Private_Code
 */
namespace linker {

static gl_uniform_storage *
get_storage(struct gl_shader_program *prog, const char *name)
{
   unsigned id;
   if (prog->UniformHash->get(id, name))
      return &prog->data->UniformStorage[id];

   assert(!"No uniform storage found!");
   return NULL;
}

void
copy_constant_to_storage(union gl_constant_value *storage,
                         const ir_constant *val,
                         const enum glsl_base_type base_type,
                         const unsigned int elements,
                         unsigned int boolean_true)
{
   for (unsigned int i = 0; i < elements; i++) {
      switch (base_type) {
      case GLSL_TYPE_UINT:
         storage[i].u = val->value.u[i];
         break;
      case GLSL_TYPE_INT:
      case GLSL_TYPE_SAMPLER:
         storage[i].i = val->value.i[i];
         break;
      case GLSL_TYPE_FLOAT:
         storage[i].f = val->value.f[i];
         break;
      case GLSL_TYPE_DOUBLE:
      case GLSL_TYPE_UINT64:
      case GLSL_TYPE_INT64:
         /* XXX need to check on big-endian */
         memcpy(&storage[i * 2].u, &val->value.d[i], sizeof(double));
         break;
      case GLSL_TYPE_BOOL:
         storage[i].b = val->value.b[i] ? boolean_true : 0;
         break;
      case GLSL_TYPE_ARRAY:
      case GLSL_TYPE_STRUCT:
      case GLSL_TYPE_IMAGE:
      case GLSL_TYPE_ATOMIC_UINT:
      case GLSL_TYPE_INTERFACE:
      case GLSL_TYPE_VOID:
      case GLSL_TYPE_SUBROUTINE:
      case GLSL_TYPE_FUNCTION:
      case GLSL_TYPE_ERROR:
      case GLSL_TYPE_UINT16:
      case GLSL_TYPE_INT16:
      case GLSL_TYPE_UINT8:
      case GLSL_TYPE_INT8:
      case GLSL_TYPE_FLOAT16:
         /* All other types should have already been filtered by other
          * paths in the caller.
          */
         assert(!"Should not get here.");
         break;
      }
   }
}

/**
 * Initialize an opaque uniform from the value of an explicit binding
 * qualifier specified in the shader.  Atomic counters are different because
 * they have no storage and should be handled elsewhere.
 */
static void
set_opaque_binding(void *mem_ctx, gl_shader_program *prog,
                   const ir_variable *var, const glsl_type *type,
                   const char *name, int *binding)
{

   if (type->is_array() && type->fields.array->is_array()) {
      const glsl_type *const element_type = type->fields.array;

      for (unsigned int i = 0; i < type->length; i++) {
         const char *element_name = ralloc_asprintf(mem_ctx, "%s[%d]", name, i);

         set_opaque_binding(mem_ctx, prog, var, element_type,
                            element_name, binding);
      }
   } else {
      struct gl_uniform_storage *const storage = get_storage(prog, name);

      if (!storage)
         return;

      const unsigned elements = MAX2(storage->array_elements, 1);

      /* Section 4.4.6 (Opaque-Uniform Layout Qualifiers) of the GLSL 4.50 spec
       * says:
       *
       *     "If the binding identifier is used with an array, the first element
       *     of the array takes the specified unit and each subsequent element
       *     takes the next consecutive unit."
       */
      for (unsigned int i = 0; i < elements; i++) {
         storage->storage[i].i = (*binding)++;
      }

      for (int sh = 0; sh < MESA_SHADER_STAGES; sh++) {
         gl_linked_shader *shader = prog->_LinkedShaders[sh];

         if (!shader)
            continue;
         if (!storage->opaque[sh].active)
            continue;

         if (storage->type->is_sampler()) {
            for (unsigned i = 0; i < elements; i++) {
               const unsigned index = storage->opaque[sh].index + i;

               if (var->data.bindless) {
                  if (index >= shader->Program->sh.NumBindlessSamplers)
                     break;
                  shader->Program->sh.BindlessSamplers[index].unit =
                     storage->storage[i].i;
                  shader->Program->sh.BindlessSamplers[index].bound = true;
                  shader->Program->sh.HasBoundBindlessSampler = true;
               } else {
                  if (index >= ARRAY_SIZE(shader->Program->SamplerUnits))
                     break;
                  shader->Program->SamplerUnits[index] =
                     storage->storage[i].i;
               }
            }
         } else if (storage->type->is_image()) {
            for (unsigned i = 0; i < elements; i++) {
               const unsigned index = storage->opaque[sh].index + i;


               if (var->data.bindless) {
                  if (index >= shader->Program->sh.NumBindlessImages)
                     break;
                  shader->Program->sh.BindlessImages[index].unit =
                     storage->storage[i].i;
                  shader->Program->sh.BindlessImages[index].bound = true;
                  shader->Program->sh.HasBoundBindlessImage = true;
               } else {
                  if (index >= ARRAY_SIZE(shader->Program->sh.ImageUnits))
                     break;
                  shader->Program->sh.ImageUnits[index] =
                     storage->storage[i].i;
               }
            }
         }
      }
   }
}

void
set_uniform_initializer(void *mem_ctx, gl_shader_program *prog,
                        const char *name, const glsl_type *type,
                        ir_constant *val, unsigned int boolean_true)
{
   const glsl_type *t_without_array = type->without_array();
   if (type->is_struct()) {
      for (unsigned int i = 0; i < type->length; i++) {
         const glsl_type *field_type = type->fields.structure[i].type;
         const char *field_name = ralloc_asprintf(mem_ctx, "%s.%s", name,
                                            type->fields.structure[i].name);
         set_uniform_initializer(mem_ctx, prog, field_name,
                                 field_type, val->get_record_field(i),
                                 boolean_true);
      }
      return;
   } else if (t_without_array->is_struct() ||
              (type->is_array() && type->fields.array->is_array())) {
      const glsl_type *const element_type = type->fields.array;

      for (unsigned int i = 0; i < type->length; i++) {
         const char *element_name = ralloc_asprintf(mem_ctx, "%s[%d]", name, i);

         set_uniform_initializer(mem_ctx, prog, element_name,
                                 element_type, val->const_elements[i],
                                 boolean_true);
      }
      return;
   }

   struct gl_uniform_storage *const storage = get_storage(prog, name);

   if (!storage)
      return;

   if (val->type->is_array()) {
      const enum glsl_base_type base_type =
         val->const_elements[0]->type->base_type;
      const unsigned int elements = val->const_elements[0]->type->components();
      unsigned int idx = 0;
      unsigned dmul = glsl_base_type_is_64bit(base_type) ? 2 : 1;

      assert(val->type->length >= storage->array_elements);
      for (unsigned int i = 0; i < storage->array_elements; i++) {
         copy_constant_to_storage(& storage->storage[idx],
                                  val->const_elements[i],
                                  base_type,
                                  elements,
                                  boolean_true);

         idx += elements * dmul;
      }
   } else {
      copy_constant_to_storage(storage->storage,
                               val,
                               val->type->base_type,
                               val->type->components(),
                               boolean_true);

      if (storage->type->is_sampler()) {
         for (int sh = 0; sh < MESA_SHADER_STAGES; sh++) {
            gl_linked_shader *shader = prog->_LinkedShaders[sh];

            if (shader && storage->opaque[sh].active) {
               unsigned index = storage->opaque[sh].index;

               shader->Program->SamplerUnits[index] = storage->storage[0].i;
            }
         }
      }
   }
}
}

void
link_set_uniform_initializers(struct gl_shader_program *prog,
                              unsigned int boolean_true)
{
   void *mem_ctx = NULL;

   for (unsigned int i = 0; i < MESA_SHADER_STAGES; i++) {
      struct gl_linked_shader *shader = prog->_LinkedShaders[i];

      if (shader == NULL)
         continue;

      foreach_in_list(ir_instruction, node, shader->ir) {
         ir_variable *const var = node->as_variable();

         if (!var || (var->data.mode != ir_var_uniform &&
             var->data.mode != ir_var_shader_storage))
            continue;

         if (!mem_ctx)
            mem_ctx = ralloc_context(NULL);

         if (var->data.explicit_binding) {
            const glsl_type *const type = var->type;

            if (var->is_in_buffer_block()) {
               /* This case is handled by link_uniform_blocks (at
                * process_block_array_leaf)
                */
            } else if (type->without_array()->is_sampler() ||
                type->without_array()->is_image()) {
               int binding = var->data.binding;
               linker::set_opaque_binding(mem_ctx, prog, var, var->type,
                                          var->name, &binding);
            } else if (type->contains_atomic()) {
               /* we don't actually need to do anything. */
            } else {
               assert(!"Explicit binding not on a sampler, UBO or atomic.");
            }
         } else if (var->constant_initializer) {
            linker::set_uniform_initializer(mem_ctx, prog, var->name,
                                            var->type, var->constant_initializer,
                                            boolean_true);
         }
      }
   }

   memcpy(prog->data->UniformDataDefaults, prog->data->UniformDataSlots,
          sizeof(union gl_constant_value) * prog->data->NumUniformDataSlots);
   ralloc_free(mem_ctx);
}
