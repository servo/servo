/*
 * Copyright © 2016 Intel Corporation
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
 * \file propagate_invariance.cpp
 * Propagate the "invariant" and "precise" qualifiers to variables used to
 * compute invariant or precise values.
 *
 * The GLSL spec (depending on what version you read) says, among the
 * conditions for getting bit-for-bit the same values on an invariant output:
 *
 *    "All operations in the consuming expressions and any intermediate
 *    expressions must be the same, with the same order of operands and same
 *    associativity, to give the same order of evaluation."
 *
 * This effectively means that if a variable is used to compute an invariant
 * value then that variable becomes invariant.  The same should apply to the
 * "precise" qualifier.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"

namespace {

class ir_invariance_propagation_visitor : public ir_hierarchical_visitor {
public:
   ir_invariance_propagation_visitor()
   {
      this->progress = false;
      this->dst_var = NULL;
   }

   virtual ~ir_invariance_propagation_visitor()
   {
      /* empty */
   }

   virtual ir_visitor_status visit_enter(ir_assignment *ir);
   virtual ir_visitor_status visit_leave(ir_assignment *ir);
   virtual ir_visitor_status visit(ir_dereference_variable *ir);

   ir_variable *dst_var;
   bool progress;
};

} /* unnamed namespace */

ir_visitor_status
ir_invariance_propagation_visitor::visit_enter(ir_assignment *ir)
{
   assert(this->dst_var == NULL);
   ir_variable *var = ir->lhs->variable_referenced();
   if (var->data.invariant || var->data.precise) {
      this->dst_var = var;
      return visit_continue;
   } else {
      return visit_continue_with_parent;
   }
}

ir_visitor_status
ir_invariance_propagation_visitor::visit_leave(ir_assignment *)
{
   this->dst_var = NULL;

   return visit_continue;
}

ir_visitor_status
ir_invariance_propagation_visitor::visit(ir_dereference_variable *ir)
{
   if (this->dst_var == NULL)
      return visit_continue;

   if (this->dst_var->data.invariant) {
      if (!ir->var->data.invariant)
         this->progress = true;

      ir->var->data.invariant = true;
   }

   if (this->dst_var->data.precise) {
      if (!ir->var->data.precise)
         this->progress = true;

      ir->var->data.precise = true;
   }

   return visit_continue;
}

void
propagate_invariance(exec_list *instructions)
{
   ir_invariance_propagation_visitor visitor;

   do {
      visitor.progress = false;
      visit_list_elements(&visitor, instructions);
   } while (visitor.progress);
}
