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
 * \file opt_array_splitting.cpp
 *
 * If an array is always dereferenced with a constant index, then
 * split it apart into its elements, making it more amenable to other
 * optimization passes.
 *
 * This skips uniform/varying arrays, which would need careful
 * handling due to their ir->location fields tying them to the GL API
 * and other shader stages.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "compiler/glsl_types.h"

static bool debug = false;

namespace {

namespace opt_array_splitting {

class variable_entry : public exec_node
{
public:
   variable_entry(ir_variable *var)
   {
      this->var = var;
      this->split = true;
      this->declaration = false;
      this->components = NULL;
      this->mem_ctx = NULL;
      if (var->type->is_array())
         this->size = var->type->length;
      else
         this->size = var->type->matrix_columns;
   }

   ir_variable *var; /* The key: the variable's pointer. */
   unsigned size; /* array length or matrix columns */

   /** Whether this array should be split or not. */
   bool split;

   /* If the variable had a decl we can work with in the instruction
    * stream.  We can't do splitting on function arguments, which
    * don't get this variable set.
    */
   bool declaration;

   ir_variable **components;

   /** ralloc_parent(this->var) -- the shader's talloc context. */
   void *mem_ctx;
};

} /* namespace */

using namespace opt_array_splitting;

/**
 * This class does a walk over the tree, coming up with the set of
 * variables that could be split by looking to see if they are arrays
 * that are only ever constant-index dereferenced.
 */
class ir_array_reference_visitor : public ir_hierarchical_visitor {
public:
   ir_array_reference_visitor(void)
   {
      this->mem_ctx = ralloc_context(NULL);
      this->variable_list.make_empty();
      this->in_whole_array_copy = false;
   }

   ~ir_array_reference_visitor(void)
   {
      ralloc_free(mem_ctx);
   }

   bool get_split_list(exec_list *instructions, bool linked);

   virtual ir_visitor_status visit(ir_variable *);
   virtual ir_visitor_status visit(ir_dereference_variable *);
   virtual ir_visitor_status visit_enter(ir_assignment *);
   virtual ir_visitor_status visit_leave(ir_assignment *);
   virtual ir_visitor_status visit_enter(ir_dereference_array *);
   virtual ir_visitor_status visit_enter(ir_function_signature *);

   variable_entry *get_variable_entry(ir_variable *var);

   /* List of variable_entry */
   exec_list variable_list;

   void *mem_ctx;

   bool in_whole_array_copy;
};

} /* namespace */

variable_entry *
ir_array_reference_visitor::get_variable_entry(ir_variable *var)
{
   assert(var);

   if (var->data.mode != ir_var_auto &&
       var->data.mode != ir_var_temporary)
      return NULL;

   if (!(var->type->is_array() || var->type->is_matrix()))
      return NULL;

   /* If the array hasn't been sized yet, we can't split it.  After
    * linking, this should be resolved.
    */
   if (var->type->is_unsized_array())
      return NULL;

   /* FIXME: arrays of arrays are not handled correctly by this pass so we
    * skip it for now. While the pass will create functioning code it actually
    * produces worse code.
    *
    * For example the array:
    *
    *    int[3][2] a;
    *
    * ends up being split up into:
    *
    *    int[3][2] a_0;
    *    int[3][2] a_1;
    *    int[3][2] a_2;
    *
    * And we end up referencing each of these new arrays for example:
    *
    *    a[0][1] will be turned into a_0[0][1]
    *    a[1][0] will be turned into a_1[1][0]
    *    a[2][0] will be turned into a_2[2][0]
    */
   if (var->type->is_array() && var->type->fields.array->is_array())
      return NULL;

   foreach_in_list(variable_entry, entry, &this->variable_list) {
      if (entry->var == var)
         return entry;
   }

   variable_entry *entry = new(mem_ctx) variable_entry(var);
   this->variable_list.push_tail(entry);
   return entry;
}


ir_visitor_status
ir_array_reference_visitor::visit(ir_variable *ir)
{
   variable_entry *entry = this->get_variable_entry(ir);

   if (entry)
      entry->declaration = true;

   return visit_continue;
}

ir_visitor_status
ir_array_reference_visitor::visit_enter(ir_assignment *ir)
{
   in_whole_array_copy =
      ir->lhs->type->is_array() && ir->whole_variable_written();

   return visit_continue;
}

ir_visitor_status
ir_array_reference_visitor::visit_leave(ir_assignment *)
{
   in_whole_array_copy = false;

   return visit_continue;
}

ir_visitor_status
ir_array_reference_visitor::visit(ir_dereference_variable *ir)
{
   variable_entry *entry = this->get_variable_entry(ir->var);

   /* Allow whole-array assignments on the LHS.  We can split those
    * by "unrolling" the assignment into component-wise assignments.
    */
   if (in_assignee && in_whole_array_copy)
      return visit_continue;

   /* If we made it to here without seeing an ir_dereference_array,
    * then the dereference of this array didn't have a constant index
    * (see the visit_continue_with_parent below), so we can't split
    * the variable.
    */
   if (entry)
      entry->split = false;

   return visit_continue;
}

ir_visitor_status
ir_array_reference_visitor::visit_enter(ir_dereference_array *ir)
{
   ir_dereference_variable *deref = ir->array->as_dereference_variable();
   if (!deref)
      return visit_continue;

   variable_entry *entry = this->get_variable_entry(deref->var);

   /* If the access to the array has a variable index, we wouldn't
    * know which split variable this dereference should go to.
    */
   if (!ir->array_index->as_constant()) {
      if (entry)
         entry->split = false;
      /* This variable indexing could come from a different array dereference
       * that also has variable indexing, that is, something like a[b[a[b[0]]]].
       * If we return visit_continue_with_parent here for the first appearence
       * of a, then we can miss that b also has indirect indexing (if this is
       * the only place in the program where such indirect indexing into b
       * happens), so keep going.
       */
      return visit_continue;
   }

   /* If the index is also array dereference, visit index. */
   if (ir->array_index->as_dereference_array())
      visit_enter(ir->array_index->as_dereference_array());

   return visit_continue_with_parent;
}

ir_visitor_status
ir_array_reference_visitor::visit_enter(ir_function_signature *ir)
{
   /* We don't have logic for array-splitting function arguments,
    * so just look at the body instructions and not the parameter
    * declarations.
    */
   visit_list_elements(this, &ir->body);
   return visit_continue_with_parent;
}

bool
ir_array_reference_visitor::get_split_list(exec_list *instructions,
                                           bool linked)
{
   visit_list_elements(this, instructions);

   /* If the shaders aren't linked yet, we can't mess with global
    * declarations, which need to be matched by name across shaders.
    */
   if (!linked) {
      foreach_in_list(ir_instruction, node, instructions) {
         ir_variable *var = node->as_variable();
         if (var) {
            variable_entry *entry = get_variable_entry(var);
            if (entry)
               entry->remove();
         }
      }
   }

   /* Trim out variables we found that we can't split. */
   foreach_in_list_safe(variable_entry, entry, &variable_list) {
      if (debug) {
         printf("array %s@%p: decl %d, split %d\n",
                entry->var->name, (void *) entry->var, entry->declaration,
                entry->split);
      }

      if (!(entry->declaration && entry->split)) {
         entry->remove();
      }
   }

   return !variable_list.is_empty();
}

/**
 * This class rewrites the dereferences of arrays that have been split
 * to use the newly created ir_variables for each component.
 */
class ir_array_splitting_visitor : public ir_rvalue_visitor {
public:
   ir_array_splitting_visitor(exec_list *vars)
   {
      this->variable_list = vars;
   }

   virtual ~ir_array_splitting_visitor()
   {
   }

   virtual ir_visitor_status visit_leave(ir_assignment *);

   void split_deref(ir_dereference **deref);
   void handle_rvalue(ir_rvalue **rvalue);
   variable_entry *get_splitting_entry(ir_variable *var);

   exec_list *variable_list;
};

variable_entry *
ir_array_splitting_visitor::get_splitting_entry(ir_variable *var)
{
   assert(var);

   foreach_in_list(variable_entry, entry, this->variable_list) {
      if (entry->var == var) {
         return entry;
      }
   }

   return NULL;
}

void
ir_array_splitting_visitor::split_deref(ir_dereference **deref)
{
   ir_dereference_array *deref_array = (*deref)->as_dereference_array();
   if (!deref_array)
      return;

   ir_dereference_variable *deref_var = deref_array->array->as_dereference_variable();
   if (!deref_var)
      return;
   ir_variable *var = deref_var->var;

   variable_entry *entry = get_splitting_entry(var);
   if (!entry)
      return;

   ir_constant *constant = deref_array->array_index->as_constant();
   assert(constant);

   if (constant->value.i[0] >= 0 && constant->value.i[0] < (int)entry->size) {
      *deref = new(entry->mem_ctx)
               ir_dereference_variable(entry->components[constant->value.i[0]]);
   } else {
      /* There was a constant array access beyond the end of the
       * array.  This might have happened due to constant folding
       * after the initial parse.  This produces an undefined value,
       * but shouldn't crash.  Just give them an uninitialized
       * variable.
       */
      ir_variable *temp = new(entry->mem_ctx) ir_variable(deref_array->type,
                                                          "undef",
                                                          ir_var_temporary);
      entry->components[0]->insert_before(temp);
      *deref = new(entry->mem_ctx) ir_dereference_variable(temp);
   }
}

void
ir_array_splitting_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_dereference *deref = (*rvalue)->as_dereference();

   if (!deref)
      return;

   split_deref(&deref);
   *rvalue = deref;
}

ir_visitor_status
ir_array_splitting_visitor::visit_leave(ir_assignment *ir)
{
   /* The normal rvalue visitor skips the LHS of assignments, but we
    * need to process those just the same.
    */
   ir_rvalue *lhs = ir->lhs;

   /* "Unroll" any whole array assignments, creating assignments for
    * each array element.  Then, do splitting on each new assignment.
    */
   if (lhs->type->is_array() && ir->whole_variable_written() &&
       get_splitting_entry(ir->whole_variable_written())) {
      void *mem_ctx = ralloc_parent(ir);

      for (unsigned i = 0; i < lhs->type->length; i++) {
         ir_rvalue *lhs_i =
            new(mem_ctx) ir_dereference_array(ir->lhs->clone(mem_ctx, NULL),
                                              new(mem_ctx) ir_constant(i));
         ir_rvalue *rhs_i =
            new(mem_ctx) ir_dereference_array(ir->rhs->clone(mem_ctx, NULL),
                                              new(mem_ctx) ir_constant(i));
         ir_rvalue *condition_i =
            ir->condition ? ir->condition->clone(mem_ctx, NULL) : NULL;

         ir_assignment *assign_i =
            new(mem_ctx) ir_assignment(lhs_i, rhs_i, condition_i);

         ir->insert_before(assign_i);
         assign_i->accept(this);
      }
      ir->remove();
      return visit_continue;
   }

   handle_rvalue(&lhs);
   ir->lhs = lhs->as_dereference();

   ir->lhs->accept(this);

   handle_rvalue(&ir->rhs);
   ir->rhs->accept(this);

   if (ir->condition) {
      handle_rvalue(&ir->condition);
      ir->condition->accept(this);
   }

   return visit_continue;
}

bool
optimize_split_arrays(exec_list *instructions, bool linked)
{
   ir_array_reference_visitor refs;
   if (!refs.get_split_list(instructions, linked))
      return false;

   void *mem_ctx = ralloc_context(NULL);

   /* Replace the decls of the arrays to be split with their split
    * components.
    */
   foreach_in_list(variable_entry, entry, &refs.variable_list) {
      const struct glsl_type *type = entry->var->type;
      const struct glsl_type *subtype;

      if (type->is_matrix())
         subtype = type->column_type();
      else
         subtype = type->fields.array;

      entry->mem_ctx = ralloc_parent(entry->var);

      entry->components = ralloc_array(mem_ctx, ir_variable *, entry->size);

      for (unsigned int i = 0; i < entry->size; i++) {
         const char *name = ralloc_asprintf(mem_ctx, "%s_%d",
                                            entry->var->name, i);
         ir_variable *new_var =
            new(entry->mem_ctx) ir_variable(subtype, name, ir_var_temporary);

         /* Do not lose memory/format qualifiers when arrays of images are
          * split.
          */
         new_var->data.memory_read_only = entry->var->data.memory_read_only;
         new_var->data.memory_write_only = entry->var->data.memory_write_only;
         new_var->data.memory_coherent = entry->var->data.memory_coherent;
         new_var->data.memory_volatile = entry->var->data.memory_volatile;
         new_var->data.memory_restrict = entry->var->data.memory_restrict;
         new_var->data.image_format = entry->var->data.image_format;

         entry->components[i] = new_var;
         entry->var->insert_before(entry->components[i]);
      }

      entry->var->remove();
   }

   ir_array_splitting_visitor split(&refs.variable_list);
   visit_list_elements(&split, instructions);

   if (debug)
      _mesa_print_ir(stdout, instructions, NULL);

   ralloc_free(mem_ctx);

   return true;

}
