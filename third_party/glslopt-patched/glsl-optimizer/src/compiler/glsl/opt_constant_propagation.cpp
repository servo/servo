/*
 * Copyright © 2010 Intel Corporation
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * constant of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, constant, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above constantright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR CONSTANTRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

/**
 * \file opt_constant_propagation.cpp
 *
 * Tracks assignments of constants to channels of variables, and
 * usage of those constant channels with direct usage of the constants.
 *
 * This can lead to constant folding and algebraic optimizations in
 * those later expressions, while causing no increase in instruction
 * count (due to constants being generally free to load from a
 * constant push buffer or as instruction immediate values) and
 * possibly reducing register pressure.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "ir_basic_block.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "util/hash_table.h"

namespace {

class acp_entry : public exec_node
{
public:
   /* override operator new from exec_node */
   DECLARE_LINEAR_ZALLOC_CXX_OPERATORS(acp_entry)

   acp_entry(ir_variable *var, unsigned write_mask, ir_constant *constant)
   {
      assert(var);
      assert(constant);
      this->var = var;
      this->write_mask = write_mask;
      this->constant = constant;
      this->initial_values = write_mask;
   }

   acp_entry(const acp_entry *src)
   {
      this->var = src->var;
      this->write_mask = src->write_mask;
      this->constant = src->constant;
      this->initial_values = src->initial_values;
   }

   ir_variable *var;
   ir_constant *constant;
   unsigned write_mask;

   /** Mask of values initially available in the constant. */
   unsigned initial_values;
};


class ir_constant_propagation_visitor : public ir_rvalue_visitor {
public:
   ir_constant_propagation_visitor()
   {
      progress = false;
      killed_all = false;
      mem_ctx = ralloc_context(0);
      this->lin_ctx = linear_alloc_parent(this->mem_ctx, 0);
      this->acp = new(mem_ctx) exec_list;
      this->kills = _mesa_pointer_hash_table_create(mem_ctx);
   }
   ~ir_constant_propagation_visitor()
   {
      ralloc_free(mem_ctx);
   }

   virtual ir_visitor_status visit_enter(class ir_loop *);
   virtual ir_visitor_status visit_enter(class ir_function_signature *);
   virtual ir_visitor_status visit_enter(class ir_function *);
   virtual ir_visitor_status visit_leave(class ir_assignment *);
   virtual ir_visitor_status visit_enter(class ir_call *);
   virtual ir_visitor_status visit_enter(class ir_if *);

   void add_constant(ir_assignment *ir);
   void constant_folding(ir_rvalue **rvalue);
   void constant_propagation(ir_rvalue **rvalue);
   void kill(ir_variable *ir, unsigned write_mask);
   void handle_if_block(exec_list *instructions, hash_table *kills, bool *killed_all);
   void handle_loop(class ir_loop *, bool keep_acp);
   void handle_rvalue(ir_rvalue **rvalue);

   /** List of acp_entry: The available constants to propagate */
   exec_list *acp;

   /**
    * Hash table of killed entries: maps variables to the mask of killed channels.
    */
   hash_table *kills;

   bool progress;

   bool killed_all;

   void *mem_ctx;
   void *lin_ctx;
};


void
ir_constant_propagation_visitor::constant_folding(ir_rvalue **rvalue)
{
   if (this->in_assignee || *rvalue == NULL)
      return;

   if (ir_constant_fold(rvalue))
      this->progress = true;

   ir_dereference_variable *var_ref = (*rvalue)->as_dereference_variable();
   if (var_ref && !var_ref->type->is_array()) {
      ir_constant *constant =
         var_ref->constant_expression_value(ralloc_parent(var_ref));
      if (constant) {
         *rvalue = constant;
         this->progress = true;
      }
   }
}

void
ir_constant_propagation_visitor::constant_propagation(ir_rvalue **rvalue) {

   if (this->in_assignee || !*rvalue)
      return;

   const glsl_type *type = (*rvalue)->type;
   if (!type->is_scalar() && !type->is_vector())
      return;

   ir_swizzle *swiz = NULL;
   ir_dereference_variable *deref = (*rvalue)->as_dereference_variable();
   if (!deref) {
      swiz = (*rvalue)->as_swizzle();
      if (!swiz)
	 return;

      deref = swiz->val->as_dereference_variable();
      if (!deref)
	 return;
   }

   ir_constant_data data;
   memset(&data, 0, sizeof(data));

   for (unsigned int i = 0; i < type->components(); i++) {
      int channel;
      acp_entry *found = NULL;

      if (swiz) {
	 switch (i) {
	 case 0: channel = swiz->mask.x; break;
	 case 1: channel = swiz->mask.y; break;
	 case 2: channel = swiz->mask.z; break;
	 case 3: channel = swiz->mask.w; break;
	 default: assert(!"shouldn't be reached"); channel = 0; break;
	 }
      } else {
	 channel = i;
      }

      foreach_in_list(acp_entry, entry, this->acp) {
	 if (entry->var == deref->var && entry->write_mask & (1 << channel)) {
	    found = entry;
	    break;
	 }
      }

      if (!found)
	 return;

      int rhs_channel = 0;
      for (int j = 0; j < 4; j++) {
	 if (j == channel)
	    break;
	 if (found->initial_values & (1 << j))
	    rhs_channel++;
      }

      switch (type->base_type) {
      case GLSL_TYPE_FLOAT:
	 data.f[i] = found->constant->value.f[rhs_channel];
	 break;
      case GLSL_TYPE_FLOAT16:
	 data.f16[i] = found->constant->value.f16[rhs_channel];
	 break;
      case GLSL_TYPE_DOUBLE:
	 data.d[i] = found->constant->value.d[rhs_channel];
	 break;
      case GLSL_TYPE_INT:
	 data.i[i] = found->constant->value.i[rhs_channel];
	 break;
      case GLSL_TYPE_UINT:
	 data.u[i] = found->constant->value.u[rhs_channel];
	 break;
      case GLSL_TYPE_BOOL:
	 data.b[i] = found->constant->value.b[rhs_channel];
	 break;
      case GLSL_TYPE_UINT64:
	 data.u64[i] = found->constant->value.u64[rhs_channel];
	 break;
      case GLSL_TYPE_INT64:
	 data.i64[i] = found->constant->value.i64[rhs_channel];
	 break;
      default:
	 assert(!"not reached");
	 break;
      }
   }

   *rvalue = new(ralloc_parent(deref)) ir_constant(type, &data);
   this->progress = true;
}

void
ir_constant_propagation_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   constant_propagation(rvalue);
   constant_folding(rvalue);
}

ir_visitor_status
ir_constant_propagation_visitor::visit_enter(ir_function_signature *ir)
{
   /* Treat entry into a function signature as a completely separate
    * block.  Any instructions at global scope will be shuffled into
    * main() at link time, so they're irrelevant to us.
    */
   exec_list *orig_acp = this->acp;
   hash_table *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->acp = new(mem_ctx) exec_list;
   this->kills = _mesa_pointer_hash_table_create(mem_ctx);
   this->killed_all = false;

   visit_list_elements(this, &ir->body);

   this->kills = orig_kills;
   this->acp = orig_acp;
   this->killed_all = orig_killed_all;

   return visit_continue_with_parent;
}

ir_visitor_status
ir_constant_propagation_visitor::visit_leave(ir_assignment *ir)
{
  constant_folding(&ir->rhs);

   if (this->in_assignee)
      return visit_continue;

   unsigned kill_mask = ir->write_mask;
   if (ir->lhs->as_dereference_array()) {
      /* The LHS of the assignment uses an array indexing operator (e.g. v[i]
       * = ...;).  Since we only try to constant propagate vectors and
       * scalars, this means that either (a) array indexing is being used to
       * select a vector component, or (b) the variable in question is neither
       * a scalar or a vector, so we don't care about it.  In the former case,
       * we want to kill the whole vector, since in general we can't predict
       * which vector component will be selected by array indexing.  In the
       * latter case, it doesn't matter what we do, so go ahead and kill the
       * whole variable anyway.
       *
       * Note that if the array index is constant (e.g. v[2] = ...;), we could
       * in principle be smarter, but we don't need to, because a future
       * optimization pass will convert it to a simple assignment with the
       * correct mask.
       */
      kill_mask = ~0;
   }
   kill(ir->lhs->variable_referenced(), kill_mask);

   add_constant(ir);

   return visit_continue;
}

ir_visitor_status
ir_constant_propagation_visitor::visit_enter(ir_function *ir)
{
   (void) ir;
   return visit_continue;
}

ir_visitor_status
ir_constant_propagation_visitor::visit_enter(ir_call *ir)
{
   /* Do constant propagation on call parameters, but skip any out params */
   foreach_two_lists(formal_node, &ir->callee->parameters,
                     actual_node, &ir->actual_parameters) {
      ir_variable *sig_param = (ir_variable *) formal_node;
      ir_rvalue *param = (ir_rvalue *) actual_node;
      if (sig_param->data.mode != ir_var_function_out
          && sig_param->data.mode != ir_var_function_inout) {
	 ir_rvalue *new_param = param;
	 handle_rvalue(&new_param);
         if (new_param != param)
	    param->replace_with(new_param);
	 else
	    param->accept(this);
      }
   }

   /* Since we're unlinked, we don't (necssarily) know the side effects of
    * this call.  So kill all copies.
    */
   acp->make_empty();
   this->killed_all = true;

   return visit_continue_with_parent;
}

void
ir_constant_propagation_visitor::handle_if_block(exec_list *instructions, hash_table *kills, bool *killed_all)
{
   exec_list *orig_acp = this->acp;
   hash_table *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->acp = new(mem_ctx) exec_list;
   this->kills = kills;
   this->killed_all = false;

   /* Populate the initial acp with a constant of the original */
   foreach_in_list(acp_entry, a, orig_acp) {
      this->acp->push_tail(new(this->lin_ctx) acp_entry(a));
   }

   visit_list_elements(this, instructions);

   *killed_all = this->killed_all;
   this->kills = orig_kills;
   this->acp = orig_acp;
   this->killed_all = orig_killed_all;
}

ir_visitor_status
ir_constant_propagation_visitor::visit_enter(ir_if *ir)
{
   ir->condition->accept(this);
   handle_rvalue(&ir->condition);

   hash_table *new_kills = _mesa_pointer_hash_table_create(mem_ctx);
   bool then_killed_all = false;
   bool else_killed_all = false;

   handle_if_block(&ir->then_instructions, new_kills, &then_killed_all);
   handle_if_block(&ir->else_instructions, new_kills, &else_killed_all);

   if (then_killed_all || else_killed_all) {
      acp->make_empty();
      killed_all = true;
   } else {
      hash_table_foreach(new_kills, htk)
         kill((ir_variable *) htk->key, (uintptr_t) htk->data);
   }

   _mesa_hash_table_destroy(new_kills, NULL);

   /* handle_if_block() already descended into the children. */
   return visit_continue_with_parent;
}

void
ir_constant_propagation_visitor::handle_loop(ir_loop *ir, bool keep_acp)
{
   exec_list *orig_acp = this->acp;
   hash_table *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->acp = new(mem_ctx) exec_list;
   this->kills = _mesa_pointer_hash_table_create(mem_ctx);
   this->killed_all = false;

   if (keep_acp) {
      foreach_in_list(acp_entry, a, orig_acp) {
         this->acp->push_tail(new(this->lin_ctx) acp_entry(a));
      }
   }

   visit_list_elements(this, &ir->body_instructions);

   if (this->killed_all) {
      orig_acp->make_empty();
   }

   hash_table *new_kills = this->kills;
   this->kills = orig_kills;
   this->acp = orig_acp;
   this->killed_all = this->killed_all || orig_killed_all;

   hash_table_foreach(new_kills, htk) {
      kill((ir_variable *) htk->key, (uintptr_t) htk->data);
   }
}

ir_visitor_status
ir_constant_propagation_visitor::visit_enter(ir_loop *ir)
{
   /* Make a conservative first pass over the loop with an empty ACP set.
    * This also removes any killed entries from the original ACP set.
    */
   handle_loop(ir, false);

   /* Then, run it again with the real ACP set, minus any killed entries.
    * This takes care of propagating values from before the loop into it.
    */
   handle_loop(ir, true);

   /* already descended into the children. */
   return visit_continue_with_parent;
}

void
ir_constant_propagation_visitor::kill(ir_variable *var, unsigned write_mask)
{
   assert(var != NULL);

   /* We don't track non-vectors. */
   if (!var->type->is_vector() && !var->type->is_scalar())
      return;

   /* Remove any entries currently in the ACP for this kill. */
   foreach_in_list_safe(acp_entry, entry, this->acp) {
      if (entry->var == var) {
	 entry->write_mask &= ~write_mask;
	 if (entry->write_mask == 0)
	    entry->remove();
      }
   }

   /* Add this writemask of the variable to the hash table of killed
    * variables in this block.
    */
   hash_entry *kill_hash_entry = _mesa_hash_table_search(this->kills, var);
   if (kill_hash_entry) {
      uintptr_t new_write_mask = ((uintptr_t) kill_hash_entry->data) | write_mask;
      kill_hash_entry->data = (void *) new_write_mask;
      return;
   }
   /* Not already in the hash table.  Make new entry. */
   _mesa_hash_table_insert(this->kills, var, (void *) uintptr_t(write_mask));
}

/**
 * Adds an entry to the available constant list if it's a plain assignment
 * of a variable to a variable.
 */
void
ir_constant_propagation_visitor::add_constant(ir_assignment *ir)
{
   acp_entry *entry;

   if (ir->condition)
      return;

   if (!ir->write_mask)
      return;

   ir_dereference_variable *deref = ir->lhs->as_dereference_variable();
   ir_constant *constant = ir->rhs->as_constant();

   if (!deref || !constant)
      return;

   /* Only do constant propagation on vectors.  Constant matrices,
    * arrays, or structures would require more work elsewhere.
    */
   if (!deref->var->type->is_vector() && !deref->var->type->is_scalar())
      return;

   /* We can't do copy propagation on buffer variables, since the underlying
    * memory storage is shared across multiple threads we can't be sure that
    * the variable value isn't modified between this assignment and the next
    * instruction where its value is read.
    */
   if (deref->var->data.mode == ir_var_shader_storage ||
       deref->var->data.mode == ir_var_shader_shared)
      return;

   entry = new(this->lin_ctx) acp_entry(deref->var, ir->write_mask, constant);
   this->acp->push_tail(entry);
}

} /* unnamed namespace */

/**
 * Does a constant propagation pass on the code present in the instruction stream.
 */
bool
do_constant_propagation(exec_list *instructions)
{
   ir_constant_propagation_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}
