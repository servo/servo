/*
 * Copyright © 2010 Intel Corporation
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
 * \file lower_discard.cpp
 *
 * This pass moves discards out of if-statements.
 *
 * Case 1: The "then" branch contains a conditional discard:
 * ---------------------------------------------------------
 *
 *    if (cond1) {
 *	 s1;
 *	 discard cond2;
 *	 s2;
 *    } else {
 *	 s3;
 *    }
 *
 * becomes:
 *
 *    temp = false;
 *    if (cond1) {
 *	 s1;
 *	 temp = cond2;
 *	 s2;
 *    } else {
 *	 s3;
 *    }
 *    discard temp;
 *
 * Case 2: The "else" branch contains a conditional discard:
 * ---------------------------------------------------------
 *
 *    if (cond1) {
 *	 s1;
 *    } else {
 *	 s2;
 *	 discard cond2;
 *	 s3;
 *    }
 *
 * becomes:
 *
 *    temp = false;
 *    if (cond1) {
 *	 s1;
 *    } else {
 *	 s2;
 *	 temp = cond2;
 *	 s3;
 *    }
 *    discard temp;
 *
 * Case 3: Both branches contain a conditional discard:
 * ----------------------------------------------------
 *
 *    if (cond1) {
 *	 s1;
 *	 discard cond2;
 *	 s2;
 *    } else {
 *	 s3;
 *	 discard cond3;
 *	 s4;
 *    }
 *
 * becomes:
 *
 *    temp = false;
 *    if (cond1) {
 *	 s1;
 *	 temp = cond2;
 *	 s2;
 *    } else {
 *	 s3;
 *	 temp = cond3;
 *	 s4;
 *    }
 *    discard temp;
 *
 * If there are multiple conditional discards, we need only deal with one of
 * them.  Repeatedly applying this pass will take care of the others.
 *
 * Unconditional discards are treated as having a condition of "true".
 */

#include "compiler/glsl_types.h"
#include "ir.h"

namespace {

class lower_discard_visitor : public ir_hierarchical_visitor {
public:
   lower_discard_visitor()
   {
      this->progress = false;
   }

   ir_visitor_status visit_leave(ir_if *);

   bool progress;
};

} /* anonymous namespace */

bool
lower_discard(exec_list *instructions)
{
   lower_discard_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}


static ir_discard *
find_discard(exec_list &instructions)
{
   foreach_in_list(ir_instruction, node, &instructions) {
      ir_discard *ir = node->as_discard();
      if (ir != NULL)
	 return ir;
   }
   return NULL;
}


static void
replace_discard(void *mem_ctx, ir_variable *var, ir_discard *ir)
{
   ir_rvalue *condition = ir->condition;

   /* For unconditional discards, use "true" as the condition. */
   if (condition == NULL)
      condition = new(mem_ctx) ir_constant(true);

   ir_assignment *assignment =
      new(mem_ctx) ir_assignment(new(mem_ctx) ir_dereference_variable(var),
                                 condition);

   ir->replace_with(assignment);
}


ir_visitor_status
lower_discard_visitor::visit_leave(ir_if *ir)
{
   ir_discard *then_discard = find_discard(ir->then_instructions);
   ir_discard *else_discard = find_discard(ir->else_instructions);

   if (then_discard == NULL && else_discard == NULL)
      return visit_continue;

   void *mem_ctx = ralloc_parent(ir);

   ir_variable *temp = new(mem_ctx) ir_variable(glsl_type::bool_type,
						"discard_cond_temp",
						ir_var_temporary);
   ir_assignment *temp_initializer =
      new(mem_ctx) ir_assignment(new(mem_ctx) ir_dereference_variable(temp),
                                 new(mem_ctx) ir_constant(false));

   ir->insert_before(temp);
   ir->insert_before(temp_initializer);

   if (then_discard != NULL)
      replace_discard(mem_ctx, temp, then_discard);

   if (else_discard != NULL)
      replace_discard(mem_ctx, temp, else_discard);

   ir_discard *discard = then_discard != NULL ? then_discard : else_discard;
   discard->condition = new(mem_ctx) ir_dereference_variable(temp);
   ir->insert_after(discard);

   this->progress = true;

   return visit_continue;
}
