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

using namespace ir_builder;

namespace {

class vector_insert_visitor : public ir_rvalue_visitor {
public:
   vector_insert_visitor(bool lower_nonconstant_index)
      : progress(false), lower_nonconstant_index(lower_nonconstant_index)
   {
      factory.instructions = &factory_instructions;
   }

   virtual ~vector_insert_visitor()
   {
      assert(factory_instructions.is_empty());
   }

   virtual void handle_rvalue(ir_rvalue **rv);

   ir_factory factory;
   exec_list factory_instructions;
   bool progress;
   bool lower_nonconstant_index;
};

} /* anonymous namespace */

void
vector_insert_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL || (*rv)->ir_type != ir_type_expression)
      return;

   ir_expression *const expr = (ir_expression *) *rv;

   if (likely(expr->operation != ir_triop_vector_insert))
      return;

   factory.mem_ctx = ralloc_parent(expr);

   ir_constant *const idx =
      expr->operands[2]->constant_expression_value(factory.mem_ctx);
   if (idx != NULL) {
      /* Replace (vector_insert (vec) (scalar) (index)) with a dereference of
       * a new temporary.  The new temporary gets assigned as
       *
       *     t = vec
       *     t.mask = scalar
       *
       * where mask is the component selected by index.
       */
      ir_variable *const temp =
         factory.make_temp(expr->operands[0]->type, "vec_tmp");

      const int mask = 1 << idx->value.i[0];

      factory.emit(assign(temp, expr->operands[0]));
      factory.emit(assign(temp, expr->operands[1], mask));

      this->progress = true;
      *rv = new(factory.mem_ctx) ir_dereference_variable(temp);
   } else if (this->lower_nonconstant_index) {
      /* Replace (vector_insert (vec) (scalar) (index)) with a dereference of
       * a new temporary.  The new temporary gets assigned as
       *
       *     t = vec
       *     if (index == 0)
       *         t.x = scalar
       *     if (index == 1)
       *         t.y = scalar
       *     if (index == 2)
       *         t.z = scalar
       *     if (index == 3)
       *         t.w = scalar
       */
      ir_variable *const temp =
         factory.make_temp(expr->operands[0]->type, "vec_tmp");

      ir_variable *const src_temp =
         factory.make_temp(expr->operands[1]->type, "src_temp");

      factory.emit(assign(temp, expr->operands[0]));
      factory.emit(assign(src_temp, expr->operands[1]));

      assert(expr->operands[2]->type == glsl_type::int_type ||
             expr->operands[2]->type == glsl_type::uint_type);

      for (unsigned i = 0; i < expr->type->vector_elements; i++) {
         ir_constant *const cmp_index =
            ir_constant::zero(factory.mem_ctx, expr->operands[2]->type);
         cmp_index->value.u[0] = i;

         ir_variable *const cmp_result =
            factory.make_temp(glsl_type::bool_type, "index_condition");

         factory.emit(assign(cmp_result,
                             equal(expr->operands[2]->clone(factory.mem_ctx,
                                                            NULL),
                                   cmp_index)));

         factory.emit(if_tree(cmp_result,
                              assign(temp, src_temp, WRITEMASK_X << i)));
      }

      this->progress = true;
      *rv = new(factory.mem_ctx) ir_dereference_variable(temp);
   }

   base_ir->insert_before(factory.instructions);
}

bool
lower_vector_insert(exec_list *instructions, bool lower_nonconstant_index)
{
   vector_insert_visitor v(lower_nonconstant_index);

   visit_list_elements(&v, instructions);

   return v.progress;
}
