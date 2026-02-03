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
 * \file lower_if_to_cond_assign.cpp
 *
 * This flattens if-statements to conditional assignments if:
 *
 * - the GPU has limited or no flow control support
 *   (controlled by max_depth)
 *
 * - small conditional branches are more expensive than conditional assignments
 *   (controlled by min_branch_cost, that's the cost for a branch to be
 *    preserved)
 *
 * It can't handle other control flow being inside of its block, such
 * as calls or loops.  Hopefully loop unrolling and inlining will take
 * care of those.
 *
 * Drivers for GPUs with no control flow support should simply call
 *
 *    lower_if_to_cond_assign(instructions)
 *
 * to attempt to flatten all if-statements.
 *
 * Some GPUs (such as i965 prior to gen6) do support control flow, but have a
 * maximum nesting depth N.  Drivers for such hardware can call
 *
 *    lower_if_to_cond_assign(instructions, N)
 *
 * to attempt to flatten any if-statements appearing at depth > N.
 */

#include "compiler/glsl_types.h"
#include "ir.h"
#include "util/set.h"
#include "util/hash_table.h" /* Needed for the hashing functions */
#include "main/macros.h" /* for MAX2 */

namespace {

class ir_if_to_cond_assign_visitor : public ir_hierarchical_visitor {
public:
   ir_if_to_cond_assign_visitor(gl_shader_stage stage,
                                unsigned max_depth,
                                unsigned min_branch_cost)
   {
      this->progress = false;
      this->stage = stage;
      this->max_depth = max_depth;
      this->min_branch_cost = min_branch_cost;
      this->depth = 0;

      this->condition_variables = _mesa_pointer_set_create(NULL);
   }

   ~ir_if_to_cond_assign_visitor()
   {
      _mesa_set_destroy(this->condition_variables, NULL);
   }

   ir_visitor_status visit_enter(ir_if *);
   ir_visitor_status visit_leave(ir_if *);

   bool found_unsupported_op;
   bool found_expensive_op;
   bool found_dynamic_arrayref;
   bool is_then;
   bool progress;
   gl_shader_stage stage;
   unsigned then_cost;
   unsigned else_cost;
   unsigned min_branch_cost;
   unsigned max_depth;
   unsigned depth;

   struct set *condition_variables;
};

} /* anonymous namespace */

bool
lower_if_to_cond_assign(gl_shader_stage stage, exec_list *instructions,
                        unsigned max_depth, unsigned min_branch_cost)
{
   if (max_depth == UINT_MAX)
      return false;

   ir_if_to_cond_assign_visitor v(stage, max_depth, min_branch_cost);

   visit_list_elements(&v, instructions);

   return v.progress;
}

static void
check_ir_node(ir_instruction *ir, void *data)
{
   ir_if_to_cond_assign_visitor *v = (ir_if_to_cond_assign_visitor *)data;

   switch (ir->ir_type) {
   case ir_type_call:
   case ir_type_discard:
   case ir_type_loop:
   case ir_type_loop_jump:
   case ir_type_return:
   case ir_type_emit_vertex:
   case ir_type_end_primitive:
   case ir_type_barrier:
      v->found_unsupported_op = true;
      break;

   case ir_type_dereference_variable: {
      ir_variable *var = ir->as_dereference_variable()->variable_referenced();

      /* Lowering branches with TCS output accesses breaks many piglit tests,
       * so don't touch them for now.
       */
      if (v->stage == MESA_SHADER_TESS_CTRL &&
          var->data.mode == ir_var_shader_out)
         v->found_unsupported_op = true;
      break;
   }

   /* SSBO, images, atomic counters are handled by ir_type_call */
   case ir_type_texture:
      v->found_expensive_op = true;
      break;

   case ir_type_dereference_array: {
      ir_dereference_array *deref = ir->as_dereference_array();

      if (deref->array_index->ir_type != ir_type_constant)
         v->found_dynamic_arrayref = true;
   } /* fall-through */
   case ir_type_expression:
   case ir_type_dereference_record:
      if (v->is_then)
         v->then_cost++;
      else
         v->else_cost++;
      break;

   default:
      break;
   }
}

static void
move_block_to_cond_assign(void *mem_ctx,
                          ir_if *if_ir, ir_rvalue *cond_expr,
                          exec_list *instructions,
                          struct set *set)
{
   foreach_in_list_safe(ir_instruction, ir, instructions) {
      if (ir->ir_type == ir_type_assignment) {
         ir_assignment *assign = (ir_assignment *)ir;

         if (_mesa_set_search(set, assign) == NULL) {
            _mesa_set_add(set, assign);

            /* If the LHS of the assignment is a condition variable that was
             * previously added, insert an additional assignment of false to
             * the variable.
             */
            const bool assign_to_cv =
               _mesa_set_search(
                  set, assign->lhs->variable_referenced()) != NULL;

            if (!assign->condition) {
               if (assign_to_cv) {
                  assign->rhs =
                     new(mem_ctx) ir_expression(ir_binop_logic_and,
                                                glsl_type::bool_type,
                                                cond_expr->clone(mem_ctx, NULL),
                                                assign->rhs);
               } else {
                  assign->condition = cond_expr->clone(mem_ctx, NULL);
               }
            } else {
               assign->condition =
                  new(mem_ctx) ir_expression(ir_binop_logic_and,
                                             glsl_type::bool_type,
                                             cond_expr->clone(mem_ctx, NULL),
                                             assign->condition);
            }
         }
      }

      /* Now, move from the if block to the block surrounding it. */
      ir->remove();
      if_ir->insert_before(ir);
   }
}

ir_visitor_status
ir_if_to_cond_assign_visitor::visit_enter(ir_if *)
{
   this->depth++;

   return visit_continue;
}

ir_visitor_status
ir_if_to_cond_assign_visitor::visit_leave(ir_if *ir)
{
   bool must_lower = this->depth-- > this->max_depth;

   /* Only flatten when beyond the GPU's maximum supported nesting depth. */
   if (!must_lower && this->min_branch_cost == 0)
      return visit_continue;

   this->found_unsupported_op = false;
   this->found_expensive_op = false;
   this->found_dynamic_arrayref = false;
   this->then_cost = 0;
   this->else_cost = 0;

   ir_assignment *assign;

   /* Check that both blocks don't contain anything we can't support. */
   this->is_then = true;
   foreach_in_list(ir_instruction, then_ir, &ir->then_instructions) {
      visit_tree(then_ir, check_ir_node, this);
   }

   this->is_then = false;
   foreach_in_list(ir_instruction, else_ir, &ir->else_instructions) {
      visit_tree(else_ir, check_ir_node, this);
   }

   if (this->found_unsupported_op)
      return visit_continue; /* can't handle inner unsupported opcodes */

   /* Skip if the branch cost is high enough or if there's an expensive op.
    *
    * Also skip if non-constant array indices were encountered, since those
    * can be out-of-bounds for a not-taken branch, and so generating an
    * assignment would be incorrect. In the case of must_lower, it's up to the
    * backend to deal with any potential fall-out (perhaps by translating the
    * assignments to hardware-predicated moves).
    */
   if (!must_lower &&
       (this->found_expensive_op ||
        this->found_dynamic_arrayref ||
        MAX2(this->then_cost, this->else_cost) >= this->min_branch_cost))
      return visit_continue;

   void *mem_ctx = ralloc_parent(ir);

   /* Store the condition to a variable.  Move all of the instructions from
    * the then-clause of the if-statement.  Use the condition variable as a
    * condition for all assignments.
    */
   ir_variable *const then_var =
      new(mem_ctx) ir_variable(glsl_type::bool_type,
                               "if_to_cond_assign_then",
                               ir_var_temporary);
   ir->insert_before(then_var);

   ir_dereference_variable *then_cond =
      new(mem_ctx) ir_dereference_variable(then_var);

   assign = new(mem_ctx) ir_assignment(then_cond, ir->condition);
   ir->insert_before(assign);

   move_block_to_cond_assign(mem_ctx, ir, then_cond,
                             &ir->then_instructions,
                             this->condition_variables);

   /* Add the new condition variable to the hash table.  This allows us to
    * find this variable when lowering other (enclosing) if-statements.
    */
   _mesa_set_add(this->condition_variables, then_var);

   /* If there are instructions in the else-clause, store the inverse of the
    * condition to a variable.  Move all of the instructions from the
    * else-clause if the if-statement.  Use the (inverse) condition variable
    * as a condition for all assignments.
    */
   if (!ir->else_instructions.is_empty()) {
      ir_variable *const else_var =
         new(mem_ctx) ir_variable(glsl_type::bool_type,
                                  "if_to_cond_assign_else",
                                  ir_var_temporary);
      ir->insert_before(else_var);

      ir_dereference_variable *else_cond =
         new(mem_ctx) ir_dereference_variable(else_var);

      ir_rvalue *inverse =
         new(mem_ctx) ir_expression(ir_unop_logic_not,
                                    then_cond->clone(mem_ctx, NULL));

      assign = new(mem_ctx) ir_assignment(else_cond, inverse);
      ir->insert_before(assign);

      move_block_to_cond_assign(mem_ctx, ir, else_cond,
                                &ir->else_instructions,
                                this->condition_variables);

      /* Add the new condition variable to the hash table.  This allows us to
       * find this variable when lowering other (enclosing) if-statements.
       */
      _mesa_set_add(this->condition_variables, else_var);
   }

   ir->remove();

   this->progress = true;

   return visit_continue;
}
