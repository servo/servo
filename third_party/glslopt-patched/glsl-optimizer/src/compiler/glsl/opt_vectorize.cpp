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
 * \file opt_vectorize.cpp
 *
 * Combines scalar assignments of the same expression (modulo swizzle) to
 * multiple channels of the same variable into a single vectorized expression
 * and assignment.
 *
 * Many generated shaders contain scalarized code. That is, they contain
 *
 * r1.x = log2(v0.x);
 * r1.y = log2(v0.y);
 * r1.z = log2(v0.z);
 *
 * rather than
 *
 * r1.xyz = log2(v0.xyz);
 *
 * We look for consecutive assignments of the same expression (modulo swizzle)
 * to each channel of the same variable.
 *
 * For instance, we want to convert these three scalar operations
 *
 * (assign (x) (var_ref r1) (expression float log2 (swiz x (var_ref v0))))
 * (assign (y) (var_ref r1) (expression float log2 (swiz y (var_ref v0))))
 * (assign (z) (var_ref r1) (expression float log2 (swiz z (var_ref v0))))
 *
 * into a single vector operation
 *
 * (assign (xyz) (var_ref r1) (expression vec3 log2 (swiz xyz (var_ref v0))))
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "program/prog_instruction.h"

namespace {

class ir_vectorize_visitor : public ir_hierarchical_visitor {
public:
   void clear()
   {
      assignment[0] = NULL;
      assignment[1] = NULL;
      assignment[2] = NULL;
      assignment[3] = NULL;
      current_assignment = NULL;
      last_assignment = NULL;
      channels = 0;
      has_swizzle = false;
   }

   ir_vectorize_visitor()
   {
      clear();
      progress = false;
   }

   virtual ir_visitor_status visit_enter(ir_assignment *);
   virtual ir_visitor_status visit_enter(ir_swizzle *);
   virtual ir_visitor_status visit_enter(ir_dereference_array *);
   virtual ir_visitor_status visit_enter(ir_expression *);
   virtual ir_visitor_status visit_enter(ir_if *);
   virtual ir_visitor_status visit_enter(ir_loop *);
   virtual ir_visitor_status visit_enter(ir_texture *);

   virtual ir_visitor_status visit_leave(ir_assignment *);

   void try_vectorize();

   ir_assignment *assignment[4];
   ir_assignment *current_assignment, *last_assignment;
   unsigned channels;
   bool has_swizzle;

   bool progress;
};

} /* unnamed namespace */

/**
 * Rewrites the swizzles and types of a right-hand side of an assignment.
 *
 * From the example above, this function would be called (by visit_tree()) on
 * the nodes of the tree (expression float log2 (swiz z   (var_ref v0))),
 * rewriting it into     (expression vec3  log2 (swiz xyz (var_ref v0))).
 *
 * The function operates on ir_expressions (and its operands) and ir_swizzles.
 * For expressions it sets a new type and swizzles any non-expression and non-
 * swizzle scalar operands into appropriately sized vector arguments. For
 * example, if combining
 *
 * (assign (x) (var_ref r1) (expression float + (swiz x (var_ref v0) (var_ref v1))))
 * (assign (y) (var_ref r1) (expression float + (swiz y (var_ref v0) (var_ref v1))))
 *
 * where v1 is a scalar, rewrite_swizzle() would insert a swizzle on
 * (var_ref v1) such that the final result was
 *
 * (assign (xy) (var_ref r1) (expression vec2 + (swiz xy (var_ref v0))
 *                                              (swiz xx (var_ref v1))))
 *
 * For swizzles, it sets a new type, and if the variable being swizzled is a
 * vector it overwrites the swizzle mask with the ir_swizzle_mask passed as the
 * data parameter. If the swizzled variable is scalar, then the swizzle was
 * added by an earlier call to rewrite_swizzle() on an expression, so the
 * mask should not be modified.
 */
static void
rewrite_swizzle(ir_instruction *ir, void *data)
{
   ir_swizzle_mask *mask = (ir_swizzle_mask *)data;

   switch (ir->ir_type) {
   case ir_type_swizzle: {
      ir_swizzle *swz = (ir_swizzle *)ir;
      if (swz->val->type->is_vector()) {
         swz->mask = *mask;
      }
      swz->type = glsl_type::get_instance(swz->type->base_type,
                                          mask->num_components, 1);
      break;
   }
   case ir_type_expression: {
      ir_expression *expr = (ir_expression *)ir;
      expr->type = glsl_type::get_instance(expr->type->base_type,
                                           mask->num_components, 1);
      for (unsigned i = 0; i < 4; i++) {
         if (expr->operands[i]) {
            ir_rvalue *rval = expr->operands[i]->as_rvalue();
            if (rval && rval->type->is_scalar() &&
                !rval->as_expression() && !rval->as_swizzle()) {
               expr->operands[i] = new(ir) ir_swizzle(rval, 0, 0, 0, 0,
                                                      mask->num_components);
            }
         }
      }
      break;
   }
   default:
      break;
   }
}

/**
 * Attempt to vectorize the previously saved assignments, and clear them from
 * consideration.
 *
 * If the assignments are able to be combined, it modifies in-place the last
 * assignment seen to be an equivalent vector form of the scalar assignments.
 * It then removes the other now obsolete scalar assignments.
 */
void
ir_vectorize_visitor::try_vectorize()
{
   if (this->last_assignment && this->channels > 1) {
      ir_swizzle_mask mask = {0, 0, 0, 0, channels, 0};

      this->last_assignment->write_mask = 0;

      for (unsigned i = 0, j = 0; i < 4; i++) {
         if (this->assignment[i]) {
            this->last_assignment->write_mask |= 1 << i;

            if (this->assignment[i] != this->last_assignment) {
               this->assignment[i]->remove();
            }

            switch (j) {
            case 0: mask.x = i; break;
            case 1: mask.y = i; break;
            case 2: mask.z = i; break;
            case 3: mask.w = i; break;
            }

            j++;
         }
      }

      visit_tree(this->last_assignment->rhs, rewrite_swizzle, &mask);

      this->progress = true;
   }
   clear();
}

/**
 * Returns whether the write mask is a single channel.
 */
static bool
single_channel_write_mask(unsigned write_mask)
{
   return write_mask != 0 && (write_mask & (write_mask - 1)) == 0;
}

/**
 * Translates single-channeled write mask to single-channeled swizzle.
 */
static unsigned
write_mask_to_swizzle(unsigned write_mask)
{
   switch (write_mask) {
   case WRITEMASK_X: return SWIZZLE_X;
   case WRITEMASK_Y: return SWIZZLE_Y;
   case WRITEMASK_Z: return SWIZZLE_Z;
   case WRITEMASK_W: return SWIZZLE_W;
   }
   unreachable("not reached");
}

/**
 * Returns whether a single-channeled write mask matches a swizzle.
 */
static bool
write_mask_matches_swizzle(unsigned write_mask,
                           const ir_swizzle *swz)
{
   return ((write_mask == WRITEMASK_X && swz->mask.x == SWIZZLE_X) ||
           (write_mask == WRITEMASK_Y && swz->mask.x == SWIZZLE_Y) ||
           (write_mask == WRITEMASK_Z && swz->mask.x == SWIZZLE_Z) ||
           (write_mask == WRITEMASK_W && swz->mask.x == SWIZZLE_W));
}

/**
 * Upon entering an ir_assignment, attempt to vectorize the currently tracked
 * assignments if the current assignment is not suitable. Keep a pointer to
 * the current assignment.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_assignment *ir)
{
   ir_dereference *lhs = this->last_assignment != NULL ?
                         this->last_assignment->lhs : NULL;
   ir_rvalue *rhs = this->last_assignment != NULL ?
                    this->last_assignment->rhs : NULL;

   if (ir->condition ||
       this->channels >= 4 ||
       !single_channel_write_mask(ir->write_mask) ||
       this->assignment[write_mask_to_swizzle(ir->write_mask)] != NULL ||
       (lhs && !ir->lhs->equals(lhs)) ||
       (rhs && !ir->rhs->equals(rhs, ir_type_swizzle))) {
      try_vectorize();
   }

   this->current_assignment = ir;

   return visit_continue;
}

/**
 * Upon entering an ir_swizzle, set ::has_swizzle if we're visiting from an
 * ir_assignment (i.e., that ::current_assignment is set) and the swizzle mask
 * matches the current assignment's write mask.
 *
 * If the write mask doesn't match the swizzle mask, remove the current
 * assignment from further consideration.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_swizzle *ir)
{
   if (this->current_assignment) {
      if (write_mask_matches_swizzle(this->current_assignment->write_mask, ir)) {
         this->has_swizzle = true;
      } else {
         this->current_assignment = NULL;
      }
   }
   return visit_continue;
}

/* Upon entering an ir_array_dereference, remove the current assignment from
 * further consideration. Since the index of an array dereference must scalar,
 * we are not able to vectorize it.
 *
 * FINISHME: If all of scalar indices are identical we could vectorize.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_dereference_array *)
{
   this->current_assignment = NULL;
   return visit_continue_with_parent;
}

/**
 * Upon entering an ir_expression, remove the current assignment from further
 * consideration if the expression operates horizontally on vectors.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_expression *ir)
{
   if (ir->is_horizontal()) {
      this->current_assignment = NULL;
      return visit_continue_with_parent;
   }
   return visit_continue;
}

/* Since there is no statement to visit between the "then" and "else"
 * instructions try to vectorize before, in between, and after them to avoid
 * combining statements from different basic blocks.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_if *ir)
{
   try_vectorize();

   visit_list_elements(this, &ir->then_instructions);
   try_vectorize();

   visit_list_elements(this, &ir->else_instructions);
   try_vectorize();

   return visit_continue_with_parent;
}

/* Since there is no statement to visit between the instructions in the body of
 * the loop and the instructions after it try to vectorize before and after the
 * body to avoid combining statements from different basic blocks.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_loop *ir)
{
   try_vectorize();

   visit_list_elements(this, &ir->body_instructions);
   try_vectorize();

   return visit_continue_with_parent;
}

/**
 * Upon entering an ir_texture, remove the current assignment from
 * further consideration. Vectorizing multiple texture lookups into one
 * is wrong.
 */
ir_visitor_status
ir_vectorize_visitor::visit_enter(ir_texture *)
{
   this->current_assignment = NULL;
   return visit_continue_with_parent;
}

/**
 * Upon leaving an ir_assignment, save a pointer to it in ::assignment[] if
 * the swizzle mask(s) found were appropriate. Also save a pointer in
 * ::last_assignment so that we can compare future assignments with it.
 *
 * Finally, clear ::current_assignment and ::has_swizzle.
 */
ir_visitor_status
ir_vectorize_visitor::visit_leave(ir_assignment *ir)
{
   if (this->has_swizzle && this->current_assignment) {
      assert(this->current_assignment == ir);

      unsigned channel = write_mask_to_swizzle(this->current_assignment->write_mask);
      this->assignment[channel] = ir;
      this->channels++;

      this->last_assignment = this->current_assignment;
   }
   this->current_assignment = NULL;
   this->has_swizzle = false;
   return visit_continue;
}

/**
 * Combines scalar assignments of the same expression (modulo swizzle) to
 * multiple channels of the same variable into a single vectorized expression
 * and assignment.
 */
bool
do_vectorize(exec_list *instructions)
{
   ir_vectorize_visitor v;

   v.run(instructions);

   /* Try to vectorize the last assignments seen. */
   v.try_vectorize();

   return v.progress;
}
