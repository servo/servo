/*
 * Copyright © 2018 Intel Corporation
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
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE.
 *
 */
#include "main/mtypes.h"
#include "glsl_types.h"
#include "linker_util.h"
#include "util/bitscan.h"
#include "util/set.h"
#include "ir_uniform.h" /* for gl_uniform_storage */

/* Utility methods shared between the GLSL IR and the NIR */

/* From the OpenGL 4.6 specification, 7.3.1.1 Naming Active Resources:
 *
 *    "For an active shader storage block member declared as an array of an
 *     aggregate type, an entry will be generated only for the first array
 *     element, regardless of its type. Such block members are referred to as
 *     top-level arrays. If the block member is an aggregate type, the
 *     enumeration rules are then applied recursively."
 */
bool
link_util_should_add_buffer_variable(struct gl_shader_program *prog,
                                     struct gl_uniform_storage *uniform,
                                     int top_level_array_base_offset,
                                     int top_level_array_size_in_bytes,
                                     int second_element_offset,
                                     int block_index)
{
   /* If the uniform is not a shader storage buffer or is not an array return
    * true.
    */
   if (!uniform->is_shader_storage || top_level_array_size_in_bytes == 0)
      return true;

   int after_top_level_array = top_level_array_base_offset +
      top_level_array_size_in_bytes;

   /* Check for a new block, or that we are not dealing with array elements of
    * a top member array other than the first element.
    */
   if (block_index != uniform->block_index ||
       uniform->offset >= after_top_level_array ||
       uniform->offset < second_element_offset) {
      return true;
   }

   return false;
}

bool
link_util_add_program_resource(struct gl_shader_program *prog,
                               struct set *resource_set,
                               GLenum type, const void *data, uint8_t stages)
{
   assert(data);

   /* If resource already exists, do not add it again. */
   if (_mesa_set_search(resource_set, data))
      return true;

   prog->data->ProgramResourceList =
      reralloc(prog->data,
               prog->data->ProgramResourceList,
               gl_program_resource,
               prog->data->NumProgramResourceList + 1);

   if (!prog->data->ProgramResourceList) {
      linker_error(prog, "Out of memory during linking.\n");
      return false;
   }

   struct gl_program_resource *res =
      &prog->data->ProgramResourceList[prog->data->NumProgramResourceList];

   res->Type = type;
   res->Data = data;
   res->StageReferences = stages;

   prog->data->NumProgramResourceList++;

   _mesa_set_add(resource_set, data);

   return true;
}

/**
 * Search through the list of empty blocks to find one that fits the current
 * uniform.
 */
int
link_util_find_empty_block(struct gl_shader_program *prog,
                           struct gl_uniform_storage *uniform)
{
   const unsigned entries = MAX2(1, uniform->array_elements);

   foreach_list_typed(struct empty_uniform_block, block, link,
                      &prog->EmptyUniformLocations) {
      /* Found a block with enough slots to fit the uniform */
      if (block->slots == entries) {
         unsigned start = block->start;
         exec_node_remove(&block->link);
         ralloc_free(block);

         return start;
      /* Found a block with more slots than needed. It can still be used. */
      } else if (block->slots > entries) {
         unsigned start = block->start;
         block->start += entries;
         block->slots -= entries;

         return start;
      }
   }

   return -1;
}

void
link_util_update_empty_uniform_locations(struct gl_shader_program *prog)
{
   struct empty_uniform_block *current_block = NULL;

   for (unsigned i = 0; i < prog->NumUniformRemapTable; i++) {
      /* We found empty space in UniformRemapTable. */
      if (prog->UniformRemapTable[i] == NULL) {
         /* We've found the beginning of a new continous block of empty slots */
         if (!current_block || current_block->start + current_block->slots != i) {
            current_block = rzalloc(prog, struct empty_uniform_block);
            current_block->start = i;
            exec_list_push_tail(&prog->EmptyUniformLocations,
                                &current_block->link);
         }

         /* The current block continues, so we simply increment its slots */
         current_block->slots++;
      }
   }
}

void
link_util_check_subroutine_resources(struct gl_shader_program *prog)
{
   unsigned mask = prog->data->linked_stages;
   while (mask) {
      const int i = u_bit_scan(&mask);
      struct gl_program *p = prog->_LinkedShaders[i]->Program;

      if (p->sh.NumSubroutineUniformRemapTable > MAX_SUBROUTINE_UNIFORM_LOCATIONS) {
         linker_error(prog, "Too many %s shader subroutine uniforms\n",
                      _mesa_shader_stage_to_string(i));
      }
   }
}

/**
 * Validate uniform resources used by a program versus the implementation limits
 */
void
link_util_check_uniform_resources(struct gl_context *ctx,
                                  struct gl_shader_program *prog)
{
   unsigned total_uniform_blocks = 0;
   unsigned total_shader_storage_blocks = 0;

   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      struct gl_linked_shader *sh = prog->_LinkedShaders[i];

      if (sh == NULL)
         continue;

      if (sh->num_uniform_components >
          ctx->Const.Program[i].MaxUniformComponents) {
         if (ctx->Const.GLSLSkipStrictMaxUniformLimitCheck) {
            linker_warning(prog, "Too many %s shader default uniform block "
                           "components, but the driver will try to optimize "
                           "them out; this is non-portable out-of-spec "
                           "behavior\n",
                           _mesa_shader_stage_to_string(i));
         } else {
            linker_error(prog, "Too many %s shader default uniform block "
                         "components\n",
                         _mesa_shader_stage_to_string(i));
         }
      }

      if (sh->num_combined_uniform_components >
          ctx->Const.Program[i].MaxCombinedUniformComponents) {
         if (ctx->Const.GLSLSkipStrictMaxUniformLimitCheck) {
            linker_warning(prog, "Too many %s shader uniform components, "
                           "but the driver will try to optimize them out; "
                           "this is non-portable out-of-spec behavior\n",
                           _mesa_shader_stage_to_string(i));
         } else {
            linker_error(prog, "Too many %s shader uniform components\n",
                         _mesa_shader_stage_to_string(i));
         }
      }

      total_shader_storage_blocks += sh->Program->info.num_ssbos;
      total_uniform_blocks += sh->Program->info.num_ubos;
   }

   if (total_uniform_blocks > ctx->Const.MaxCombinedUniformBlocks) {
      linker_error(prog, "Too many combined uniform blocks (%d/%d)\n",
                   total_uniform_blocks, ctx->Const.MaxCombinedUniformBlocks);
   }

   if (total_shader_storage_blocks > ctx->Const.MaxCombinedShaderStorageBlocks) {
      linker_error(prog, "Too many combined shader storage blocks (%d/%d)\n",
                   total_shader_storage_blocks,
                   ctx->Const.MaxCombinedShaderStorageBlocks);
   }

   for (unsigned i = 0; i < prog->data->NumUniformBlocks; i++) {
      if (prog->data->UniformBlocks[i].UniformBufferSize >
          ctx->Const.MaxUniformBlockSize) {
         linker_error(prog, "Uniform block %s too big (%d/%d)\n",
                      prog->data->UniformBlocks[i].Name,
                      prog->data->UniformBlocks[i].UniformBufferSize,
                      ctx->Const.MaxUniformBlockSize);
      }
   }

   for (unsigned i = 0; i < prog->data->NumShaderStorageBlocks; i++) {
      if (prog->data->ShaderStorageBlocks[i].UniformBufferSize >
          ctx->Const.MaxShaderStorageBlockSize) {
         linker_error(prog, "Shader storage block %s too big (%d/%d)\n",
                      prog->data->ShaderStorageBlocks[i].Name,
                      prog->data->ShaderStorageBlocks[i].UniformBufferSize,
                      ctx->Const.MaxShaderStorageBlockSize);
      }
   }
}

void
link_util_calculate_subroutine_compat(struct gl_shader_program *prog)
{
   unsigned mask = prog->data->linked_stages;
   while (mask) {
      const int i = u_bit_scan(&mask);
      struct gl_program *p = prog->_LinkedShaders[i]->Program;

      for (unsigned j = 0; j < p->sh.NumSubroutineUniformRemapTable; j++) {
         if (p->sh.SubroutineUniformRemapTable[j] == INACTIVE_UNIFORM_EXPLICIT_LOCATION)
            continue;

         struct gl_uniform_storage *uni = p->sh.SubroutineUniformRemapTable[j];

         if (!uni)
            continue;

         int count = 0;
         if (p->sh.NumSubroutineFunctions == 0) {
            linker_error(prog, "subroutine uniform %s defined but no valid functions found\n", uni->type->name);
            continue;
         }
         for (unsigned f = 0; f < p->sh.NumSubroutineFunctions; f++) {
            struct gl_subroutine_function *fn = &p->sh.SubroutineFunctions[f];
            for (int k = 0; k < fn->num_compat_types; k++) {
               if (fn->types[k] == uni->type) {
                  count++;
                  break;
               }
            }
         }
         uni->num_compatible_subroutines = count;
      }
   }
}

/**
 * Recursive part of the public mark_array_elements_referenced function.
 *
 * The recursion occurs when an entire array-of- is accessed.  See the
 * implementation for more details.
 *
 * \param dr                List of array_deref_range elements to be
 *                          processed.
 * \param count             Number of array_deref_range elements to be
 *                          processed.
 * \param scale             Current offset scale.
 * \param linearized_index  Current accumulated linearized array index.
 */
void
_mark_array_elements_referenced(const struct array_deref_range *dr,
                                unsigned count, unsigned scale,
                                unsigned linearized_index,
                                BITSET_WORD *bits)
{
   /* Walk through the list of array dereferences in least- to
    * most-significant order.  Along the way, accumulate the current
    * linearized offset and the scale factor for each array-of-.
    */
   for (unsigned i = 0; i < count; i++) {
      if (dr[i].index < dr[i].size) {
         linearized_index += dr[i].index * scale;
         scale *= dr[i].size;
      } else {
         /* For each element in the current array, update the count and
          * offset, then recurse to process the remaining arrays.
          *
          * There is some inefficency here if the last eBITSET_WORD *bitslement in the
          * array_deref_range list specifies the entire array.  In that case,
          * the loop will make recursive calls with count == 0.  In the call,
          * all that will happen is the bit will be set.
          */
         for (unsigned j = 0; j < dr[i].size; j++) {
            _mark_array_elements_referenced(&dr[i + 1],
                                            count - (i + 1),
                                            scale * dr[i].size,
                                            linearized_index + (j * scale),
                                            bits);
         }

         return;
      }
   }

   BITSET_SET(bits, linearized_index);
}

/**
 * Mark a set of array elements as accessed.
 *
 * If every \c array_deref_range is for a single index, only a single
 * element will be marked.  If any \c array_deref_range is for an entire
 * array-of-, then multiple elements will be marked.
 *
 * Items in the \c array_deref_range list appear in least- to
 * most-significant order.  This is the \b opposite order the indices
 * appear in the GLSL shader text.  An array access like
 *
 *     x = y[1][i][3];
 *
 * would appear as
 *
 *     { { 3, n }, { m, m }, { 1, p } }
 *
 * where n, m, and p are the sizes of the arrays-of-arrays.
 *
 * The set of marked array elements can later be queried by
 * \c ::is_linearized_index_referenced.
 *
 * \param dr     List of array_deref_range elements to be processed.
 * \param count  Number of array_deref_range elements to be processed.
 */
void
link_util_mark_array_elements_referenced(const struct array_deref_range *dr,
                                         unsigned count, unsigned array_depth,
                                         BITSET_WORD *bits)
{
   if (count != array_depth)
      return;

   _mark_array_elements_referenced(dr, count, 1, 0, bits);
}
