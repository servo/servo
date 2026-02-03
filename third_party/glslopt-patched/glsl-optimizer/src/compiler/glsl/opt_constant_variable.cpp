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
 * \file opt_constant_variable.cpp
 *
 * Marks variables assigned a single constant value over the course
 * of the program as constant.
 *
 * The goal here is to trigger further constant folding and then dead
 * code elimination.  This is common with vector/matrix constructors
 * and calls to builtin functions.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "util/hash_table.h"

namespace {

struct assignment_entry {
   int assignment_count;
   ir_variable *var;
   ir_constant *constval;
   bool our_scope;
};

class ir_constant_variable_visitor : public ir_hierarchical_visitor {
public:
   using ir_hierarchical_visitor::visit;
   using ir_hierarchical_visitor::visit_enter;

   virtual ir_visitor_status visit_enter(ir_dereference_variable *);
   virtual ir_visitor_status visit(ir_variable *);
   virtual ir_visitor_status visit_enter(ir_assignment *);
   virtual ir_visitor_status visit_enter(ir_call *);

   struct hash_table *ht;
};

} /* unnamed namespace */

static struct assignment_entry *
get_assignment_entry(ir_variable *var, struct hash_table *ht)
{
   struct hash_entry *hte = _mesa_hash_table_search(ht, var);
   struct assignment_entry *entry;

   if (hte) {
      entry = (struct assignment_entry *) hte->data;
   } else {
      entry = (struct assignment_entry *) calloc(1, sizeof(*entry));
      entry->var = var;
      _mesa_hash_table_insert(ht, var, entry);
   }

   return entry;
}

ir_visitor_status
ir_constant_variable_visitor::visit(ir_variable *ir)
{
   struct assignment_entry *entry = get_assignment_entry(ir, this->ht);
   entry->our_scope = true;
   return visit_continue;
}

/* Skip derefs of variables so that we can detect declarations. */
ir_visitor_status
ir_constant_variable_visitor::visit_enter(ir_dereference_variable *ir)
{
   (void)ir;
   return visit_continue_with_parent;
}

ir_visitor_status
ir_constant_variable_visitor::visit_enter(ir_assignment *ir)
{
   ir_constant *constval;
   struct assignment_entry *entry;

   entry = get_assignment_entry(ir->lhs->variable_referenced(), this->ht);
   assert(entry);
   entry->assignment_count++;

   /* If there's more than one assignment, don't bother - we won't do anything
    * with this variable anyway, and continuing just wastes memory cloning
    * constant expressions.
    */
   if (entry->assignment_count > 1)
      return visit_continue;

   /* If it's already constant, don't do the work. */
   if (entry->var->constant_value)
      return visit_continue;

   /* OK, now find if we actually have all the right conditions for
    * this to be a constant value assigned to the var.
    */
   if (ir->condition)
      return visit_continue;

   ir_variable *var = ir->whole_variable_written();
   if (!var)
      return visit_continue;

   /* Ignore buffer variables, since the underlying storage is shared
    * and we can't be sure that this variable won't be written by another
    * thread.
    */
   if (var->data.mode == ir_var_shader_storage ||
       var->data.mode == ir_var_shader_shared)
      return visit_continue;

   constval = ir->rhs->constant_expression_value(ralloc_parent(ir));
   if (!constval)
      return visit_continue;

   /* Mark this entry as having a constant assignment (if the
    * assignment count doesn't go >1).  do_constant_variable will fix
    * up the variable with the constant value later.
    */
   entry->constval = constval;

   return visit_continue;
}

ir_visitor_status
ir_constant_variable_visitor::visit_enter(ir_call *ir)
{
   /* Mark any out parameters as assigned to */
   foreach_two_lists(formal_node, &ir->callee->parameters,
                     actual_node, &ir->actual_parameters) {
      ir_rvalue *param_rval = (ir_rvalue *) actual_node;
      ir_variable *param = (ir_variable *) formal_node;

      if (param->data.mode == ir_var_function_out ||
	  param->data.mode == ir_var_function_inout) {
	 ir_variable *var = param_rval->variable_referenced();
	 struct assignment_entry *entry;

	 assert(var);
	 entry = get_assignment_entry(var, this->ht);
	 entry->assignment_count++;
      }

      /* We don't know if the variable passed to this function has been
       * assigned a value or if it is undefined, so for now we always assume
       * it has been assigned a value. Once functions have been inlined any
       * further potential optimisations will be taken care of.
       */
      struct assignment_entry *entry;
      entry = get_assignment_entry(param, this->ht);
      entry->assignment_count++;
   }

   /* Mark the return storage as having been assigned to */
   if (ir->return_deref != NULL) {
      ir_variable *var = ir->return_deref->variable_referenced();
      struct assignment_entry *entry;

      assert(var);
      entry = get_assignment_entry(var, this->ht);
      entry->assignment_count++;
   }

   return visit_continue;
}

/**
 * Does a copy propagation pass on the code present in the instruction stream.
 */
bool
do_constant_variable(exec_list *instructions)
{
   bool progress = false;
   ir_constant_variable_visitor v;

   v.ht = _mesa_pointer_hash_table_create(NULL);
   v.run(instructions);

   hash_table_foreach(v.ht, hte) {
      struct assignment_entry *entry = (struct assignment_entry *) hte->data;

      if (entry->assignment_count == 1 && entry->constval && entry->our_scope) {
	 entry->var->constant_value = entry->constval;
	 progress = true;
      }
      hte->data = NULL;
      free(entry);
   }
   _mesa_hash_table_destroy(v.ht, NULL);

   return progress;
}

bool
do_constant_variable_unlinked(exec_list *instructions)
{
   bool progress = false;

   foreach_in_list(ir_instruction, ir, instructions) {
      ir_function *f = ir->as_function();
      if (f) {
	 foreach_in_list(ir_function_signature, sig, &f->signatures) {
	    if (do_constant_variable(&sig->body))
	       progress = true;
	 }
      }
   }

   return progress;
}
