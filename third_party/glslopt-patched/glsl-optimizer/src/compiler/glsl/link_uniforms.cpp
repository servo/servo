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

#include "ir.h"
#include "linker.h"
#include "ir_uniform.h"
#include "glsl_symbol_table.h"
#include "program.h"
#include "string_to_uint_map.h"
#include "ir_array_refcount.h"

#include "main/mtypes.h"
#include "util/strndup.h"

/**
 * \file link_uniforms.cpp
 * Assign locations for GLSL uniforms.
 *
 * \author Ian Romanick <ian.d.romanick@intel.com>
 */

/**
 * Used by linker to indicate uniforms that have no location set.
 */
#define UNMAPPED_UNIFORM_LOC ~0u

static char*
get_top_level_name(const char *name)
{
   const char *first_dot = strchr(name, '.');
   const char *first_square_bracket = strchr(name, '[');
   int name_size = 0;

   /* The ARB_program_interface_query spec says:
    *
    *     "For the property TOP_LEVEL_ARRAY_SIZE, a single integer identifying
    *     the number of active array elements of the top-level shader storage
    *     block member containing to the active variable is written to
    *     <params>.  If the top-level block member is not declared as an
    *     array, the value one is written to <params>.  If the top-level block
    *     member is an array with no declared size, the value zero is written
    *     to <params>."
    */

   /* The buffer variable is on top level.*/
   if (!first_square_bracket && !first_dot)
      name_size = strlen(name);
   else if ((!first_square_bracket ||
            (first_dot && first_dot < first_square_bracket)))
      name_size = first_dot - name;
   else
      name_size = first_square_bracket - name;

   return strndup(name, name_size);
}

static char*
get_var_name(const char *name)
{
   const char *first_dot = strchr(name, '.');

   if (!first_dot)
      return strdup(name);

   return strndup(first_dot+1, strlen(first_dot) - 1);
}

static bool
is_top_level_shader_storage_block_member(const char* name,
                                         const char* interface_name,
                                         const char* field_name)
{
   bool result = false;

   /* If the given variable is already a top-level shader storage
    * block member, then return array_size = 1.
    * We could have two possibilities: if we have an instanced
    * shader storage block or not instanced.
    *
    * For the first, we check create a name as it was in top level and
    * compare it with the real name. If they are the same, then
    * the variable is already at top-level.
    *
    * Full instanced name is: interface name + '.' + var name +
    *    NULL character
    */
   int name_length = strlen(interface_name) + 1 + strlen(field_name) + 1;
   char *full_instanced_name = (char *) calloc(name_length, sizeof(char));
   if (!full_instanced_name) {
      fprintf(stderr, "%s: Cannot allocate space for name\n", __func__);
      return false;
   }

   snprintf(full_instanced_name, name_length, "%s.%s",
            interface_name, field_name);

   /* Check if its top-level shader storage block member of an
    * instanced interface block, or of a unnamed interface block.
    */
   if (strcmp(name, full_instanced_name) == 0 ||
       strcmp(name, field_name) == 0)
      result = true;

   free(full_instanced_name);
   return result;
}

static int
get_array_size(struct gl_uniform_storage *uni, const glsl_struct_field *field,
               char *interface_name, char *var_name)
{
   /* The ARB_program_interface_query spec says:
    *
    *     "For the property TOP_LEVEL_ARRAY_SIZE, a single integer identifying
    *     the number of active array elements of the top-level shader storage
    *     block member containing to the active variable is written to
    *     <params>.  If the top-level block member is not declared as an
    *     array, the value one is written to <params>.  If the top-level block
    *     member is an array with no declared size, the value zero is written
    *     to <params>."
    */
   if (is_top_level_shader_storage_block_member(uni->name,
                                                interface_name,
                                                var_name))
      return  1;
   else if (field->type->is_array())
      return field->type->length;

   return 1;
}

static int
get_array_stride(struct gl_uniform_storage *uni, const glsl_type *iface,
                 const glsl_struct_field *field, char *interface_name,
                 char *var_name, bool use_std430_as_default)
{
   /* The ARB_program_interface_query spec says:
    *
    *     "For the property TOP_LEVEL_ARRAY_STRIDE, a single integer
    *     identifying the stride between array elements of the top-level
    *     shader storage block member containing the active variable is
    *     written to <params>.  For top-level block members declared as
    *     arrays, the value written is the difference, in basic machine units,
    *     between the offsets of the active variable for consecutive elements
    *     in the top-level array.  For top-level block members not declared as
    *     an array, zero is written to <params>."
    */
   if (field->type->is_array()) {
      const enum glsl_matrix_layout matrix_layout =
         glsl_matrix_layout(field->matrix_layout);
      bool row_major = matrix_layout == GLSL_MATRIX_LAYOUT_ROW_MAJOR;
      const glsl_type *array_type = field->type->fields.array;

      if (is_top_level_shader_storage_block_member(uni->name,
                                                   interface_name,
                                                   var_name))
         return 0;

      if (GLSL_INTERFACE_PACKING_STD140 ==
          iface->get_internal_ifc_packing(use_std430_as_default)) {
         if (array_type->is_struct() || array_type->is_array())
            return glsl_align(array_type->std140_size(row_major), 16);
         else
            return MAX2(array_type->std140_base_alignment(row_major), 16);
      } else {
         return array_type->std430_array_stride(row_major);
      }
   }
   return 0;
}

static void
calculate_array_size_and_stride(struct gl_shader_program *shProg,
                                struct gl_uniform_storage *uni,
                                bool use_std430_as_default)
{
   if (!uni->is_shader_storage)
      return;

   int block_index = uni->block_index;
   int array_size = -1;
   int array_stride = -1;
   char *var_name = get_top_level_name(uni->name);
   char *interface_name =
      get_top_level_name(uni->is_shader_storage ?
                         shProg->data->ShaderStorageBlocks[block_index].Name :
                         shProg->data->UniformBlocks[block_index].Name);

   if (strcmp(var_name, interface_name) == 0) {
      /* Deal with instanced array of SSBOs */
      char *temp_name = get_var_name(uni->name);
      if (!temp_name) {
         linker_error(shProg, "Out of memory during linking.\n");
         goto write_top_level_array_size_and_stride;
      }
      free(var_name);
      var_name = get_top_level_name(temp_name);
      free(temp_name);
      if (!var_name) {
         linker_error(shProg, "Out of memory during linking.\n");
         goto write_top_level_array_size_and_stride;
      }
   }

   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      const gl_linked_shader *sh = shProg->_LinkedShaders[i];
      if (sh == NULL)
         continue;

      foreach_in_list(ir_instruction, node, sh->ir) {
         ir_variable *var = node->as_variable();
         if (!var || !var->get_interface_type() ||
             var->data.mode != ir_var_shader_storage)
            continue;

         const glsl_type *iface = var->get_interface_type();

         if (strcmp(interface_name, iface->name) != 0)
            continue;

         for (unsigned i = 0; i < iface->length; i++) {
            const glsl_struct_field *field = &iface->fields.structure[i];
            if (strcmp(field->name, var_name) != 0)
               continue;

            array_stride = get_array_stride(uni, iface, field, interface_name,
                                            var_name, use_std430_as_default);
            array_size = get_array_size(uni, field, interface_name, var_name);
            goto write_top_level_array_size_and_stride;
         }
      }
   }
write_top_level_array_size_and_stride:
   free(interface_name);
   free(var_name);
   uni->top_level_array_stride = array_stride;
   uni->top_level_array_size = array_size;
}

void
program_resource_visitor::process(const glsl_type *type, const char *name,
                                  bool use_std430_as_default)
{
   assert(type->without_array()->is_struct()
          || type->without_array()->is_interface());

   unsigned record_array_count = 1;
   char *name_copy = ralloc_strdup(NULL, name);

   enum glsl_interface_packing packing =
      type->get_internal_ifc_packing(use_std430_as_default);

   recursion(type, &name_copy, strlen(name), false, NULL, packing, false,
             record_array_count, NULL);
   ralloc_free(name_copy);
}

void
program_resource_visitor::process(ir_variable *var, bool use_std430_as_default)
{
   const glsl_type *t =
      var->data.from_named_ifc_block ? var->get_interface_type() : var->type;
   process(var, t, use_std430_as_default);
}

void
program_resource_visitor::process(ir_variable *var, const glsl_type *var_type,
                                  bool use_std430_as_default)
{
   unsigned record_array_count = 1;
   const bool row_major =
      var->data.matrix_layout == GLSL_MATRIX_LAYOUT_ROW_MAJOR;

   enum glsl_interface_packing packing = var->get_interface_type() ?
      var->get_interface_type()->
         get_internal_ifc_packing(use_std430_as_default) :
      var->type->get_internal_ifc_packing(use_std430_as_default);

   const glsl_type *t = var_type;
   const glsl_type *t_without_array = t->without_array();

   /* false is always passed for the row_major parameter to the other
    * processing functions because no information is available to do
    * otherwise.  See the warning in linker.h.
    */
   if (t_without_array->is_struct() ||
              (t->is_array() && t->fields.array->is_array())) {
      char *name = ralloc_strdup(NULL, var->name);
      recursion(var->type, &name, strlen(name), row_major, NULL, packing,
                false, record_array_count, NULL);
      ralloc_free(name);
   } else if (t_without_array->is_interface()) {
      char *name = ralloc_strdup(NULL, t_without_array->name);
      const glsl_struct_field *ifc_member = var->data.from_named_ifc_block ?
         &t_without_array->
            fields.structure[t_without_array->field_index(var->name)] : NULL;

      recursion(t, &name, strlen(name), row_major, NULL, packing,
                false, record_array_count, ifc_member);
      ralloc_free(name);
   } else {
      this->set_record_array_count(record_array_count);
      this->visit_field(t, var->name, row_major, NULL, packing, false);
   }
}

void
program_resource_visitor::recursion(const glsl_type *t, char **name,
                                    size_t name_length, bool row_major,
                                    const glsl_type *record_type,
                                    const enum glsl_interface_packing packing,
                                    bool last_field,
                                    unsigned record_array_count,
                                    const glsl_struct_field *named_ifc_member)
{
   /* Records need to have each field processed individually.
    *
    * Arrays of records need to have each array element processed
    * individually, then each field of the resulting array elements processed
    * individually.
    */
   if (t->is_interface() && named_ifc_member) {
      ralloc_asprintf_rewrite_tail(name, &name_length, ".%s",
                                   named_ifc_member->name);
      recursion(named_ifc_member->type, name, name_length, row_major, NULL,
                packing, false, record_array_count, NULL);
   } else if (t->is_struct() || t->is_interface()) {
      if (record_type == NULL && t->is_struct())
         record_type = t;

      if (t->is_struct())
         this->enter_record(t, *name, row_major, packing);

      for (unsigned i = 0; i < t->length; i++) {
         const char *field = t->fields.structure[i].name;
         size_t new_length = name_length;

         if (t->is_interface() && t->fields.structure[i].offset != -1)
            this->set_buffer_offset(t->fields.structure[i].offset);

         /* Append '.field' to the current variable name. */
         if (name_length == 0) {
            ralloc_asprintf_rewrite_tail(name, &new_length, "%s", field);
         } else {
            ralloc_asprintf_rewrite_tail(name, &new_length, ".%s", field);
         }

         /* The layout of structures at the top level of the block is set
          * during parsing.  For matrices contained in multiple levels of
          * structures in the block, the inner structures have no layout.
          * These cases must potentially inherit the layout from the outer
          * levels.
          */
         bool field_row_major = row_major;
         const enum glsl_matrix_layout matrix_layout =
            glsl_matrix_layout(t->fields.structure[i].matrix_layout);
         if (matrix_layout == GLSL_MATRIX_LAYOUT_ROW_MAJOR) {
            field_row_major = true;
         } else if (matrix_layout == GLSL_MATRIX_LAYOUT_COLUMN_MAJOR) {
            field_row_major = false;
         }

         recursion(t->fields.structure[i].type, name, new_length,
                   field_row_major,
                   record_type,
                   packing,
                   (i + 1) == t->length, record_array_count, NULL);

         /* Only the first leaf-field of the record gets called with the
          * record type pointer.
          */
         record_type = NULL;
      }

      if (t->is_struct()) {
         (*name)[name_length] = '\0';
         this->leave_record(t, *name, row_major, packing);
      }
   } else if (t->without_array()->is_struct() ||
              t->without_array()->is_interface() ||
              (t->is_array() && t->fields.array->is_array())) {
      if (record_type == NULL && t->fields.array->is_struct())
         record_type = t->fields.array;

      unsigned length = t->length;

      /* Shader storage block unsized arrays: add subscript [0] to variable
       * names.
       */
      if (t->is_unsized_array())
         length = 1;

      record_array_count *= length;

      for (unsigned i = 0; i < length; i++) {
         size_t new_length = name_length;

         /* Append the subscript to the current variable name */
         ralloc_asprintf_rewrite_tail(name, &new_length, "[%u]", i);

         recursion(t->fields.array, name, new_length, row_major,
                   record_type,
                   packing,
                   (i + 1) == t->length, record_array_count,
                   named_ifc_member);

         /* Only the first leaf-field of the record gets called with the
          * record type pointer.
          */
         record_type = NULL;
      }
   } else {
      this->set_record_array_count(record_array_count);
      this->visit_field(t, *name, row_major, record_type, packing, last_field);
   }
}

void
program_resource_visitor::enter_record(const glsl_type *, const char *, bool,
                                       const enum glsl_interface_packing)
{
}

void
program_resource_visitor::leave_record(const glsl_type *, const char *, bool,
                                       const enum glsl_interface_packing)
{
}

void
program_resource_visitor::set_buffer_offset(unsigned)
{
}

void
program_resource_visitor::set_record_array_count(unsigned)
{
}

namespace {

/**
 * Class to help calculate the storage requirements for a set of uniforms
 *
 * As uniforms are added to the active set the number of active uniforms and
 * the storage requirements for those uniforms are accumulated.  The active
 * uniforms are added to the hash table supplied to the constructor.
 *
 * If the same uniform is added multiple times (i.e., once for each shader
 * target), it will only be accounted once.
 */
class count_uniform_size : public program_resource_visitor {
public:
   count_uniform_size(struct string_to_uint_map *map,
                      struct string_to_uint_map *hidden_map,
                      bool use_std430_as_default)
      : num_active_uniforms(0), num_hidden_uniforms(0), num_values(0),
        num_shader_samplers(0), num_shader_images(0),
        num_shader_uniform_components(0), num_shader_subroutines(0),
        is_buffer_block(false), is_shader_storage(false), map(map),
        hidden_map(hidden_map), current_var(NULL),
        use_std430_as_default(use_std430_as_default)
   {
      /* empty */
   }

   void start_shader()
   {
      this->num_shader_samplers = 0;
      this->num_shader_images = 0;
      this->num_shader_uniform_components = 0;
      this->num_shader_subroutines = 0;
   }

   void process(ir_variable *var)
   {
      this->current_var = var;
      this->is_buffer_block = var->is_in_buffer_block();
      this->is_shader_storage = var->is_in_shader_storage_block();
      if (var->is_interface_instance())
         program_resource_visitor::process(var->get_interface_type(),
                                           var->get_interface_type()->name,
                                           use_std430_as_default);
      else
         program_resource_visitor::process(var, use_std430_as_default);
   }

   /**
    * Total number of active uniforms counted
    */
   unsigned num_active_uniforms;

   unsigned num_hidden_uniforms;

   /**
    * Number of data values required to back the storage for the active uniforms
    */
   unsigned num_values;

   /**
    * Number of samplers used
    */
   unsigned num_shader_samplers;

   /**
    * Number of images used
    */
   unsigned num_shader_images;

   /**
    * Number of uniforms used in the current shader
    */
   unsigned num_shader_uniform_components;

   /**
    * Number of subroutine uniforms used
    */
   unsigned num_shader_subroutines;

   bool is_buffer_block;
   bool is_shader_storage;

   struct string_to_uint_map *map;

private:
   virtual void visit_field(const glsl_type *type, const char *name,
                            bool /* row_major */,
                            const glsl_type * /* record_type */,
                            const enum glsl_interface_packing,
                            bool /* last_field */)
   {
      assert(!type->without_array()->is_struct());
      assert(!type->without_array()->is_interface());
      assert(!(type->is_array() && type->fields.array->is_array()));

      /* Count the number of samplers regardless of whether the uniform is
       * already in the hash table.  The hash table prevents adding the same
       * uniform for multiple shader targets, but in this case we want to
       * count it for each shader target.
       */
      const unsigned values = type->component_slots();
      if (type->contains_subroutine()) {
         this->num_shader_subroutines += values;
      } else if (type->contains_sampler() && !current_var->data.bindless) {
         /* Samplers (bound or bindless) are counted as two components as
          * specified by ARB_bindless_texture. */
         this->num_shader_samplers += values / 2;
      } else if (type->contains_image() && !current_var->data.bindless) {
         /* Images (bound or bindless) are counted as two components as
          * specified by ARB_bindless_texture. */
         this->num_shader_images += values / 2;

         /* As drivers are likely to represent image uniforms as
          * scalar indices, count them against the limit of uniform
          * components in the default block.  The spec allows image
          * uniforms to use up no more than one scalar slot.
          */
         if (!is_shader_storage)
            this->num_shader_uniform_components += values;
      } else {
         /* Accumulate the total number of uniform slots used by this shader.
          * Note that samplers do not count against this limit because they
          * don't use any storage on current hardware.
          */
         if (!is_buffer_block)
            this->num_shader_uniform_components += values;
      }

      /* If the uniform is already in the map, there's nothing more to do.
       */
      unsigned id;
      if (this->map->get(id, name))
         return;

      if (this->current_var->data.how_declared == ir_var_hidden) {
         this->hidden_map->put(this->num_hidden_uniforms, name);
         this->num_hidden_uniforms++;
      } else {
         this->map->put(this->num_active_uniforms-this->num_hidden_uniforms,
                        name);
      }

      /* Each leaf uniform occupies one entry in the list of active
       * uniforms.
       */
      this->num_active_uniforms++;

      if(!is_gl_identifier(name) && !is_shader_storage && !is_buffer_block)
         this->num_values += values;
   }

   struct string_to_uint_map *hidden_map;

   /**
    * Current variable being processed.
    */
   ir_variable *current_var;

   bool use_std430_as_default;
};

} /* anonymous namespace */

unsigned
link_calculate_matrix_stride(const glsl_type *matrix, bool row_major,
                             enum glsl_interface_packing packing)
{
   const unsigned N = matrix->is_double() ? 8 : 4;
   const unsigned items =
      row_major ? matrix->matrix_columns : matrix->vector_elements;

   assert(items <= 4);

   /* Matrix stride for std430 mat2xY matrices are not rounded up to
    * vec4 size.
    *
    * Section 7.6.2.2 "Standard Uniform Block Layout" of the OpenGL 4.3 spec
    * says:
    *
    *    2. If the member is a two- or four-component vector with components
    *       consuming N basic machine units, the base alignment is 2N or 4N,
    *       respectively.
    *    ...
    *    4. If the member is an array of scalars or vectors, the base
    *       alignment and array stride are set to match the base alignment of
    *       a single array element, according to rules (1), (2), and (3), and
    *       rounded up to the base alignment of a vec4.
    *    ...
    *    7. If the member is a row-major matrix with C columns and R rows, the
    *       matrix is stored identically to an array of R row vectors with C
    *       components each, according to rule (4).
    *    ...
    *
    *    When using the std430 storage layout, shader storage blocks will be
    *    laid out in buffer storage identically to uniform and shader storage
    *    blocks using the std140 layout, except that the base alignment and
    *    stride of arrays of scalars and vectors in rule 4 and of structures
    *    in rule 9 are not rounded up a multiple of the base alignment of a
    *    vec4.
    */
   return packing == GLSL_INTERFACE_PACKING_STD430
      ? (items < 3 ? items * N : glsl_align(items * N, 16))
      : glsl_align(items * N, 16);
}

/**
 * Class to help parcel out pieces of backing storage to uniforms
 *
 * Each uniform processed has some range of the \c gl_constant_value
 * structures associated with it.  The association is done by finding
 * the uniform in the \c string_to_uint_map and using the value from
 * the map to connect that slot in the \c gl_uniform_storage table
 * with the next available slot in the \c gl_constant_value array.
 *
 * \warning
 * This class assumes that every uniform that will be processed is
 * already in the \c string_to_uint_map.  In addition, it assumes that
 * the \c gl_uniform_storage and \c gl_constant_value arrays are "big
 * enough."
 */
class parcel_out_uniform_storage : public program_resource_visitor {
public:
   parcel_out_uniform_storage(struct gl_shader_program *prog,
                              struct string_to_uint_map *map,
                              struct gl_uniform_storage *uniforms,
                              union gl_constant_value *values,
                              bool use_std430_as_default)
      : prog(prog), map(map), uniforms(uniforms),
        use_std430_as_default(use_std430_as_default), values(values),
        bindless_targets(NULL), bindless_access(NULL),
        shader_storage_blocks_write_access(0)
   {
   }

   virtual ~parcel_out_uniform_storage()
   {
      free(this->bindless_targets);
      free(this->bindless_access);
   }

   void start_shader(gl_shader_stage shader_type)
   {
      assert(shader_type < MESA_SHADER_STAGES);
      this->shader_type = shader_type;

      this->shader_samplers_used = 0;
      this->shader_shadow_samplers = 0;
      this->next_sampler = 0;
      this->next_image = 0;
      this->next_subroutine = 0;
      this->record_array_count = 1;
      memset(this->targets, 0, sizeof(this->targets));

      this->num_bindless_samplers = 0;
      this->next_bindless_sampler = 0;
      free(this->bindless_targets);
      this->bindless_targets = NULL;

      this->num_bindless_images = 0;
      this->next_bindless_image = 0;
      free(this->bindless_access);
      this->bindless_access = NULL;
      this->shader_storage_blocks_write_access = 0;
   }

   void set_and_process(ir_variable *var)
   {
      current_var = var;
      field_counter = 0;
      this->record_next_sampler = new string_to_uint_map;
      this->record_next_bindless_sampler = new string_to_uint_map;
      this->record_next_image = new string_to_uint_map;
      this->record_next_bindless_image = new string_to_uint_map;

      buffer_block_index = -1;
      if (var->is_in_buffer_block()) {
         struct gl_uniform_block *blks = var->is_in_shader_storage_block() ?
            prog->data->ShaderStorageBlocks : prog->data->UniformBlocks;
         unsigned num_blks = var->is_in_shader_storage_block() ?
            prog->data->NumShaderStorageBlocks : prog->data->NumUniformBlocks;
         bool is_interface_array =
            var->is_interface_instance() && var->type->is_array();

         if (is_interface_array) {
            unsigned l = strlen(var->get_interface_type()->name);

            for (unsigned i = 0; i < num_blks; i++) {
               if (strncmp(var->get_interface_type()->name, blks[i].Name, l)
                   == 0 && blks[i].Name[l] == '[') {
                  buffer_block_index = i;
                  break;
               }
            }
         } else {
            for (unsigned i = 0; i < num_blks; i++) {
               if (strcmp(var->get_interface_type()->name, blks[i].Name) == 0) {
                  buffer_block_index = i;
                  break;
               }
            }
         }
         assert(buffer_block_index != -1);

         if (var->is_in_shader_storage_block() &&
             !var->data.memory_read_only) {
            unsigned array_size = is_interface_array ?
                                     var->type->array_size() : 1;

            STATIC_ASSERT(MAX_SHADER_STORAGE_BUFFERS <= 32);

            /* Shaders that use too many SSBOs will fail to compile, which
             * we don't care about.
             *
             * This is true for shaders that do not use too many SSBOs:
             */
            if (buffer_block_index + array_size <= 32) {
               shader_storage_blocks_write_access |=
                  u_bit_consecutive(buffer_block_index, array_size);
            }
         }

         /* Uniform blocks that were specified with an instance name must be
          * handled a little bit differently.  The name of the variable is the
          * name used to reference the uniform block instead of being the name
          * of a variable within the block.  Therefore, searching for the name
          * within the block will fail.
          */
         if (var->is_interface_instance()) {
            ubo_byte_offset = 0;
            process(var->get_interface_type(),
                    var->get_interface_type()->name,
                    use_std430_as_default);
         } else {
            const struct gl_uniform_block *const block =
               &blks[buffer_block_index];

            assert(var->data.location != -1);

            const struct gl_uniform_buffer_variable *const ubo_var =
               &block->Uniforms[var->data.location];

            ubo_byte_offset = ubo_var->Offset;
            process(var, use_std430_as_default);
         }
      } else {
         /* Store any explicit location and reset data location so we can
          * reuse this variable for storing the uniform slot number.
          */
         this->explicit_location = current_var->data.location;
         current_var->data.location = -1;

         process(var, use_std430_as_default);
      }
      delete this->record_next_sampler;
      delete this->record_next_bindless_sampler;
      delete this->record_next_image;
      delete this->record_next_bindless_image;
   }

   int buffer_block_index;
   int ubo_byte_offset;
   gl_shader_stage shader_type;

private:
   bool set_opaque_indices(const glsl_type *base_type,
                           struct gl_uniform_storage *uniform,
                           const char *name, unsigned &next_index,
                           struct string_to_uint_map *record_next_index)
   {
      assert(base_type->is_sampler() || base_type->is_image());

      if (this->record_array_count > 1) {
         unsigned inner_array_size = MAX2(1, uniform->array_elements);
         char *name_copy = ralloc_strdup(NULL, name);

         /* Remove all array subscripts from the sampler/image name */
         char *str_start;
         const char *str_end;
         while((str_start = strchr(name_copy, '[')) &&
               (str_end = strchr(name_copy, ']'))) {
            memmove(str_start, str_end + 1, 1 + strlen(str_end + 1));
         }

         unsigned index = 0;
         if (record_next_index->get(index, name_copy)) {
            /* In this case, we've already seen this uniform so we just use the
             * next sampler/image index recorded the last time we visited.
             */
            uniform->opaque[shader_type].index = index;
            index = inner_array_size + uniform->opaque[shader_type].index;
            record_next_index->put(index, name_copy);

            ralloc_free(name_copy);
            /* Return as everything else has already been initialised in a
             * previous pass.
             */
            return false;
         } else {
            /* We've never seen this uniform before so we need to allocate
             * enough indices to store it.
             *
             * Nested struct arrays behave like arrays of arrays so we need to
             * increase the index by the total number of elements of the
             * sampler/image in case there is more than one sampler/image
             * inside the structs. This allows the offset to be easily
             * calculated for indirect indexing.
             */
            uniform->opaque[shader_type].index = next_index;
            next_index += inner_array_size * this->record_array_count;

            /* Store the next index for future passes over the struct array
             */
            index = uniform->opaque[shader_type].index + inner_array_size;
            record_next_index->put(index, name_copy);
            ralloc_free(name_copy);
         }
      } else {
         /* Increment the sampler/image by 1 for non-arrays and by the number
          * of array elements for arrays.
          */
         uniform->opaque[shader_type].index = next_index;
         next_index += MAX2(1, uniform->array_elements);
      }
      return true;
   }

   void handle_samplers(const glsl_type *base_type,
                        struct gl_uniform_storage *uniform, const char *name)
   {
      if (base_type->is_sampler()) {
         uniform->opaque[shader_type].active = true;

         const gl_texture_index target = base_type->sampler_index();
         const unsigned shadow = base_type->sampler_shadow;

         if (current_var->data.bindless) {
            if (!set_opaque_indices(base_type, uniform, name,
                                    this->next_bindless_sampler,
                                    this->record_next_bindless_sampler))
               return;

            this->num_bindless_samplers = this->next_bindless_sampler;

            this->bindless_targets = (gl_texture_index *)
               realloc(this->bindless_targets,
                       this->num_bindless_samplers * sizeof(gl_texture_index));

            for (unsigned i = uniform->opaque[shader_type].index;
                 i < this->num_bindless_samplers;
                 i++) {
               this->bindless_targets[i] = target;
            }
         } else {
            if (!set_opaque_indices(base_type, uniform, name,
                                    this->next_sampler,
                                    this->record_next_sampler))
               return;

            for (unsigned i = uniform->opaque[shader_type].index;
                 i < MIN2(this->next_sampler, MAX_SAMPLERS);
                 i++) {
               this->targets[i] = target;
               this->shader_samplers_used |= 1U << i;
               this->shader_shadow_samplers |= shadow << i;
            }
         }
      }
   }

   void handle_images(const glsl_type *base_type,
                      struct gl_uniform_storage *uniform, const char *name)
   {
      if (base_type->is_image()) {
         uniform->opaque[shader_type].active = true;

         /* Set image access qualifiers */
         const GLenum access =
            current_var->data.memory_read_only ?
            (current_var->data.memory_write_only ? GL_NONE :
                                                   GL_READ_ONLY) :
            (current_var->data.memory_write_only ? GL_WRITE_ONLY :
                                                   GL_READ_WRITE);

         if (current_var->data.bindless) {
            if (!set_opaque_indices(base_type, uniform, name,
                                    this->next_bindless_image,
                                    this->record_next_bindless_image))
               return;

            this->num_bindless_images = this->next_bindless_image;

            this->bindless_access = (GLenum *)
               realloc(this->bindless_access,
                       this->num_bindless_images * sizeof(GLenum));

            for (unsigned i = uniform->opaque[shader_type].index;
                 i < this->num_bindless_images;
                 i++) {
               this->bindless_access[i] = access;
            }
         } else {
            if (!set_opaque_indices(base_type, uniform, name,
                                    this->next_image,
                                    this->record_next_image))
               return;

            for (unsigned i = uniform->opaque[shader_type].index;
                 i < MIN2(this->next_image, MAX_IMAGE_UNIFORMS);
                 i++) {
               prog->_LinkedShaders[shader_type]->Program->sh.ImageAccess[i] = access;
            }
         }
      }
   }

   void handle_subroutines(const glsl_type *base_type,
                           struct gl_uniform_storage *uniform)
   {
      if (base_type->is_subroutine()) {
         uniform->opaque[shader_type].index = this->next_subroutine;
         uniform->opaque[shader_type].active = true;

         prog->_LinkedShaders[shader_type]->Program->sh.NumSubroutineUniforms++;

         /* Increment the subroutine index by 1 for non-arrays and by the
          * number of array elements for arrays.
          */
         this->next_subroutine += MAX2(1, uniform->array_elements);

      }
   }

   virtual void set_buffer_offset(unsigned offset)
   {
      this->ubo_byte_offset = offset;
   }

   virtual void set_record_array_count(unsigned record_array_count)
   {
      this->record_array_count = record_array_count;
   }

   virtual void enter_record(const glsl_type *type, const char *,
                             bool row_major,
                             const enum glsl_interface_packing packing)
   {
      assert(type->is_struct());
      if (this->buffer_block_index == -1)
         return;
      if (packing == GLSL_INTERFACE_PACKING_STD430)
         this->ubo_byte_offset = glsl_align(
            this->ubo_byte_offset, type->std430_base_alignment(row_major));
      else
         this->ubo_byte_offset = glsl_align(
            this->ubo_byte_offset, type->std140_base_alignment(row_major));
   }

   virtual void leave_record(const glsl_type *type, const char *,
                             bool row_major,
                             const enum glsl_interface_packing packing)
   {
      assert(type->is_struct());
      if (this->buffer_block_index == -1)
         return;
      if (packing == GLSL_INTERFACE_PACKING_STD430)
         this->ubo_byte_offset = glsl_align(
            this->ubo_byte_offset, type->std430_base_alignment(row_major));
      else
         this->ubo_byte_offset = glsl_align(
            this->ubo_byte_offset, type->std140_base_alignment(row_major));
   }

   virtual void visit_field(const glsl_type *type, const char *name,
                            bool row_major, const glsl_type * /* record_type */,
                            const enum glsl_interface_packing packing,
                            bool /* last_field */)
   {
      assert(!type->without_array()->is_struct());
      assert(!type->without_array()->is_interface());
      assert(!(type->is_array() && type->fields.array->is_array()));

      unsigned id;
      bool found = this->map->get(id, name);
      assert(found);

      if (!found)
         return;

      const glsl_type *base_type;
      if (type->is_array()) {
         this->uniforms[id].array_elements = type->length;
         base_type = type->fields.array;
      } else {
         this->uniforms[id].array_elements = 0;
         base_type = type;
      }

      /* Initialise opaque data */
      this->uniforms[id].opaque[shader_type].index = ~0;
      this->uniforms[id].opaque[shader_type].active = false;

      if (current_var->data.used || base_type->is_subroutine())
         this->uniforms[id].active_shader_mask |= 1 << shader_type;

      /* This assigns uniform indices to sampler and image uniforms. */
      handle_samplers(base_type, &this->uniforms[id], name);
      handle_images(base_type, &this->uniforms[id], name);
      handle_subroutines(base_type, &this->uniforms[id]);

      /* For array of arrays or struct arrays the base location may have
       * already been set so don't set it again.
       */
      if (buffer_block_index == -1 && current_var->data.location == -1) {
         current_var->data.location = id;
      }

      /* If there is already storage associated with this uniform or if the
       * uniform is set as builtin, it means that it was set while processing
       * an earlier shader stage.  For example, we may be processing the
       * uniform in the fragment shader, but the uniform was already processed
       * in the vertex shader.
       */
      if (this->uniforms[id].storage != NULL || this->uniforms[id].builtin) {
         return;
      }

      /* Assign explicit locations. */
      if (current_var->data.explicit_location) {
         /* Set sequential locations for struct fields. */
         if (current_var->type->without_array()->is_struct() ||
             current_var->type->is_array_of_arrays()) {
            const unsigned entries = MAX2(1, this->uniforms[id].array_elements);
            this->uniforms[id].remap_location =
               this->explicit_location + field_counter;
            field_counter += entries;
         } else {
            this->uniforms[id].remap_location = this->explicit_location;
         }
      } else {
         /* Initialize to to indicate that no location is set */
         this->uniforms[id].remap_location = UNMAPPED_UNIFORM_LOC;
      }

      this->uniforms[id].name = ralloc_strdup(this->uniforms, name);
      this->uniforms[id].type = base_type;
      this->uniforms[id].num_driver_storage = 0;
      this->uniforms[id].driver_storage = NULL;
      this->uniforms[id].atomic_buffer_index = -1;
      this->uniforms[id].hidden =
         current_var->data.how_declared == ir_var_hidden;
      this->uniforms[id].builtin = is_gl_identifier(name);

      this->uniforms[id].is_shader_storage =
         current_var->is_in_shader_storage_block();
      this->uniforms[id].is_bindless = current_var->data.bindless;

      /* Do not assign storage if the uniform is a builtin or buffer object */
      if (!this->uniforms[id].builtin &&
          !this->uniforms[id].is_shader_storage &&
          this->buffer_block_index == -1)
         this->uniforms[id].storage = this->values;

      if (this->buffer_block_index != -1) {
         this->uniforms[id].block_index = this->buffer_block_index;

         unsigned alignment = type->std140_base_alignment(row_major);
         if (packing == GLSL_INTERFACE_PACKING_STD430)
            alignment = type->std430_base_alignment(row_major);
         this->ubo_byte_offset = glsl_align(this->ubo_byte_offset, alignment);
         this->uniforms[id].offset = this->ubo_byte_offset;
         if (packing == GLSL_INTERFACE_PACKING_STD430)
            this->ubo_byte_offset += type->std430_size(row_major);
         else
            this->ubo_byte_offset += type->std140_size(row_major);

         if (type->is_array()) {
            if (packing == GLSL_INTERFACE_PACKING_STD430)
               this->uniforms[id].array_stride =
                  type->without_array()->std430_array_stride(row_major);
            else
               this->uniforms[id].array_stride =
                  glsl_align(type->without_array()->std140_size(row_major),
                             16);
         } else {
            this->uniforms[id].array_stride = 0;
         }

         if (type->without_array()->is_matrix()) {
            this->uniforms[id].matrix_stride =
               link_calculate_matrix_stride(type->without_array(),
                                            row_major,
                                            packing);
            this->uniforms[id].row_major = row_major;
         } else {
            this->uniforms[id].matrix_stride = 0;
            this->uniforms[id].row_major = false;
         }
      } else {
         this->uniforms[id].block_index = -1;
         this->uniforms[id].offset = -1;
         this->uniforms[id].array_stride = -1;
         this->uniforms[id].matrix_stride = -1;
         this->uniforms[id].row_major = false;
      }

      if (!this->uniforms[id].builtin &&
          !this->uniforms[id].is_shader_storage &&
          this->buffer_block_index == -1)
         this->values += type->component_slots();

      calculate_array_size_and_stride(prog, &this->uniforms[id],
                                      use_std430_as_default);
   }

   /**
    * Current program being processed.
    */
   struct gl_shader_program *prog;

   struct string_to_uint_map *map;

   struct gl_uniform_storage *uniforms;
   unsigned next_sampler;
   unsigned next_bindless_sampler;
   unsigned next_image;
   unsigned next_bindless_image;
   unsigned next_subroutine;

   bool use_std430_as_default;

   /**
    * Field counter is used to take care that uniform structures
    * with explicit locations get sequential locations.
    */
   unsigned field_counter;

   /**
    * Current variable being processed.
    */
   ir_variable *current_var;

   /* Used to store the explicit location from current_var so that we can
    * reuse the location field for storing the uniform slot id.
    */
   int explicit_location;

   /* Stores total struct array elements including nested structs */
   unsigned record_array_count;

   /* Map for temporarily storing next sampler index when handling samplers in
    * struct arrays.
    */
   struct string_to_uint_map *record_next_sampler;

   /* Map for temporarily storing next imager index when handling images in
    * struct arrays.
    */
   struct string_to_uint_map *record_next_image;

   /* Map for temporarily storing next bindless sampler index when handling
    * bindless samplers in struct arrays.
    */
   struct string_to_uint_map *record_next_bindless_sampler;

   /* Map for temporarily storing next bindless image index when handling
    * bindless images in struct arrays.
    */
   struct string_to_uint_map *record_next_bindless_image;

public:
   union gl_constant_value *values;

   gl_texture_index targets[MAX_SAMPLERS];

   /**
    * Mask of samplers used by the current shader stage.
    */
   unsigned shader_samplers_used;

   /**
    * Mask of samplers used by the current shader stage for shadows.
    */
   unsigned shader_shadow_samplers;

   /**
    * Number of bindless samplers used by the current shader stage.
    */
   unsigned num_bindless_samplers;

   /**
    * Texture targets for bindless samplers used by the current stage.
    */
   gl_texture_index *bindless_targets;

   /**
    * Number of bindless images used by the current shader stage.
    */
   unsigned num_bindless_images;

   /**
    * Access types for bindless images used by the current stage.
    */
   GLenum *bindless_access;

   /**
    * Bitmask of shader storage blocks not declared as read-only.
    */
   unsigned shader_storage_blocks_write_access;
};

static bool
variable_is_referenced(ir_array_refcount_visitor &v, ir_variable *var)
{
   ir_array_refcount_entry *const entry = v.get_variable_entry(var);

   return entry->is_referenced;

}

/**
 * Walks the IR and update the references to uniform blocks in the
 * ir_variables to point at linked shader's list (previously, they
 * would point at the uniform block list in one of the pre-linked
 * shaders).
 */
static void
link_update_uniform_buffer_variables(struct gl_linked_shader *shader,
                                     unsigned stage)
{
   ir_array_refcount_visitor v;

   v.run(shader->ir);

   foreach_in_list(ir_instruction, node, shader->ir) {
      ir_variable *const var = node->as_variable();

      if (var == NULL || !var->is_in_buffer_block())
         continue;

      assert(var->data.mode == ir_var_uniform ||
             var->data.mode == ir_var_shader_storage);

      unsigned num_blocks = var->data.mode == ir_var_uniform ?
         shader->Program->info.num_ubos : shader->Program->info.num_ssbos;
      struct gl_uniform_block **blks = var->data.mode == ir_var_uniform ?
         shader->Program->sh.UniformBlocks :
         shader->Program->sh.ShaderStorageBlocks;

      if (var->is_interface_instance()) {
         const ir_array_refcount_entry *const entry = v.get_variable_entry(var);

         if (entry->is_referenced) {
            /* Since this is an interface instance, the instance type will be
             * same as the array-stripped variable type.  If the variable type
             * is an array, then the block names will be suffixed with [0]
             * through [n-1].  Unlike for non-interface instances, there will
             * not be structure types here, so the only name sentinel that we
             * have to worry about is [.
             */
            assert(var->type->without_array() == var->get_interface_type());
            const char sentinel = var->type->is_array() ? '[' : '\0';

            const ptrdiff_t len = strlen(var->get_interface_type()->name);
            for (unsigned i = 0; i < num_blocks; i++) {
               const char *const begin = blks[i]->Name;
               const char *const end = strchr(begin, sentinel);

               if (end == NULL)
                  continue;

               if (len != (end - begin))
                  continue;

               /* Even when a match is found, do not "break" here.  This could
                * be an array of instances, and all elements of the array need
                * to be marked as referenced.
                */
               if (strncmp(begin, var->get_interface_type()->name, len) == 0 &&
                   (!var->type->is_array() ||
                    entry->is_linearized_index_referenced(blks[i]->linearized_array_index))) {
                  blks[i]->stageref |= 1U << stage;
               }
            }
         }

         var->data.location = 0;
         continue;
      }

      bool found = false;
      char sentinel = '\0';

      if (var->type->is_struct()) {
         sentinel = '.';
      } else if (var->type->is_array() && (var->type->fields.array->is_array()
                 || var->type->without_array()->is_struct())) {
         sentinel = '[';
      }

      const unsigned l = strlen(var->name);
      for (unsigned i = 0; i < num_blocks; i++) {
         for (unsigned j = 0; j < blks[i]->NumUniforms; j++) {
            if (sentinel) {
               const char *begin = blks[i]->Uniforms[j].Name;
               const char *end = strchr(begin, sentinel);

               if (end == NULL)
                  continue;

               if ((ptrdiff_t) l != (end - begin))
                  continue;

               found = strncmp(var->name, begin, l) == 0;
            } else {
               found = strcmp(var->name, blks[i]->Uniforms[j].Name) == 0;
            }

            if (found) {
               var->data.location = j;

               if (variable_is_referenced(v, var))
                  blks[i]->stageref |= 1U << stage;

               break;
            }
         }

         if (found)
            break;
      }
      assert(found);
   }
}

/**
 * Combine the hidden uniform hash map with the uniform hash map so that the
 * hidden uniforms will be given indicies at the end of the uniform storage
 * array.
 */
static void
assign_hidden_uniform_slot_id(const char *name, unsigned hidden_id,
                              void *closure)
{
   count_uniform_size *uniform_size = (count_uniform_size *) closure;
   unsigned hidden_uniform_start = uniform_size->num_active_uniforms -
      uniform_size->num_hidden_uniforms;

   uniform_size->map->put(hidden_uniform_start + hidden_id, name);
}

static void
link_setup_uniform_remap_tables(struct gl_context *ctx,
                                struct gl_shader_program *prog)
{
   unsigned total_entries = prog->NumExplicitUniformLocations;
   unsigned empty_locs = prog->NumUniformRemapTable - total_entries;

   /* Reserve all the explicit locations of the active uniforms. */
   for (unsigned i = 0; i < prog->data->NumUniformStorage; i++) {
      if (prog->data->UniformStorage[i].type->is_subroutine() ||
          prog->data->UniformStorage[i].is_shader_storage)
         continue;

      if (prog->data->UniformStorage[i].remap_location !=
          UNMAPPED_UNIFORM_LOC) {
         /* How many new entries for this uniform? */
         const unsigned entries =
            MAX2(1, prog->data->UniformStorage[i].array_elements);

         /* Set remap table entries point to correct gl_uniform_storage. */
         for (unsigned j = 0; j < entries; j++) {
            unsigned element_loc =
               prog->data->UniformStorage[i].remap_location + j;
            assert(prog->UniformRemapTable[element_loc] ==
                   INACTIVE_UNIFORM_EXPLICIT_LOCATION);
            prog->UniformRemapTable[element_loc] =
               &prog->data->UniformStorage[i];
         }
      }
   }

   /* Reserve locations for rest of the uniforms. */
   for (unsigned i = 0; i < prog->data->NumUniformStorage; i++) {

      if (prog->data->UniformStorage[i].type->is_subroutine() ||
          prog->data->UniformStorage[i].is_shader_storage)
         continue;

      /* Built-in uniforms should not get any location. */
      if (prog->data->UniformStorage[i].builtin)
         continue;

      /* Explicit ones have been set already. */
      if (prog->data->UniformStorage[i].remap_location != UNMAPPED_UNIFORM_LOC)
         continue;

      /* how many new entries for this uniform? */
      const unsigned entries =
         MAX2(1, prog->data->UniformStorage[i].array_elements);

      /* Find UniformRemapTable for empty blocks where we can fit this uniform. */
      int chosen_location = -1;

      if (empty_locs)
         chosen_location = link_util_find_empty_block(prog, &prog->data->UniformStorage[i]);

      /* Add new entries to the total amount for checking against MAX_UNIFORM-
       * _LOCATIONS. This only applies to the default uniform block (-1),
       * because locations of uniform block entries are not assignable.
       */
      if (prog->data->UniformStorage[i].block_index == -1)
         total_entries += entries;

      if (chosen_location != -1) {
         empty_locs -= entries;
      } else {
         chosen_location = prog->NumUniformRemapTable;

         /* resize remap table to fit new entries */
         prog->UniformRemapTable =
            reralloc(prog,
                     prog->UniformRemapTable,
                     gl_uniform_storage *,
                     prog->NumUniformRemapTable + entries);
         prog->NumUniformRemapTable += entries;
      }

      /* set pointers for this uniform */
      for (unsigned j = 0; j < entries; j++)
         prog->UniformRemapTable[chosen_location + j] =
            &prog->data->UniformStorage[i];

      /* set the base location in remap table for the uniform */
      prog->data->UniformStorage[i].remap_location = chosen_location;
   }

   /* Verify that total amount of entries for explicit and implicit locations
    * is less than MAX_UNIFORM_LOCATIONS.
    */

   if (total_entries > ctx->Const.MaxUserAssignableUniformLocations) {
      linker_error(prog, "count of uniform locations > MAX_UNIFORM_LOCATIONS"
                   "(%u > %u)", total_entries,
                   ctx->Const.MaxUserAssignableUniformLocations);
   }

   /* Reserve all the explicit locations of the active subroutine uniforms. */
   for (unsigned i = 0; i < prog->data->NumUniformStorage; i++) {
      if (!prog->data->UniformStorage[i].type->is_subroutine())
         continue;

      if (prog->data->UniformStorage[i].remap_location == UNMAPPED_UNIFORM_LOC)
         continue;

      /* How many new entries for this uniform? */
      const unsigned entries =
         MAX2(1, prog->data->UniformStorage[i].array_elements);

      unsigned mask = prog->data->linked_stages;
      while (mask) {
         const int j = u_bit_scan(&mask);
         struct gl_program *p = prog->_LinkedShaders[j]->Program;

         if (!prog->data->UniformStorage[i].opaque[j].active)
            continue;

         /* Set remap table entries point to correct gl_uniform_storage. */
         for (unsigned k = 0; k < entries; k++) {
            unsigned element_loc =
               prog->data->UniformStorage[i].remap_location + k;
            assert(p->sh.SubroutineUniformRemapTable[element_loc] ==
                   INACTIVE_UNIFORM_EXPLICIT_LOCATION);
            p->sh.SubroutineUniformRemapTable[element_loc] =
               &prog->data->UniformStorage[i];
         }
      }
   }

   /* reserve subroutine locations */
   for (unsigned i = 0; i < prog->data->NumUniformStorage; i++) {
      if (!prog->data->UniformStorage[i].type->is_subroutine())
         continue;

      if (prog->data->UniformStorage[i].remap_location !=
          UNMAPPED_UNIFORM_LOC)
         continue;

      const unsigned entries =
         MAX2(1, prog->data->UniformStorage[i].array_elements);

      unsigned mask = prog->data->linked_stages;
      while (mask) {
         const int j = u_bit_scan(&mask);
         struct gl_program *p = prog->_LinkedShaders[j]->Program;

         if (!prog->data->UniformStorage[i].opaque[j].active)
            continue;

         p->sh.SubroutineUniformRemapTable =
            reralloc(p,
                     p->sh.SubroutineUniformRemapTable,
                     gl_uniform_storage *,
                     p->sh.NumSubroutineUniformRemapTable + entries);

         for (unsigned k = 0; k < entries; k++) {
            p->sh.SubroutineUniformRemapTable[p->sh.NumSubroutineUniformRemapTable + k] =
               &prog->data->UniformStorage[i];
         }
         prog->data->UniformStorage[i].remap_location =
            p->sh.NumSubroutineUniformRemapTable;
         p->sh.NumSubroutineUniformRemapTable += entries;
      }
   }
}

static void
link_assign_uniform_storage(struct gl_context *ctx,
                            struct gl_shader_program *prog,
                            const unsigned num_data_slots)
{
   /* On the outside chance that there were no uniforms, bail out.
    */
   if (prog->data->NumUniformStorage == 0)
      return;

   unsigned int boolean_true = ctx->Const.UniformBooleanTrue;

   union gl_constant_value *data;
   if (prog->data->UniformStorage == NULL) {
      prog->data->UniformStorage = rzalloc_array(prog->data,
                                                 struct gl_uniform_storage,
                                                 prog->data->NumUniformStorage);
      data = rzalloc_array(prog->data->UniformStorage,
                           union gl_constant_value, num_data_slots);
      prog->data->UniformDataDefaults =
         rzalloc_array(prog->data->UniformStorage,
                       union gl_constant_value, num_data_slots);
   } else {
      data = prog->data->UniformDataSlots;
   }

#ifndef NDEBUG
   union gl_constant_value *data_end = &data[num_data_slots];
#endif

   parcel_out_uniform_storage parcel(prog, prog->UniformHash,
                                     prog->data->UniformStorage, data,
                                     ctx->Const.UseSTD430AsDefaultPacking);

   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      struct gl_linked_shader *shader = prog->_LinkedShaders[i];

      if (!shader)
         continue;

      parcel.start_shader((gl_shader_stage)i);

      foreach_in_list(ir_instruction, node, shader->ir) {
         ir_variable *const var = node->as_variable();

         if ((var == NULL) || (var->data.mode != ir_var_uniform &&
                               var->data.mode != ir_var_shader_storage))
            continue;

         parcel.set_and_process(var);
      }

      shader->Program->SamplersUsed = parcel.shader_samplers_used;
      shader->shadow_samplers = parcel.shader_shadow_samplers;
      shader->Program->sh.ShaderStorageBlocksWriteAccess =
         parcel.shader_storage_blocks_write_access;

      if (parcel.num_bindless_samplers > 0) {
         shader->Program->sh.NumBindlessSamplers = parcel.num_bindless_samplers;
         shader->Program->sh.BindlessSamplers =
            rzalloc_array(shader->Program, gl_bindless_sampler,
                          parcel.num_bindless_samplers);
         for (unsigned j = 0; j < parcel.num_bindless_samplers; j++) {
            shader->Program->sh.BindlessSamplers[j].target =
               parcel.bindless_targets[j];
         }
      }

      if (parcel.num_bindless_images > 0) {
         shader->Program->sh.NumBindlessImages = parcel.num_bindless_images;
         shader->Program->sh.BindlessImages =
            rzalloc_array(shader->Program, gl_bindless_image,
                          parcel.num_bindless_images);
         for (unsigned j = 0; j < parcel.num_bindless_images; j++) {
            shader->Program->sh.BindlessImages[j].access =
               parcel.bindless_access[j];
         }
      }

      STATIC_ASSERT(ARRAY_SIZE(shader->Program->sh.SamplerTargets) ==
                    ARRAY_SIZE(parcel.targets));
      for (unsigned j = 0; j < ARRAY_SIZE(parcel.targets); j++)
         shader->Program->sh.SamplerTargets[j] = parcel.targets[j];
   }

#ifndef NDEBUG
   for (unsigned i = 0; i < prog->data->NumUniformStorage; i++) {
      assert(prog->data->UniformStorage[i].storage != NULL ||
             prog->data->UniformStorage[i].builtin ||
             prog->data->UniformStorage[i].is_shader_storage ||
             prog->data->UniformStorage[i].block_index != -1);
   }

   assert(parcel.values == data_end);
#endif

   link_setup_uniform_remap_tables(ctx, prog);

   /* Set shader cache fields */
   prog->data->NumUniformDataSlots = num_data_slots;
   prog->data->UniformDataSlots = data;

   link_set_uniform_initializers(prog, boolean_true);
}

void
link_assign_uniform_locations(struct gl_shader_program *prog,
                              struct gl_context *ctx)
{
   ralloc_free(prog->data->UniformStorage);
   prog->data->UniformStorage = NULL;
   prog->data->NumUniformStorage = 0;

   if (prog->UniformHash != NULL) {
      prog->UniformHash->clear();
   } else {
      prog->UniformHash = new string_to_uint_map;
   }

   /* First pass: Count the uniform resources used by the user-defined
    * uniforms.  While this happens, each active uniform will have an index
    * assigned to it.
    *
    * Note: this is *NOT* the index that is returned to the application by
    * glGetUniformLocation.
    */
   struct string_to_uint_map *hiddenUniforms = new string_to_uint_map;
   count_uniform_size uniform_size(prog->UniformHash, hiddenUniforms,
                                   ctx->Const.UseSTD430AsDefaultPacking);
   for (unsigned i = 0; i < MESA_SHADER_STAGES; i++) {
      struct gl_linked_shader *sh = prog->_LinkedShaders[i];

      if (sh == NULL)
         continue;

      link_update_uniform_buffer_variables(sh, i);

      /* Reset various per-shader target counts.
       */
      uniform_size.start_shader();

      foreach_in_list(ir_instruction, node, sh->ir) {
         ir_variable *const var = node->as_variable();

         if ((var == NULL) || (var->data.mode != ir_var_uniform &&
                               var->data.mode != ir_var_shader_storage))
            continue;

         uniform_size.process(var);
      }

      if (uniform_size.num_shader_samplers >
          ctx->Const.Program[i].MaxTextureImageUnits) {
         linker_error(prog, "Too many %s shader texture samplers\n",
                      _mesa_shader_stage_to_string(i));
         continue;
      }

      if (uniform_size.num_shader_images >
          ctx->Const.Program[i].MaxImageUniforms) {
         linker_error(prog, "Too many %s shader image uniforms (%u > %u)\n",
                      _mesa_shader_stage_to_string(i),
                      sh->Program->info.num_images,
                      ctx->Const.Program[i].MaxImageUniforms);
         continue;
      }

      sh->Program->info.num_textures = uniform_size.num_shader_samplers;
      sh->Program->info.num_images = uniform_size.num_shader_images;
      sh->num_uniform_components = uniform_size.num_shader_uniform_components;
      sh->num_combined_uniform_components = sh->num_uniform_components;

      for (unsigned i = 0; i < sh->Program->info.num_ubos; i++) {
         sh->num_combined_uniform_components +=
            sh->Program->sh.UniformBlocks[i]->UniformBufferSize / 4;
      }
   }

   if (prog->data->LinkStatus == LINKING_FAILURE) {
      delete hiddenUniforms;
      return;
   }

   prog->data->NumUniformStorage = uniform_size.num_active_uniforms;
   prog->data->NumHiddenUniforms = uniform_size.num_hidden_uniforms;

   /* assign hidden uniforms a slot id */
   hiddenUniforms->iterate(assign_hidden_uniform_slot_id, &uniform_size);
   delete hiddenUniforms;

   link_assign_uniform_storage(ctx, prog, uniform_size.num_values);
}
