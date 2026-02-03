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
 * \file ir_function_can_inline.cpp
 *
 * Determines if we can inline a function call using ir_function_inlining.cpp.
 *
 * The primary restriction is that we can't return from the function other
 * than as the last instruction.  In lower_jumps.cpp, we can lower return
 * statements not at the end of the function to other control flow in order to
 * deal with this restriction.
 */

#include "ir.h"

class ir_function_can_inline_visitor : public ir_hierarchical_visitor {
public:
   ir_function_can_inline_visitor()
   {
      this->num_returns = 0;
   }

   virtual ir_visitor_status visit_enter(ir_return *);

   int num_returns;
};

ir_visitor_status
ir_function_can_inline_visitor::visit_enter(ir_return *ir)
{
   (void) ir;
   this->num_returns++;
   return visit_continue;
}

bool
can_inline(ir_call *call)
{
   ir_function_can_inline_visitor v;
   const ir_function_signature *callee = call->callee;
   if (!callee->is_defined)
      return false;

   v.run((exec_list *) &callee->body);

   /* If the function is empty (no last instruction) or does not end with a
    * return statement, we need to count the implicit return.
    */
   ir_instruction *last = (ir_instruction *)callee->body.get_tail();
   if (last == NULL || !last->as_return())
      v.num_returns++;

   return v.num_returns == 1;
}
