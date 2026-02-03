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

static void try_add_loop_terminator(loop_variable_state *ls, ir_if *ir);

static bool all_expression_operands_are_loop_constant(ir_rvalue *,
						      hash_table *);

static ir_rvalue *get_basic_induction_increment(ir_assignment *, hash_table *);

/**
 * Find an initializer of a variable outside a loop
 *
 * Works backwards from the loop to find the pre-loop value of the variable.
 * This is used, for example, to find the initial value of loop induction
 * variables.
 *
 * \param loop  Loop where \c var is an induction variable
 * \param var   Variable whose initializer is to be found
 *
 * \return
 * The \c ir_rvalue assigned to the variable outside the loop.  May return
 * \c NULL if no initializer can be found.
 */
static ir_rvalue *
find_initial_value(ir_loop *loop, ir_variable *var)
{
   for (exec_node *node = loop->prev; !node->is_head_sentinel();
        node = node->prev) {
      ir_instruction *ir = (ir_instruction *) node;

      switch (ir->ir_type) {
      case ir_type_call:
      case ir_type_loop:
      case ir_type_loop_jump:
      case ir_type_return:
      case ir_type_if:
         return NULL;

      case ir_type_function:
      case ir_type_function_signature:
         assert(!"Should not get here.");
         return NULL;

      case ir_type_assignment: {
         ir_assignment *assign = ir->as_assignment();
         ir_variable *assignee = assign->lhs->whole_variable_referenced();

         if (assignee == var)
            return (assign->condition != NULL) ? NULL : assign->rhs;

         break;
      }

      default:
         break;
      }
   }

   return NULL;
}


static int
calculate_iterations(ir_rvalue *from, ir_rvalue *to, ir_rvalue *increment,
                     enum ir_expression_operation op, bool continue_from_then,
                     bool swap_compare_operands, bool inc_before_terminator)
{
   if (from == NULL || to == NULL || increment == NULL)
      return -1;

   void *mem_ctx = ralloc_context(NULL);

   ir_expression *const sub =
      new(mem_ctx) ir_expression(ir_binop_sub, from->type, to, from);

   ir_expression *const div =
      new(mem_ctx) ir_expression(ir_binop_div, sub->type, sub, increment);

   ir_constant *iter = div->constant_expression_value(mem_ctx);
   if (iter == NULL) {
      ralloc_free(mem_ctx);
      return -1;
   }

   if (!iter->type->is_integer_32()) {
      const ir_expression_operation op = iter->type->is_double()
         ? ir_unop_d2i : ir_unop_f2i;
      ir_rvalue *cast =
         new(mem_ctx) ir_expression(op, glsl_type::int_type, iter, NULL);

      iter = cast->constant_expression_value(mem_ctx);
   }

   int iter_value = iter->get_int_component(0);

   /* Code after this block works under assumption that iterator will be
    * incremented or decremented until it hits the limit,
    * however the loop condition can be false on the first iteration.
    * Handle such loops first.
    */
   {
      ir_rvalue *first_value = from;
      if (inc_before_terminator) {
         first_value =
            new(mem_ctx) ir_expression(ir_binop_add, from->type, from, increment);
      }

      ir_expression *cmp = swap_compare_operands
            ? new(mem_ctx) ir_expression(op, glsl_type::bool_type, to, first_value)
            : new(mem_ctx) ir_expression(op, glsl_type::bool_type, first_value, to);
      if (continue_from_then)
         cmp = new(mem_ctx) ir_expression(ir_unop_logic_not, cmp);

      ir_constant *const cmp_result = cmp->constant_expression_value(mem_ctx);
      assert(cmp_result != NULL);
      if (cmp_result->get_bool_component(0)) {
         ralloc_free(mem_ctx);
         return 0;
      }
   }

   /* Make sure that the calculated number of iterations satisfies the exit
    * condition.  This is needed to catch off-by-one errors and some types of
    * ill-formed loops.  For example, we need to detect that the following
    * loop does not have a maximum iteration count.
    *
    *    for (float x = 0.0; x != 0.9; x += 0.2)
    *        ;
    */
   const int bias[] = { -1, 0, 1 };
   bool valid_loop = false;

   for (unsigned i = 0; i < ARRAY_SIZE(bias); i++) {
      /* Increment may be of type int, uint or float. */
      switch (increment->type->base_type) {
      case GLSL_TYPE_INT:
         iter = new(mem_ctx) ir_constant(iter_value + bias[i]);
         break;
      case GLSL_TYPE_UINT:
         iter = new(mem_ctx) ir_constant(unsigned(iter_value + bias[i]));
         break;
      case GLSL_TYPE_FLOAT:
         iter = new(mem_ctx) ir_constant(float(iter_value + bias[i]));
         break;
      case GLSL_TYPE_DOUBLE:
         iter = new(mem_ctx) ir_constant(double(iter_value + bias[i]));
         break;
      default:
          unreachable("Unsupported type for loop iterator.");
      }

      ir_expression *const mul =
         new(mem_ctx) ir_expression(ir_binop_mul, increment->type, iter,
                                    increment);

      ir_expression *const add =
         new(mem_ctx) ir_expression(ir_binop_add, mul->type, mul, from);

      ir_expression *cmp = swap_compare_operands
         ? new(mem_ctx) ir_expression(op, glsl_type::bool_type, to, add)
         : new(mem_ctx) ir_expression(op, glsl_type::bool_type, add, to);
      if (continue_from_then)
         cmp = new(mem_ctx) ir_expression(ir_unop_logic_not, cmp);

      ir_constant *const cmp_result = cmp->constant_expression_value(mem_ctx);

      assert(cmp_result != NULL);
      if (cmp_result->get_bool_component(0)) {
         iter_value += bias[i];
         valid_loop = true;
         break;
      }
   }

   ralloc_free(mem_ctx);

   if (inc_before_terminator) {
      iter_value--;
   }

   return (valid_loop) ? iter_value : -1;
}

static bool
incremented_before_terminator(ir_loop *loop, ir_variable *var,
                              ir_if *terminator)
{
   for (exec_node *node = loop->body_instructions.get_head();
        !node->is_tail_sentinel();
        node = node->get_next()) {
      ir_instruction *ir = (ir_instruction *) node;

      switch (ir->ir_type) {
      case ir_type_if:
         if (ir->as_if() == terminator)
            return false;
         break;

      case ir_type_assignment: {
         ir_assignment *assign = ir->as_assignment();
         ir_variable *assignee = assign->lhs->whole_variable_referenced();

         if (assignee == var) {
            assert(assign->condition == NULL);
            return true;
         }

         break;
      }

      default:
         break;
      }
   }

   unreachable("Unable to find induction variable");
}

/**
 * Record the fact that the given loop variable was referenced inside the loop.
 *
 * \arg in_assignee is true if the reference was on the LHS of an assignment.
 *
 * \arg in_conditional_code_or_nested_loop is true if the reference occurred
 * inside an if statement or a nested loop.
 *
 * \arg current_assignment is the ir_assignment node that the loop variable is
 * on the LHS of, if any (ignored if \c in_assignee is false).
 */
void
loop_variable::record_reference(bool in_assignee,
                                bool in_conditional_code_or_nested_loop,
                                ir_assignment *current_assignment)
{
   if (in_assignee) {
      assert(current_assignment != NULL);

      if (in_conditional_code_or_nested_loop ||
          current_assignment->condition != NULL) {
         this->conditional_or_nested_assignment = true;
      }

      if (this->first_assignment == NULL) {
         assert(this->num_assignments == 0);

         this->first_assignment = current_assignment;
      }

      this->num_assignments++;
   } else if (this->first_assignment == current_assignment) {
      /* This catches the case where the variable is used in the RHS of an
       * assignment where it is also in the LHS.
       */
      this->read_before_write = true;
   }
}


loop_state::loop_state()
{
   this->ht = _mesa_pointer_hash_table_create(NULL);
   this->mem_ctx = ralloc_context(NULL);
   this->loop_found = false;
}


loop_state::~loop_state()
{
   _mesa_hash_table_destroy(this->ht, NULL);
   ralloc_free(this->mem_ctx);
}


loop_variable_state *
loop_state::insert(ir_loop *ir)
{
   loop_variable_state *ls = new(this->mem_ctx) loop_variable_state;

   _mesa_hash_table_insert(this->ht, ir, ls);
   this->loop_found = true;

   return ls;
}


loop_variable_state *
loop_state::get(const ir_loop *ir)
{
   hash_entry *entry = _mesa_hash_table_search(this->ht, ir);
   return entry ? (loop_variable_state *) entry->data : NULL;
}


loop_variable *
loop_variable_state::get(const ir_variable *ir)
{
   if (ir == NULL)
      return NULL;

   hash_entry *entry = _mesa_hash_table_search(this->var_hash, ir);
   return entry ? (loop_variable *) entry->data : NULL;
}


loop_variable *
loop_variable_state::insert(ir_variable *var)
{
   void *mem_ctx = ralloc_parent(this);
   loop_variable *lv = rzalloc(mem_ctx, loop_variable);

   lv->var = var;

   _mesa_hash_table_insert(this->var_hash, lv->var, lv);
   this->variables.push_tail(lv);

   return lv;
}


loop_terminator *
loop_variable_state::insert(ir_if *if_stmt, bool continue_from_then)
{
   void *mem_ctx = ralloc_parent(this);
   loop_terminator *t = new(mem_ctx) loop_terminator();

   t->ir = if_stmt;
   t->continue_from_then = continue_from_then;

   this->terminators.push_tail(t);

   return t;
}


/**
 * If the given variable already is recorded in the state for this loop,
 * return the corresponding loop_variable object that records information
 * about it.
 *
 * Otherwise, create a new loop_variable object to record information about
 * the variable, and set its \c read_before_write field appropriately based on
 * \c in_assignee.
 *
 * \arg in_assignee is true if this variable was encountered on the LHS of an
 * assignment.
 */
loop_variable *
loop_variable_state::get_or_insert(ir_variable *var, bool in_assignee)
{
   loop_variable *lv = this->get(var);

   if (lv == NULL) {
      lv = this->insert(var);
      lv->read_before_write = !in_assignee;
   }

   return lv;
}


namespace {

class loop_analysis : public ir_hierarchical_visitor {
public:
   loop_analysis(loop_state *loops);

   virtual ir_visitor_status visit(ir_loop_jump *);
   virtual ir_visitor_status visit(ir_dereference_variable *);

   virtual ir_visitor_status visit_enter(ir_call *);

   virtual ir_visitor_status visit_enter(ir_loop *);
   virtual ir_visitor_status visit_leave(ir_loop *);
   virtual ir_visitor_status visit_enter(ir_assignment *);
   virtual ir_visitor_status visit_leave(ir_assignment *);
   virtual ir_visitor_status visit_enter(ir_if *);
   virtual ir_visitor_status visit_leave(ir_if *);

   loop_state *loops;

   int if_statement_depth;

   ir_assignment *current_assignment;

   exec_list state;
};

} /* anonymous namespace */

loop_analysis::loop_analysis(loop_state *loops)
   : loops(loops), if_statement_depth(0), current_assignment(NULL)
{
   /* empty */
}


ir_visitor_status
loop_analysis::visit(ir_loop_jump *ir)
{
   (void) ir;

   assert(!this->state.is_empty());

   loop_variable_state *const ls =
      (loop_variable_state *) this->state.get_head();

   ls->num_loop_jumps++;

   return visit_continue;
}


ir_visitor_status
loop_analysis::visit_enter(ir_call *)
{
   /* Mark every loop that we're currently analyzing as containing an ir_call
    * (even those at outer nesting levels).
    */
   foreach_in_list(loop_variable_state, ls, &this->state) {
      ls->contains_calls = true;
   }

   return visit_continue_with_parent;
}


ir_visitor_status
loop_analysis::visit(ir_dereference_variable *ir)
{
   /* If we're not somewhere inside a loop, there's nothing to do.
    */
   if (this->state.is_empty())
      return visit_continue;

   bool nested = false;

   foreach_in_list(loop_variable_state, ls, &this->state) {
      ir_variable *var = ir->variable_referenced();
      loop_variable *lv = ls->get_or_insert(var, this->in_assignee);

      lv->record_reference(this->in_assignee,
                           nested || this->if_statement_depth > 0,
                           this->current_assignment);
      nested = true;
   }

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_enter(ir_loop *ir)
{
   loop_variable_state *ls = this->loops->insert(ir);
   this->state.push_head(ls);

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_leave(ir_loop *ir)
{
   loop_variable_state *const ls =
      (loop_variable_state *) this->state.pop_head();

   /* Function calls may contain side effects.  These could alter any of our
    * variables in ways that cannot be known, and may even terminate shader
    * execution (say, calling discard in the fragment shader).  So we can't
    * rely on any of our analysis about assignments to variables.
    *
    * We could perform some conservative analysis (prove there's no statically
    * possible assignment, etc.) but it isn't worth it for now; function
    * inlining will allow us to unroll loops anyway.
    */
   if (ls->contains_calls)
      return visit_continue;

   foreach_in_list(ir_instruction, node, &ir->body_instructions) {
      /* Skip over declarations at the start of a loop.
       */
      if (node->as_variable())
	 continue;

      ir_if *if_stmt = ((ir_instruction *) node)->as_if();

      if (if_stmt != NULL)
         try_add_loop_terminator(ls, if_stmt);
   }


   foreach_in_list_safe(loop_variable, lv, &ls->variables) {
      /* Move variables that are already marked as being loop constant to
       * a separate list.  These trivially don't need to be tested.
       */
      if (lv->is_loop_constant()) {
	 lv->remove();
	 ls->constants.push_tail(lv);
      }
   }

   /* Each variable assigned in the loop that isn't already marked as being loop
    * constant might still be loop constant.  The requirements at this point
    * are:
    *
    *    - Variable is written before it is read.
    *
    *    - Only one assignment to the variable.
    *
    *    - All operands on the RHS of the assignment are also loop constants.
    *
    * The last requirement is the reason for the progress loop.  A variable
    * marked as a loop constant on one pass may allow other variables to be
    * marked as loop constant on following passes.
    */
   bool progress;
   do {
      progress = false;

      foreach_in_list_safe(loop_variable, lv, &ls->variables) {
	 if (lv->conditional_or_nested_assignment || (lv->num_assignments > 1))
	    continue;

	 /* Process the RHS of the assignment.  If all of the variables
	  * accessed there are loop constants, then add this
	  */
	 ir_rvalue *const rhs = lv->first_assignment->rhs;
	 if (all_expression_operands_are_loop_constant(rhs, ls->var_hash)) {
	    lv->rhs_clean = true;

	    if (lv->is_loop_constant()) {
	       progress = true;

	       lv->remove();
	       ls->constants.push_tail(lv);
	    }
	 }
      }
   } while (progress);

   /* The remaining variables that are not loop invariant might be loop
    * induction variables.
    */
   foreach_in_list_safe(loop_variable, lv, &ls->variables) {
      /* If there is more than one assignment to a variable, it cannot be a
       * loop induction variable.  This isn't strictly true, but this is a
       * very simple induction variable detector, and it can't handle more
       * complex cases.
       */
      if (lv->num_assignments > 1)
	 continue;

      /* All of the variables with zero assignments in the loop are loop
       * invariant, and they should have already been filtered out.
       */
      assert(lv->num_assignments == 1);
      assert(lv->first_assignment != NULL);

      /* The assignment to the variable in the loop must be unconditional and
       * not inside a nested loop.
       */
      if (lv->conditional_or_nested_assignment)
	 continue;

      /* Basic loop induction variables have a single assignment in the loop
       * that has the form 'VAR = VAR + i' or 'VAR = VAR - i' where i is a
       * loop invariant.
       */
      ir_rvalue *const inc =
	 get_basic_induction_increment(lv->first_assignment, ls->var_hash);
      if (inc != NULL) {
	 lv->increment = inc;

	 lv->remove();
	 ls->induction_variables.push_tail(lv);
      }
   }

   /* Search the loop terminating conditions for those of the form 'i < c'
    * where i is a loop induction variable, c is a constant, and < is any
    * relative operator.  From each of these we can infer an iteration count.
    * Also figure out which terminator (if any) produces the smallest
    * iteration count--this is the limiting terminator.
    */
   foreach_in_list(loop_terminator, t, &ls->terminators) {
      ir_if *if_stmt = t->ir;

      /* If-statements can be either 'if (expr)' or 'if (deref)'.  We only care
       * about the former here.
       */
      ir_expression *cond = if_stmt->condition->as_expression();
      if (cond == NULL)
	 continue;

      switch (cond->operation) {
      case ir_binop_less:
      case ir_binop_gequal: {
	 /* The expressions that we care about will either be of the form
	  * 'counter < limit' or 'limit < counter'.  Figure out which is
	  * which.
	  */
	 ir_rvalue *counter = cond->operands[0]->as_dereference_variable();
	 ir_constant *limit = cond->operands[1]->as_constant();
	 enum ir_expression_operation cmp = cond->operation;
         bool swap_compare_operands = false;

	 if (limit == NULL) {
	    counter = cond->operands[1]->as_dereference_variable();
	    limit = cond->operands[0]->as_constant();
            swap_compare_operands = true;
	 }

	 if ((counter == NULL) || (limit == NULL))
	    break;

	 ir_variable *var = counter->variable_referenced();

	 ir_rvalue *init = find_initial_value(ir, var);

         loop_variable *lv = ls->get(var);
         if (lv != NULL && lv->is_induction_var()) {
            bool inc_before_terminator =
               incremented_before_terminator(ir, var, t->ir);

            t->iterations = calculate_iterations(init, limit, lv->increment,
                                                 cmp, t->continue_from_then,
                                                 swap_compare_operands,
                                                 inc_before_terminator);

            if (t->iterations >= 0 &&
                (ls->limiting_terminator == NULL ||
                 t->iterations < ls->limiting_terminator->iterations)) {
               ls->limiting_terminator = t;
            }
         }
         break;
      }

      default:
         break;
      }
   }

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_enter(ir_if *ir)
{
   (void) ir;

   if (!this->state.is_empty())
      this->if_statement_depth++;

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_leave(ir_if *ir)
{
   (void) ir;

   if (!this->state.is_empty())
      this->if_statement_depth--;

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_enter(ir_assignment *ir)
{
   /* If we're not somewhere inside a loop, there's nothing to do.
    */
   if (this->state.is_empty())
      return visit_continue_with_parent;

   this->current_assignment = ir;

   return visit_continue;
}

ir_visitor_status
loop_analysis::visit_leave(ir_assignment *ir)
{
   /* Since the visit_enter exits with visit_continue_with_parent for this
    * case, the loop state stack should never be empty here.
    */
   assert(!this->state.is_empty());

   assert(this->current_assignment == ir);
   this->current_assignment = NULL;

   return visit_continue;
}


class examine_rhs : public ir_hierarchical_visitor {
public:
   examine_rhs(hash_table *loop_variables)
   {
      this->only_uses_loop_constants = true;
      this->loop_variables = loop_variables;
   }

   virtual ir_visitor_status visit(ir_dereference_variable *ir)
   {
      hash_entry *entry = _mesa_hash_table_search(this->loop_variables,
                                                  ir->var);
      loop_variable *lv = entry ? (loop_variable *) entry->data : NULL;

      assert(lv != NULL);

      if (lv->is_loop_constant()) {
	 return visit_continue;
      } else {
	 this->only_uses_loop_constants = false;
	 return visit_stop;
      }
   }

   hash_table *loop_variables;
   bool only_uses_loop_constants;
};


bool
all_expression_operands_are_loop_constant(ir_rvalue *ir, hash_table *variables)
{
   examine_rhs v(variables);

   ir->accept(&v);

   return v.only_uses_loop_constants;
}


ir_rvalue *
get_basic_induction_increment(ir_assignment *ir, hash_table *var_hash)
{
   /* The RHS must be a binary expression.
    */
   ir_expression *const rhs = ir->rhs->as_expression();
   if ((rhs == NULL)
       || ((rhs->operation != ir_binop_add)
	   && (rhs->operation != ir_binop_sub)))
      return NULL;

   /* One of the of operands of the expression must be the variable assigned.
    * If the operation is subtraction, the variable in question must be the
    * "left" operand.
    */
   ir_variable *const var = ir->lhs->variable_referenced();

   ir_variable *const op0 = rhs->operands[0]->variable_referenced();
   ir_variable *const op1 = rhs->operands[1]->variable_referenced();

   if (((op0 != var) && (op1 != var))
       || ((op1 == var) && (rhs->operation == ir_binop_sub)))
      return NULL;

   ir_rvalue *inc = (op0 == var) ? rhs->operands[1] : rhs->operands[0];

   if (inc->as_constant() == NULL) {
      ir_variable *const inc_var = inc->variable_referenced();
      if (inc_var != NULL) {
         hash_entry *entry = _mesa_hash_table_search(var_hash, inc_var);
         loop_variable *lv = entry ? (loop_variable *) entry->data : NULL;

         if (lv == NULL || !lv->is_loop_constant()) {
            assert(lv != NULL);
            inc = NULL;
         }
      } else
	 inc = NULL;
   }

   if ((inc != NULL) && (rhs->operation == ir_binop_sub)) {
      void *mem_ctx = ralloc_parent(ir);

      inc = new(mem_ctx) ir_expression(ir_unop_neg,
				       inc->type,
				       inc->clone(mem_ctx, NULL),
				       NULL);
   }

   return inc;
}


/**
 * Detect whether an if-statement is a loop terminating condition, if so
 * add it to the list of loop terminators.
 *
 * Detects if-statements of the form
 *
 *  (if (expression bool ...) (...then_instrs...break))
 *
 *     or
 *
 *  (if (expression bool ...) ... (...else_instrs...break))
 */
void
try_add_loop_terminator(loop_variable_state *ls, ir_if *ir)
{
   ir_instruction *inst = (ir_instruction *) ir->then_instructions.get_tail();
   ir_instruction *else_inst =
      (ir_instruction *) ir->else_instructions.get_tail();

   if (is_break(inst) || is_break(else_inst))
      ls->insert(ir, is_break(else_inst));
}


loop_state *
analyze_loop_variables(exec_list *instructions)
{
   loop_state *loops = new loop_state;
   loop_analysis v(loops);

   v.run(instructions);
   return v.loops;
}
