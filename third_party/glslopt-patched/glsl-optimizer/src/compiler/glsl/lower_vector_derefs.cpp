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
#include "ir.h"
#include "ir_builder.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "main/mtypes.h"

using namespace ir_builder;

namespace {

class vector_deref_visitor : public ir_rvalue_enter_visitor {
public:
   vector_deref_visitor(void *mem_ctx, gl_shader_stage shader_stage)
      : progress(false), shader_stage(shader_stage),
        factory(&factory_instructions, mem_ctx)
   {
   }

   virtual ~vector_deref_visitor()
   {
   }

   virtual void handle_rvalue(ir_rvalue **rv);
   virtual ir_visitor_status visit_enter(ir_assignment *ir);

   bool progress;
   gl_shader_stage shader_stage;
   exec_list factory_instructions;
   ir_factory factory;
};

} /* anonymous namespace */

ir_visitor_status
vector_deref_visitor::visit_enter(ir_assignment *ir)
{
   if (!ir->lhs || ir->lhs->ir_type != ir_type_dereference_array)
      return ir_rvalue_enter_visitor::visit_enter(ir);

   ir_dereference_array *const deref = (ir_dereference_array *) ir->lhs;
   if (!deref->array->type->is_vector())
      return ir_rvalue_enter_visitor::visit_enter(ir);

   /* SSBOs and shared variables are backed by memory and may be accessed by
    * multiple threads simultaneously.  It's not safe to lower a single
    * component store to a load-vec-store because it may race with writes to
    * other components.
    */
   ir_variable *var = deref->variable_referenced();
   if (var->data.mode == ir_var_shader_storage ||
       var->data.mode == ir_var_shader_shared)
      return ir_rvalue_enter_visitor::visit_enter(ir);

   ir_rvalue *const new_lhs = deref->array;

   void *mem_ctx = ralloc_parent(ir);
   ir_constant *old_index_constant =
      deref->array_index->constant_expression_value(mem_ctx);
   if (!old_index_constant) {
      if (shader_stage == MESA_SHADER_TESS_CTRL &&
          deref->variable_referenced()->data.mode == ir_var_shader_out) {
         /* Tessellation control shader outputs act as if they have memory
          * backing them and if we have writes from multiple threads
          * targeting the same vec4 (this can happen for patch outputs), the
          * load-vec-store pattern of ir_triop_vector_insert doesn't work.
          * Instead, we have to lower to a series of conditional write-masked
          * assignments.
          */
         ir_variable *const src_temp =
            factory.make_temp(ir->rhs->type, "scalar_tmp");

         /* The newly created variable declaration goes before the assignment
          * because we're going to set it as the new LHS.
          */
         ir->insert_before(factory.instructions);
         ir->set_lhs(new(mem_ctx) ir_dereference_variable(src_temp));

         ir_variable *const arr_index =
            factory.make_temp(deref->array_index->type, "index_tmp");
         factory.emit(assign(arr_index, deref->array_index));

         for (unsigned i = 0; i < new_lhs->type->vector_elements; i++) {
            ir_constant *const cmp_index =
               ir_constant::zero(factory.mem_ctx, deref->array_index->type);
            cmp_index->value.u[0] = i;

            ir_rvalue *const lhs_clone = new_lhs->clone(factory.mem_ctx, NULL);
            ir_dereference_variable *const src_temp_deref =
               new(mem_ctx) ir_dereference_variable(src_temp);

            if (new_lhs->ir_type != ir_type_swizzle) {
               assert(lhs_clone->as_dereference());
               ir_assignment *cond_assign =
                  new(mem_ctx) ir_assignment(lhs_clone->as_dereference(),
                                             src_temp_deref,
                                             equal(arr_index, cmp_index),
                                             WRITEMASK_X << i);
               factory.emit(cond_assign);
            } else {
               ir_assignment *cond_assign =
                  new(mem_ctx) ir_assignment(swizzle(lhs_clone, i, 1),
                                             src_temp_deref,
                                             equal(arr_index, cmp_index));
               factory.emit(cond_assign);
            }
         }
         ir->insert_after(factory.instructions);
      } else {
         ir->rhs = new(mem_ctx) ir_expression(ir_triop_vector_insert,
                                              new_lhs->type,
                                              new_lhs->clone(mem_ctx, NULL),
                                              ir->rhs,
                                              deref->array_index);
         ir->write_mask = (1 << new_lhs->type->vector_elements) - 1;
         ir->set_lhs(new_lhs);
      }
   } else if (new_lhs->ir_type != ir_type_swizzle) {
      ir->set_lhs(new_lhs);
      ir->write_mask = 1 << old_index_constant->get_uint_component(0);
   } else {
      /* If the "new" LHS is a swizzle, use the set_lhs helper to instead
       * swizzle the RHS.
       */
      unsigned component[1] = { old_index_constant->get_uint_component(0) };
      ir->set_lhs(new(mem_ctx) ir_swizzle(new_lhs, component, 1));
   }

   return ir_rvalue_enter_visitor::visit_enter(ir);
}

void
vector_deref_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL || (*rv)->ir_type != ir_type_dereference_array)
      return;

   ir_dereference_array *const deref = (ir_dereference_array *) *rv;
   if (!deref->array->type->is_vector())
      return;

   /* Back-ends need to be able to handle derefs on vectors for SSBOs, UBOs,
    * and shared variables.  They have to handle it for writes anyway so we
    * may as well require it for reads.
    */
   ir_variable *var = deref->variable_referenced();
   if (var && (var->data.mode == ir_var_shader_storage ||
               var->data.mode == ir_var_shader_shared ||
               (var->data.mode == ir_var_uniform &&
                var->get_interface_type())))
      return;

   void *mem_ctx = ralloc_parent(deref);
   *rv = new(mem_ctx) ir_expression(ir_binop_vector_extract,
                                    deref->array,
                                    deref->array_index);
}

bool
lower_vector_derefs(gl_linked_shader *shader)
{
   vector_deref_visitor v(shader->ir, shader->Stage);

   visit_list_elements(&v, shader->ir);

   return v.progress;
}
