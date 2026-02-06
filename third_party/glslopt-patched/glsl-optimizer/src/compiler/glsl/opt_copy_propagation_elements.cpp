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
 * \file opt_copy_propagation_elements.cpp
 *
 * Replaces usage of recently-copied components of variables with the
 * previous copy of the variable.
 *
 * This should reduce the number of MOV instructions in the generated
 * programs and help triggering other optimizations that live in GLSL
 * level.
 */

#include "ir.h"
#include "ir_rvalue_visitor.h"
#include "ir_basic_block.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "util/hash_table.h"
#include "util/set.h"

static bool debug = false;

namespace {

class acp_entry
{
public:
   DECLARE_LINEAR_ZALLOC_CXX_OPERATORS(acp_entry)

   /* If set, rhs_full indicates that this ACP entry represents a
    * whole-variable copy.  The rhs_element[] array will still be filled,
    * to allow the swizzling from its components in case the variable
    * was a vector (and to simplify some of the erase() and write_vector()
    * logic).
    */

   ir_variable *rhs_full;
   ir_variable *rhs_element[4];
   unsigned rhs_channel[4];

   /* Set of variables that use the variable associated with this acp_entry as
    * RHS.  This has the "reverse references" of rhs_full/rhs_element.  It is
    * used to speed up invalidating those references when the acp_entry
    * changes.
    */
   set *dsts;
};

class copy_propagation_state {
public:
   DECLARE_RZALLOC_CXX_OPERATORS(copy_propagation_state);

   static
   copy_propagation_state* create(void *mem_ctx)
   {
      return new (mem_ctx) copy_propagation_state(NULL);
   }

   copy_propagation_state* clone()
   {
      return new (ralloc_parent(this)) copy_propagation_state(this);
   }

   void erase_all()
   {
      /* Individual elements were allocated from a linear allocator, so will
       * be destroyed when the state is destroyed.
       */
      _mesa_hash_table_clear(acp, NULL);
      fallback = NULL;
   }

   void erase(ir_variable *var, unsigned write_mask)
   {
      acp_entry *entry = pull_acp(var);
      entry->rhs_full = NULL;

      for (int i = 0; i < 4; i++) {
         if (!entry->rhs_element[i])
            continue;
         if ((write_mask & (1 << i)) == 0)
            continue;

         ir_variable *to_remove = entry->rhs_element[i];
         entry->rhs_element[i] = NULL;
         remove_unused_var_from_dsts(entry, var, to_remove);
      }

      /* TODO: Check write mask, and possibly not clear everything. */

      /* For any usage of our variable on the RHS, clear it out. */
      set_foreach(entry->dsts, set_entry) {
         ir_variable *dst_var = (ir_variable *)set_entry->key;
         acp_entry *dst_entry = pull_acp(dst_var);
         for (int i = 0; i < 4; i++) {
            if (dst_entry->rhs_element[i] == var)
               dst_entry->rhs_element[i] = NULL;
         }
         if (dst_entry->rhs_full == var)
            dst_entry->rhs_full = NULL;
         _mesa_set_remove(entry->dsts, set_entry);
      }
   }

   acp_entry *read(ir_variable *var)
   {
      for (copy_propagation_state *s = this; s != NULL; s = s->fallback) {
         hash_entry *ht_entry = _mesa_hash_table_search(s->acp, var);
         if (ht_entry)
            return (acp_entry *) ht_entry->data;
      }
      return NULL;
   }

   void write_elements(ir_variable *lhs, ir_variable *rhs, unsigned write_mask, int swizzle[4])
   {
      acp_entry *lhs_entry = pull_acp(lhs);
      lhs_entry->rhs_full = NULL;

      for (int i = 0; i < 4; i++) {
         if ((write_mask & (1 << i)) == 0)
            continue;
         ir_variable *to_remove = lhs_entry->rhs_element[i];
         lhs_entry->rhs_element[i] = rhs;
         lhs_entry->rhs_channel[i] = swizzle[i];

         remove_unused_var_from_dsts(lhs_entry, lhs, to_remove);
      }

      acp_entry *rhs_entry = pull_acp(rhs);
      _mesa_set_add(rhs_entry->dsts, lhs);
   }

   void write_full(ir_variable *lhs, ir_variable *rhs)
   {
      acp_entry *lhs_entry = pull_acp(lhs);
      if (lhs_entry->rhs_full == rhs)
         return;

      if (lhs_entry->rhs_full) {
         remove_from_dsts(lhs_entry->rhs_full, lhs);
      } else if (lhs->type->is_vector()) {
         for (int i = 0; i < 4; i++) {
            if (lhs_entry->rhs_element[i])
               remove_from_dsts(lhs_entry->rhs_element[i], lhs);
         }
      }

      lhs_entry->rhs_full = rhs;
      acp_entry *rhs_entry = pull_acp(rhs);
      _mesa_set_add(rhs_entry->dsts, lhs);

      if (lhs->type->is_vector()) {
         for (int i = 0; i < 4; i++) {
            lhs_entry->rhs_element[i] = rhs;
            lhs_entry->rhs_channel[i] = i;
         }
      }
   }

   void remove_unused_var_from_dsts(acp_entry *lhs_entry, ir_variable *lhs, ir_variable *var)
   {
      if (!var)
         return;

      /* If lhs still uses var, don't remove anything. */
      for (int j = 0; j < 4; j++) {
         if (lhs_entry->rhs_element[j] == var)
            return;
      }

      acp_entry *element = pull_acp(var);
      assert(element);
      _mesa_set_remove_key(element->dsts, lhs);
   }

private:
   explicit copy_propagation_state(copy_propagation_state *fallback)
   {
      this->fallback = fallback;
      /* Use 'this' as context for the table, no explicit destruction
       * needed later.
       */
      acp = _mesa_pointer_hash_table_create(this);
      lin_ctx = linear_alloc_parent(this, 0);
   }

   acp_entry *pull_acp(ir_variable *var)
   {
      hash_entry *ht_entry = _mesa_hash_table_search(acp, var);
      if (ht_entry)
         return (acp_entry *) ht_entry->data;

      /* If not found, create one and copy data from fallback if available. */
      acp_entry *entry = new(lin_ctx) acp_entry();
      _mesa_hash_table_insert(acp, var, entry);

      bool found = false;
      for (copy_propagation_state *s = fallback; s != NULL; s = s->fallback) {
         hash_entry *fallback_ht_entry = _mesa_hash_table_search(s->acp, var);
         if (fallback_ht_entry) {
            acp_entry *fallback_entry = (acp_entry *) fallback_ht_entry->data;
            *entry = *fallback_entry;
            entry->dsts = _mesa_set_clone(fallback_entry->dsts, this);
            found = true;
            break;
         }
      }

      if (!found) {
         entry->dsts = _mesa_pointer_set_create(this);
      }

      return entry;
   }

   void
   remove_from_dsts(ir_variable *var, ir_variable *to_remove)
   {
      acp_entry *entry = pull_acp(var);
      assert(entry);
      _mesa_set_remove_key(entry->dsts, to_remove);
   }

   /** Available Copy to Propagate table, from variable to the entry
    *  containing the current sources that can be used. */
   hash_table *acp;

   /** When a state is cloned, entries are copied on demand from fallback. */
   copy_propagation_state *fallback;

   void *lin_ctx;
};

class kill_entry : public exec_node
{
public:
   /* override operator new from exec_node */
   DECLARE_LINEAR_ZALLOC_CXX_OPERATORS(kill_entry)

   kill_entry(ir_variable *var, int write_mask)
   {
      this->var = var;
      this->write_mask = write_mask;
   }

   ir_variable *var;
   unsigned int write_mask;
};

class ir_copy_propagation_elements_visitor : public ir_rvalue_visitor {
public:
   ir_copy_propagation_elements_visitor()
   {
      this->progress = false;
      this->killed_all = false;
      this->mem_ctx = ralloc_context(NULL);
      this->lin_ctx = linear_alloc_parent(this->mem_ctx, 0);
      this->shader_mem_ctx = NULL;
      this->kills = new(mem_ctx) exec_list;
      this->state = copy_propagation_state::create(mem_ctx);
   }
   ~ir_copy_propagation_elements_visitor()
   {
      ralloc_free(mem_ctx);
   }

   virtual ir_visitor_status visit(ir_dereference_variable *);

   void handle_loop(ir_loop *, bool keep_acp);
   virtual ir_visitor_status visit_enter(class ir_loop *);
   virtual ir_visitor_status visit_enter(class ir_function_signature *);
   virtual ir_visitor_status visit_leave(class ir_assignment *);
   virtual ir_visitor_status visit_enter(class ir_call *);
   virtual ir_visitor_status visit_enter(class ir_if *);
   virtual ir_visitor_status visit_leave(class ir_swizzle *);

   void handle_rvalue(ir_rvalue **rvalue);

   void add_copy(ir_assignment *ir);
   void kill(kill_entry *k);
   void handle_if_block(exec_list *instructions, exec_list *kills, bool *killed_all);

   copy_propagation_state *state;

   /**
    * List of kill_entry: The variables whose values were killed in this
    * block.
    */
   exec_list *kills;

   bool progress;

   bool killed_all;

   /* Context for our local data structures. */
   void *mem_ctx;
   void *lin_ctx;
   /* Context for allocating new shader nodes. */
   void *shader_mem_ctx;
};

} /* unnamed namespace */

ir_visitor_status
ir_copy_propagation_elements_visitor::visit(ir_dereference_variable *ir)
{
   if (this->in_assignee)
      return visit_continue;

   const acp_entry *entry = state->read(ir->var);
   if (entry && entry->rhs_full) {
      ir->var = (ir_variable *) entry->rhs_full;
      progress = true;
   }

   return visit_continue;
}

ir_visitor_status
ir_copy_propagation_elements_visitor::visit_enter(ir_function_signature *ir)
{
   /* Treat entry into a function signature as a completely separate
    * block.  Any instructions at global scope will be shuffled into
    * main() at link time, so they're irrelevant to us.
    */
   exec_list *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->kills = new(mem_ctx) exec_list;
   this->killed_all = false;

   copy_propagation_state *orig_state = state;
   this->state = copy_propagation_state::create(mem_ctx);

   visit_list_elements(this, &ir->body);

   delete this->state;
   this->state = orig_state;

   ralloc_free(this->kills);
   this->kills = orig_kills;
   this->killed_all = orig_killed_all;

   return visit_continue_with_parent;
}

ir_visitor_status
ir_copy_propagation_elements_visitor::visit_leave(ir_assignment *ir)
{
   ir_dereference_variable *lhs = ir->lhs->as_dereference_variable();
   ir_variable *var = ir->lhs->variable_referenced();

   kill_entry *k;

   if (lhs && var->type->is_vector())
      k = new(this->lin_ctx) kill_entry(var, ir->write_mask);
   else
      k = new(this->lin_ctx) kill_entry(var, ~0);

   kill(k);

   add_copy(ir);

   return visit_continue;
}

ir_visitor_status
ir_copy_propagation_elements_visitor::visit_leave(ir_swizzle *)
{
   /* Don't visit the values of swizzles since they are handled while
    * visiting the swizzle itself.
    */
   return visit_continue;
}

/**
 * Replaces dereferences of ACP RHS variables with ACP LHS variables.
 *
 * This is where the actual copy propagation occurs.  Note that the
 * rewriting of ir_dereference means that the ir_dereference instance
 * must not be shared by multiple IR operations!
 */
void
ir_copy_propagation_elements_visitor::handle_rvalue(ir_rvalue **ir)
{
   int swizzle_chan[4];
   ir_dereference_variable *deref_var;
   ir_variable *source[4] = {NULL, NULL, NULL, NULL};
   int source_chan[4] = {0, 0, 0, 0};
   int chans;
   bool noop_swizzle = true;

   if (!*ir)
      return;

   ir_swizzle *swizzle = (*ir)->as_swizzle();
   if (swizzle) {
      deref_var = swizzle->val->as_dereference_variable();
      if (!deref_var)
	 return;

      swizzle_chan[0] = swizzle->mask.x;
      swizzle_chan[1] = swizzle->mask.y;
      swizzle_chan[2] = swizzle->mask.z;
      swizzle_chan[3] = swizzle->mask.w;
      chans = swizzle->type->vector_elements;
   } else {
      deref_var = (*ir)->as_dereference_variable();
      if (!deref_var)
	 return;

      swizzle_chan[0] = 0;
      swizzle_chan[1] = 1;
      swizzle_chan[2] = 2;
      swizzle_chan[3] = 3;
      chans = deref_var->type->vector_elements;
   }

   if (this->in_assignee)
      return;

   ir_variable *var = deref_var->var;

   /* Try to find ACP entries covering swizzle_chan[], hoping they're
    * the same source variable.
    */

   const acp_entry *entry = state->read(var);
   if (entry) {
      for (int c = 0; c < chans; c++) {
         unsigned index = swizzle_chan[c];
         ir_variable *src = entry->rhs_element[index];
         if (!src)
            continue;
         source[c] = src;
         source_chan[c] = entry->rhs_channel[index];
         if (source_chan[c] != swizzle_chan[c])
            noop_swizzle = false;
      }
   }

   /* Make sure all channels are copying from the same source variable. */
   if (!source[0])
      return;
   for (int c = 1; c < chans; c++) {
      if (source[c] != source[0])
	 return;
   }

   if (!shader_mem_ctx)
      shader_mem_ctx = ralloc_parent(deref_var);

   /* Don't pointlessly replace the rvalue with itself (or a noop swizzle
    * of itself, which would just be deleted by opt_noop_swizzle).
    */
   if (source[0] == var && noop_swizzle)
      return;

   if (debug) {
      printf("Copy propagation from:\n");
      (*ir)->print();
   }

   deref_var = new(shader_mem_ctx) ir_dereference_variable(source[0]);
   *ir = new(shader_mem_ctx) ir_swizzle(deref_var,
					source_chan[0],
					source_chan[1],
					source_chan[2],
					source_chan[3],
					chans);
   progress = true;

   if (debug) {
      printf("to:\n");
      (*ir)->print();
      printf("\n");
   }
}


ir_visitor_status
ir_copy_propagation_elements_visitor::visit_enter(ir_call *ir)
{
   /* Do copy propagation on call parameters, but skip any out params */
   foreach_two_lists(formal_node, &ir->callee->parameters,
                     actual_node, &ir->actual_parameters) {
      ir_variable *sig_param = (ir_variable *) formal_node;
      ir_rvalue *ir = (ir_rvalue *) actual_node;
      if (sig_param->data.mode != ir_var_function_out
          && sig_param->data.mode != ir_var_function_inout) {
         ir->accept(this);
      }
   }

   if (!ir->callee->is_intrinsic()) {
      state->erase_all();
      this->killed_all = true;
   } else {
      if (ir->return_deref) {
         kill(new(this->lin_ctx) kill_entry(ir->return_deref->var, ~0));
      }

      foreach_two_lists(formal_node, &ir->callee->parameters,
                        actual_node, &ir->actual_parameters) {
         ir_variable *sig_param = (ir_variable *) formal_node;
         if (sig_param->data.mode == ir_var_function_out ||
             sig_param->data.mode == ir_var_function_inout) {
            ir_rvalue *ir = (ir_rvalue *) actual_node;
            ir_variable *var = ir->variable_referenced();
            kill(new(this->lin_ctx) kill_entry(var, ~0));
         }
      }
   }

   return visit_continue_with_parent;
}

void
ir_copy_propagation_elements_visitor::handle_if_block(exec_list *instructions, exec_list *kills, bool *killed_all)
{
   exec_list *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->kills = kills;
   this->killed_all = false;

   /* Populate the initial acp with a copy of the original */
   copy_propagation_state *orig_state = state;
   this->state = orig_state->clone();

   visit_list_elements(this, instructions);

   delete this->state;
   this->state = orig_state;

   *killed_all = this->killed_all;
   this->kills = orig_kills;
   this->killed_all = orig_killed_all;
}

ir_visitor_status
ir_copy_propagation_elements_visitor::visit_enter(ir_if *ir)
{
   ir->condition->accept(this);

   exec_list *new_kills = new(mem_ctx) exec_list;
   bool then_killed_all = false;
   bool else_killed_all = false;

   handle_if_block(&ir->then_instructions, new_kills, &then_killed_all);
   handle_if_block(&ir->else_instructions, new_kills, &else_killed_all);

   if (then_killed_all || else_killed_all) {
      state->erase_all();
      killed_all = true;
   } else {
      foreach_in_list_safe(kill_entry, k, new_kills)
         kill(k);
   }

   ralloc_free(new_kills);

   /* handle_if_block() already descended into the children. */
   return visit_continue_with_parent;
}

void
ir_copy_propagation_elements_visitor::handle_loop(ir_loop *ir, bool keep_acp)
{
   exec_list *orig_kills = this->kills;
   bool orig_killed_all = this->killed_all;

   this->kills = new(mem_ctx) exec_list;
   this->killed_all = false;

   copy_propagation_state *orig_state = state;

   if (keep_acp) {
      /* Populate the initial acp with a copy of the original */
      this->state = orig_state->clone();
   } else {
      this->state = copy_propagation_state::create(mem_ctx);
   }

   visit_list_elements(this, &ir->body_instructions);

   delete this->state;
   this->state = orig_state;

   if (this->killed_all)
      this->state->erase_all();

   exec_list *new_kills = this->kills;
   this->kills = orig_kills;
   this->killed_all = this->killed_all || orig_killed_all;

   foreach_in_list_safe(kill_entry, k, new_kills) {
      kill(k);
   }

   ralloc_free(new_kills);
}

ir_visitor_status
ir_copy_propagation_elements_visitor::visit_enter(ir_loop *ir)
{
   handle_loop(ir, false);
   handle_loop(ir, true);

   /* already descended into the children. */
   return visit_continue_with_parent;
}

/* Remove any entries currently in the ACP for this kill. */
void
ir_copy_propagation_elements_visitor::kill(kill_entry *k)
{
   state->erase(k->var, k->write_mask);

   /* If we were on a list, remove ourselves before inserting */
   if (k->next)
      k->remove();

   this->kills->push_tail(k);
}

/**
 * Adds directly-copied channels between vector variables to the available
 * copy propagation list.
 */
void
ir_copy_propagation_elements_visitor::add_copy(ir_assignment *ir)
{
   if (ir->condition)
      return;

   {
      ir_variable *lhs_var = ir->whole_variable_written();
      ir_dereference_variable *rhs = ir->rhs->as_dereference_variable();

      if (lhs_var != NULL && rhs && rhs->var != NULL && lhs_var != rhs->var) {
         if (lhs_var->data.mode == ir_var_shader_storage ||
             lhs_var->data.mode == ir_var_shader_shared ||
             rhs->var->data.mode == ir_var_shader_storage ||
             rhs->var->data.mode == ir_var_shader_shared ||
             lhs_var->data.precise != rhs->var->data.precise) {
            return;
         }
         state->write_full(lhs_var, rhs->var);
         return;
      }
   }

   int orig_swizzle[4] = {0, 1, 2, 3};
   int swizzle[4];

   ir_dereference_variable *lhs = ir->lhs->as_dereference_variable();
   if (!lhs || !(lhs->type->is_scalar() || lhs->type->is_vector()))
      return;

   if (lhs->var->data.mode == ir_var_shader_storage ||
       lhs->var->data.mode == ir_var_shader_shared)
      return;

   ir_dereference_variable *rhs = ir->rhs->as_dereference_variable();
   if (!rhs) {
      ir_swizzle *swiz = ir->rhs->as_swizzle();
      if (!swiz)
	 return;

      rhs = swiz->val->as_dereference_variable();
      if (!rhs)
	 return;

      orig_swizzle[0] = swiz->mask.x;
      orig_swizzle[1] = swiz->mask.y;
      orig_swizzle[2] = swiz->mask.z;
      orig_swizzle[3] = swiz->mask.w;
   }

   if (rhs->var->data.mode == ir_var_shader_storage ||
       rhs->var->data.mode == ir_var_shader_shared)
      return;

   /* Move the swizzle channels out to the positions they match in the
    * destination.  We don't want to have to rewrite the swizzle[]
    * array every time we clear a bit of the write_mask.
    */
   int j = 0;
   for (int i = 0; i < 4; i++) {
      if (ir->write_mask & (1 << i))
	 swizzle[i] = orig_swizzle[j++];
   }

   int write_mask = ir->write_mask;
   if (lhs->var == rhs->var) {
      /* If this is a copy from the variable to itself, then we need
       * to be sure not to include the updated channels from this
       * instruction in the set of new source channels to be
       * copy-propagated from.
       */
      for (int i = 0; i < 4; i++) {
	 if (ir->write_mask & (1 << orig_swizzle[i]))
	    write_mask &= ~(1 << i);
      }
   }

   if (lhs->var->data.precise != rhs->var->data.precise)
      return;

   state->write_elements(lhs->var, rhs->var, write_mask, swizzle);
}

bool
do_copy_propagation_elements(exec_list *instructions)
{
   ir_copy_propagation_elements_visitor v;

   visit_list_elements(&v, instructions);

   return v.progress;
}
