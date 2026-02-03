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
 * \file lower_texture_projection.cpp
 *
 * IR lower pass to perform the division of texture coordinates by the texture
 * projector if present.
 *
 * Many GPUs have a texture sampling opcode that takes the projector
 * and does the divide internally, thus the presence of the projector
 * in the IR.  For GPUs that don't, this saves the driver needing the
 * logic for handling the divide.
 *
 * \author Eric Anholt <eric@anholt.net>
 */

#include "ir.h"

namespace {

class lower_texture_projection_visitor : public ir_hierarchical_visitor {
public:
   lower_texture_projection_visitor()
   {
      progress = false;
   }

   ir_visitor_status visit_leave(ir_texture *ir);

   bool progress;
};

} /* anonymous namespace */

ir_visitor_status
lower_texture_projection_visitor::visit_leave(ir_texture *ir)
{
   if (!ir->projector)
      return visit_continue;

   void *mem_ctx = ralloc_parent(ir);

   ir_variable *var = new(mem_ctx) ir_variable(ir->projector->type,
					       "projector", ir_var_temporary);
   base_ir->insert_before(var);
   ir_dereference *deref = new(mem_ctx) ir_dereference_variable(var);
   ir_expression *expr = new(mem_ctx) ir_expression(ir_unop_rcp,
						    ir->projector->type,
						    ir->projector,
						    NULL);
   ir_assignment *assign = new(mem_ctx) ir_assignment(deref, expr);
   base_ir->insert_before(assign);

   deref = new(mem_ctx) ir_dereference_variable(var);
   ir->coordinate = new(mem_ctx) ir_expression(ir_binop_mul,
					       ir->coordinate->type,
					       ir->coordinate,
					       deref);

   if (ir->shadow_comparator) {
      deref = new(mem_ctx) ir_dereference_variable(var);
      ir->shadow_comparator = new(mem_ctx) ir_expression(ir_binop_mul,
						  ir->shadow_comparator->type,
						  ir->shadow_comparator,
						  deref);
   }

   ir->projector = NULL;

   progress = true;
   return visit_continue;
}

bool
do_lower_texture_projection(exec_list *instructions)
{
   lower_texture_projection_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}
