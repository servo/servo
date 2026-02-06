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
 * \file lower_vec_index_to_swizzle.cpp
 *
 * Turns constant indexing into vector types to swizzles.  This will
 * let other swizzle-aware optimization passes catch these constructs,
 * and codegen backends not have to worry about this case.
 */

#include "ir.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "main/macros.h"

namespace {

class ir_vec_index_to_swizzle_visitor : public ir_rvalue_visitor {
public:
   ir_vec_index_to_swizzle_visitor()
   {
      progress = false;
   }

   ir_rvalue *convert_vector_extract_to_swizzle(ir_rvalue *val);

   virtual void handle_rvalue(ir_rvalue **);

   bool progress;
};

} /* anonymous namespace */

void
ir_vec_index_to_swizzle_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL)
      return;

   ir_expression *const expr = (*rv)->as_expression();
   if (expr == NULL || expr->operation != ir_binop_vector_extract)
      return;

   void *mem_ctx = ralloc_parent(expr);
   ir_constant *const idx =
      expr->operands[1]->constant_expression_value(mem_ctx);
   if (idx == NULL)
      return;

   this->progress = true;

   /* Page 40 of the GLSL 1.20 spec says:
    *
    *     "When indexing with non-constant expressions, behavior is undefined
    *     if the index is negative, or greater than or equal to the size of
    *     the vector."
    *
    * The quoted spec text mentions non-constant expressions, but this code
    * operates on constants.  These constants are the result of non-constant
    * expressions that have been optimized to constants.  The common case here
    * is a loop counter from an unrolled loop that is used to index a vector.
    *
    * The ir_swizzle constructor gets angry if the index is negative or too
    * large.  For simplicity sake, just clamp the index to [0, size-1].
    */
   const int i = CLAMP(idx->value.i[0], 0,
                       (int) expr->operands[0]->type->vector_elements - 1);

   *rv = new(mem_ctx) ir_swizzle(expr->operands[0], i, 0, 0, 0, 1);
}

bool
do_vec_index_to_swizzle(exec_list *instructions)
{
   ir_vec_index_to_swizzle_visitor v;

   v.run(instructions);

   return v.progress;
}
