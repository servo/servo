/*
 * Copyright © 2019 Google, Inc
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
 * \file lower_precision.cpp
 */

#include "main/macros.h"
#include "compiler/glsl_types.h"
#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "ir_rvalue_visitor.h"
#include "util/half_float.h"
#include "util/set.h"
#include "util/hash_table.h"
#include <vector>

namespace {

class find_precision_visitor : public ir_rvalue_enter_visitor {
public:
   find_precision_visitor();
   ~find_precision_visitor();

   virtual void handle_rvalue(ir_rvalue **rvalue);
   virtual ir_visitor_status visit_enter(ir_call *ir);

   ir_function_signature *map_builtin(ir_function_signature *sig);

   bool progress;

   /* Set of rvalues that can be lowered. This will be filled in by
    * find_lowerable_rvalues_visitor. Only the root node of a lowerable section
    * will be added to this set.
    */
   struct set *lowerable_rvalues;

   /**
    * A mapping of builtin signature functions to lowered versions. This is
    * filled in lazily when a lowered version is needed.
    */
   struct hash_table *lowered_builtins;
   /**
    * A temporary hash table only used in order to clone functions.
    */
   struct hash_table *clone_ht;

   void *lowered_builtin_mem_ctx;
};

class find_lowerable_rvalues_visitor : public ir_hierarchical_visitor {
public:
   enum can_lower_state {
      UNKNOWN,
      CANT_LOWER,
      SHOULD_LOWER,
   };

   enum parent_relation {
      /* The parent performs a further operation involving the result from the
       * child and can be lowered along with it.
       */
      COMBINED_OPERATION,
      /* The parent instruction’s operation is independent of the child type so
       * the child should be lowered separately.
       */
      INDEPENDENT_OPERATION,
   };

   struct stack_entry {
      ir_instruction *instr;
      enum can_lower_state state;
      /* List of child rvalues that can be lowered. When this stack entry is
       * popped, if this node itself can’t be lowered than all of the children
       * are root nodes to lower so we will add them to lowerable_rvalues.
       * Otherwise if this node can also be lowered then we won’t add the
       * children because we only want to add the topmost lowerable nodes to
       * lowerable_rvalues and the children will be lowered as part of lowering
       * this node.
       */
      std::vector<ir_instruction *> lowerable_children;
   };

   find_lowerable_rvalues_visitor(struct set *result);

   static void stack_enter(class ir_instruction *ir, void *data);
   static void stack_leave(class ir_instruction *ir, void *data);

   virtual ir_visitor_status visit(ir_constant *ir);
   virtual ir_visitor_status visit(ir_dereference_variable *ir);

   virtual ir_visitor_status visit_enter(ir_dereference_record *ir);
   virtual ir_visitor_status visit_enter(ir_dereference_array *ir);
   virtual ir_visitor_status visit_enter(ir_texture *ir);
   virtual ir_visitor_status visit_enter(ir_expression *ir);

   virtual ir_visitor_status visit_leave(ir_assignment *ir);
   virtual ir_visitor_status visit_leave(ir_call *ir);

   static can_lower_state handle_precision(const glsl_type *type,
                                           int precision);

   static parent_relation get_parent_relation(ir_instruction *parent,
                                              ir_instruction *child);

   std::vector<stack_entry> stack;
   struct set *lowerable_rvalues;

   void pop_stack_entry();
   void add_lowerable_children(const stack_entry &entry);
};

class lower_precision_visitor : public ir_rvalue_visitor {
public:
   virtual void handle_rvalue(ir_rvalue **rvalue);
   virtual ir_visitor_status visit_enter(ir_dereference_array *);
   virtual ir_visitor_status visit_enter(ir_dereference_record *);
   virtual ir_visitor_status visit_enter(ir_call *ir);
   virtual ir_visitor_status visit_enter(ir_texture *ir);
   virtual ir_visitor_status visit_leave(ir_expression *);
};

bool
can_lower_type(const glsl_type *type)
{
   /* Don’t lower any expressions involving non-float types except bool and
    * texture samplers. This will rule out operations that change the type such
    * as conversion to ints. Instead it will end up lowering the arguments
    * instead and adding a final conversion to float32. We want to handle
    * boolean types so that it will do comparisons as 16-bit.
    */

   switch (type->base_type) {
   case GLSL_TYPE_FLOAT:
   case GLSL_TYPE_BOOL:
   case GLSL_TYPE_SAMPLER:
      return true;

   default:
      return false;
   }
}

find_lowerable_rvalues_visitor::find_lowerable_rvalues_visitor(struct set *res)
{
   lowerable_rvalues = res;
   callback_enter = stack_enter;
   callback_leave = stack_leave;
   data_enter = this;
   data_leave = this;
}

void
find_lowerable_rvalues_visitor::stack_enter(class ir_instruction *ir,
                                            void *data)
{
   find_lowerable_rvalues_visitor *state =
      (find_lowerable_rvalues_visitor *) data;

   /* Add a new stack entry for this instruction */
   stack_entry entry;

   entry.instr = ir;
   entry.state = state->in_assignee ? CANT_LOWER : UNKNOWN;

   state->stack.push_back(entry);
}

void
find_lowerable_rvalues_visitor::add_lowerable_children(const stack_entry &entry)
{
   /* We can’t lower this node so if there were any pending children then they
    * are all root lowerable nodes and we should add them to the set.
    */
   for (auto &it : entry.lowerable_children)
      _mesa_set_add(lowerable_rvalues, it);
}

void
find_lowerable_rvalues_visitor::pop_stack_entry()
{
   const stack_entry &entry = stack.back();

   if (stack.size() >= 2) {
      /* Combine this state into the parent state, unless the parent operation
       * doesn’t have any relation to the child operations
       */
      stack_entry &parent = stack.end()[-2];
      parent_relation rel = get_parent_relation(parent.instr, entry.instr);

      if (rel == COMBINED_OPERATION) {
         switch (entry.state) {
         case CANT_LOWER:
            parent.state = CANT_LOWER;
            break;
         case SHOULD_LOWER:
            if (parent.state == UNKNOWN)
               parent.state = SHOULD_LOWER;
            break;
         case UNKNOWN:
            break;
         }
      }
   }

   if (entry.state == SHOULD_LOWER) {
      ir_rvalue *rv = entry.instr->as_rvalue();

      if (rv == NULL) {
         add_lowerable_children(entry);
      } else if (stack.size() >= 2) {
         stack_entry &parent = stack.end()[-2];

         switch (get_parent_relation(parent.instr, rv)) {
         case COMBINED_OPERATION:
            /* We only want to add the toplevel lowerable instructions to the
             * lowerable set. Therefore if there is a parent then instead of
             * adding this instruction to the set we will queue depending on
             * the result of the parent instruction.
             */
            parent.lowerable_children.push_back(entry.instr);
            break;
         case INDEPENDENT_OPERATION:
            _mesa_set_add(lowerable_rvalues, rv);
            break;
         }
      } else {
         /* This is a toplevel node so add it directly to the lowerable
          * set.
          */
         _mesa_set_add(lowerable_rvalues, rv);
      }
   } else if (entry.state == CANT_LOWER) {
      add_lowerable_children(entry);
   }

   stack.pop_back();
}

void
find_lowerable_rvalues_visitor::stack_leave(class ir_instruction *ir,
                                            void *data)
{
   find_lowerable_rvalues_visitor *state =
      (find_lowerable_rvalues_visitor *) data;

   state->pop_stack_entry();
}

enum find_lowerable_rvalues_visitor::can_lower_state
find_lowerable_rvalues_visitor::handle_precision(const glsl_type *type,
                                                 int precision)
{
   if (!can_lower_type(type))
      return CANT_LOWER;

   switch (precision) {
   case GLSL_PRECISION_NONE:
      return UNKNOWN;
   case GLSL_PRECISION_HIGH:
      return CANT_LOWER;
   case GLSL_PRECISION_MEDIUM:
   case GLSL_PRECISION_LOW:
      return SHOULD_LOWER;
   }

   return CANT_LOWER;
}

enum find_lowerable_rvalues_visitor::parent_relation
find_lowerable_rvalues_visitor::get_parent_relation(ir_instruction *parent,
                                                    ir_instruction *child)
{
   /* If the parent is a dereference instruction then the only child could be
    * for example an array dereference and that should be lowered independently
    * of the parent.
    */
   if (parent->as_dereference())
      return INDEPENDENT_OPERATION;

   /* The precision of texture sampling depend on the precision of the sampler.
    * The rest of the arguments don’t matter so we can treat it as an
    * independent operation.
    */
   if (parent->as_texture())
      return INDEPENDENT_OPERATION;

   return COMBINED_OPERATION;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit(ir_constant *ir)
{
   stack_enter(ir, this);

   if (!can_lower_type(ir->type))
      stack.back().state = CANT_LOWER;

   stack_leave(ir, this);

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit(ir_dereference_variable *ir)
{
   stack_enter(ir, this);

   if (stack.back().state == UNKNOWN)
      stack.back().state = handle_precision(ir->type, ir->precision());

   stack_leave(ir, this);

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_enter(ir_dereference_record *ir)
{
   ir_hierarchical_visitor::visit_enter(ir);

   if (stack.back().state == UNKNOWN)
      stack.back().state = handle_precision(ir->type, ir->precision());

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_enter(ir_dereference_array *ir)
{
   ir_hierarchical_visitor::visit_enter(ir);

   if (stack.back().state == UNKNOWN)
      stack.back().state = handle_precision(ir->type, ir->precision());

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_enter(ir_texture *ir)
{
   ir_hierarchical_visitor::visit_enter(ir);

   if (stack.back().state == UNKNOWN) {
      /* The precision of the sample value depends on the precision of the
       * sampler.
       */
      stack.back().state = handle_precision(ir->type,
                                            ir->sampler->precision());
   }

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_enter(ir_expression *ir)
{
   ir_hierarchical_visitor::visit_enter(ir);

   if (!can_lower_type(ir->type))
      stack.back().state = CANT_LOWER;

   /* Don't lower precision for derivative calculations */
   if (ir->operation == ir_unop_dFdx ||
         ir->operation == ir_unop_dFdx_coarse ||
         ir->operation == ir_unop_dFdx_fine ||
         ir->operation == ir_unop_dFdy ||
         ir->operation == ir_unop_dFdy_coarse ||
         ir->operation == ir_unop_dFdy_fine) {
      stack.back().state = CANT_LOWER;
   }

   return visit_continue;
}

static bool
is_lowerable_builtin(ir_call *ir,
                     const struct set *lowerable_rvalues)
{
   if (!ir->callee->is_builtin())
      return false;

   assert(ir->callee->return_precision == GLSL_PRECISION_NONE);

   foreach_in_list(ir_rvalue, param, &ir->actual_parameters) {
      if (!param->as_constant() &&
          _mesa_set_search(lowerable_rvalues, param) == NULL)
         return false;
   }

   return true;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_leave(ir_call *ir)
{
   ir_hierarchical_visitor::visit_leave(ir);

   /* Special case for handling temporary variables generated by the compiler
    * for function calls. If we assign to one of these using a function call
    * that has a lowerable return type then we can assume the temporary
    * variable should have a medium precision too.
    */

   /* Do nothing if the return type is void. */
   if (!ir->return_deref)
      return visit_continue;

   ir_variable *var = ir->return_deref->variable_referenced();

   assert(var->data.mode == ir_var_temporary);

   unsigned return_precision = ir->callee->return_precision;

   /* If the call is to a builtin, then the function won’t have a return
    * precision and we should determine it from the precision of the arguments.
    */
   if (is_lowerable_builtin(ir, lowerable_rvalues))
      return_precision = GLSL_PRECISION_MEDIUM;

   can_lower_state lower_state =
      handle_precision(var->type, return_precision);

   if (lower_state == SHOULD_LOWER) {
      /* There probably shouldn’t be any situations where multiple ir_call
       * instructions write to the same temporary?
       */
      assert(var->data.precision == GLSL_PRECISION_NONE);
      var->data.precision = GLSL_PRECISION_MEDIUM;
   } else {
      var->data.precision = GLSL_PRECISION_HIGH;
   }

   return visit_continue;
}

ir_visitor_status
find_lowerable_rvalues_visitor::visit_leave(ir_assignment *ir)
{
   ir_hierarchical_visitor::visit_leave(ir);

   /* Special case for handling temporary variables generated by the compiler.
    * If we assign to one of these using a lowered precision then we can assume
    * the temporary variable should have a medium precision too.
    */
   ir_variable *var = ir->lhs->variable_referenced();

   if (var->data.mode == ir_var_temporary) {
      if (_mesa_set_search(lowerable_rvalues, ir->rhs)) {
         /* Only override the precision if this is the first assignment. For
          * temporaries such as the ones generated for the ?: operator there
          * can be multiple assignments with different precisions. This way we
          * get the highest precision of all of the assignments.
          */
         if (var->data.precision == GLSL_PRECISION_NONE)
            var->data.precision = GLSL_PRECISION_MEDIUM;
      } else if (!ir->rhs->as_constant()) {
         var->data.precision = GLSL_PRECISION_HIGH;
      }
   }

   return visit_continue;
}

void
find_lowerable_rvalues(exec_list *instructions,
                       struct set *result)
{
   find_lowerable_rvalues_visitor v(result);

   visit_list_elements(&v, instructions);

   assert(v.stack.empty());
}

static ir_rvalue *
convert_precision(int op, ir_rvalue *ir)
{
   unsigned base_type = (op == ir_unop_f2fmp ?
                        GLSL_TYPE_FLOAT16 : GLSL_TYPE_FLOAT);
   const glsl_type *desired_type;
   desired_type = glsl_type::get_instance(base_type,
                             ir->type->vector_elements,
                             ir->type->matrix_columns);

   void *mem_ctx = ralloc_parent(ir);
   return new(mem_ctx) ir_expression(op, desired_type, ir, NULL);
}

void
lower_precision_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   ir_rvalue *ir = *rvalue;

   if (ir == NULL)
      return;

   if (ir->as_dereference()) {
      if (!ir->type->is_boolean())
         *rvalue = convert_precision(ir_unop_f2fmp, ir);
   } else if (ir->type->is_float()) {
      ir->type = glsl_type::get_instance(GLSL_TYPE_FLOAT16,
                                         ir->type->vector_elements,
                                         ir->type->matrix_columns,
                                         ir->type->explicit_stride,
                                         ir->type->interface_row_major);

      ir_constant *const_ir = ir->as_constant();

      if (const_ir) {
         ir_constant_data value;

         for (unsigned i = 0; i < ARRAY_SIZE(value.f16); i++)
            value.f16[i] = _mesa_float_to_half(const_ir->value.f[i]);

         const_ir->value = value;
      }
   }
}

ir_visitor_status
lower_precision_visitor::visit_enter(ir_dereference_record *ir)
{
   /* We don’t want to lower the variable */
   return visit_continue_with_parent;
}

ir_visitor_status
lower_precision_visitor::visit_enter(ir_dereference_array *ir)
{
   /* We don’t want to convert the array index or the variable. If the array
    * index itself is lowerable that will be handled separately.
    */
   return visit_continue_with_parent;
}

ir_visitor_status
lower_precision_visitor::visit_enter(ir_call *ir)
{
   /* We don’t want to convert the arguments. These will be handled separately.
    */
   return visit_continue_with_parent;
}

ir_visitor_status
lower_precision_visitor::visit_enter(ir_texture *ir)
{
   /* We don’t want to convert the arguments. These will be handled separately.
    */
   return visit_continue_with_parent;
}

ir_visitor_status
lower_precision_visitor::visit_leave(ir_expression *ir)
{
   ir_rvalue_visitor::visit_leave(ir);

   /* If the expression is a conversion operation to or from bool then fix the
    * operation.
    */
   switch (ir->operation) {
   case ir_unop_b2f:
      ir->operation = ir_unop_b2f16;
      break;
   case ir_unop_f2b:
      ir->operation = ir_unop_f162b;
      break;
   default:
      break;
   }

   return visit_continue;
}

void
find_precision_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   /* Checking the precision of rvalue can be lowered first throughout
    * find_lowerable_rvalues_visitor.
    * Once it found the precision of rvalue can be lowered, then we can
    * add conversion f2fmp through lower_precision_visitor.
    */
   if (*rvalue == NULL)
      return;

   struct set_entry *entry = _mesa_set_search(lowerable_rvalues, *rvalue);

   if (!entry)
      return;

   _mesa_set_remove(lowerable_rvalues, entry);

   /* If the entire expression is just a variable dereference then trying to
    * lower it will just directly add pointless to and from conversions without
    * any actual operation in-between. Although these will eventually get
    * optimised out, avoiding generating them here also avoids breaking inout
    * parameters to functions.
    */
   if ((*rvalue)->as_dereference())
      return;

   lower_precision_visitor v;

   (*rvalue)->accept(&v);
   v.handle_rvalue(rvalue);

   /* We don’t need to add the final conversion if the final type has been
    * converted to bool
    */
   if ((*rvalue)->type->base_type != GLSL_TYPE_BOOL)
      *rvalue = convert_precision(ir_unop_f162f, *rvalue);

   progress = true;
}

ir_visitor_status
find_precision_visitor::visit_enter(ir_call *ir)
{
   ir_rvalue_enter_visitor::visit_enter(ir);

   /* If this is a call to a builtin and the find_lowerable_rvalues_visitor
    * overrode the precision of the temporary return variable, then we can
    * replace the builtin implementation with a lowered version.
    */

   if (!ir->callee->is_builtin() ||
       ir->return_deref == NULL ||
       ir->return_deref->variable_referenced()->data.precision !=
       GLSL_PRECISION_MEDIUM)
      return visit_continue;

   ir->callee = map_builtin(ir->callee);
   ir->generate_inline(ir);
   ir->remove();

   return visit_continue_with_parent;
}

ir_function_signature *
find_precision_visitor::map_builtin(ir_function_signature *sig)
{
   if (lowered_builtins == NULL) {
      lowered_builtins = _mesa_pointer_hash_table_create(NULL);
      clone_ht =_mesa_pointer_hash_table_create(NULL);
      lowered_builtin_mem_ctx = ralloc_context(NULL);
   } else {
      struct hash_entry *entry = _mesa_hash_table_search(lowered_builtins, sig);
      if (entry)
         return (ir_function_signature *) entry->data;
   }

   ir_function_signature *lowered_sig =
      sig->clone(lowered_builtin_mem_ctx, clone_ht);

   foreach_in_list(ir_variable, param, &lowered_sig->parameters) {
      param->data.precision = GLSL_PRECISION_MEDIUM;
   }

   lower_precision(&lowered_sig->body);

   _mesa_hash_table_clear(clone_ht, NULL);

   _mesa_hash_table_insert(lowered_builtins, sig, lowered_sig);

   return lowered_sig;
}

find_precision_visitor::find_precision_visitor()
   : progress(false),
     lowerable_rvalues(_mesa_pointer_set_create(NULL)),
     lowered_builtins(NULL),
     clone_ht(NULL),
     lowered_builtin_mem_ctx(NULL)
{
}

find_precision_visitor::~find_precision_visitor()
{
   _mesa_set_destroy(lowerable_rvalues, NULL);

   if (lowered_builtins) {
      _mesa_hash_table_destroy(lowered_builtins, NULL);
      _mesa_hash_table_destroy(clone_ht, NULL);
      ralloc_free(lowered_builtin_mem_ctx);
   }
}

}

bool
lower_precision(exec_list *instructions)
{
   find_precision_visitor v;

   find_lowerable_rvalues(instructions, v.lowerable_rvalues);

   visit_list_elements(&v, instructions);

   return v.progress;
}
