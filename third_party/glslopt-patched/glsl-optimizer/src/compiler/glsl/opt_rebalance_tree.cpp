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
 * \file opt_rebalance_tree.cpp
 *
 * Rebalances a reduction expression tree.
 *
 * For reduction operations (e.g., x + y + z + w) we generate an expression
 * tree like
 *
 *        +
 *       / \
 *      +   w
 *     / \
 *    +   z
 *   / \
 *  x   y
 *
 * which we can rebalance into
 *
 *       +
 *      / \
 *     /   \
 *    +     +
 *   / \   / \
 *  x   y z   w
 *
 * to get a better instruction scheduling.
 *
 * See "Tree Rebalancing in Optimal Editor Time and Space" by Quentin F. Stout
 * and Bette L. Warren.
 *
 * Also see http://penguin.ewu.edu/~trolfe/DSWpaper/ for a very readable
 * explanation of the of the tree_to_vine() (rightward rotation) and
 * vine_to_tree() (leftward rotation) algorithms.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "main/macros.h" /* for MAX2 */

/* The DSW algorithm generates a degenerate tree (really, a linked list) in
 * tree_to_vine(). We'd rather not leave a binary expression with only one
 * operand, so trivial modifications (the ternary operators below) are needed
 * to ensure that we only rotate around the ir_expression nodes of the tree.
 */
static unsigned
tree_to_vine(ir_expression *root)
{
   unsigned size = 0;
   ir_rvalue *vine_tail = root;
   ir_rvalue *remainder = root->operands[1];

   while (remainder != NULL) {
      ir_expression *remainder_temp = remainder->as_expression();
      ir_expression *remainder_left = remainder_temp ?
         remainder_temp->operands[0]->as_expression() : NULL;

      if (remainder_left == NULL) {
         /* move vine_tail down one */
         vine_tail = remainder;
         remainder = remainder->as_expression() ?
            ((ir_expression *)remainder)->operands[1] : NULL;
         size++;
      } else {
         /* rotate */
         ir_expression *tempptr = remainder_left;
         ((ir_expression *)remainder)->operands[0] = tempptr->operands[1];
         tempptr->operands[1] = remainder;
         remainder = tempptr;
         ((ir_expression *)vine_tail)->operands[1] = tempptr;
      }
   }

   return size;
}

static void
compression(ir_expression *root, unsigned count)
{
   ir_expression *scanner = root;

   for (unsigned i = 0; i < count; i++) {
      ir_expression *child = (ir_expression *)scanner->operands[1];
      scanner->operands[1] = child->operands[1];
      scanner = (ir_expression *)scanner->operands[1];
      child->operands[1] = scanner->operands[0];
      scanner->operands[0] = child;
   }
}

static void
vine_to_tree(ir_expression *root, unsigned size)
{
   int n = size - 1;
   for (int m = n / 2; m > 0; m = n / 2) {
      compression(root, m);
      n -= m + 1;
   }
}

namespace {

class ir_rebalance_visitor : public ir_rvalue_enter_visitor {
public:
   ir_rebalance_visitor()
   {
      progress = false;
   }

   virtual ir_visitor_status visit_enter(ir_assignment *ir);

   void handle_rvalue(ir_rvalue **rvalue);

   bool progress;
};

struct is_reduction_data {
   ir_expression_operation operation;
   const glsl_type *type;
   unsigned num_expr;
   bool is_reduction;
   bool contains_constant;
};

} /* anonymous namespace */

ir_visitor_status
ir_rebalance_visitor::visit_enter(ir_assignment *ir)
{
   ir_variable *var = ir->lhs->variable_referenced();
   if (var->data.invariant || var->data.precise) {
      /* If we're assigning to an invariant variable, just bail.  Tree
       * rebalancing (reassociation) isn't precision-safe.
       */
      return visit_continue_with_parent;
   } else {
      return visit_continue;
   }
}

static bool
is_reduction_operation(ir_expression_operation operation)
{
   switch (operation) {
   case ir_binop_add:
   case ir_binop_mul:
   case ir_binop_bit_and:
   case ir_binop_bit_xor:
   case ir_binop_bit_or:
   case ir_binop_logic_and:
   case ir_binop_logic_xor:
   case ir_binop_logic_or:
   case ir_binop_min:
   case ir_binop_max:
      return true;
   default:
      return false;
   }
}

/* Note that this function does not attempt to recognize that reduction trees
 * are already balanced.
 *
 * We return false from this function for a number of reasons other than an
 * expression tree not being a mathematical reduction. Namely,
 *
 *    - if the tree contains multiple constants that we may be able to combine.
 *    - if the tree contains matrices:
 *       - they might contain vec4's with many constant components that we can
 *         simplify after splitting.
 *       - applying the matrix chain ordering optimization is more than just
 *         balancing an expression tree.
 *    - if the tree contains operations on multiple types.
 *    - if the tree contains ir_dereference_{array,record}, since foo[a+b] + c
 *      would trick the visiting pass.
 */
static void
is_reduction(ir_instruction *ir, void *data)
{
   struct is_reduction_data *ird = (struct is_reduction_data *)data;
   if (!ird->is_reduction)
      return;

   /* We don't want to balance a tree that contains multiple constants, since
    * we'll be able to constant fold them if they're not in separate subtrees.
    */
   if (ir->as_constant()) {
      if (ird->contains_constant) {
         ird->is_reduction = false;
      }
      ird->contains_constant = true;
      return;
   }

   /* Array/record dereferences have subtrees that are not part of the expr
    * tree we're balancing. Skip trees containing them.
    */
   if (ir->ir_type == ir_type_dereference_array ||
       ir->ir_type == ir_type_dereference_record) {
      ird->is_reduction = false;
      return;
   }

   ir_expression *expr = ir->as_expression();
   if (!expr)
      return;

   /* Non-constant matrices might still contain constant vec4 that we can
    * constant fold once split up. Handling matrices will need some more
    * work.
    */
   if (expr->type->is_matrix() ||
       expr->operands[0]->type->is_matrix() ||
       (expr->operands[1] && expr->operands[1]->type->is_matrix())) {
      ird->is_reduction = false;
      return;
   }

   if (ird->type != NULL && ird->type != expr->type) {
      ird->is_reduction = false;
      return;
   }
   ird->type = expr->type;

   ird->num_expr++;
   if (is_reduction_operation(expr->operation)) {
      if (ird->operation != 0 && ird->operation != expr->operation)
         ird->is_reduction = false;
      ird->operation = expr->operation;
   } else {
      ird->is_reduction = false;
   }
}

static ir_rvalue *
handle_expression(ir_expression *expr)
{
   struct is_reduction_data ird;
   ird.operation = (ir_expression_operation)0;
   ird.type = NULL;
   ird.num_expr = 0;
   ird.is_reduction = true;
   ird.contains_constant = false;

   visit_tree(expr, is_reduction, (void *)&ird);

   if (ird.is_reduction && ird.num_expr > 2) {
      ir_constant z = ir_constant(0.0f);
      ir_expression pseudo_root = ir_expression(ir_binop_add, &z, expr);

      unsigned size = tree_to_vine(&pseudo_root);
      vine_to_tree(&pseudo_root, size);

      expr = (ir_expression *)pseudo_root.operands[1];
   }
   return expr;
}

static void
update_types(ir_instruction *ir, void *)
{
   ir_expression *expr = ir->as_expression();
   if (!expr)
      return;

   const glsl_type *const new_type =
      glsl_type::get_instance(expr->type->base_type,
                              MAX2(expr->operands[0]->type->vector_elements,
                                   expr->operands[1]->type->vector_elements),
                              1);
   assert(new_type != glsl_type::error_type);
   expr->type = new_type;
}

void
ir_rebalance_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_expression *expr = (*rvalue)->as_expression();
   if (!expr || !is_reduction_operation(expr->operation))
      return;

   ir_rvalue *new_rvalue = handle_expression(expr);

   /* If we failed to rebalance the tree (e.g., because it wasn't a reduction,
    * or some other set of cases) new_rvalue will point to the same root as
    * before.
    *
    * Similarly, if the tree rooted at *rvalue was a reduction and was already
    * balanced, the algorithm will rearrange the tree but will ultimately
    * return an identical tree, so this check will handle that as well and
    * will not set progress = true.
    */
   if (new_rvalue == *rvalue)
      return;

   visit_tree(new_rvalue, NULL, NULL, update_types);

   *rvalue = new_rvalue;
   this->progress = true;
}

bool
do_rebalance_tree(exec_list *instructions)
{
   ir_rebalance_visitor v;

   v.run(instructions);

   return v.progress;
}
