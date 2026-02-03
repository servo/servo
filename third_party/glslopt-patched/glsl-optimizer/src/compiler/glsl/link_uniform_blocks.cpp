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
#include "link_uniform_block_active_visitor.h"
#include "util/hash_table.h"
#include "program.h"
#include "main/errors.h"
#include "main/mtypes.h"

namespace {

class ubo_visitor : public program_resource_visitor {
public:
   ubo_visitor(void *mem_ctx, gl_uniform_buffer_variable *variables,
               unsigned num_variables, struct gl_shader_program *prog,
               bool use_std430_as_default)
      : index(0), offset(0), buffer_size(0), variables(variables),
        num_variables(num_variables), mem_ctx(mem_ctx),
        is_array_instance(false), prog(prog),
        use_std430_as_default(use_std430_as_default)
   {
      /* empty */
   }

   void process(const glsl_type *type, const char *name)
   {
      this->offset = 0;
      this->buffer_size = 0;
      this->is_array_instance = strchr(name, ']') != NULL;
      this->program_resource_visitor::process(type, name,
                                              use_std430_as_default);
   }

   unsigned index;
   unsigned offset;
   unsigned buffer_size;
   gl_uniform_buffer_variable *variables;
   unsigned num_variables;
   void *mem_ctx;
   bool is_array_instance;
   struct gl_shader_program *prog;

private:
   virtual void enter_record(const glsl_type *type, const char *,
                             bool row_major,
                             const enum glsl_interface_packing packing)
   {
      assert(type->is_struct());
      if (packing == GLSL_INTERFACE_PACKING_STD430)
         this->offset = glsl_align(
            this->offset, type->std430_base_alignment(row_major));
      else
         this->offset = glsl_align(
            this->offset, type->std140_base_alignment(row_major));
   }

   virtual void leave_record(const glsl_type *type, const char *,
                             bool row_major,
                             const enum glsl_interface_packing packing)
   {
      assert(type->is_struct());

      /* If this is the last field of a structure, apply rule #9.  The
       * ARB_uniform_buffer_object spec says:
       *
       *    The structure may have padding at the end; the base offset of the
       *    member following the sub-structure is rounded up to the next
       *    multiple of the base alignment of the structure.
       */
      if (packing == GLSL_INTERFACE_PACKING_STD430)
         this->offset = glsl_align(
            this->offset, type->std430_base_alignment(row_major));
      else
         this->offset = glsl_align(
            this->offset, type->std140_base_alignment(row_major));
   }

   virtual void set_buffer_offset(unsigned offset)
   {
      this->offset = offset;
   }

   virtual void visit_field(const glsl_type *type, const char *name,
                            bool row_major, const glsl_type *,
                            const enum glsl_interface_packing packing,
                            bool last_field)
   {
      assert(this->index < this->num_variables);

      gl_uniform_buffer_variable *v = &this->variables[this->index++];

      v->Name = ralloc_strdup(mem_ctx, name);
      v->Type = type;
      v->RowMajor = type->without_array()->is_matrix() && row_major;

      if (this->is_array_instance) {
         v->IndexName = ralloc_strdup(mem_ctx, name);

         char *open_bracket = strchr(v->IndexName, '[');
         assert(open_bracket != NULL);

         char *close_bracket = strchr(open_bracket, '.') - 1;
         assert(close_bracket != NULL);

         /* Length of the tail without the ']' but with the NUL.
          */
         unsigned len = strlen(close_bracket + 1) + 1;

         memmove(open_bracket, close_bracket + 1, len);
      } else {
         v->IndexName = v->Name;
      }

      unsigned alignment = 0;
      unsigned size = 0;

      /* The ARB_program_interface_query spec says:
       *
       *    If the final member of an active shader storage block is array
       *    with no declared size, the minimum buffer size is computed
       *    assuming the array was declared as an array with one element.
       *
       * For that reason, we use the base type of the unsized array to
       * calculate its size. We don't need to check if the unsized array is
       * the last member of a shader storage block (that check was already
       * done by the parser).
       */
      const glsl_type *type_for_size = type;
      if (type->is_unsized_array()) {
         if (!last_field) {
            linker_error(prog, "unsized array `%s' definition: "
                         "only last member of a shader storage block "
                         "can be defined as unsized array",
                         name);
         }

         type_for_size = type->without_array();
      }

      if (packing == GLSL_INTERFACE_PACKING_STD430) {
         alignment = type->std430_base_alignment(v->RowMajor);
         size = type_for_size->std430_size(v->RowMajor);
      } else {
         alignment = type->std140_base_alignment(v->RowMajor);
         size = type_for_size->std140_size(v->RowMajor);
      }

      this->offset = glsl_align(this->offset, alignment);
      v->Offset = this->offset;

      this->offset += size;

      /* The ARB_uniform_buffer_object spec says:
       *
       *    For uniform blocks laid out according to [std140] rules, the
       *    minimum buffer object size returned by the UNIFORM_BLOCK_DATA_SIZE
       *    query is derived by taking the offset of the last basic machine
       *    unit consumed by the last uniform of the uniform block (including
       *    any end-of-array or end-of-structure padding), adding one, and
       *    rounding up to the next multiple of the base alignment required
       *    for a vec4.
       */
      this->buffer_size = glsl_align(this->offset, 16);
   }

   bool use_std430_as_default;
};

class count_block_size : public program_resource_visitor {
public:
   count_block_size() : num_active_uniforms(0)
   {
      /* empty */
   }

   unsigned num_active_uniforms;

private:
   virtual void visit_field(const glsl_type * /* type */,
                            const char * /* name */,
                            bool /* row_major */,
                            const glsl_type * /* record_type */,
                            const enum glsl_interface_packing,
                            bool /* last_field */)
   {
      this->num_active_uniforms++;
   }
};

} /* anonymous namespace */

struct block {
   const glsl_type *type;
   bool has_instance_name;
};

static void process_block_array_leaf(const char *name, gl_uniform_block *blocks,
                                     ubo_visitor *parcel,
                                     gl_uniform_buffer_variable *variables,
                                     const struct link_uniform_block_active *const b,
                                     unsigned *block_index,
                                     unsigned binding_offset,
                                     unsigned linearized_index,
                                     struct gl_context *ctx,
                                     struct gl_shader_program *prog);

/**
 *
 * \param first_index Value of \c block_index for the first element of the
 *                    array.
 */
static void
process_block_array(struct uniform_block_array_elements *ub_array, char **name,
                    size_t name_length, gl_uniform_block *blocks,
                    ubo_visitor *parcel, gl_uniform_buffer_variable *variables,
                    const struct link_uniform_block_active *const b,
                    unsigned *block_index, unsigned binding_offset,
                    struct gl_context *ctx, struct gl_shader_program *prog,
                    unsigned first_index)
{
   for (unsigned j = 0; j < ub_array->num_array_elements; j++) {
      size_t new_length = name_length;

      unsigned int element_idx = ub_array->array_elements[j];
      /* Append the subscript to the current variable name */
      ralloc_asprintf_rewrite_tail(name, &new_length, "[%u]", element_idx);

      if (ub_array->array) {
         unsigned binding_stride = binding_offset + (element_idx *
                                   ub_array->array->aoa_size);
         process_block_array(ub_array->array, name, new_length, blocks,
                             parcel, variables, b, block_index,
                             binding_stride, ctx, prog, first_index);
      } else {
         process_block_array_leaf(*name, blocks,
                                  parcel, variables, b, block_index,
                                  binding_offset + element_idx,
                                  *block_index - first_index, ctx, prog);
      }
   }
}

static void
process_block_array_leaf(const char *name,
                         gl_uniform_block *blocks,
                         ubo_visitor *parcel, gl_uniform_buffer_variable *variables,
                         const struct link_uniform_block_active *const b,
                         unsigned *block_index, unsigned binding_offset,
                         unsigned linearized_index,
                         struct gl_context *ctx, struct gl_shader_program *prog)
{
   unsigned i = *block_index;
   const glsl_type *type =  b->type->without_array();

   blocks[i].Name = ralloc_strdup(blocks, name);
   blocks[i].Uniforms = &variables[(*parcel).index];

   /* The ARB_shading_language_420pack spec says:
    *
    *    If the binding identifier is used with a uniform block instanced as
    *    an array then the first element of the array takes the specified
    *    block binding and each subsequent element takes the next consecutive
    *    uniform block binding point.
    */
   blocks[i].Binding = (b->has_binding) ? b->binding + binding_offset : 0;

   blocks[i].UniformBufferSize = 0;
   blocks[i]._Packing = glsl_interface_packing(type->interface_packing);
   blocks[i]._RowMajor = type->get_interface_row_major();
   blocks[i].linearized_array_index = linearized_index;

   parcel->process(type, b->has_instance_name ? blocks[i].Name : "");

   blocks[i].UniformBufferSize = parcel->buffer_size;

   /* Check SSBO size is lower than maximum supported size for SSBO */
   if (b->is_shader_storage &&
       parcel->buffer_size > ctx->Const.MaxShaderStorageBlockSize) {
      linker_error(prog, "shader storage block `%s' has size %d, "
                   "which is larger than the maximum allowed (%d)",
                   b->type->name,
                   parcel->buffer_size,
                   ctx->Const.MaxShaderStorageBlockSize);
   }
   blocks[i].NumUniforms =
      (unsigned)(ptrdiff_t)(&variables[parcel->index] - blocks[i].Uniforms);

   *block_index = *block_index + 1;
}

/* This function resizes the array types of the block so that later we can use
 * this new size to correctly calculate the offest for indirect indexing.
 */
static const glsl_type *
resize_block_array(const glsl_type *type,
                   struct uniform_block_array_elements *ub_array)
{
   if (type->is_array()) {
      struct uniform_block_array_elements *child_array =
         type->fields.array->is_array() ? ub_array->array : NULL;
      const glsl_type *new_child_type =
         resize_block_array(type->fields.array, child_array);

      const glsl_type *new_type =
         glsl_type::get_array_instance(new_child_type,
                                       ub_array->num_array_elements);
      ub_array->ir->array->type = new_type;
      return new_type;
   } else {
      return type;
   }
}

static void
create_buffer_blocks(void *mem_ctx, struct gl_context *ctx,
                     struct gl_shader_program *prog,
                     struct gl_uniform_block **out_blks, unsigned num_blocks,
                     struct hash_table *block_hash, unsigned num_variables,
                     bool create_ubo_blocks)
{
   if (num_blocks == 0) {
      assert(num_variables == 0);
      return;
   }

   assert(num_variables != 0);

   /* Allocate storage to hold all of the information related to uniform
    * blocks that can be queried through the API.
    */
   struct gl_uniform_block *blocks =
      rzalloc_array(mem_ctx, gl_uniform_block, num_blocks);
   gl_uniform_buffer_variable *variables =
      ralloc_array(blocks, gl_uniform_buffer_variable, num_variables);

   /* Add each variable from each uniform block to the API tracking
    * structures.
    */
   ubo_visitor parcel(blocks, variables, num_variables, prog,
                      ctx->Const.UseSTD430AsDefaultPacking);

   unsigned i = 0;
   hash_table_foreach (block_hash, entry) {
      const struct link_uniform_block_active *const b =
         (const struct link_uniform_block_active *) entry->data;
      const glsl_type *block_type = b->type;

      if ((create_ubo_blocks && !b->is_shader_storage) ||
          (!create_ubo_blocks && b->is_shader_storage)) {

         if (b->array != NULL) {
            char *name = ralloc_strdup(NULL,
                                       block_type->without_array()->name);
            size_t name_length = strlen(name);

            assert(b->has_instance_name);
            process_block_array(b->array, &name, name_length, blocks, &parcel,
                                variables, b, &i, 0, ctx, prog,
                                i);
            ralloc_free(name);
         } else {
            process_block_array_leaf(block_type->name, blocks, &parcel,
                                     variables, b, &i, 0,
                                     0, ctx, prog);
         }
      }
   }

   *out_blks = blocks;

   assert(parcel.index == num_variables);
}

void
link_uniform_blocks(void *mem_ctx,
                    struct gl_context *ctx,
                    struct gl_shader_program *prog,
                    struct gl_linked_shader *shader,
                    struct gl_uniform_block **ubo_blocks,
                    unsigned *num_ubo_blocks,
                    struct gl_uniform_block **ssbo_blocks,
                    unsigned *num_ssbo_blocks)
{
   /* This hash table will track all of the uniform blocks that have been
    * encountered.  Since blocks with the same block-name must be the same,
    * the hash is organized by block-name.
    */
   struct hash_table *block_hash =
      _mesa_hash_table_create(mem_ctx, _mesa_hash_string,
                              _mesa_key_string_equal);

   if (block_hash == NULL) {
      _mesa_error_no_memory(__func__);
      linker_error(prog, "out of memory\n");
      return;
   }

   /* Determine which uniform blocks are active. */
   link_uniform_block_active_visitor v(mem_ctx, block_hash, prog);
   visit_list_elements(&v, shader->ir);

   /* Count the number of active uniform blocks.  Count the total number of
    * active slots in those uniform blocks.
    */
   unsigned num_ubo_variables = 0;
   unsigned num_ssbo_variables = 0;
   count_block_size block_size;

   hash_table_foreach (block_hash, entry) {
      struct link_uniform_block_active *const b =
         (struct link_uniform_block_active *) entry->data;

      assert((b->array != NULL) == b->type->is_array());

      if (b->array != NULL &&
          (b->type->without_array()->interface_packing ==
           GLSL_INTERFACE_PACKING_PACKED)) {
         b->type = resize_block_array(b->type, b->array);
         b->var->type = b->type;
         b->var->data.max_array_access = b->type->length - 1;
      }

      block_size.num_active_uniforms = 0;
      block_size.process(b->type->without_array(), "",
                         ctx->Const.UseSTD430AsDefaultPacking);

      if (b->array != NULL) {
         unsigned aoa_size = b->type->arrays_of_arrays_size();
         if (b->is_shader_storage) {
            *num_ssbo_blocks += aoa_size;
            num_ssbo_variables += aoa_size * block_size.num_active_uniforms;
         } else {
            *num_ubo_blocks += aoa_size;
            num_ubo_variables += aoa_size * block_size.num_active_uniforms;
         }
      } else {
         if (b->is_shader_storage) {
            (*num_ssbo_blocks)++;
            num_ssbo_variables += block_size.num_active_uniforms;
         } else {
            (*num_ubo_blocks)++;
            num_ubo_variables += block_size.num_active_uniforms;
         }
      }

   }

   create_buffer_blocks(mem_ctx, ctx, prog, ubo_blocks, *num_ubo_blocks,
                        block_hash, num_ubo_variables, true);
   create_buffer_blocks(mem_ctx, ctx, prog, ssbo_blocks, *num_ssbo_blocks,
                        block_hash, num_ssbo_variables, false);

   _mesa_hash_table_destroy(block_hash, NULL);
}

static bool
link_uniform_blocks_are_compatible(const gl_uniform_block *a,
                                   const gl_uniform_block *b)
{
   assert(strcmp(a->Name, b->Name) == 0);

   /* Page 35 (page 42 of the PDF) in section 4.3.7 of the GLSL 1.50 spec says:
    *
    *    Matched block names within an interface (as defined above) must match
    *    in terms of having the same number of declarations with the same
    *    sequence of types and the same sequence of member names, as well as
    *    having the same member-wise layout qualification....if a matching
    *    block is declared as an array, then the array sizes must also
    *    match... Any mismatch will generate a link error.
    *
    * Arrays are not yet supported, so there is no check for that.
    */
   if (a->NumUniforms != b->NumUniforms)
      return false;

   if (a->_Packing != b->_Packing)
      return false;

   if (a->_RowMajor != b->_RowMajor)
      return false;

   if (a->Binding != b->Binding)
      return false;

   for (unsigned i = 0; i < a->NumUniforms; i++) {
      if (strcmp(a->Uniforms[i].Name, b->Uniforms[i].Name) != 0)
         return false;

      if (a->Uniforms[i].Type != b->Uniforms[i].Type)
         return false;

      if (a->Uniforms[i].RowMajor != b->Uniforms[i].RowMajor)
         return false;
   }

   return true;
}

/**
 * Merges a uniform block into an array of uniform blocks that may or
 * may not already contain a copy of it.
 *
 * Returns the index of the new block in the array.
 */
int
link_cross_validate_uniform_block(void *mem_ctx,
                                  struct gl_uniform_block **linked_blocks,
                                  unsigned int *num_linked_blocks,
                                  struct gl_uniform_block *new_block)
{
   for (unsigned int i = 0; i < *num_linked_blocks; i++) {
      struct gl_uniform_block *old_block = &(*linked_blocks)[i];

      if (strcmp(old_block->Name, new_block->Name) == 0)
         return link_uniform_blocks_are_compatible(old_block, new_block)
            ? i : -1;
   }

   *linked_blocks = reralloc(mem_ctx, *linked_blocks,
                             struct gl_uniform_block,
                             *num_linked_blocks + 1);
   int linked_block_index = (*num_linked_blocks)++;
   struct gl_uniform_block *linked_block = &(*linked_blocks)[linked_block_index];

   memcpy(linked_block, new_block, sizeof(*new_block));
   linked_block->Uniforms = ralloc_array(*linked_blocks,
                                         struct gl_uniform_buffer_variable,
                                         linked_block->NumUniforms);

   memcpy(linked_block->Uniforms,
          new_block->Uniforms,
          sizeof(*linked_block->Uniforms) * linked_block->NumUniforms);

   linked_block->Name = ralloc_strdup(*linked_blocks, linked_block->Name);

   for (unsigned int i = 0; i < linked_block->NumUniforms; i++) {
      struct gl_uniform_buffer_variable *ubo_var =
         &linked_block->Uniforms[i];

      if (ubo_var->Name == ubo_var->IndexName) {
         ubo_var->Name = ralloc_strdup(*linked_blocks, ubo_var->Name);
         ubo_var->IndexName = ubo_var->Name;
      } else {
         ubo_var->Name = ralloc_strdup(*linked_blocks, ubo_var->Name);
         ubo_var->IndexName = ralloc_strdup(*linked_blocks, ubo_var->IndexName);
      }
   }

   return linked_block_index;
}
