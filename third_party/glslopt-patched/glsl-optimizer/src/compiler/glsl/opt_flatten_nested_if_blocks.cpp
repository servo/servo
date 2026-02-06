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

/**
 * \file opt_flatten_nested_if_blocks.cpp
 *
 * Flattens nested if blocks such as:
 *
 * if (x) {
 *    if (y) {
 *       ...
 *    }
 * }
 *
 * into a single if block with a combined condition:
 *
 * if (x && y) {
 *    ...
 * }
 */

#include "ir.h"
#include "ir_builder.h"

using namespace ir_builder;

namespace {

class nested_if_flattener : public ir_hierarchical_visitor {
public:
   nested_if_flattener()
   {
      progress = false;
   }

   ir_visitor_status visit_leave(ir_if *);
   ir_visitor_status visit_enter(ir_assignment *);

   bool progress;
};

} /* unnamed namespace */

/* We only care about the top level "if" instructions, so don't
 * descend into expressions.
 */
ir_visitor_status
nested_if_flattener::visit_enter(ir_assignment *ir)
{
   (void) ir;
   return visit_continue_with_parent;
}

bool
opt_flatten_nested_if_blocks(exec_list *instructions)
{
   nested_if_flattener v;

   v.run(instructions);
   return v.progress;
}


ir_visitor_status
nested_if_flattener::visit_leave(ir_if *ir)
{
   /* Only handle a single ir_if within the then clause of an ir_if.  No extra
    * instructions, no else clauses, nothing.
    */
   if (ir->then_instructions.is_empty() || !ir->else_instructions.is_empty())
      return visit_continue;

   ir_if *inner = ((ir_instruction *) ir->then_instructions.get_head_raw())->as_if();
   if (!inner || !inner->next->is_tail_sentinel() ||
       !inner->else_instructions.is_empty())
      return visit_continue;

   ir->condition = logic_and(ir->condition, inner->condition);
   inner->then_instructions.move_nodes_to(&ir->then_instructions);

   progress = true;
   return visit_continue;
}
