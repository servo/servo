/*
 * Copyright © 2010 Luca Barbieri
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
 * \file lower_jumps.cpp
 *
 * This pass lowers jumps (break, continue, and return) to if/else structures.
 *
 * It can be asked to:
 * 1. Pull jumps out of ifs where possible
 * 2. Remove all "continue"s, replacing them with an "execute flag"
 * 3. Replace all "break" with a single conditional one at the end of the loop
 * 4. Replace all "return"s with a single return at the end of the function,
 *    for the main function and/or other functions
 *
 * Applying this pass gives several benefits:
 * 1. All functions can be inlined.
 * 2. nv40 and other pre-DX10 chips without "continue" can be supported
 * 3. nv30 and other pre-DX10 chips with no control flow at all are better
 *    supported
 *
 * Continues are lowered by adding a per-loop "execute flag", initialized to
 * true, that when cleared inhibits all execution until the end of the loop.
 *
 * Breaks are lowered to continues, plus setting a "break flag" that is checked
 * at the end of the loop, and trigger the unique "break".
 *
 * Returns are lowered to breaks/continues, plus adding a "return flag" that
 * causes loops to break again out of their enclosing loops until all the
 * loops are exited: then the "execute flag" logic will ignore everything
 * until the end of the function.
 *
 * Note that "continue" and "return" can also be implemented by adding
 * a dummy loop and using break.
 * However, this is bad for hardware with limited nesting depth, and
 * prevents further optimization, and thus is not currently performed.
 */

#include "compiler/glsl_types.h"
#include <string.h>
#include "ir.h"

/**
 * Enum recording the result of analyzing how control flow might exit
 * an IR node.
 *
 * Each possible value of jump_strength indicates a strictly stronger
 * guarantee on control flow than the previous value.
 *
 * The ordering of strengths roughly reflects the way jumps are
 * lowered: jumps with higher strength tend to be lowered to jumps of
 * lower strength.  Accordingly, strength is used as a heuristic to
 * determine which lowering to perform first.
 *
 * This enum is also used by get_jump_strength() to categorize
 * instructions as either break, continue, return, or other.  When
 * used in this fashion, strength_always_clears_execute_flag is not
 * used.
 *
 * The control flow analysis made by this optimization pass makes two
 * simplifying assumptions:
 *
 * - It ignores discard instructions, since they are lowered by a
 *   separate pass (lower_discard.cpp).
 *
 * - It assumes it is always possible for control to flow from a loop
 *   to the instruction immediately following it.  Technically, this
 *   is not true (since all execution paths through the loop might
 *   jump back to the top, or return from the function).
 *
 * Both of these simplifying assumtions are safe, since they can never
 * cause reachable code to be incorrectly classified as unreachable;
 * they can only do the opposite.
 */
enum jump_strength
{
   /**
    * Analysis has produced no guarantee on how control flow might
    * exit this IR node.  It might fall out the bottom (with or
    * without clearing the execute flag, if present), or it might
    * continue to the top of the innermost enclosing loop, break out
    * of it, or return from the function.
    */
   strength_none,

   /**
    * The only way control can fall out the bottom of this node is
    * through a code path that clears the execute flag.  It might also
    * continue to the top of the innermost enclosing loop, break out
    * of it, or return from the function.
    */
   strength_always_clears_execute_flag,

   /**
    * Control cannot fall out the bottom of this node.  It might
    * continue to the top of the innermost enclosing loop, break out
    * of it, or return from the function.
    */
   strength_continue,

   /**
    * Control cannot fall out the bottom of this node, or continue the
    * top of the innermost enclosing loop.  It can only break out of
    * it or return from the function.
    */
   strength_break,

   /**
    * Control cannot fall out the bottom of this node, continue to the
    * top of the innermost enclosing loop, or break out of it.  It can
    * only return from the function.
    */
   strength_return
};

namespace {

struct block_record
{
   /* minimum jump strength (of lowered IR, not pre-lowering IR)
    *
    * If the block ends with a jump, must be the strength of the jump.
    * Otherwise, the jump would be dead and have been deleted before)
    *
    * If the block doesn't end with a jump, it can be different than strength_none if all paths before it lead to some jump
    * (e.g. an if with a return in one branch, and a break in the other, while not lowering them)
    * Note that identical jumps are usually unified though.
    */
   jump_strength min_strength;

   /* can anything clear the execute flag? */
   bool may_clear_execute_flag;

   block_record()
   {
      this->min_strength = strength_none;
      this->may_clear_execute_flag = false;
   }
};

struct loop_record
{
   ir_function_signature* signature;
   ir_loop* loop;

   /* used to avoid lowering the break used to represent lowered breaks */
   unsigned nesting_depth;
   bool in_if_at_the_end_of_the_loop;

   bool may_set_return_flag;

   ir_variable* break_flag;
   ir_variable* execute_flag; /* cleared to emulate continue */

   loop_record(ir_function_signature* p_signature = 0, ir_loop* p_loop = 0)
   {
      this->signature = p_signature;
      this->loop = p_loop;
      this->nesting_depth = 0;
      this->in_if_at_the_end_of_the_loop = false;
      this->may_set_return_flag = false;
      this->break_flag = 0;
      this->execute_flag = 0;
   }

   ir_variable* get_execute_flag()
   {
      /* also supported for the "function loop" */
      if(!this->execute_flag) {
         exec_list& list = this->loop ? this->loop->body_instructions : signature->body;
         this->execute_flag = new(this->signature) ir_variable(glsl_type::bool_type, "execute_flag", ir_var_temporary);
         list.push_head(new(this->signature) ir_assignment(new(this->signature) ir_dereference_variable(execute_flag), new(this->signature) ir_constant(true)));
         list.push_head(this->execute_flag);
      }
      return this->execute_flag;
   }

   ir_variable* get_break_flag()
   {
      assert(this->loop);
      if(!this->break_flag) {
         this->break_flag = new(this->signature) ir_variable(glsl_type::bool_type, "break_flag", ir_var_temporary);
         this->loop->insert_before(this->break_flag);
         this->loop->insert_before(new(this->signature) ir_assignment(new(this->signature) ir_dereference_variable(break_flag), new(this->signature) ir_constant(false)));
      }
      return this->break_flag;
   }
};

struct function_record
{
   ir_function_signature* signature;
   ir_variable* return_flag; /* used to break out of all loops and then jump to the return instruction */
   ir_variable* return_value;
   bool lower_return;
   unsigned nesting_depth;

   function_record(ir_function_signature* p_signature = 0,
                   bool lower_return = false)
   {
      this->signature = p_signature;
      this->return_flag = 0;
      this->return_value = 0;
      this->nesting_depth = 0;
      this->lower_return = lower_return;
   }

   ir_variable* get_return_flag()
   {
      if(!this->return_flag) {
         this->return_flag = new(this->signature) ir_variable(glsl_type::bool_type, "return_flag", ir_var_temporary);
         this->signature->body.push_head(new(this->signature) ir_assignment(new(this->signature) ir_dereference_variable(return_flag), new(this->signature) ir_constant(false)));
         this->signature->body.push_head(this->return_flag);
      }
      return this->return_flag;
   }

   ir_variable* get_return_value()
   {
      if(!this->return_value) {
         assert(!this->signature->return_type->is_void());
         return_value = new(this->signature) ir_variable(this->signature->return_type, "return_value", ir_var_temporary);
         this->signature->body.push_head(this->return_value);
      }
      return this->return_value;
   }
};

struct ir_lower_jumps_visitor : public ir_control_flow_visitor {
   /* Postconditions: on exit of any visit() function:
    *
    * ANALYSIS: this->block.min_strength,
    * this->block.may_clear_execute_flag, and
    * this->loop.may_set_return_flag are updated to reflect the
    * characteristics of the visited statement.
    *
    * DEAD_CODE_ELIMINATION: If this->block.min_strength is not
    * strength_none, the visited node is at the end of its exec_list.
    * In other words, any unreachable statements that follow the
    * visited statement in its exec_list have been removed.
    *
    * CONTAINED_JUMPS_LOWERED: If the visited statement contains other
    * statements, then should_lower_jump() is false for all of the
    * return, break, or continue statements it contains.
    *
    * Note that visiting a jump does not lower it.  That is the
    * responsibility of the statement (or function signature) that
    * contains the jump.
    */

   using ir_control_flow_visitor::visit;

   bool progress;

   struct function_record function;
   struct loop_record loop;
   struct block_record block;

   bool pull_out_jumps;
   bool lower_continue;
   bool lower_break;
   bool lower_sub_return;
   bool lower_main_return;

   ir_lower_jumps_visitor()
      : progress(false),
        pull_out_jumps(false),
        lower_continue(false),
        lower_break(false),
        lower_sub_return(false),
        lower_main_return(false)
   {
   }

   void truncate_after_instruction(exec_node *ir)
   {
      if (!ir)
         return;

      while (!ir->get_next()->is_tail_sentinel()) {
         ((ir_instruction *)ir->get_next())->remove();
         this->progress = true;
      }
   }

   void move_outer_block_inside(ir_instruction *ir, exec_list *inner_block)
   {
      while (!ir->get_next()->is_tail_sentinel()) {
         ir_instruction *move_ir = (ir_instruction *)ir->get_next();

         move_ir->remove();
         inner_block->push_tail(move_ir);
      }
   }

   /**
    * Insert the instructions necessary to lower a return statement,
    * before the given return instruction.
    */
   void insert_lowered_return(ir_return *ir)
   {
      ir_variable* return_flag = this->function.get_return_flag();
      if(!this->function.signature->return_type->is_void()) {
         ir_variable* return_value = this->function.get_return_value();
         ir->insert_before(
            new(ir) ir_assignment(
               new (ir) ir_dereference_variable(return_value),
               ir->value));
      }
      ir->insert_before(
         new(ir) ir_assignment(
            new (ir) ir_dereference_variable(return_flag),
            new (ir) ir_constant(true)));
      this->loop.may_set_return_flag = true;
   }

   /**
    * If the given instruction is a return, lower it to instructions
    * that store the return value (if there is one), set the return
    * flag, and then break.
    *
    * It is safe to pass NULL to this function.
    */
   void lower_return_unconditionally(ir_instruction *ir)
   {
      if (get_jump_strength(ir) != strength_return) {
         return;
      }
      insert_lowered_return((ir_return*)ir);
      ir->replace_with(new(ir) ir_loop_jump(ir_loop_jump::jump_break));
   }

   /**
    * Create the necessary instruction to replace a break instruction.
    */
   ir_instruction *create_lowered_break()
   {
      void *ctx = this->function.signature;
      return new(ctx) ir_assignment(
          new(ctx) ir_dereference_variable(this->loop.get_break_flag()),
          new(ctx) ir_constant(true));
   }

   /**
    * If the given instruction is a break, lower it to an instruction
    * that sets the break flag, without consulting
    * should_lower_jump().
    *
    * It is safe to pass NULL to this function.
    */
   void lower_break_unconditionally(ir_instruction *ir)
   {
      if (get_jump_strength(ir) != strength_break) {
         return;
      }
      ir->replace_with(create_lowered_break());
   }

   /**
    * If the block ends in a conditional or unconditional break, lower
    * it, even though should_lower_jump() says it needn't be lowered.
    */
   void lower_final_breaks(exec_list *block)
   {
      ir_instruction *ir = (ir_instruction *) block->get_tail();
      lower_break_unconditionally(ir);
      ir_if *ir_if = ir->as_if();
      if (ir_if) {
          lower_break_unconditionally(
              (ir_instruction *) ir_if->then_instructions.get_tail());
          lower_break_unconditionally(
              (ir_instruction *) ir_if->else_instructions.get_tail());
      }
   }

   virtual void visit(class ir_loop_jump * ir)
   {
      /* Eliminate all instructions after each one, since they are
       * unreachable.  This satisfies the DEAD_CODE_ELIMINATION
       * postcondition.
       */
      truncate_after_instruction(ir);

      /* Set this->block.min_strength based on this instruction.  This
       * satisfies the ANALYSIS postcondition.  It is not necessary to
       * update this->block.may_clear_execute_flag or
       * this->loop.may_set_return_flag, because an unlowered jump
       * instruction can't change any flags.
       */
      this->block.min_strength = ir->is_break() ? strength_break : strength_continue;

      /* The CONTAINED_JUMPS_LOWERED postcondition is already
       * satisfied, because jump statements can't contain other
       * statements.
       */
   }

   virtual void visit(class ir_return * ir)
   {
      /* Eliminate all instructions after each one, since they are
       * unreachable.  This satisfies the DEAD_CODE_ELIMINATION
       * postcondition.
       */
      truncate_after_instruction(ir);

      /* Set this->block.min_strength based on this instruction.  This
       * satisfies the ANALYSIS postcondition.  It is not necessary to
       * update this->block.may_clear_execute_flag or
       * this->loop.may_set_return_flag, because an unlowered return
       * instruction can't change any flags.
       */
      this->block.min_strength = strength_return;

      /* The CONTAINED_JUMPS_LOWERED postcondition is already
       * satisfied, because jump statements can't contain other
       * statements.
       */
   }

   virtual void visit(class ir_discard * ir)
   {
      /* Nothing needs to be done.  The ANALYSIS and
       * DEAD_CODE_ELIMINATION postconditions are already satisfied,
       * because discard statements are ignored by this optimization
       * pass.  The CONTAINED_JUMPS_LOWERED postcondition is already
       * satisfied, because discard statements can't contain other
       * statements.
       */
      (void) ir;
   }

   virtual void visit(class ir_precision_statement * ir)
   {
      /* Nothing needs to be done. */
   }

   virtual void visit(class ir_typedecl_statement * ir)
   {
      /* Nothing needs to be done. */
   }

   enum jump_strength get_jump_strength(ir_instruction* ir)
   {
      if(!ir)
         return strength_none;
      else if(ir->ir_type == ir_type_loop_jump) {
         if(((ir_loop_jump*)ir)->is_break())
            return strength_break;
         else
            return strength_continue;
      } else if(ir->ir_type == ir_type_return)
         return strength_return;
      else
         return strength_none;
   }

   bool should_lower_jump(ir_jump* ir)
   {
      unsigned strength = get_jump_strength(ir);
      bool lower;
      switch(strength)
      {
      case strength_none:
         lower = false; /* don't change this, code relies on it */
         break;
      case strength_continue:
         lower = lower_continue;
         break;
      case strength_break:
         assert(this->loop.loop);
         /* never lower "canonical break" */
         if(ir->get_next()->is_tail_sentinel() && (this->loop.nesting_depth == 0
               || (this->loop.nesting_depth == 1 && this->loop.in_if_at_the_end_of_the_loop)))
            lower = false;
         else
            lower = lower_break;
         break;
      case strength_return:
         /* never lower return at the end of a this->function */
         if(this->function.nesting_depth == 0 && ir->get_next()->is_tail_sentinel())
            lower = false;
         else
            lower = this->function.lower_return;
         break;
      }
      return lower;
   }

   block_record visit_block(exec_list* list)
   {
      /* Note: since visiting a node may change that node's next
       * pointer, we can't use visit_exec_list(), because
       * visit_exec_list() caches the node's next pointer before
       * visiting it.  So we use foreach_in_list() instead.
       *
       * foreach_in_list() isn't safe if the node being visited gets
       * removed, but fortunately this visitor doesn't do that.
       */

      block_record saved_block = this->block;
      this->block = block_record();
      foreach_in_list(ir_instruction, node, list) {
         node->accept(this);
      }
      block_record ret = this->block;
      this->block = saved_block;
      return ret;
   }

   virtual void visit(ir_if *ir)
   {
      if(this->loop.nesting_depth == 0 && ir->get_next()->is_tail_sentinel())
         this->loop.in_if_at_the_end_of_the_loop = true;

      ++this->function.nesting_depth;
      ++this->loop.nesting_depth;

      block_record block_records[2];
      ir_jump* jumps[2];

      /* Recursively lower nested jumps.  This satisfies the
       * CONTAINED_JUMPS_LOWERED postcondition, except in the case of
       * unconditional jumps at the end of ir->then_instructions and
       * ir->else_instructions, which are handled below.
       */
      block_records[0] = visit_block(&ir->then_instructions);
      block_records[1] = visit_block(&ir->else_instructions);

retry: /* we get here if we put code after the if inside a branch */

      /* Determine which of ir->then_instructions and
       * ir->else_instructions end with an unconditional jump.
       */
      for(unsigned i = 0; i < 2; ++i) {
         exec_list& list = i ? ir->else_instructions : ir->then_instructions;
         jumps[i] = 0;
         if(!list.is_empty() && get_jump_strength((ir_instruction*)list.get_tail()))
            jumps[i] = (ir_jump*)list.get_tail();
      }

      /* Loop until we have satisfied the CONTAINED_JUMPS_LOWERED
       * postcondition by lowering jumps in both then_instructions and
       * else_instructions.
       */
      for(;;) {
         /* Determine the types of the jumps that terminate
          * ir->then_instructions and ir->else_instructions.
          */
         jump_strength jump_strengths[2];

         for(unsigned i = 0; i < 2; ++i) {
            if(jumps[i]) {
               jump_strengths[i] = block_records[i].min_strength;
               assert(jump_strengths[i] == get_jump_strength(jumps[i]));
            } else
               jump_strengths[i] = strength_none;
         }

         /* If both code paths end in a jump, and the jumps are the
          * same, and we are pulling out jumps, replace them with a
          * single jump that comes after the if instruction.  The new
          * jump will be visited next, and it will be lowered if
          * necessary by the loop or conditional that encloses it.
          */
         if(pull_out_jumps && jump_strengths[0] == jump_strengths[1]) {
            bool unify = true;
            if(jump_strengths[0] == strength_continue)
               ir->insert_after(new(ir) ir_loop_jump(ir_loop_jump::jump_continue));
            else if(jump_strengths[0] == strength_break)
               ir->insert_after(new(ir) ir_loop_jump(ir_loop_jump::jump_break));
            /* FINISHME: unify returns with identical expressions */
            else if(jump_strengths[0] == strength_return && this->function.signature->return_type->is_void())
               ir->insert_after(new(ir) ir_return(NULL));
	    else
	       unify = false;

            if(unify) {
               jumps[0]->remove();
               jumps[1]->remove();
               this->progress = true;

               /* Update jumps[] to reflect the fact that the jumps
                * are gone, and update block_records[] to reflect the
                * fact that control can now flow to the next
                * instruction.
                */
               jumps[0] = 0;
               jumps[1] = 0;
               block_records[0].min_strength = strength_none;
               block_records[1].min_strength = strength_none;

               /* The CONTAINED_JUMPS_LOWERED postcondition is now
                * satisfied, so we can break out of the loop.
                */
               break;
            }
         }

         /* lower a jump: if both need to lowered, start with the strongest one, so that
          * we might later unify the lowered version with the other one
          */
         bool should_lower[2];
         for(unsigned i = 0; i < 2; ++i)
            should_lower[i] = should_lower_jump(jumps[i]);

         int lower;
         if(should_lower[1] && should_lower[0])
            lower = jump_strengths[1] > jump_strengths[0];
         else if(should_lower[0])
            lower = 0;
         else if(should_lower[1])
            lower = 1;
         else
            /* Neither code path ends in a jump that needs to be
             * lowered, so the CONTAINED_JUMPS_LOWERED postcondition
             * is satisfied and we can break out of the loop.
             */
            break;

         if(jump_strengths[lower] == strength_return) {
            /* To lower a return, we create a return flag (if the
             * function doesn't have one already) and add instructions
             * that: 1. store the return value (if this function has a
             * non-void return) and 2. set the return flag
             */
            insert_lowered_return((ir_return*)jumps[lower]);
            if(this->loop.loop) {
               /* If we are in a loop, replace the return instruction
                * with a break instruction, and then loop so that the
                * break instruction can be lowered if necessary.
                */
               ir_loop_jump* lowered = 0;
               lowered = new(ir) ir_loop_jump(ir_loop_jump::jump_break);
               /* Note: we must update block_records and jumps to
                * reflect the fact that the control path has been
                * altered from a return to a break.
                */
               block_records[lower].min_strength = strength_break;
               jumps[lower]->replace_with(lowered);
               jumps[lower] = lowered;
            } else {
               /* If we are not in a loop, we then proceed as we would
                * for a continue statement (set the execute flag to
                * false to prevent the rest of the function from
                * executing).
                */
               goto lower_continue;
            }
            this->progress = true;
         } else if(jump_strengths[lower] == strength_break) {
            /* To lower a break, we create a break flag (if the loop
             * doesn't have one already) and add an instruction that
             * sets it.
             *
             * Then we proceed as we would for a continue statement
             * (set the execute flag to false to prevent the rest of
             * the loop body from executing).
             *
             * The visit() function for the loop will ensure that the
             * break flag is checked after executing the loop body.
             */
            jumps[lower]->insert_before(create_lowered_break());
            goto lower_continue;
         } else if(jump_strengths[lower] == strength_continue) {
lower_continue:
            /* To lower a continue, we create an execute flag (if the
             * loop doesn't have one already) and replace the continue
             * with an instruction that clears it.
             *
             * Note that this code path gets exercised when lowering
             * return statements that are not inside a loop, so
             * this->loop must be initialized even outside of loops.
             */
            ir_variable* execute_flag = this->loop.get_execute_flag();
            jumps[lower]->replace_with(new(ir) ir_assignment(new (ir) ir_dereference_variable(execute_flag), new (ir) ir_constant(false)));
            /* Note: we must update block_records and jumps to reflect
             * the fact that the control path has been altered to an
             * instruction that clears the execute flag.
             */
            jumps[lower] = 0;
            block_records[lower].min_strength = strength_always_clears_execute_flag;
            block_records[lower].may_clear_execute_flag = true;
            this->progress = true;

            /* Let the loop run again, in case the other branch of the
             * if needs to be lowered too.
             */
         }
      }

      /* move out a jump out if possible */
      if(pull_out_jumps) {
         /* If one of the branches ends in a jump, and control cannot
          * fall out the bottom of the other branch, then we can move
          * the jump after the if.
          *
          * Set move_out to the branch we are moving a jump out of.
          */
         int move_out = -1;
         if(jumps[0] && block_records[1].min_strength >= strength_continue)
            move_out = 0;
         else if(jumps[1] && block_records[0].min_strength >= strength_continue)
            move_out = 1;

         if(move_out >= 0)
         {
            jumps[move_out]->remove();
            ir->insert_after(jumps[move_out]);
            /* Note: we must update block_records and jumps to reflect
             * the fact that the jump has been moved out of the if.
             */
            jumps[move_out] = 0;
            block_records[move_out].min_strength = strength_none;
            this->progress = true;
         }
      }

      /* Now satisfy the ANALYSIS postcondition by setting
       * this->block.min_strength and
       * this->block.may_clear_execute_flag based on the
       * characteristics of the two branches.
       */
      if(block_records[0].min_strength < block_records[1].min_strength)
         this->block.min_strength = block_records[0].min_strength;
      else
         this->block.min_strength = block_records[1].min_strength;
      this->block.may_clear_execute_flag = this->block.may_clear_execute_flag || block_records[0].may_clear_execute_flag || block_records[1].may_clear_execute_flag;

      /* Now we need to clean up the instructions that follow the
       * if.
       *
       * If those instructions are unreachable, then satisfy the
       * DEAD_CODE_ELIMINATION postcondition by eliminating them.
       * Otherwise that postcondition is already satisfied.
       */
      if(this->block.min_strength)
         truncate_after_instruction(ir);
      else if(this->block.may_clear_execute_flag)
      {
         /* If the "if" instruction might clear the execute flag, then
          * we need to guard any instructions that follow so that they
          * are only executed if the execute flag is set.
          *
          * If one of the branches of the "if" always clears the
          * execute flag, and the other branch never clears it, then
          * this is easy: just move all the instructions following the
          * "if" into the branch that never clears it.
          */
         int move_into = -1;
         if(block_records[0].min_strength && !block_records[1].may_clear_execute_flag)
            move_into = 1;
         else if(block_records[1].min_strength && !block_records[0].may_clear_execute_flag)
            move_into = 0;

         if(move_into >= 0) {
            assert(!block_records[move_into].min_strength && !block_records[move_into].may_clear_execute_flag); /* otherwise, we just truncated */

            exec_list* list = move_into ? &ir->else_instructions : &ir->then_instructions;
            exec_node* next = ir->get_next();
            if(!next->is_tail_sentinel()) {
               move_outer_block_inside(ir, list);

               /* If any instructions moved, then we need to visit
                * them (since they are now inside the "if").  Since
                * block_records[move_into] is in its default state
                * (see assertion above), we can safely replace
                * block_records[move_into] with the result of this
                * analysis.
                */
               exec_list list;
               list.head_sentinel.next = next;
               block_records[move_into] = visit_block(&list);

               /*
                * Then we need to re-start our jump lowering, since one
                * of the instructions we moved might be a jump that
                * needs to be lowered.
                */
               this->progress = true;
               goto retry;
            }
         } else {
            /* If we get here, then the simple case didn't apply; we
             * need to actually guard the instructions that follow.
             *
             * To avoid creating unnecessarily-deep nesting, first
             * look through the instructions that follow and unwrap
             * any instructions that that are already wrapped in the
             * appropriate guard.
             */
            ir_instruction* ir_after;
            for(ir_after = (ir_instruction*)ir->get_next(); !ir_after->is_tail_sentinel();)
            {
               ir_if* ir_if = ir_after->as_if();
               if(ir_if && ir_if->else_instructions.is_empty()) {
                  ir_dereference_variable* ir_if_cond_deref = ir_if->condition->as_dereference_variable();
                  if(ir_if_cond_deref && ir_if_cond_deref->var == this->loop.execute_flag) {
                     ir_instruction* ir_next = (ir_instruction*)ir_after->get_next();
                     ir_after->insert_before(&ir_if->then_instructions);
                     ir_after->remove();
                     ir_after = ir_next;
                     continue;
                  }
               }
               ir_after = (ir_instruction*)ir_after->get_next();

               /* only set this if we find any unprotected instruction */
               this->progress = true;
            }

            /* Then, wrap all the instructions that follow in a single
             * guard.
             */
            if(!ir->get_next()->is_tail_sentinel()) {
               assert(this->loop.execute_flag);
               ir_if* if_execute = new(ir) ir_if(new(ir) ir_dereference_variable(this->loop.execute_flag));
               move_outer_block_inside(ir, &if_execute->then_instructions);
               ir->insert_after(if_execute);
            }
         }
      }
      --this->loop.nesting_depth;
      --this->function.nesting_depth;
   }

   virtual void visit(ir_loop *ir)
   {
      /* Visit the body of the loop, with a fresh data structure in
       * this->loop so that the analysis we do here won't bleed into
       * enclosing loops.
       *
       * We assume that all code after a loop is reachable from the
       * loop (see comments on enum jump_strength), so the
       * DEAD_CODE_ELIMINATION postcondition is automatically
       * satisfied, as is the block.min_strength portion of the
       * ANALYSIS postcondition.
       *
       * The block.may_clear_execute_flag portion of the ANALYSIS
       * postcondition is automatically satisfied because execute
       * flags do not propagate outside of loops.
       *
       * The loop.may_set_return_flag portion of the ANALYSIS
       * postcondition is handled below.
       */
      ++this->function.nesting_depth;
      loop_record saved_loop = this->loop;
      this->loop = loop_record(this->function.signature, ir);

      /* Recursively lower nested jumps.  This satisfies the
       * CONTAINED_JUMPS_LOWERED postcondition, except in the case of
       * an unconditional continue or return at the bottom of the
       * loop, which are handled below.
       */
      block_record body = visit_block(&ir->body_instructions);

      /* If the loop ends in an unconditional continue, eliminate it
       * because it is redundant.
       */
      ir_instruction *ir_last
         = (ir_instruction *) ir->body_instructions.get_tail();
      if (get_jump_strength(ir_last) == strength_continue) {
         ir_last->remove();
      }

      /* If the loop ends in an unconditional return, and we are
       * lowering returns, lower it.
       */
      if (this->function.lower_return)
         lower_return_unconditionally(ir_last);

      if(body.min_strength >= strength_break) {
         /* FINISHME: If the min_strength of the loop body is
          * strength_break or strength_return, that means that it
          * isn't a loop at all, since control flow always leaves the
          * body of the loop via break or return.  In principle the
          * loop could be eliminated in this case.  This optimization
          * is not implemented yet.
          */
      }

      if(this->loop.break_flag) {
         /* We only get here if we are lowering breaks */
         assert (lower_break);

         /* If a break flag was generated while visiting the body of
          * the loop, then at least one break was lowered, so we need
          * to generate an if statement at the end of the loop that
          * does a "break" if the break flag is set.  The break we
          * generate won't violate the CONTAINED_JUMPS_LOWERED
          * postcondition, because should_lower_jump() always returns
          * false for a break that happens at the end of a loop.
          *
          * However, if the loop already ends in a conditional or
          * unconditional break, then we need to lower that break,
          * because it won't be at the end of the loop anymore.
          */
         lower_final_breaks(&ir->body_instructions);

         ir_if* break_if = new(ir) ir_if(new(ir) ir_dereference_variable(this->loop.break_flag));
         break_if->then_instructions.push_tail(new(ir) ir_loop_jump(ir_loop_jump::jump_break));
         ir->body_instructions.push_tail(break_if);
      }

      /* If the body of the loop may set the return flag, then at
       * least one return was lowered to a break, so we need to ensure
       * that the return flag is checked after the body of the loop is
       * executed.
       */
      if(this->loop.may_set_return_flag) {
         assert(this->function.return_flag);
         /* Generate the if statement to check the return flag */
         ir_if* return_if = new(ir) ir_if(new(ir) ir_dereference_variable(this->function.return_flag));
         /* Note: we also need to propagate the knowledge that the
          * return flag may get set to the outer context.  This
          * satisfies the loop.may_set_return_flag part of the
          * ANALYSIS postcondition.
          */
         saved_loop.may_set_return_flag = true;
         if(saved_loop.loop)
            /* If this loop is nested inside another one, then the if
             * statement that we generated should break out of that
             * loop if the return flag is set.  Caller will lower that
             * break statement if necessary.
             */
            return_if->then_instructions.push_tail(new(ir) ir_loop_jump(ir_loop_jump::jump_break));
         else {
            /* Otherwise, ensure that the instructions that follow are only
             * executed if the return flag is clear.  We can do that by moving
             * those instructions into the else clause of the generated if
             * statement.
             */
            move_outer_block_inside(ir, &return_if->else_instructions);

            /* In case the loop is embedded inside an if add a new return to
             * the return flag then branch and let a future pass tidy it up.
             */
            if (this->function.signature->return_type->is_void())
               return_if->then_instructions.push_tail(new(ir) ir_return(NULL));
            else {
               assert(this->function.return_value);
               ir_variable* return_value = this->function.return_value;
               return_if->then_instructions.push_tail(
                  new(ir) ir_return(new(ir) ir_dereference_variable(return_value)));
            }
         }

         ir->insert_after(return_if);
      }

      this->loop = saved_loop;
      --this->function.nesting_depth;
   }

   virtual void visit(ir_function_signature *ir)
   {
      /* these are not strictly necessary */
      assert(!this->function.signature);
      assert(!this->loop.loop);

      bool lower_return;
      if (strcmp(ir->function_name(), "main") == 0)
         lower_return = lower_main_return;
      else
         lower_return = lower_sub_return;

      function_record saved_function = this->function;
      loop_record saved_loop = this->loop;
      this->function = function_record(ir, lower_return);
      this->loop = loop_record(ir);

      assert(!this->loop.loop);

      /* Visit the body of the function to lower any jumps that occur
       * in it, except possibly an unconditional return statement at
       * the end of it.
       */
      visit_block(&ir->body);

      /* If the body ended in an unconditional return of non-void,
       * then we don't need to lower it because it's the one canonical
       * return.
       *
       * If the body ended in a return of void, eliminate it because
       * it is redundant.
       */
      if (ir->return_type->is_void() &&
          get_jump_strength((ir_instruction *) ir->body.get_tail())) {
         ir_jump *jump = (ir_jump *) ir->body.get_tail();
         assert (jump->ir_type == ir_type_return);
         jump->remove();
      }

      if(this->function.return_value)
         ir->body.push_tail(new(ir) ir_return(new (ir) ir_dereference_variable(this->function.return_value)));

      this->loop = saved_loop;
      this->function = saved_function;
   }

   virtual void visit(class ir_function * ir)
   {
      visit_block(&ir->signatures);
   }
};

} /* anonymous namespace */

bool
do_lower_jumps(exec_list *instructions, bool pull_out_jumps, bool lower_sub_return, bool lower_main_return, bool lower_continue, bool lower_break)
{
   ir_lower_jumps_visitor v;
   v.pull_out_jumps = pull_out_jumps;
   v.lower_continue = lower_continue;
   v.lower_break = lower_break;
   v.lower_sub_return = lower_sub_return;
   v.lower_main_return = lower_main_return;

   bool progress_ever = false;
   do {
      v.progress = false;
      visit_exec_list(instructions, &v);
      progress_ever = v.progress || progress_ever;
   } while (v.progress);

   return progress_ever;
}
