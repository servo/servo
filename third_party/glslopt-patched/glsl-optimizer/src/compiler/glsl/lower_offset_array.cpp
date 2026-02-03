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
 * \file lower_offset_array.cpp
 *
 * IR lower pass to decompose ir_texture ir_tg4 with an array of offsets
 * into four ir_tg4s with a single ivec2 offset, select the .w component of each,
 * and return those four values packed into a gvec4.
 *
 * \author Chris Forbes <chrisf@ijw.co.nz>
 */

#include "compiler/glsl_types.h"
#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "ir_rvalue_visitor.h"

using namespace ir_builder;

class lower_offset_array_visitor : public ir_rvalue_visitor {
public:
   lower_offset_array_visitor()
   {
      progress = false;
   }

   void handle_rvalue(ir_rvalue **rv);

   bool progress;
};

void
lower_offset_array_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL || (*rv)->ir_type != ir_type_texture)
      return;

   ir_texture *ir = (ir_texture *) *rv;
   if (ir->op != ir_tg4 || !ir->offset || !ir->offset->type->is_array())
      return;

   void *mem_ctx = ralloc_parent(ir);

   ir_variable *var =
      new (mem_ctx) ir_variable(ir->type, "result", ir_var_temporary);
   base_ir->insert_before(var);

   for (int i = 0; i < 4; i++) {
      ir_texture *tex = ir->clone(mem_ctx, NULL);
      tex->offset = new (mem_ctx) ir_dereference_array(tex->offset,
            new (mem_ctx) ir_constant(i));

      base_ir->insert_before(assign(var, swizzle_w(tex), 1 << i));
   }

   *rv = new (mem_ctx) ir_dereference_variable(var);

   progress = true;
}

bool
lower_offset_arrays(exec_list *instructions)
{
   lower_offset_array_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}
