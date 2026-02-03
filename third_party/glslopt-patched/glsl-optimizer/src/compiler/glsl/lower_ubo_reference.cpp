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

/**
 * \file lower_ubo_reference.cpp
 *
 * IR lower pass to replace dereferences of variables in a uniform
 * buffer object with usage of ir_binop_ubo_load expressions, each of
 * which can read data up to the size of a vec4.
 *
 * This relieves drivers of the responsibility to deal with tricky UBO
 * layout issues like std140 structures and row_major matrices on
 * their own.
 */

#include "lower_buffer_access.h"
#include "ir_builder.h"
#include "main/macros.h"
#include "glsl_parser_extras.h"
#include "main/mtypes.h"

using namespace ir_builder;

namespace {
class lower_ubo_reference_visitor :
      public lower_buffer_access::lower_buffer_access {
public:
   lower_ubo_reference_visitor(struct gl_linked_shader *shader,
                               bool clamp_block_indices,
                               bool use_std430_as_default)
   : shader(shader), clamp_block_indices(clamp_block_indices),
     struct_field(NULL), variable(NULL)
   {
      this->use_std430_as_default = use_std430_as_default;
   }

   void handle_rvalue(ir_rvalue **rvalue);
   ir_visitor_status visit_enter(ir_assignment *ir);

   void setup_for_load_or_store(void *mem_ctx,
                                ir_variable *var,
                                ir_rvalue *deref,
                                ir_rvalue **offset,
                                unsigned *const_offset,
                                bool *row_major,
                                const glsl_type **matrix_type,
                                enum glsl_interface_packing packing);
   uint32_t ssbo_access_params();
   ir_expression *ubo_load(void *mem_ctx, const struct glsl_type *type,
			   ir_rvalue *offset);
   ir_call *ssbo_load(void *mem_ctx, const struct glsl_type *type,
                      ir_rvalue *offset);

   bool check_for_buffer_array_copy(ir_assignment *ir);
   bool check_for_buffer_struct_copy(ir_assignment *ir);
   void check_for_ssbo_store(ir_assignment *ir);
   void write_to_memory(void *mem_ctx, ir_dereference *deref, ir_variable *var,
                        ir_variable *write_var, unsigned write_mask);
   ir_call *ssbo_store(void *mem_ctx, ir_rvalue *deref, ir_rvalue *offset,
                       unsigned write_mask);

   enum {
      ubo_load_access,
      ssbo_load_access,
      ssbo_store_access,
      ssbo_unsized_array_length_access,
      ssbo_atomic_access,
   } buffer_access_type;

   void insert_buffer_access(void *mem_ctx, ir_dereference *deref,
                             const glsl_type *type, ir_rvalue *offset,
                             unsigned mask, int channel);

   ir_visitor_status visit_enter(class ir_expression *);
   ir_expression *calculate_ssbo_unsized_array_length(ir_expression *expr);
   void check_ssbo_unsized_array_length_expression(class ir_expression *);
   void check_ssbo_unsized_array_length_assignment(ir_assignment *ir);

   ir_expression *process_ssbo_unsized_array_length(ir_rvalue **,
                                                    ir_dereference *,
                                                    ir_variable *);
   ir_expression *emit_ssbo_get_buffer_size(void *mem_ctx);

   unsigned calculate_unsized_array_stride(ir_dereference *deref,
                                           enum glsl_interface_packing packing);

   ir_call *lower_ssbo_atomic_intrinsic(ir_call *ir);
   ir_call *check_for_ssbo_atomic_intrinsic(ir_call *ir);
   ir_visitor_status visit_enter(ir_call *ir);
   ir_visitor_status visit_enter(ir_texture *ir);

   struct gl_linked_shader *shader;
   bool clamp_block_indices;
   const struct glsl_struct_field *struct_field;
   ir_variable *variable;
   ir_rvalue *uniform_block;
   bool progress;
};

/**
 * Determine the name of the interface block field
 *
 * This is the name of the specific member as it would appear in the
 * \c gl_uniform_buffer_variable::Name field in the shader's
 * \c UniformBlocks array.
 */
static const char *
interface_field_name(void *mem_ctx, char *base_name, ir_rvalue *d,
                     ir_rvalue **nonconst_block_index)
{
   *nonconst_block_index = NULL;
   char *name_copy = NULL;
   size_t base_length = 0;

   /* Loop back through the IR until we find the uniform block */
   ir_rvalue *ir = d;
   while (ir != NULL) {
      switch (ir->ir_type) {
      case ir_type_dereference_variable: {
         /* Exit loop */
         ir = NULL;
         break;
      }

      case ir_type_dereference_record: {
         ir_dereference_record *r = (ir_dereference_record *) ir;
         ir = r->record->as_dereference();

         /* If we got here it means any previous array subscripts belong to
          * block members and not the block itself so skip over them in the
          * next pass.
          */
         d = ir;
         break;
      }

      case ir_type_dereference_array: {
         ir_dereference_array *a = (ir_dereference_array *) ir;
         ir = a->array->as_dereference();
         break;
      }

      case ir_type_swizzle: {
         ir_swizzle *s = (ir_swizzle *) ir;
         ir = s->val->as_dereference();
         /* Skip swizzle in the next pass */
         d = ir;
         break;
      }

      default:
         assert(!"Should not get here.");
         break;
      }
   }

   while (d != NULL) {
      switch (d->ir_type) {
      case ir_type_dereference_variable: {
         ir_dereference_variable *v = (ir_dereference_variable *) d;
         if (name_copy != NULL &&
             v->var->is_interface_instance() &&
             v->var->type->is_array()) {
            return name_copy;
         } else {
            *nonconst_block_index = NULL;
            return base_name;
         }

         break;
      }

      case ir_type_dereference_array: {
         ir_dereference_array *a = (ir_dereference_array *) d;
         size_t new_length;

         if (name_copy == NULL) {
            name_copy = ralloc_strdup(mem_ctx, base_name);
            base_length = strlen(name_copy);
         }

         /* For arrays of arrays we start at the innermost array and work our
          * way out so we need to insert the subscript at the base of the
          * name string rather than just attaching it to the end.
          */
         new_length = base_length;
         ir_constant *const_index = a->array_index->as_constant();
         char *end = ralloc_strdup(NULL, &name_copy[new_length]);
         if (!const_index) {
            ir_rvalue *array_index = a->array_index;
            if (array_index->type != glsl_type::uint_type)
               array_index = i2u(array_index);

            if (a->array->type->is_array() &&
                a->array->type->fields.array->is_array()) {
               ir_constant *base_size = new(mem_ctx)
                  ir_constant(a->array->type->fields.array->arrays_of_arrays_size());
               array_index = mul(array_index, base_size);
            }

            if (*nonconst_block_index) {
               *nonconst_block_index = add(*nonconst_block_index, array_index);
            } else {
               *nonconst_block_index = array_index;
            }

            ralloc_asprintf_rewrite_tail(&name_copy, &new_length, "[0]%s",
                                         end);
         } else {
            ralloc_asprintf_rewrite_tail(&name_copy, &new_length, "[%d]%s",
                                         const_index->get_uint_component(0),
                                         end);
         }
         ralloc_free(end);

         d = a->array->as_dereference();

         break;
      }

      default:
         assert(!"Should not get here.");
         break;
      }
   }

   assert(!"Should not get here.");
   return NULL;
}

static ir_rvalue *
clamp_to_array_bounds(void *mem_ctx, ir_rvalue *index, const glsl_type *type)
{
   assert(type->is_array());

   const unsigned array_size = type->arrays_of_arrays_size();

   ir_constant *max_index = new(mem_ctx) ir_constant(array_size - 1);
   max_index->type = index->type;

   ir_constant *zero = new(mem_ctx) ir_constant(0);
   zero->type = index->type;

   if (index->type->base_type == GLSL_TYPE_INT)
      index = max2(index, zero);
   index = min2(index, max_index);

   return index;
}

void
lower_ubo_reference_visitor::setup_for_load_or_store(void *mem_ctx,
                                                     ir_variable *var,
                                                     ir_rvalue *deref,
                                                     ir_rvalue **offset,
                                                     unsigned *const_offset,
                                                     bool *row_major,
                                                     const glsl_type **matrix_type,
                                                     enum glsl_interface_packing packing)
{
   /* Determine the name of the interface block */
   ir_rvalue *nonconst_block_index;
   const char *const field_name =
      interface_field_name(mem_ctx, (char *) var->get_interface_type()->name,
                           deref, &nonconst_block_index);

   if (nonconst_block_index && clamp_block_indices) {
      nonconst_block_index =
         clamp_to_array_bounds(mem_ctx, nonconst_block_index, var->type);
   }

   /* Locate the block by interface name */
   unsigned num_blocks;
   struct gl_uniform_block **blocks;
   if (this->buffer_access_type != ubo_load_access) {
      num_blocks = shader->Program->info.num_ssbos;
      blocks = shader->Program->sh.ShaderStorageBlocks;
   } else {
      num_blocks = shader->Program->info.num_ubos;
      blocks = shader->Program->sh.UniformBlocks;
   }
   this->uniform_block = NULL;
   for (unsigned i = 0; i < num_blocks; i++) {
      if (strcmp(field_name, blocks[i]->Name) == 0) {

         ir_constant *index = new(mem_ctx) ir_constant(i);

         if (nonconst_block_index) {
            this->uniform_block = add(nonconst_block_index, index);
         } else {
            this->uniform_block = index;
         }

         if (var->is_interface_instance()) {
            *const_offset = 0;
         } else {
            *const_offset = blocks[i]->Uniforms[var->data.location].Offset;
         }

         break;
      }
   }

   assert(this->uniform_block);

   this->struct_field = NULL;
   setup_buffer_access(mem_ctx, deref, offset, const_offset, row_major,
                       matrix_type, &this->struct_field, packing);
}

void
lower_ubo_reference_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_dereference *deref = (*rvalue)->as_dereference();
   if (!deref)
      return;

   ir_variable *var = deref->variable_referenced();
   if (!var || !var->is_in_buffer_block())
      return;

   void *mem_ctx = ralloc_parent(shader->ir);

   ir_rvalue *offset = NULL;
   unsigned const_offset;
   bool row_major;
   const glsl_type *matrix_type;

   enum glsl_interface_packing packing =
      var->get_interface_type()->
         get_internal_ifc_packing(use_std430_as_default);

   this->buffer_access_type =
      var->is_in_shader_storage_block() ?
      ssbo_load_access : ubo_load_access;
   this->variable = var;

   /* Compute the offset to the start if the dereference as well as other
    * information we need to configure the write
    */
   setup_for_load_or_store(mem_ctx, var, deref,
                           &offset, &const_offset,
                           &row_major, &matrix_type,
                           packing);
   assert(offset);

   /* Now that we've calculated the offset to the start of the
    * dereference, walk over the type and emit loads into a temporary.
    */
   const glsl_type *type = (*rvalue)->type;
   ir_variable *load_var = new(mem_ctx) ir_variable(type,
						    "ubo_load_temp",
						    ir_var_temporary);
   base_ir->insert_before(load_var);

   ir_variable *load_offset = new(mem_ctx) ir_variable(glsl_type::uint_type,
						       "ubo_load_temp_offset",
						       ir_var_temporary);
   base_ir->insert_before(load_offset);
   base_ir->insert_before(assign(load_offset, offset));

   deref = new(mem_ctx) ir_dereference_variable(load_var);
   emit_access(mem_ctx, false, deref, load_offset, const_offset,
               row_major, matrix_type, packing, 0);
   *rvalue = deref;

   progress = true;
}

ir_expression *
lower_ubo_reference_visitor::ubo_load(void *mem_ctx,
                                      const glsl_type *type,
				      ir_rvalue *offset)
{
   ir_rvalue *block_ref = this->uniform_block->clone(mem_ctx, NULL);
   return new(mem_ctx)
      ir_expression(ir_binop_ubo_load,
                    type,
                    block_ref,
                    offset);

}

static bool
shader_storage_buffer_object(const _mesa_glsl_parse_state *state)
{
   return state->has_shader_storage_buffer_objects();
}

uint32_t
lower_ubo_reference_visitor::ssbo_access_params()
{
   assert(variable);

   if (variable->is_interface_instance()) {
      assert(struct_field);

      return ((struct_field->memory_coherent ? ACCESS_COHERENT : 0) |
              (struct_field->memory_restrict ? ACCESS_RESTRICT : 0) |
              (struct_field->memory_volatile ? ACCESS_VOLATILE : 0));
   } else {
      return ((variable->data.memory_coherent ? ACCESS_COHERENT : 0) |
              (variable->data.memory_restrict ? ACCESS_RESTRICT : 0) |
              (variable->data.memory_volatile ? ACCESS_VOLATILE : 0));
   }
}

ir_call *
lower_ubo_reference_visitor::ssbo_store(void *mem_ctx,
                                        ir_rvalue *deref,
                                        ir_rvalue *offset,
                                        unsigned write_mask)
{
   exec_list sig_params;

   ir_variable *block_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "block_ref" , ir_var_function_in);
   sig_params.push_tail(block_ref);

   ir_variable *offset_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset" , ir_var_function_in);
   sig_params.push_tail(offset_ref);

   ir_variable *val_ref = new(mem_ctx)
      ir_variable(deref->type, "value" , ir_var_function_in);
   sig_params.push_tail(val_ref);

   ir_variable *writemask_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "write_mask" , ir_var_function_in);
   sig_params.push_tail(writemask_ref);

   ir_variable *access_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "access" , ir_var_function_in);
   sig_params.push_tail(access_ref);

   ir_function_signature *sig = new(mem_ctx)
      ir_function_signature(glsl_type::void_type, shader_storage_buffer_object);
   assert(sig);
   sig->replace_parameters(&sig_params);
   sig->intrinsic_id = ir_intrinsic_ssbo_store;

   ir_function *f = new(mem_ctx) ir_function("__intrinsic_store_ssbo");
   f->add_signature(sig);

   exec_list call_params;
   call_params.push_tail(this->uniform_block->clone(mem_ctx, NULL));
   call_params.push_tail(offset->clone(mem_ctx, NULL));
   call_params.push_tail(deref->clone(mem_ctx, NULL));
   call_params.push_tail(new(mem_ctx) ir_constant(write_mask));
   call_params.push_tail(new(mem_ctx) ir_constant(ssbo_access_params()));
   return new(mem_ctx) ir_call(sig, NULL, &call_params);
}

ir_call *
lower_ubo_reference_visitor::ssbo_load(void *mem_ctx,
                                       const struct glsl_type *type,
                                       ir_rvalue *offset)
{
   exec_list sig_params;

   ir_variable *block_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "block_ref" , ir_var_function_in);
   sig_params.push_tail(block_ref);

   ir_variable *offset_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset_ref" , ir_var_function_in);
   sig_params.push_tail(offset_ref);

   ir_variable *access_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "access" , ir_var_function_in);
   sig_params.push_tail(access_ref);

   ir_function_signature *sig =
      new(mem_ctx) ir_function_signature(type, shader_storage_buffer_object);
   assert(sig);
   sig->replace_parameters(&sig_params);
   sig->intrinsic_id = ir_intrinsic_ssbo_load;

   ir_function *f = new(mem_ctx) ir_function("__intrinsic_load_ssbo");
   f->add_signature(sig);

   ir_variable *result = new(mem_ctx)
      ir_variable(type, "ssbo_load_result", ir_var_temporary);
   base_ir->insert_before(result);
   ir_dereference_variable *deref_result = new(mem_ctx)
      ir_dereference_variable(result);

   exec_list call_params;
   call_params.push_tail(this->uniform_block->clone(mem_ctx, NULL));
   call_params.push_tail(offset->clone(mem_ctx, NULL));
   call_params.push_tail(new(mem_ctx) ir_constant(ssbo_access_params()));

   return new(mem_ctx) ir_call(sig, deref_result, &call_params);
}

void
lower_ubo_reference_visitor::insert_buffer_access(void *mem_ctx,
                                                  ir_dereference *deref,
                                                  const glsl_type *type,
                                                  ir_rvalue *offset,
                                                  unsigned mask,
                                                  int channel)
{
   switch (this->buffer_access_type) {
   case ubo_load_access:
      base_ir->insert_before(assign(deref->clone(mem_ctx, NULL),
                                    ubo_load(mem_ctx, type, offset),
                                    mask));
      break;
   case ssbo_load_access: {
      ir_call *load_ssbo = ssbo_load(mem_ctx, type, offset);
      base_ir->insert_before(load_ssbo);
      ir_rvalue *value = load_ssbo->return_deref->as_rvalue()->clone(mem_ctx, NULL);
      ir_assignment *assignment =
         assign(deref->clone(mem_ctx, NULL), value, mask);
      base_ir->insert_before(assignment);
      break;
   }
   case ssbo_store_access:
      if (channel >= 0) {
         base_ir->insert_after(ssbo_store(mem_ctx,
                                          swizzle(deref, channel, 1),
                                          offset, 1));
      } else {
         base_ir->insert_after(ssbo_store(mem_ctx, deref, offset, mask));
      }
      break;
   default:
      unreachable("invalid buffer_access_type in insert_buffer_access");
   }
}

void
lower_ubo_reference_visitor::write_to_memory(void *mem_ctx,
                                             ir_dereference *deref,
                                             ir_variable *var,
                                             ir_variable *write_var,
                                             unsigned write_mask)
{
   ir_rvalue *offset = NULL;
   unsigned const_offset;
   bool row_major;
   const glsl_type *matrix_type;

   enum glsl_interface_packing packing =
      var->get_interface_type()->
         get_internal_ifc_packing(use_std430_as_default);

   this->buffer_access_type = ssbo_store_access;
   this->variable = var;

   /* Compute the offset to the start if the dereference as well as other
    * information we need to configure the write
    */
   setup_for_load_or_store(mem_ctx, var, deref,
                           &offset, &const_offset,
                           &row_major, &matrix_type,
                           packing);
   assert(offset);

   /* Now emit writes from the temporary to memory */
   ir_variable *write_offset =
      new(mem_ctx) ir_variable(glsl_type::uint_type,
                               "ssbo_store_temp_offset",
                               ir_var_temporary);

   base_ir->insert_before(write_offset);
   base_ir->insert_before(assign(write_offset, offset));

   deref = new(mem_ctx) ir_dereference_variable(write_var);
   emit_access(mem_ctx, true, deref, write_offset, const_offset,
               row_major, matrix_type, packing, write_mask);
}

ir_visitor_status
lower_ubo_reference_visitor::visit_enter(ir_expression *ir)
{
   check_ssbo_unsized_array_length_expression(ir);
   return rvalue_visit(ir);
}

ir_expression *
lower_ubo_reference_visitor::calculate_ssbo_unsized_array_length(ir_expression *expr)
{
   if (expr->operation !=
       ir_expression_operation(ir_unop_ssbo_unsized_array_length))
      return NULL;

   ir_rvalue *rvalue = expr->operands[0]->as_rvalue();
   if (!rvalue ||
       !rvalue->type->is_array() || !rvalue->type->is_unsized_array())
      return NULL;

   ir_dereference *deref = expr->operands[0]->as_dereference();
   if (!deref)
      return NULL;

   ir_variable *var = expr->operands[0]->variable_referenced();
   if (!var || !var->is_in_shader_storage_block())
      return NULL;
   return process_ssbo_unsized_array_length(&rvalue, deref, var);
}

void
lower_ubo_reference_visitor::check_ssbo_unsized_array_length_expression(ir_expression *ir)
{
   if (ir->operation ==
       ir_expression_operation(ir_unop_ssbo_unsized_array_length)) {
         /* Don't replace this unop if it is found alone. It is going to be
          * removed by the optimization passes or replaced if it is part of
          * an ir_assignment or another ir_expression.
          */
         return;
   }

   for (unsigned i = 0; i < ir->num_operands; i++) {
      if (ir->operands[i]->ir_type != ir_type_expression)
         continue;
      ir_expression *expr = (ir_expression *) ir->operands[i];
      ir_expression *temp = calculate_ssbo_unsized_array_length(expr);
      if (!temp)
         continue;

      delete expr;
      ir->operands[i] = temp;
   }
}

void
lower_ubo_reference_visitor::check_ssbo_unsized_array_length_assignment(ir_assignment *ir)
{
   if (!ir->rhs || ir->rhs->ir_type != ir_type_expression)
      return;

   ir_expression *expr = (ir_expression *) ir->rhs;
   ir_expression *temp = calculate_ssbo_unsized_array_length(expr);
   if (!temp)
      return;

   delete expr;
   ir->rhs = temp;
   return;
}

ir_expression *
lower_ubo_reference_visitor::emit_ssbo_get_buffer_size(void *mem_ctx)
{
   ir_rvalue *block_ref = this->uniform_block->clone(mem_ctx, NULL);
   return new(mem_ctx) ir_expression(ir_unop_get_buffer_size,
                                     glsl_type::int_type,
                                     block_ref);
}

unsigned
lower_ubo_reference_visitor::calculate_unsized_array_stride(ir_dereference *deref,
                                                            enum glsl_interface_packing packing)
{
   unsigned array_stride = 0;

   switch (deref->ir_type) {
   case ir_type_dereference_variable:
   {
      ir_dereference_variable *deref_var = (ir_dereference_variable *)deref;
      const struct glsl_type *unsized_array_type = NULL;
      /* An unsized array can be sized by other lowering passes, so pick
       * the first field of the array which has the data type of the unsized
       * array.
       */
      unsized_array_type = deref_var->var->type->fields.array;

      /* Whether or not the field is row-major (because it might be a
       * bvec2 or something) does not affect the array itself. We need
       * to know whether an array element in its entirety is row-major.
       */
      const bool array_row_major =
         is_dereferenced_thing_row_major(deref_var);

      if (packing == GLSL_INTERFACE_PACKING_STD430) {
         array_stride = unsized_array_type->std430_array_stride(array_row_major);
      } else {
         array_stride = unsized_array_type->std140_size(array_row_major);
         array_stride = glsl_align(array_stride, 16);
      }
      break;
   }
   case ir_type_dereference_record:
   {
      ir_dereference_record *deref_record = (ir_dereference_record *) deref;
      ir_dereference *interface_deref =
         deref_record->record->as_dereference();
      assert(interface_deref != NULL);
      const struct glsl_type *interface_type = interface_deref->type;
      unsigned record_length = interface_type->length;
      /* Unsized array is always the last element of the interface */
      const struct glsl_type *unsized_array_type =
         interface_type->fields.structure[record_length - 1].type->fields.array;

      const bool array_row_major =
         is_dereferenced_thing_row_major(deref_record);

      if (packing == GLSL_INTERFACE_PACKING_STD430) {
         array_stride = unsized_array_type->std430_array_stride(array_row_major);
      } else {
         array_stride = unsized_array_type->std140_size(array_row_major);
         array_stride = glsl_align(array_stride, 16);
      }
      break;
   }
   default:
      unreachable("Unsupported dereference type");
   }
   return array_stride;
}

ir_expression *
lower_ubo_reference_visitor::process_ssbo_unsized_array_length(ir_rvalue **rvalue,
                                                               ir_dereference *deref,
                                                               ir_variable *var)
{
   void *mem_ctx = ralloc_parent(*rvalue);

   ir_rvalue *base_offset = NULL;
   unsigned const_offset;
   bool row_major;
   const glsl_type *matrix_type;

   enum glsl_interface_packing packing =
      var->get_interface_type()->
         get_internal_ifc_packing(use_std430_as_default);
   int unsized_array_stride =
      calculate_unsized_array_stride(deref, packing);

   this->buffer_access_type = ssbo_unsized_array_length_access;
   this->variable = var;

   /* Compute the offset to the start if the dereference as well as other
    * information we need to calculate the length.
    */
   setup_for_load_or_store(mem_ctx, var, deref,
                           &base_offset, &const_offset,
                           &row_major, &matrix_type,
                           packing);
   /* array.length() =
    *  max((buffer_object_size - offset_of_array) / stride_of_array, 0)
    */
   ir_expression *buffer_size = emit_ssbo_get_buffer_size(mem_ctx);

   ir_expression *offset_of_array = new(mem_ctx)
      ir_expression(ir_binop_add, base_offset,
                    new(mem_ctx) ir_constant(const_offset));
   ir_expression *offset_of_array_int = new(mem_ctx)
      ir_expression(ir_unop_u2i, offset_of_array);

   ir_expression *sub = new(mem_ctx)
      ir_expression(ir_binop_sub, buffer_size, offset_of_array_int);
   ir_expression *div =  new(mem_ctx)
      ir_expression(ir_binop_div, sub,
                    new(mem_ctx) ir_constant(unsized_array_stride));
   ir_expression *max = new(mem_ctx)
      ir_expression(ir_binop_max, div, new(mem_ctx) ir_constant(0));

   return max;
}

void
lower_ubo_reference_visitor::check_for_ssbo_store(ir_assignment *ir)
{
   if (!ir || !ir->lhs)
      return;

   ir_rvalue *rvalue = ir->lhs->as_rvalue();
   if (!rvalue)
      return;

   ir_dereference *deref = ir->lhs->as_dereference();
   if (!deref)
      return;

   ir_variable *var = ir->lhs->variable_referenced();
   if (!var || !var->is_in_shader_storage_block())
      return;

   /* We have a write to a buffer variable, so declare a temporary and rewrite
    * the assignment so that the temporary is the LHS.
    */
   void *mem_ctx = ralloc_parent(shader->ir);

   const glsl_type *type = rvalue->type;
   ir_variable *write_var = new(mem_ctx) ir_variable(type,
                                                     "ssbo_store_temp",
                                                     ir_var_temporary);
   base_ir->insert_before(write_var);
   ir->lhs = new(mem_ctx) ir_dereference_variable(write_var);

   /* Now we have to write the value assigned to the temporary back to memory */
   write_to_memory(mem_ctx, deref, var, write_var, ir->write_mask);
   progress = true;
}

static bool
is_buffer_backed_variable(ir_variable *var)
{
   return var->is_in_buffer_block() ||
          var->data.mode == ir_var_shader_shared;
}

bool
lower_ubo_reference_visitor::check_for_buffer_array_copy(ir_assignment *ir)
{
   if (!ir || !ir->lhs || !ir->rhs)
      return false;

   /* LHS and RHS must be arrays
    * FIXME: arrays of arrays?
    */
   if (!ir->lhs->type->is_array() || !ir->rhs->type->is_array())
      return false;

   /* RHS must be a buffer-backed variable. This is what can cause the problem
    * since it would lead to a series of loads that need to live until we
    * see the writes to the LHS.
    */
   ir_variable *rhs_var = ir->rhs->variable_referenced();
   if (!rhs_var || !is_buffer_backed_variable(rhs_var))
      return false;

   /* Split the array copy into individual element copies to reduce
    * register pressure
    */
   ir_dereference *rhs_deref = ir->rhs->as_dereference();
   if (!rhs_deref)
      return false;

   ir_dereference *lhs_deref = ir->lhs->as_dereference();
   if (!lhs_deref)
      return false;

   assert(lhs_deref->type->length == rhs_deref->type->length);
   void *mem_ctx = ralloc_parent(shader->ir);

   for (unsigned i = 0; i < lhs_deref->type->length; i++) {
      ir_dereference *lhs_i =
         new(mem_ctx) ir_dereference_array(lhs_deref->clone(mem_ctx, NULL),
                                           new(mem_ctx) ir_constant(i));

      ir_dereference *rhs_i =
         new(mem_ctx) ir_dereference_array(rhs_deref->clone(mem_ctx, NULL),
                                           new(mem_ctx) ir_constant(i));
      ir->insert_after(assign(lhs_i, rhs_i));
   }

   ir->remove();
   progress = true;
   return true;
}

bool
lower_ubo_reference_visitor::check_for_buffer_struct_copy(ir_assignment *ir)
{
   if (!ir || !ir->lhs || !ir->rhs)
      return false;

   /* LHS and RHS must be records */
   if (!ir->lhs->type->is_struct() || !ir->rhs->type->is_struct())
      return false;

   /* RHS must be a buffer-backed variable. This is what can cause the problem
    * since it would lead to a series of loads that need to live until we
    * see the writes to the LHS.
    */
   ir_variable *rhs_var = ir->rhs->variable_referenced();
   if (!rhs_var || !is_buffer_backed_variable(rhs_var))
      return false;

   /* Split the struct copy into individual element copies to reduce
    * register pressure
    */
   ir_dereference *rhs_deref = ir->rhs->as_dereference();
   if (!rhs_deref)
      return false;

   ir_dereference *lhs_deref = ir->lhs->as_dereference();
   if (!lhs_deref)
      return false;

   assert(lhs_deref->type == rhs_deref->type);
   void *mem_ctx = ralloc_parent(shader->ir);

   for (unsigned i = 0; i < lhs_deref->type->length; i++) {
      const char *field_name = lhs_deref->type->fields.structure[i].name;
      ir_dereference *lhs_field =
         new(mem_ctx) ir_dereference_record(lhs_deref->clone(mem_ctx, NULL),
                                            field_name);
      ir_dereference *rhs_field =
         new(mem_ctx) ir_dereference_record(rhs_deref->clone(mem_ctx, NULL),
                                            field_name);
      ir->insert_after(assign(lhs_field, rhs_field));
   }

   ir->remove();
   progress = true;
   return true;
}

ir_visitor_status
lower_ubo_reference_visitor::visit_enter(ir_assignment *ir)
{
   /* Array and struct copies could involve large amounts of load/store
    * operations. To improve register pressure we want to special-case
    * these and split them into individual element copies.
    * This way we avoid emitting all the loads for the RHS first and
    * all the writes for the LHS second and register usage is more
    * efficient.
    */
   if (check_for_buffer_array_copy(ir))
      return visit_continue_with_parent;

   if (check_for_buffer_struct_copy(ir))
      return visit_continue_with_parent;

   check_ssbo_unsized_array_length_assignment(ir);
   check_for_ssbo_store(ir);
   return rvalue_visit(ir);
}

/* Lowers the intrinsic call to a new internal intrinsic that swaps the
 * access to the buffer variable in the first parameter by an offset
 * and block index. This involves creating the new internal intrinsic
 * (i.e. the new function signature).
 */
ir_call *
lower_ubo_reference_visitor::lower_ssbo_atomic_intrinsic(ir_call *ir)
{
   /* SSBO atomics usually have 2 parameters, the buffer variable and an
    * integer argument. The exception is CompSwap, that has an additional
    * integer parameter.
    */
   int param_count = ir->actual_parameters.length();
   assert(param_count == 2 || param_count == 3);

   /* First argument must be a scalar integer buffer variable */
   exec_node *param = ir->actual_parameters.get_head();
   ir_instruction *inst = (ir_instruction *) param;
   assert(inst->ir_type == ir_type_dereference_variable ||
          inst->ir_type == ir_type_dereference_array ||
          inst->ir_type == ir_type_dereference_record ||
          inst->ir_type == ir_type_swizzle);

   ir_rvalue *deref = (ir_rvalue *) inst;
   assert(deref->type->is_scalar() &&
          (deref->type->is_integer_32() || deref->type->is_float()));

   ir_variable *var = deref->variable_referenced();
   assert(var);

   /* Compute the offset to the start if the dereference and the
    * block index
    */
   void *mem_ctx = ralloc_parent(shader->ir);

   ir_rvalue *offset = NULL;
   unsigned const_offset;
   bool row_major;
   const glsl_type *matrix_type;

   enum glsl_interface_packing packing =
      var->get_interface_type()->
         get_internal_ifc_packing(use_std430_as_default);

   this->buffer_access_type = ssbo_atomic_access;
   this->variable = var;

   setup_for_load_or_store(mem_ctx, var, deref,
                           &offset, &const_offset,
                           &row_major, &matrix_type,
                           packing);
   assert(offset);
   assert(!row_major);
   assert(matrix_type == NULL);

   ir_rvalue *deref_offset =
      add(offset, new(mem_ctx) ir_constant(const_offset));
   ir_rvalue *block_index = this->uniform_block->clone(mem_ctx, NULL);

   /* Create the new internal function signature that will take a block
    * index and offset instead of a buffer variable
    */
   exec_list sig_params;
   ir_variable *sig_param = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "block_ref" , ir_var_function_in);
   sig_params.push_tail(sig_param);

   sig_param = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset" , ir_var_function_in);
   sig_params.push_tail(sig_param);

   const glsl_type *type = deref->type->get_scalar_type();
   sig_param = new(mem_ctx)
         ir_variable(type, "data1", ir_var_function_in);
   sig_params.push_tail(sig_param);

   if (param_count == 3) {
      sig_param = new(mem_ctx)
            ir_variable(type, "data2", ir_var_function_in);
      sig_params.push_tail(sig_param);
   }

   ir_function_signature *sig =
      new(mem_ctx) ir_function_signature(deref->type,
                                         shader_storage_buffer_object);
   assert(sig);
   sig->replace_parameters(&sig_params);

   assert(ir->callee->intrinsic_id >= ir_intrinsic_generic_load);
   assert(ir->callee->intrinsic_id <= ir_intrinsic_generic_atomic_comp_swap);
   sig->intrinsic_id = MAP_INTRINSIC_TO_TYPE(ir->callee->intrinsic_id, ssbo);

   char func_name[64];
   sprintf(func_name, "%s_ssbo", ir->callee_name());
   ir_function *f = new(mem_ctx) ir_function(func_name);
   f->add_signature(sig);

   /* Now, create the call to the internal intrinsic */
   exec_list call_params;
   call_params.push_tail(block_index);
   call_params.push_tail(deref_offset);
   param = ir->actual_parameters.get_head()->get_next();
   ir_rvalue *param_as_rvalue = ((ir_instruction *) param)->as_rvalue();
   call_params.push_tail(param_as_rvalue->clone(mem_ctx, NULL));
   if (param_count == 3) {
      param = param->get_next();
      param_as_rvalue = ((ir_instruction *) param)->as_rvalue();
      call_params.push_tail(param_as_rvalue->clone(mem_ctx, NULL));
   }
   ir_dereference_variable *return_deref =
      ir->return_deref->clone(mem_ctx, NULL);
   return new(mem_ctx) ir_call(sig, return_deref, &call_params);
}

ir_call *
lower_ubo_reference_visitor::check_for_ssbo_atomic_intrinsic(ir_call *ir)
{
   exec_list& params = ir->actual_parameters;

   if (params.length() < 2 || params.length() > 3)
      return ir;

   ir_rvalue *rvalue =
      ((ir_instruction *) params.get_head())->as_rvalue();
   if (!rvalue)
      return ir;

   ir_variable *var = rvalue->variable_referenced();
   if (!var || !var->is_in_shader_storage_block())
      return ir;

   const enum ir_intrinsic_id id = ir->callee->intrinsic_id;
   if (id == ir_intrinsic_generic_atomic_add ||
       id == ir_intrinsic_generic_atomic_min ||
       id == ir_intrinsic_generic_atomic_max ||
       id == ir_intrinsic_generic_atomic_and ||
       id == ir_intrinsic_generic_atomic_or ||
       id == ir_intrinsic_generic_atomic_xor ||
       id == ir_intrinsic_generic_atomic_exchange ||
       id == ir_intrinsic_generic_atomic_comp_swap) {
      return lower_ssbo_atomic_intrinsic(ir);
   }

   return ir;
}


ir_visitor_status
lower_ubo_reference_visitor::visit_enter(ir_call *ir)
{
   ir_call *new_ir = check_for_ssbo_atomic_intrinsic(ir);
   if (new_ir != ir) {
      progress = true;
      base_ir->replace_with(new_ir);
      return visit_continue_with_parent;
   }

   return rvalue_visit(ir);
}


ir_visitor_status
lower_ubo_reference_visitor::visit_enter(ir_texture *ir)
{
   ir_dereference *sampler = ir->sampler;

   if (sampler->ir_type == ir_type_dereference_record) {
      handle_rvalue((ir_rvalue **)&ir->sampler);
      return visit_continue_with_parent;
   }

   return rvalue_visit(ir);
}


} /* unnamed namespace */

void
lower_ubo_reference(struct gl_linked_shader *shader,
                    bool clamp_block_indices, bool use_std430_as_default)
{
   lower_ubo_reference_visitor v(shader, clamp_block_indices,
                                 use_std430_as_default);

   /* Loop over the instructions lowering references, because we take
    * a deref of a UBO array using a UBO dereference as the index will
    * produce a collection of instructions all of which have cloned
    * UBO dereferences for that array index.
    */
   do {
      v.progress = false;
      visit_list_elements(&v, shader->ir);
   } while (v.progress);
}
