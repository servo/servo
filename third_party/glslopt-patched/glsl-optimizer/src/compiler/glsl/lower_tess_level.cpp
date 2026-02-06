/*
 * Copyright © 2014 Intel Corporation
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the

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
 * \file lower_tess_level.cpp
 *
 * This pass accounts for the difference between the way gl_TessLevelOuter
 * and gl_TessLevelInner is declared in standard GLSL (as an array of
 * floats), and the way it is frequently implemented in hardware (as a vec4
 * and vec2).
 *
 * The declaration of gl_TessLevel* is replaced with a declaration
 * of gl_TessLevel*MESA, and any references to gl_TessLevel* are
 * translated to refer to gl_TessLevel*MESA with the appropriate
 * swizzling of array indices.  For instance:
 *
 *   gl_TessLevelOuter[i]
 *
 * is translated into:
 *
 *   gl_TessLevelOuterMESA[i]
 *
 * Since some hardware may not internally represent gl_TessLevel* as a pair
 * of vec4's, this lowering pass is optional.  To enable it, set the
 * LowerTessLevel flag in gl_shader_compiler_options to true.
 */

#include "glsl_symbol_table.h"
#include "ir_rvalue_visitor.h"
#include "ir.h"
#include "program/prog_instruction.h" /* For WRITEMASK_* */
#include "main/mtypes.h"

namespace {

class lower_tess_level_visitor : public ir_rvalue_visitor {
public:
   explicit lower_tess_level_visitor(gl_shader_stage shader_stage)
      : progress(false), old_tess_level_outer_var(NULL),
        old_tess_level_inner_var(NULL), new_tess_level_outer_var(NULL),
        new_tess_level_inner_var(NULL), shader_stage(shader_stage)
   {
   }

   virtual ir_visitor_status visit(ir_variable *);
   bool is_tess_level_array(ir_rvalue *ir);
   ir_rvalue *lower_tess_level_array(ir_rvalue *ir);
   virtual ir_visitor_status visit_leave(ir_assignment *);
   void visit_new_assignment(ir_assignment *ir);
   virtual ir_visitor_status visit_leave(ir_call *);

   virtual void handle_rvalue(ir_rvalue **rvalue);

   void fix_lhs(ir_assignment *);

   bool progress;

   /**
    * Pointer to the declaration of gl_TessLevel*, if found.
    */
   ir_variable *old_tess_level_outer_var;
   ir_variable *old_tess_level_inner_var;

   /**
    * Pointer to the newly-created gl_TessLevel*MESA variables.
    */
   ir_variable *new_tess_level_outer_var;
   ir_variable *new_tess_level_inner_var;

   /**
    * Type of shader we are compiling (e.g. MESA_SHADER_TESS_CTRL)
    */
   const gl_shader_stage shader_stage;
};

} /* anonymous namespace */

/**
 * Replace any declaration of gl_TessLevel* as an array of floats with a
 * declaration of gl_TessLevel*MESA as a vec4.
 */
ir_visitor_status
lower_tess_level_visitor::visit(ir_variable *ir)
{
   if ((!ir->name) ||
       ((strcmp(ir->name, "gl_TessLevelInner") != 0) &&
        (strcmp(ir->name, "gl_TessLevelOuter") != 0)))
      return visit_continue;

   assert (ir->type->is_array());

   if (strcmp(ir->name, "gl_TessLevelOuter") == 0) {
      if (this->old_tess_level_outer_var)
         return visit_continue;

      old_tess_level_outer_var = ir;
      assert(ir->type->fields.array == glsl_type::float_type);

      /* Clone the old var so that we inherit all of its properties */
      new_tess_level_outer_var = ir->clone(ralloc_parent(ir), NULL);

      /* And change the properties that we need to change */
      new_tess_level_outer_var->name = ralloc_strdup(new_tess_level_outer_var,
                                                "gl_TessLevelOuterMESA");
      new_tess_level_outer_var->type = glsl_type::vec4_type;
      new_tess_level_outer_var->data.max_array_access = 0;

      ir->replace_with(new_tess_level_outer_var);
   } else if (strcmp(ir->name, "gl_TessLevelInner") == 0) {
      if (this->old_tess_level_inner_var)
         return visit_continue;

      old_tess_level_inner_var = ir;
      assert(ir->type->fields.array == glsl_type::float_type);

      /* Clone the old var so that we inherit all of its properties */
      new_tess_level_inner_var = ir->clone(ralloc_parent(ir), NULL);

      /* And change the properties that we need to change */
      new_tess_level_inner_var->name = ralloc_strdup(new_tess_level_inner_var,
                                                "gl_TessLevelInnerMESA");
      new_tess_level_inner_var->type = glsl_type::vec2_type;
      new_tess_level_inner_var->data.max_array_access = 0;

      ir->replace_with(new_tess_level_inner_var);
   } else {
      assert(0);
   }

   this->progress = true;

   return visit_continue;
}


/**
 * Determine whether the given rvalue describes an array of floats that
 * needs to be lowered to a vec4; that is, determine whether it
 * matches one of the following patterns:
 *
 * - gl_TessLevelOuter
 * - gl_TessLevelInner
 */
bool
lower_tess_level_visitor::is_tess_level_array(ir_rvalue *ir)
{
   if (!ir->type->is_array())
      return false;
   if (ir->type->fields.array != glsl_type::float_type)
      return false;

   if (this->old_tess_level_outer_var) {
      if (ir->variable_referenced() == this->old_tess_level_outer_var)
         return true;
   }
   if (this->old_tess_level_inner_var) {
      if (ir->variable_referenced() == this->old_tess_level_inner_var)
         return true;
   }
   return false;
}


/**
 * If the given ir satisfies is_tess_level_array(), return new ir
 * representing its lowered equivalent.  That is, map:
 *
 * - gl_TessLevelOuter => gl_TessLevelOuterMESA
 * - gl_TessLevelInner => gl_TessLevelInnerMESA
 *
 * Otherwise return NULL.
 */
ir_rvalue *
lower_tess_level_visitor::lower_tess_level_array(ir_rvalue *ir)
{
   if (!ir->type->is_array())
      return NULL;
   if (ir->type->fields.array != glsl_type::float_type)
      return NULL;

   ir_variable **new_var = NULL;

   if (this->old_tess_level_outer_var) {
      if (ir->variable_referenced() == this->old_tess_level_outer_var)
         new_var = &this->new_tess_level_outer_var;
   }
   if (this->old_tess_level_inner_var) {
      if (ir->variable_referenced() == this->old_tess_level_inner_var)
         new_var = &this->new_tess_level_inner_var;
   }

   if (new_var == NULL)
      return NULL;

   assert(ir->as_dereference_variable());
   return new(ralloc_parent(ir)) ir_dereference_variable(*new_var);
}


void
lower_tess_level_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL)
      return;

   ir_dereference_array *const array_deref = (*rv)->as_dereference_array();
   if (array_deref == NULL)
      return;

   /* Replace any expression that indexes one of the floats in gl_TessLevel*
    * with an expression that indexes into one of the vec4's
    * gl_TessLevel*MESA and accesses the appropriate component.
    */
   ir_rvalue *lowered_vec4 =
      this->lower_tess_level_array(array_deref->array);
   if (lowered_vec4 != NULL) {
      this->progress = true;
      void *mem_ctx = ralloc_parent(array_deref);

      ir_expression *const expr =
         new(mem_ctx) ir_expression(ir_binop_vector_extract,
                                    lowered_vec4,
                                    array_deref->array_index);

      *rv = expr;
   }
}

void
lower_tess_level_visitor::fix_lhs(ir_assignment *ir)
{
   if (ir->lhs->ir_type != ir_type_expression)
      return;
   void *mem_ctx = ralloc_parent(ir);
   ir_expression *const expr = (ir_expression *) ir->lhs;

   /* The expression must be of the form:
    *
    *     (vector_extract gl_TessLevel*MESA, j).
    */
   assert(expr->operation == ir_binop_vector_extract);
   assert(expr->operands[0]->ir_type == ir_type_dereference_variable);
   assert((expr->operands[0]->type == glsl_type::vec4_type) ||
          (expr->operands[0]->type == glsl_type::vec2_type));

   ir_dereference *const new_lhs = (ir_dereference *) expr->operands[0];

   ir_constant *old_index_constant =
      expr->operands[1]->constant_expression_value(mem_ctx);
   if (!old_index_constant) {
      ir->rhs = new(mem_ctx) ir_expression(ir_triop_vector_insert,
                                           expr->operands[0]->type,
                                           new_lhs->clone(mem_ctx, NULL),
                                           ir->rhs,
                                           expr->operands[1]);
   }
   ir->set_lhs(new_lhs);

   if (old_index_constant) {
      /* gl_TessLevel* is being accessed via a constant index.  Don't bother
       * creating a vector insert op. Just use a write mask.
       */
      ir->write_mask = 1 << old_index_constant->get_int_component(0);
   } else {
      ir->write_mask = (1 << expr->operands[0]->type->vector_elements) - 1;
   }
}

/**
 * Replace any assignment having a gl_TessLevel* (undereferenced) as
 * its LHS or RHS with a sequence of assignments, one for each component of
 * the array.  Each of these assignments is lowered to refer to
 * gl_TessLevel*MESA as appropriate.
 */
ir_visitor_status
lower_tess_level_visitor::visit_leave(ir_assignment *ir)
{
   /* First invoke the base class visitor.  This causes handle_rvalue() to be
    * called on ir->rhs and ir->condition.
    */
   ir_rvalue_visitor::visit_leave(ir);

   if (this->is_tess_level_array(ir->lhs) ||
       this->is_tess_level_array(ir->rhs)) {
      /* LHS or RHS of the assignment is the entire gl_TessLevel* array.
       * Since we are
       * reshaping gl_TessLevel* from an array of floats to a
       * vec4, this isn't going to work as a bulk assignment anymore, so
       * unroll it to element-by-element assignments and lower each of them.
       *
       * Note: to unroll into element-by-element assignments, we need to make
       * clones of the LHS and RHS.  This is safe because expressions and
       * l-values are side-effect free.
       */
      void *ctx = ralloc_parent(ir);
      int array_size = ir->lhs->type->array_size();
      for (int i = 0; i < array_size; ++i) {
         ir_dereference_array *new_lhs = new(ctx) ir_dereference_array(
            ir->lhs->clone(ctx, NULL), new(ctx) ir_constant(i));
         ir_dereference_array *new_rhs = new(ctx) ir_dereference_array(
            ir->rhs->clone(ctx, NULL), new(ctx) ir_constant(i));
         this->handle_rvalue((ir_rvalue **) &new_rhs);

         /* Handle the LHS after creating the new assignment.  This must
          * happen in this order because handle_rvalue may replace the old LHS
          * with an ir_expression of ir_binop_vector_extract.  Since this is
          * not a valide l-value, this will cause an assertion in the
          * ir_assignment constructor to fail.
          *
          * If this occurs, replace the mangled LHS with a dereference of the
          * vector, and replace the RHS with an ir_triop_vector_insert.
          */
         ir_assignment *const assign = new(ctx) ir_assignment(new_lhs, new_rhs);
         this->handle_rvalue((ir_rvalue **) &assign->lhs);
         this->fix_lhs(assign);

         this->base_ir->insert_before(assign);
      }
      ir->remove();

      return visit_continue;
   }

   /* Handle the LHS as if it were an r-value.  Normally
    * rvalue_visit(ir_assignment *) only visits the RHS, but we need to lower
    * expressions in the LHS as well.
    *
    * This may cause the LHS to get replaced with an ir_expression of
    * ir_binop_vector_extract.  If this occurs, replace it with a dereference
    * of the vector, and replace the RHS with an ir_triop_vector_insert.
    */
   handle_rvalue((ir_rvalue **)&ir->lhs);
   this->fix_lhs(ir);

   return rvalue_visit(ir);
}


/**
 * Set up base_ir properly and call visit_leave() on a newly created
 * ir_assignment node.  This is used in cases where we have to insert an
 * ir_assignment in a place where we know the hierarchical visitor won't see
 * it.
 */
void
lower_tess_level_visitor::visit_new_assignment(ir_assignment *ir)
{
   ir_instruction *old_base_ir = this->base_ir;
   this->base_ir = ir;
   ir->accept(this);
   this->base_ir = old_base_ir;
}


/**
 * If a gl_TessLevel* variable appears as an argument in an ir_call
 * expression, replace it with a temporary variable, and make sure the ir_call
 * is preceded and/or followed by assignments that copy the contents of the
 * temporary variable to and/or from gl_TessLevel*.  Each of these
 * assignments is then lowered to refer to gl_TessLevel*MESA.
 */
ir_visitor_status
lower_tess_level_visitor::visit_leave(ir_call *ir)
{
   void *ctx = ralloc_parent(ir);

   const exec_node *formal_param_node = ir->callee->parameters.get_head_raw();
   const exec_node *actual_param_node = ir->actual_parameters.get_head_raw();
   while (!actual_param_node->is_tail_sentinel()) {
      ir_variable *formal_param = (ir_variable *) formal_param_node;
      ir_rvalue *actual_param = (ir_rvalue *) actual_param_node;

      /* Advance formal_param_node and actual_param_node now so that we can
       * safely replace actual_param with another node, if necessary, below.
       */
      formal_param_node = formal_param_node->next;
      actual_param_node = actual_param_node->next;

      if (!this->is_tess_level_array(actual_param))
         continue;

      /* User is trying to pass a whole gl_TessLevel* array to a function
       * call.  Since we are reshaping gl_TessLevel* from an array of floats
       * to a vec4, this isn't going to work anymore, so use a temporary
       * array instead.
       */
      ir_variable *temp = new(ctx) ir_variable(
         actual_param->type, "temp_tess_level", ir_var_temporary);
      this->base_ir->insert_before(temp);
      actual_param->replace_with(
         new(ctx) ir_dereference_variable(temp));
      if (formal_param->data.mode == ir_var_function_in
          || formal_param->data.mode == ir_var_function_inout) {
         /* Copy from gl_TessLevel* to the temporary before the call.
          * Since we are going to insert this copy before the current
          * instruction, we need to visit it afterwards to make sure it
          * gets lowered.
          */
         ir_assignment *new_assignment = new(ctx) ir_assignment(
            new(ctx) ir_dereference_variable(temp),
            actual_param->clone(ctx, NULL));
         this->base_ir->insert_before(new_assignment);
         this->visit_new_assignment(new_assignment);
      }
      if (formal_param->data.mode == ir_var_function_out
          || formal_param->data.mode == ir_var_function_inout) {
         /* Copy from the temporary to gl_TessLevel* after the call.
          * Since visit_list_elements() has already decided which
          * instruction it's going to visit next, we need to visit
          * afterwards to make sure it gets lowered.
          */
         ir_assignment *new_assignment = new(ctx) ir_assignment(
            actual_param->clone(ctx, NULL),
            new(ctx) ir_dereference_variable(temp));
         this->base_ir->insert_after(new_assignment);
         this->visit_new_assignment(new_assignment);
      }
   }

   return rvalue_visit(ir);
}


bool
lower_tess_level(gl_linked_shader *shader)
{
   if ((shader->Stage != MESA_SHADER_TESS_CTRL) &&
       (shader->Stage != MESA_SHADER_TESS_EVAL))
      return false;

   lower_tess_level_visitor v(shader->Stage);

   visit_list_elements(&v, shader->ir);

   if (v.new_tess_level_outer_var)
      shader->symbols->add_variable(v.new_tess_level_outer_var);
   if (v.new_tess_level_inner_var)
      shader->symbols->add_variable(v.new_tess_level_inner_var);

   return v.progress;
}
