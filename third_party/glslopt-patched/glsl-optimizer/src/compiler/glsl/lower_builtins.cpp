/*
 * Copyright © 2019 Google, Inc.
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
 * \file lower_builtins.cpp
 *
 * Inline calls to builtin functions.
 */

#include "ir.h"
#include "ir_optimization.h"

namespace {

class lower_builtins_visitor : public ir_hierarchical_visitor {
public:
   lower_builtins_visitor() : progress(false) { }
   ir_visitor_status visit_leave(ir_call *);
   bool progress;
};

}

bool
lower_builtins(exec_list *instructions)
{
   lower_builtins_visitor v;
   visit_list_elements(&v, instructions);
   return v.progress;
}

ir_visitor_status
lower_builtins_visitor::visit_leave(ir_call *ir)
{
   if (!ir->callee->is_builtin())
      return visit_continue;

   ir->generate_inline(ir);
   ir->remove();

   this->progress = true;

   return visit_continue;
}
