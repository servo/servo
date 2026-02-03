/*
 * Copyright © 2014 Intel Corporation
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
 * \file opt_conditional_discard.cpp
 *
 * Replace
 *
 *    if (cond) discard;
 *
 * with
 *
 *    (discard <condition>)
 */

#include "compiler/glsl_types.h"
#include "ir.h"

namespace {

class opt_conditional_discard_visitor : public ir_hierarchical_visitor {
public:
   opt_conditional_discard_visitor()
   {
      progress = false;
   }

   ir_visitor_status visit_leave(ir_if *);

   bool progress;
};

} /* anonymous namespace */

bool
opt_conditional_discard(exec_list *instructions)
{
   opt_conditional_discard_visitor v;
   v.run(instructions);
   return v.progress;
}

ir_visitor_status
opt_conditional_discard_visitor::visit_leave(ir_if *ir)
{
   /* Look for "if (...) discard" with no else clause or extra statements. */
   if (ir->then_instructions.is_empty() ||
       !ir->then_instructions.get_head_raw()->next->is_tail_sentinel() ||
       !((ir_instruction *) ir->then_instructions.get_head_raw())->as_discard() ||
       !ir->else_instructions.is_empty())
      return visit_continue;

   /* Move the condition and replace the ir_if with the ir_discard. */
   ir_discard *discard = (ir_discard *) ir->then_instructions.get_head_raw();
   if (!discard->condition)
      discard->condition = ir->condition;
   else {
      void *ctx = ralloc_parent(ir);
      discard->condition = new(ctx) ir_expression(ir_binop_logic_and,
                                                  ir->condition,
                                                  discard->condition);
   }
   ir->replace_with(discard);

   progress = true;

   return visit_continue;
}
