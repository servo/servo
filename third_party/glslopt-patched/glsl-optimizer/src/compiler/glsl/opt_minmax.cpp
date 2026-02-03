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
 * \file opt_minmax.cpp
 *
 * Drop operands from an expression tree of only min/max operations if they
 * can be proven to not contribute to the final result.
 *
 * The algorithm is similar to alpha-beta pruning on a minmax search.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "ir_builder.h"
#include "program/prog_instruction.h"
#include "compiler/glsl_types.h"
#include "main/macros.h"
#include "util/half_float.h"

using namespace ir_builder;

namespace {

enum compare_components_result {
   LESS,
   LESS_OR_EQUAL,
   EQUAL,
   GREATER_OR_EQUAL,
   GREATER,
   MIXED
};

class minmax_range {
public:
   minmax_range(ir_constant *low = NULL, ir_constant *high = NULL)
   {
      this->low = low;
      this->high = high;
   }

   /* low is the lower limit of the range, high is the higher limit. NULL on
    * low means negative infinity (unlimited) and on high positive infinity
    * (unlimited). Because of the two interpretations of the value NULL,
    * arbitrary comparison between ir_constants is impossible.
    */
   ir_constant *low;
   ir_constant *high;
};

class ir_minmax_visitor : public ir_rvalue_enter_visitor {
public:
   ir_minmax_visitor()
      : progress(false)
   {
   }

   ir_rvalue *prune_expression(ir_expression *expr, minmax_range baserange);

   void handle_rvalue(ir_rvalue **rvalue);

   bool progress;
};

/*
 * Returns LESS if all vector components of `a' are strictly lower than of `b',
 * GREATER if all vector components of `a' are strictly greater than of `b',
 * MIXED if some vector components of `a' are strictly lower than of `b' while
 * others are strictly greater, or EQUAL otherwise.
 */
static enum compare_components_result
compare_components(ir_constant *a, ir_constant *b)
{
   assert(a != NULL);
   assert(b != NULL);

   assert(a->type->base_type == b->type->base_type);

   unsigned a_inc = a->type->is_scalar() ? 0 : 1;
   unsigned b_inc = b->type->is_scalar() ? 0 : 1;
   unsigned components = MAX2(a->type->components(), b->type->components());

   bool foundless = false;
   bool foundgreater = false;
   bool foundequal = false;

   for (unsigned i = 0, c0 = 0, c1 = 0;
        i < components;
        c0 += a_inc, c1 += b_inc, ++i) {
      switch (a->type->base_type) {
      case GLSL_TYPE_UINT:
         if (a->value.u[c0] < b->value.u[c1])
            foundless = true;
         else if (a->value.u[c0] > b->value.u[c1])
            foundgreater = true;
         else
            foundequal = true;
         break;
      case GLSL_TYPE_INT:
         if (a->value.i[c0] < b->value.i[c1])
            foundless = true;
         else if (a->value.i[c0] > b->value.i[c1])
            foundgreater = true;
         else
            foundequal = true;
         break;
      case GLSL_TYPE_FLOAT16: {
         float af = _mesa_half_to_float(a->value.f16[c0]);
         float bf = _mesa_half_to_float(b->value.f16[c1]);
         if (af < bf)
            foundless = true;
         else if (af > bf)
            foundgreater = true;
         else
            foundequal = true;
         break;
      }
      case GLSL_TYPE_FLOAT:
         if (a->value.f[c0] < b->value.f[c1])
            foundless = true;
         else if (a->value.f[c0] > b->value.f[c1])
            foundgreater = true;
         else
            foundequal = true;
         break;
      case GLSL_TYPE_DOUBLE:
         if (a->value.d[c0] < b->value.d[c1])
            foundless = true;
         else if (a->value.d[c0] > b->value.d[c1])
            foundgreater = true;
         else
            foundequal = true;
         break;
      default:
         unreachable("not reached");
      }
   }

   if (foundless && foundgreater) {
      /* Some components are strictly lower, others are strictly greater */
      return MIXED;
   }

   if (foundequal) {
       /* It is not mixed, but it is not strictly lower or greater */
      if (foundless)
         return LESS_OR_EQUAL;
      if (foundgreater)
         return GREATER_OR_EQUAL;
      return EQUAL;
   }

   /* All components are strictly lower or strictly greater */
   return foundless ? LESS : GREATER;
}

static ir_constant *
combine_constant(bool ismin, ir_constant *a, ir_constant *b)
{
   void *mem_ctx = ralloc_parent(a);
   ir_constant *c = a->clone(mem_ctx, NULL);
   for (unsigned i = 0; i < c->type->components(); i++) {
      switch (c->type->base_type) {
      case GLSL_TYPE_UINT:
         if ((ismin && b->value.u[i] < c->value.u[i]) ||
             (!ismin && b->value.u[i] > c->value.u[i]))
            c->value.u[i] = b->value.u[i];
         break;
      case GLSL_TYPE_INT:
         if ((ismin && b->value.i[i] < c->value.i[i]) ||
             (!ismin && b->value.i[i] > c->value.i[i]))
            c->value.i[i] = b->value.i[i];
         break;
      case GLSL_TYPE_FLOAT16: {
         float bf = _mesa_half_to_float(b->value.f16[i]);
         float cf = _mesa_half_to_float(c->value.f16[i]);
         if ((ismin && bf < cf) || (!ismin && bf > cf))
            c->value.f16[i] = b->value.f16[i];
         break;
      }
      case GLSL_TYPE_FLOAT:
         if ((ismin && b->value.f[i] < c->value.f[i]) ||
             (!ismin && b->value.f[i] > c->value.f[i]))
            c->value.f[i] = b->value.f[i];
         break;
      case GLSL_TYPE_DOUBLE:
         if ((ismin && b->value.d[i] < c->value.d[i]) ||
             (!ismin && b->value.d[i] > c->value.d[i]))
            c->value.d[i] = b->value.d[i];
         break;
      default:
         assert(!"not reached");
      }
   }
   return c;
}

static ir_constant *
smaller_constant(ir_constant *a, ir_constant *b)
{
   assert(a != NULL);
   assert(b != NULL);

   enum compare_components_result ret = compare_components(a, b);
   if (ret == MIXED)
      return combine_constant(true, a, b);
   else if (ret < EQUAL)
      return a;
   else
      return b;
}

static ir_constant *
larger_constant(ir_constant *a, ir_constant *b)
{
   assert(a != NULL);
   assert(b != NULL);

   enum compare_components_result ret = compare_components(a, b);
   if (ret == MIXED)
      return combine_constant(false, a, b);
   else if (ret < EQUAL)
      return b;
   else
      return a;
}

/* Combines two ranges by doing an element-wise min() / max() depending on the
 * operation.
 */
static minmax_range
combine_range(minmax_range r0, minmax_range r1, bool ismin)
{
   minmax_range ret;

   if (!r0.low) {
      ret.low = ismin ? r0.low : r1.low;
   } else if (!r1.low) {
      ret.low = ismin ? r1.low : r0.low;
   } else {
      ret.low = ismin ? smaller_constant(r0.low, r1.low) :
         larger_constant(r0.low, r1.low);
   }

   if (!r0.high) {
      ret.high = ismin ? r1.high : r0.high;
   } else if (!r1.high) {
      ret.high = ismin ? r0.high : r1.high;
   } else {
      ret.high = ismin ? smaller_constant(r0.high, r1.high) :
         larger_constant(r0.high, r1.high);
   }

   return ret;
}

/* Returns a range so that lower limit is the larger of the two lower limits,
 * and higher limit is the smaller of the two higher limits.
 */
static minmax_range
range_intersection(minmax_range r0, minmax_range r1)
{
   minmax_range ret;

   if (!r0.low)
      ret.low = r1.low;
   else if (!r1.low)
      ret.low = r0.low;
   else
      ret.low = larger_constant(r0.low, r1.low);

   if (!r0.high)
      ret.high = r1.high;
   else if (!r1.high)
      ret.high = r0.high;
   else
      ret.high = smaller_constant(r0.high, r1.high);

   return ret;
}

static minmax_range
get_range(ir_rvalue *rval)
{
   ir_expression *expr = rval->as_expression();
   if (expr && (expr->operation == ir_binop_min ||
                expr->operation == ir_binop_max)) {
      minmax_range r0 = get_range(expr->operands[0]);
      minmax_range r1 = get_range(expr->operands[1]);
      return combine_range(r0, r1, expr->operation == ir_binop_min);
   }

   ir_constant *c = rval->as_constant();
   if (c) {
      return minmax_range(c, c);
   }

   return minmax_range();
}

/**
 * Prunes a min/max expression considering the base range of the parent
 * min/max expression.
 *
 * @param baserange the range that the parents of this min/max expression
 * in the min/max tree will clamp its value to.
 */
ir_rvalue *
ir_minmax_visitor::prune_expression(ir_expression *expr, minmax_range baserange)
{
   assert(expr->operation == ir_binop_min ||
          expr->operation == ir_binop_max);

   bool ismin = expr->operation == ir_binop_min;
   minmax_range limits[2];

   /* Recurse to get the ranges for each of the subtrees of this
    * expression. We need to do this as a separate step because we need to
    * know the ranges of each of the subtrees before we prune either one.
    * Consider something like this:
    *
    *        max
    *     /       \
    *    max     max
    *   /   \   /   \
    *  3    a   b    2
    *
    * We would like to prune away the max on the bottom-right, but to do so
    * we need to know the range of the expression on the left beforehand,
    * and there's no guarantee that we will visit either subtree in a
    * particular order.
    */
   for (unsigned i = 0; i < 2; ++i)
      limits[i] = get_range(expr->operands[i]);

   for (unsigned i = 0; i < 2; ++i) {
      bool is_redundant = false;

      enum compare_components_result cr = LESS;
      if (ismin) {
         /* If this operand will always be greater than the other one, it's
          * redundant.
          */
         if (limits[i].low && limits[1 - i].high) {
               cr = compare_components(limits[i].low, limits[1 - i].high);
            if (cr >= EQUAL && cr != MIXED)
               is_redundant = true;
         }
         /* If this operand is always greater than baserange, then even if
          * it's smaller than the other one it'll get clamped, so it's
          * redundant.
          */
         if (!is_redundant && limits[i].low && baserange.high) {
            cr = compare_components(limits[i].low, baserange.high);
            if (cr > EQUAL && cr != MIXED)
               is_redundant = true;
         }
      } else {
         /* If this operand will always be lower than the other one, it's
          * redundant.
          */
         if (limits[i].high && limits[1 - i].low) {
            cr = compare_components(limits[i].high, limits[1 - i].low);
            if (cr <= EQUAL)
               is_redundant = true;
         }
         /* If this operand is always lower than baserange, then even if
          * it's greater than the other one it'll get clamped, so it's
          * redundant.
          */
         if (!is_redundant && limits[i].high && baserange.low) {
            cr = compare_components(limits[i].high, baserange.low);
            if (cr < EQUAL)
               is_redundant = true;
         }
      }

      if (is_redundant) {
         progress = true;

         /* Recurse if necessary. */
         ir_expression *op_expr = expr->operands[1 - i]->as_expression();
         if (op_expr && (op_expr->operation == ir_binop_min ||
                         op_expr->operation == ir_binop_max)) {
            return prune_expression(op_expr, baserange);
         }

         return expr->operands[1 - i];
      } else if (cr == MIXED) {
         /* If we have mixed vector operands, we can try to resolve the minmax
          * expression by doing a component-wise minmax:
          *
          *             min                          min
          *           /    \                       /    \
          *         min     a       ===>        [1,1]    a
          *       /    \
          *    [1,3]   [3,1]
          *
          */
         ir_constant *a = expr->operands[0]->as_constant();
         ir_constant *b = expr->operands[1]->as_constant();
         if (a && b)
            return combine_constant(ismin, a, b);
      }
   }

   /* Now recurse to operands giving them the proper baserange. The baserange
    * to pass is the intersection of our baserange and the other operand's
    * limit with one of the ranges unlimited. If we can't compute a valid
    * intersection, we use the current baserange.
    */
   for (unsigned i = 0; i < 2; ++i) {
      ir_expression *op_expr = expr->operands[i]->as_expression();
      if (op_expr && (op_expr->operation == ir_binop_min ||
                      op_expr->operation == ir_binop_max)) {
         /* We can only compute a new baserange for this operand if we managed
          * to compute a valid range for the other operand.
          */
         if (ismin)
            limits[1 - i].low = NULL;
         else
            limits[1 - i].high = NULL;
         minmax_range base = range_intersection(limits[1 - i], baserange);
         expr->operands[i] = prune_expression(op_expr, base);
      }
   }

   /* If we got here we could not discard any of the operands of the minmax
    * expression, but we can still try to resolve the expression if both
    * operands are constant. We do this after the loop above, to make sure
    * that if our operands are minmax expressions we have tried to prune them
    * first (hopefully reducing them to constants).
    */
   ir_constant *a = expr->operands[0]->as_constant();
   ir_constant *b = expr->operands[1]->as_constant();
   if (a && b)
      return combine_constant(ismin, a, b);

   return expr;
}

static ir_rvalue *
swizzle_if_required(ir_expression *expr, ir_rvalue *rval)
{
   if (expr->type->is_vector() && rval->type->is_scalar()) {
      return swizzle(rval, SWIZZLE_XXXX, expr->type->vector_elements);
   } else {
      return rval;
   }
}

void
ir_minmax_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_expression *expr = (*rvalue)->as_expression();
   if (!expr || (expr->operation != ir_binop_min &&
                 expr->operation != ir_binop_max))
      return;

   ir_rvalue *new_rvalue = prune_expression(expr, minmax_range());
   if (new_rvalue == *rvalue)
      return;

   /* If the expression type is a vector and the optimization leaves a scalar
    * as the result, we need to turn it into a vector.
    */
   *rvalue = swizzle_if_required(expr, new_rvalue);

   progress = true;
}

}

bool
do_minmax_prune(exec_list *instructions)
{
   ir_minmax_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}
