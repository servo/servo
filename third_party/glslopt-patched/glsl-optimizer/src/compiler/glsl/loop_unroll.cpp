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

#include "compiler/glsl_types.h"
#include "loop_analysis.h"
#include "ir_hierarchical_visitor.h"

#include "main/mtypes.h"

namespace {

class loop_unroll_visitor : public ir_hierarchical_visitor {
public:
   loop_unroll_visitor(loop_state *state,
                       const struct gl_shader_compiler_options *options)
   {
      this->state = state;
      this->progress = false;
      this->options = options;
   }

   virtual ir_visitor_status visit_leave(ir_loop *ir);
   void simple_unroll(ir_loop *ir, int iterations);
   void complex_unroll(ir_loop *ir, int iterations,
                       bool continue_from_then_branch,
                       bool limiting_term_first,
                       bool lt_continue_from_then_branch);
   void splice_post_if_instructions(ir_if *ir_if, exec_list *splice_dest);

   loop_state *state;

   bool progress;
   const struct gl_shader_compiler_options *options;
};

} /* anonymous namespace */

class loop_unroll_count : public ir_hierarchical_visitor {
public:
   int nodes;
   bool unsupported_variable_indexing;
   bool array_indexed_by_induction_var_with_exact_iterations;
   /* If there are nested loops, the node count will be inaccurate. */
   bool nested_loop;

   loop_unroll_count(exec_list *list, loop_variable_state *ls,
                     const struct gl_shader_compiler_options *options)
      : ls(ls), options(options)
   {
      nodes = 0;
      nested_loop = false;
      unsupported_variable_indexing = false;
      array_indexed_by_induction_var_with_exact_iterations = false;

      run(list);
   }

   virtual ir_visitor_status visit_enter(ir_assignment *)
   {
      nodes++;
      return visit_continue;
   }

   virtual ir_visitor_status visit_enter(ir_expression *)
   {
      nodes++;
      return visit_continue;
   }

   virtual ir_visitor_status visit_enter(ir_loop *)
   {
      nested_loop = true;
      return visit_continue;
   }

   virtual ir_visitor_status visit_enter(ir_dereference_array *ir)
   {
      /* Force unroll in case of dynamic indexing with sampler arrays
       * when EmitNoIndirectSampler is set.
       */
      if (options->EmitNoIndirectSampler) {
         if ((ir->array->type->is_array() &&
              ir->array->type->contains_sampler()) &&
             !ir->array_index->constant_expression_value(ralloc_parent(ir))) {
            unsupported_variable_indexing = true;
            return visit_continue;
         }
      }

      /* Check for arrays variably-indexed by a loop induction variable.
       * Unrolling the loop may convert that access into constant-indexing.
       *
       * Many drivers don't support particular kinds of variable indexing,
       * and have to resort to using lower_variable_index_to_cond_assign to
       * handle it.  This results in huge amounts of horrible code, so we'd
       * like to avoid that if possible.  Here, we just note that it will
       * happen.
       */
      if ((ir->array->type->is_array() || ir->array->type->is_matrix()) &&
          !ir->array_index->as_constant()) {
         ir_variable *array = ir->array->variable_referenced();
         loop_variable *lv = ls->get(ir->array_index->variable_referenced());
         if (array && lv && lv->is_induction_var()) {
            /* If an array is indexed by a loop induction variable, and the
             * array size is exactly the number of loop iterations, this is
             * probably a simple for-loop trying to access each element in
             * turn; the application may expect it to be unrolled.
             */
            if (int(array->type->length) == ls->limiting_terminator->iterations)
               array_indexed_by_induction_var_with_exact_iterations = true;

            switch (array->data.mode) {
            case ir_var_auto:
            case ir_var_temporary:
            case ir_var_const_in:
            case ir_var_function_in:
            case ir_var_function_out:
            case ir_var_function_inout:
               if (options->EmitNoIndirectTemp)
                  unsupported_variable_indexing = true;
               break;
            case ir_var_uniform:
            case ir_var_shader_storage:
               if (options->EmitNoIndirectUniform)
                  unsupported_variable_indexing = true;
               break;
            case ir_var_shader_in:
               if (options->EmitNoIndirectInput)
                  unsupported_variable_indexing = true;
               break;
            case ir_var_shader_out:
               if (options->EmitNoIndirectOutput)
                  unsupported_variable_indexing = true;
               break;
            }
         }
      }
      return visit_continue;
   }

private:
   loop_variable_state *ls;
   const struct gl_shader_compiler_options *options;
};


/**
 * Unroll a loop which does not contain any jumps.  For example, if the input
 * is:
 *
 *     (loop (...) ...instrs...)
 *
 * And the iteration count is 3, the output will be:
 *
 *     ...instrs... ...instrs... ...instrs...
 */
void
loop_unroll_visitor::simple_unroll(ir_loop *ir, int iterations)
{
   void *const mem_ctx = ralloc_parent(ir);
   loop_variable_state *const ls = this->state->get(ir);

   /* If there are no terminators, then the loop iteration count must be 1.
    * This is the 'do { } while (false);' case.
    */
   assert(!ls->terminators.is_empty() || iterations == 1);

   ir_instruction *first_ir =
      (ir_instruction *) ir->body_instructions.get_head();

   if (!first_ir) {
      /* The loop is empty remove it and return */
      ir->remove();
      return;
   }

   ir_if *limit_if = NULL;
   bool exit_branch_has_instructions = false;
   if (ls->limiting_terminator) {
      limit_if = ls->limiting_terminator->ir;
      ir_instruction *ir_if_last = (ir_instruction *)
         limit_if->then_instructions.get_tail();

      if (is_break(ir_if_last)) {
         if (ir_if_last != limit_if->then_instructions.get_head())
            exit_branch_has_instructions = true;

         splice_post_if_instructions(limit_if, &limit_if->else_instructions);
         ir_if_last->remove();
      } else {
         ir_if_last = (ir_instruction *)
            limit_if->else_instructions.get_tail();
         assert(is_break(ir_if_last));

         if (ir_if_last != limit_if->else_instructions.get_head())
            exit_branch_has_instructions = true;

         splice_post_if_instructions(limit_if, &limit_if->then_instructions);
         ir_if_last->remove();
      }
   }

   /* Because 'iterations' is the number of times we pass over the *entire*
    * loop body before hitting the first break, we need to bump the number of
    * iterations if the limiting terminator is not the first instruction in
    * the loop, or it the exit branch contains instructions. This ensures we
    * execute any instructions before the terminator or in its exit branch.
    */
   if (!ls->terminators.is_empty() &&
       (limit_if != first_ir->as_if() || exit_branch_has_instructions))
      iterations++;

   for (int i = 0; i < iterations; i++) {
      exec_list copy_list;

      copy_list.make_empty();
      clone_ir_list(mem_ctx, &copy_list, &ir->body_instructions);

      ir->insert_before(&copy_list);
   }

   /* The loop has been replaced by the unrolled copies.  Remove the original
    * loop from the IR sequence.
    */
   ir->remove();

   this->progress = true;
}


/**
 * Unroll a loop whose last statement is an ir_if.  If \c
 * continue_from_then_branch is true, the loop is repeated only when the
 * "then" branch of the if is taken; otherwise it is repeated only when the
 * "else" branch of the if is taken.
 *
 * For example, if the input is:
 *
 *     (loop (...)
 *      ...body...
 *      (if (cond)
 *          (...then_instrs...)
 *        (...else_instrs...)))
 *
 * And the iteration count is 3, and \c continue_from_then_branch is true,
 * then the output will be:
 *
 *     ...body...
 *     (if (cond)
 *         (...then_instrs...
 *          ...body...
 *          (if (cond)
 *              (...then_instrs...
 *               ...body...
 *               (if (cond)
 *                   (...then_instrs...)
 *                 (...else_instrs...)))
 *            (...else_instrs...)))
 *       (...else_instrs))
 */
void
loop_unroll_visitor::complex_unroll(ir_loop *ir, int iterations,
                                    bool second_term_then_continue,
                                    bool extra_iteration_required,
                                    bool first_term_then_continue)
{
   void *const mem_ctx = ralloc_parent(ir);
   ir_instruction *ir_to_replace = ir;

   /* Because 'iterations' is the number of times we pass over the *entire*
    * loop body before hitting the first break, we need to bump the number of
    * iterations if the limiting terminator is not the first instruction in
    * the loop, or it the exit branch contains instructions. This ensures we
    * execute any instructions before the terminator or in its exit branch.
    */
   if (extra_iteration_required)
      iterations++;

   for (int i = 0; i < iterations; i++) {
      exec_list copy_list;

      copy_list.make_empty();
      clone_ir_list(mem_ctx, &copy_list, &ir->body_instructions);

      ir_if *ir_if = ((ir_instruction *) copy_list.get_tail())->as_if();
      assert(ir_if != NULL);

      exec_list *const first_list = first_term_then_continue
         ? &ir_if->then_instructions : &ir_if->else_instructions;
      ir_if = ((ir_instruction *) first_list->get_tail())->as_if();

      ir_to_replace->insert_before(&copy_list);
      ir_to_replace->remove();

      /* placeholder that will be removed in the next iteration */
      ir_to_replace =
         new(mem_ctx) ir_loop_jump(ir_loop_jump::jump_continue);

      exec_list *const second_term_continue_list = second_term_then_continue
         ? &ir_if->then_instructions : &ir_if->else_instructions;

      second_term_continue_list->push_tail(ir_to_replace);
   }

   ir_to_replace->remove();

   this->progress = true;
}


/**
 * Move all of the instructions which follow \c ir_if to the end of
 * \c splice_dest.
 *
 * For example, in the code snippet:
 *
 *     (if (cond)
 *         (...then_instructions...
 *          break)
 *       (...else_instructions...))
 *     ...post_if_instructions...
 *
 * If \c ir_if points to the "if" instruction, and \c splice_dest points to
 * (...else_instructions...), the code snippet is transformed into:
 *
 *     (if (cond)
 *         (...then_instructions...
 *          break)
 *       (...else_instructions...
 *        ...post_if_instructions...))
 */
void
loop_unroll_visitor::splice_post_if_instructions(ir_if *ir_if,
                                                 exec_list *splice_dest)
{
   while (!ir_if->get_next()->is_tail_sentinel()) {
      ir_instruction *move_ir = (ir_instruction *) ir_if->get_next();

      move_ir->remove();
      splice_dest->push_tail(move_ir);
   }
}

static bool
exit_branch_has_instructions(ir_if *term_if, bool lt_then_continue)
{
   if (lt_then_continue) {
      if (term_if->else_instructions.get_head() ==
          term_if->else_instructions.get_tail())
         return false;
   } else {
      if (term_if->then_instructions.get_head() ==
          term_if->then_instructions.get_tail())
         return false;
   }

   return true;
}

ir_visitor_status
loop_unroll_visitor::visit_leave(ir_loop *ir)
{
   loop_variable_state *const ls = this->state->get(ir);

   /* If we've entered a loop that hasn't been analyzed, something really,
    * really bad has happened.
    */
   if (ls == NULL) {
      assert(ls != NULL);
      return visit_continue;
   }

   /* Limiting terminator may have iteration count of zero,
    * this is a valid case because the loop may break during
    * the first iteration.
    */

   /* Remove the conditional break statements associated with all terminators
    * that are associated with a fixed iteration count, except for the one
    * associated with the limiting terminator--that one needs to stay, since
    * it terminates the loop.  Exception: if the loop still has a normative
    * bound, then that terminates the loop, so we don't even need the limiting
    * terminator.
    */
   foreach_in_list_safe(loop_terminator, t, &ls->terminators) {
      if (t->iterations < 0)
         continue;

      exec_list *branch_instructions;
      if (t != ls->limiting_terminator) {
         ir_instruction *ir_if_last = (ir_instruction *)
            t->ir->then_instructions.get_tail();
         if (is_break(ir_if_last)) {
            branch_instructions = &t->ir->else_instructions;
         } else {
            branch_instructions = &t->ir->then_instructions;
            assert(is_break((ir_instruction *)
                            t->ir->else_instructions.get_tail()));
         }

         exec_list copy_list;
         copy_list.make_empty();
         clone_ir_list(ir, &copy_list, branch_instructions);

         t->ir->insert_before(&copy_list);
         t->ir->remove();

         assert(ls->num_loop_jumps > 0);
         ls->num_loop_jumps--;

         /* Also remove it from the terminator list */
         t->remove();

         this->progress = true;
      }
   }

   if (ls->limiting_terminator == NULL) {
      ir_instruction *last_ir =
         (ir_instruction *) ir->body_instructions.get_tail();

      /* If a loop has no induction variable and the last instruction is
       * a break, unroll the loop with a count of 1.  This is the classic
       *
       *    do {
       *        // ...
       *    } while (false)
       *
       * that is used to wrap multi-line macros.
       *
       * If num_loop_jumps is not zero, last_ir cannot be NULL... there has to
       * be at least num_loop_jumps instructions in the loop.
       */
      if (ls->num_loop_jumps == 1 && is_break(last_ir)) {
         last_ir->remove();

         simple_unroll(ir, 1);
      }

      /* Don't try to unroll loops where the number of iterations is not known
       * at compile-time.
       */
      return visit_continue;
   }

   int iterations = ls->limiting_terminator->iterations;

   const int max_iterations = options->MaxUnrollIterations;

   /* Don't try to unroll loops that have zillions of iterations either.
    */
   if (iterations > max_iterations)
      return visit_continue;

   /* Don't try to unroll nested loops and loops with a huge body.
    */
   loop_unroll_count count(&ir->body_instructions, ls, options);

   bool loop_too_large =
      count.nested_loop || count.nodes * iterations > max_iterations * 5;

   if (loop_too_large && !count.unsupported_variable_indexing &&
       !count.array_indexed_by_induction_var_with_exact_iterations)
      return visit_continue;

   /* Note: the limiting terminator contributes 1 to ls->num_loop_jumps.
    * We'll be removing the limiting terminator before we unroll.
    */
   assert(ls->num_loop_jumps > 0);
   unsigned predicted_num_loop_jumps = ls->num_loop_jumps - 1;

   if (predicted_num_loop_jumps > 1)
      return visit_continue;

   if (predicted_num_loop_jumps == 0) {
      simple_unroll(ir, iterations);
      return visit_continue;
   }

   ir_instruction *last_ir = (ir_instruction *) ir->body_instructions.get_tail();
   assert(last_ir != NULL);

   if (is_break(last_ir)) {
      /* If the only loop-jump is a break at the end of the loop, the loop
       * will execute exactly once.  Remove the break and use the simple
       * unroller with an iteration count of 1.
       */
      last_ir->remove();

      simple_unroll(ir, 1);
      return visit_continue;
   }

   /* Complex unrolling can only handle two terminators. One with an unknown
    * iteration count and one with a known iteration count. We have already
    * made sure we have a known iteration count above and removed any
    * unreachable terminators with a known count. Here we make sure there
    * isn't any additional unknown terminators, or any other jumps nested
    * inside futher ifs.
    */
   if (ls->num_loop_jumps != 2 || ls->terminators.length() != 2)
      return visit_continue;

   ir_instruction *first_ir =
      (ir_instruction *) ir->body_instructions.get_head();

   unsigned term_count = 0;
   bool first_term_then_continue = false;
   foreach_in_list(loop_terminator, t, &ls->terminators) {
      ir_if *ir_if = t->ir->as_if();
      assert(ir_if != NULL);

      ir_instruction *ir_if_last =
         (ir_instruction *) ir_if->then_instructions.get_tail();

      if (is_break(ir_if_last)) {
         splice_post_if_instructions(ir_if, &ir_if->else_instructions);
         ir_if_last->remove();
         if (term_count == 1) {
            bool ebi =
               exit_branch_has_instructions(ls->limiting_terminator->ir,
                                            first_term_then_continue);
            complex_unroll(ir, iterations, false,
                           first_ir->as_if() != ls->limiting_terminator->ir ||
                           ebi,
                           first_term_then_continue);
            return visit_continue;
         }
      } else {
         ir_if_last =
            (ir_instruction *) ir_if->else_instructions.get_tail();

         assert(is_break(ir_if_last));
         if (is_break(ir_if_last)) {
            splice_post_if_instructions(ir_if, &ir_if->then_instructions);
            ir_if_last->remove();
            if (term_count == 1) {
               bool ebi =
                  exit_branch_has_instructions(ls->limiting_terminator->ir,
                                               first_term_then_continue);
               complex_unroll(ir, iterations, true,
                              first_ir->as_if() != ls->limiting_terminator->ir ||
                              ebi,
                              first_term_then_continue);
               return visit_continue;
            } else {
               first_term_then_continue = true;
            }
         }
      }

      term_count++;
   }

   /* Did not find the break statement.  It must be in a complex if-nesting,
    * so don't try to unroll.
    */
   return visit_continue;
}


bool
unroll_loops(exec_list *instructions, loop_state *ls,
             const struct gl_shader_compiler_options *options)
{
   loop_unroll_visitor v(ls, options);

   v.run(instructions);

   return v.progress;
}
