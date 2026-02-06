/*
 * Copyright © 2011 Intel Corporation
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
 * \file lower_distance.cpp
 *
 * This pass accounts for the difference between the way
 * gl_ClipDistance is declared in standard GLSL (as an array of
 * floats), and the way it is frequently implemented in hardware (as
 * a pair of vec4s, with four clip distances packed into each).
 *
 * The declaration of gl_ClipDistance is replaced with a declaration
 * of gl_ClipDistanceMESA, and any references to gl_ClipDistance are
 * translated to refer to gl_ClipDistanceMESA with the appropriate
 * swizzling of array indices.  For instance:
 *
 *   gl_ClipDistance[i]
 *
 * is translated into:
 *
 *   gl_ClipDistanceMESA[i>>2][i&3]
 *
 * Since some hardware may not internally represent gl_ClipDistance as a pair
 * of vec4's, this lowering pass is optional.  To enable it, set the
 * LowerCombinedClipCullDistance flag in gl_shader_compiler_options to true.
 */

#include "main/macros.h"
#include "glsl_symbol_table.h"
#include "ir_rvalue_visitor.h"
#include "ir.h"
#include "program/prog_instruction.h" /* For WRITEMASK_* */
#include "main/mtypes.h"

#define GLSL_CLIP_VAR_NAME "gl_ClipDistanceMESA"

namespace {

class lower_distance_visitor : public ir_rvalue_visitor {
public:
   explicit lower_distance_visitor(gl_shader_stage shader_stage,
                                   const char *in_name, int total_size,
                                   int offset)
      : progress(false), old_distance_out_var(NULL),
        old_distance_in_var(NULL), new_distance_out_var(NULL),
        new_distance_in_var(NULL), shader_stage(shader_stage),
        in_name(in_name), total_size(total_size), offset(offset)
   {
   }

   explicit lower_distance_visitor(gl_shader_stage shader_stage,
                                   const char *in_name,
                                   const lower_distance_visitor *orig,
                                   int offset)
      : progress(false),
        old_distance_out_var(NULL),
        old_distance_in_var(NULL),
        new_distance_out_var(orig->new_distance_out_var),
        new_distance_in_var(orig->new_distance_in_var),
        shader_stage(shader_stage),
        in_name(in_name),
        total_size(orig->total_size),
        offset(offset)
   {
   }

   virtual ir_visitor_status visit(ir_variable *);
   void create_indices(ir_rvalue*, ir_rvalue *&, ir_rvalue *&);
   bool is_distance_vec8(ir_rvalue *ir);
   ir_rvalue *lower_distance_vec8(ir_rvalue *ir);
   virtual ir_visitor_status visit_leave(ir_assignment *);
   void visit_new_assignment(ir_assignment *ir);
   virtual ir_visitor_status visit_leave(ir_call *);

   virtual void handle_rvalue(ir_rvalue **rvalue);

   void fix_lhs(ir_assignment *);

   bool progress;

   /**
    * Pointer to the declaration of gl_ClipDistance, if found.
    *
    * Note:
    *
    * - the in_var is for geometry and both tessellation shader inputs only.
    *
    * - since gl_ClipDistance is available in tessellation control,
    *   tessellation evaluation and geometry shaders as both an input
    *   and an output, it's possible for both old_distance_out_var
    *   and old_distance_in_var to be non-null.
    */
   ir_variable *old_distance_out_var;
   ir_variable *old_distance_in_var;

   /**
    * Pointer to the newly-created gl_ClipDistanceMESA variable.
    */
   ir_variable *new_distance_out_var;
   ir_variable *new_distance_in_var;

   /**
    * Type of shader we are compiling (e.g. MESA_SHADER_VERTEX)
    */
   const gl_shader_stage shader_stage;
   const char *in_name;
   int total_size;
   int offset;
};

} /* anonymous namespace */

/**
 * Replace any declaration of 'in_name' as an array of floats with a
 * declaration of gl_ClipDistanceMESA as an array of vec4's.
 */
ir_visitor_status
lower_distance_visitor::visit(ir_variable *ir)
{
   ir_variable **old_var;
   ir_variable **new_var;

   if (!ir->name || strcmp(ir->name, in_name) != 0)
      return visit_continue;
   assert (ir->type->is_array());

   if (ir->data.mode == ir_var_shader_out) {
      if (this->old_distance_out_var)
         return visit_continue;
      old_var = &old_distance_out_var;
      new_var = &new_distance_out_var;
   } else if (ir->data.mode == ir_var_shader_in) {
      if (this->old_distance_in_var)
         return visit_continue;
      old_var = &old_distance_in_var;
      new_var = &new_distance_in_var;
   } else {
      unreachable("not reached");
   }

   this->progress = true;

   *old_var = ir;

   if (!(*new_var)) {
      unsigned new_size = (total_size + 3) / 4;

      /* Clone the old var so that we inherit all of its properties */
      *new_var = ir->clone(ralloc_parent(ir), NULL);
      (*new_var)->name = ralloc_strdup(*new_var, GLSL_CLIP_VAR_NAME);
      (*new_var)->data.location = VARYING_SLOT_CLIP_DIST0;

      if (!ir->type->fields.array->is_array()) {
         /* gl_ClipDistance (used for vertex, tessellation evaluation and
          * geometry output, and fragment input).
          */
         assert((ir->data.mode == ir_var_shader_in &&
                 this->shader_stage == MESA_SHADER_FRAGMENT) ||
                (ir->data.mode == ir_var_shader_out &&
                 (this->shader_stage == MESA_SHADER_VERTEX ||
                  this->shader_stage == MESA_SHADER_TESS_EVAL ||
                  this->shader_stage == MESA_SHADER_GEOMETRY)));

         assert (ir->type->fields.array == glsl_type::float_type);
         (*new_var)->data.max_array_access = new_size - 1;

         /* And change the properties that we need to change */
         (*new_var)->type = glsl_type::get_array_instance(glsl_type::vec4_type,
                                                          new_size);
      } else {
         /* 2D gl_ClipDistance (used for tessellation control, tessellation
          * evaluation and geometry input, and tessellation control output).
          */
         assert((ir->data.mode == ir_var_shader_in &&
                 (this->shader_stage == MESA_SHADER_GEOMETRY ||
                  this->shader_stage == MESA_SHADER_TESS_EVAL)) ||
                this->shader_stage == MESA_SHADER_TESS_CTRL);

         assert (ir->type->fields.array->fields.array == glsl_type::float_type);

         /* And change the properties that we need to change */
         (*new_var)->type = glsl_type::get_array_instance(
                            glsl_type::get_array_instance(glsl_type::vec4_type,
                                                          new_size),
                            ir->type->array_size());
      }
      ir->replace_with(*new_var);
   } else {
      ir->remove();
   }

   return visit_continue;
}


/**
 * Create the necessary GLSL rvalues to index into gl_ClipDistanceMESA based
 * on the rvalue previously used to index into gl_ClipDistance.
 *
 * \param array_index Selects one of the vec4's in gl_ClipDistanceMESA
 * \param swizzle_index Selects a component within the vec4 selected by
 *        array_index.
 */
void
lower_distance_visitor::create_indices(ir_rvalue *old_index,
                                            ir_rvalue *&array_index,
                                            ir_rvalue *&swizzle_index)
{
   void *ctx = ralloc_parent(old_index);

   /* Make sure old_index is a signed int so that the bitwise "shift" and
    * "and" operations below type check properly.
    */
   if (old_index->type != glsl_type::int_type) {
      assert (old_index->type == glsl_type::uint_type);
      old_index = new(ctx) ir_expression(ir_unop_u2i, old_index);
   }

   ir_constant *old_index_constant =
      old_index->constant_expression_value(ctx);
   if (old_index_constant) {
      /* gl_ClipDistance is being accessed via a constant index.  Don't bother
       * creating expressions to calculate the lowered indices.  Just create
       * constants.
       */
      int const_val = old_index_constant->get_int_component(0) + offset;
      array_index = new(ctx) ir_constant(const_val / 4);
      swizzle_index = new(ctx) ir_constant(const_val % 4);
   } else {
      /* Create a variable to hold the value of old_index (so that we
       * don't compute it twice).
       */
      ir_variable *old_index_var = new(ctx) ir_variable(
         glsl_type::int_type, "distance_index", ir_var_temporary);
      this->base_ir->insert_before(old_index_var);
      this->base_ir->insert_before(new(ctx) ir_assignment(
         new(ctx) ir_dereference_variable(old_index_var), old_index));

      /* Create the expression distance_index / 4.  Do this as a bit
       * shift because that's likely to be more efficient.
       */
      array_index = new(ctx) ir_expression(
         ir_binop_rshift,
         new(ctx) ir_expression(ir_binop_add,
                                new(ctx) ir_dereference_variable(old_index_var),
                                new(ctx) ir_constant(offset)),
         new(ctx) ir_constant(2));

      /* Create the expression distance_index % 4.  Do this as a bitwise
       * AND because that's likely to be more efficient.
       */
      swizzle_index = new(ctx) ir_expression(
         ir_binop_bit_and,
         new(ctx) ir_expression(ir_binop_add,
                                new(ctx) ir_dereference_variable(old_index_var),
                                new(ctx) ir_constant(offset)),
         new(ctx) ir_constant(3));
   }
}


/**
 * Determine whether the given rvalue describes an array of 8 floats that
 * needs to be lowered to an array of 2 vec4's; that is, determine whether it
 * matches one of the following patterns:
 *
 * - gl_ClipDistance (if gl_ClipDistance is 1D)
 * - gl_ClipDistance[i] (if gl_ClipDistance is 2D)
 */
bool
lower_distance_visitor::is_distance_vec8(ir_rvalue *ir)
{
   /* Note that geometry shaders contain gl_ClipDistance both as an input
    * (which is a 2D array) and an output (which is a 1D array), so it's
    * possible for both this->old_distance_out_var and
    * this->old_distance_in_var to be non-NULL in the same shader.
    */

   if (!ir->type->is_array())
      return false;
   if (ir->type->fields.array != glsl_type::float_type)
      return false;

   if (this->old_distance_out_var) {
      if (ir->variable_referenced() == this->old_distance_out_var)
         return true;
   }
   if (this->old_distance_in_var) {
      assert(this->shader_stage == MESA_SHADER_TESS_CTRL ||
             this->shader_stage == MESA_SHADER_TESS_EVAL ||
             this->shader_stage == MESA_SHADER_GEOMETRY ||
             this->shader_stage == MESA_SHADER_FRAGMENT);

      if (ir->variable_referenced() == this->old_distance_in_var)
         return true;
   }
   return false;
}


/**
 * If the given ir satisfies is_distance_vec8(), return new ir
 * representing its lowered equivalent.  That is, map:
 *
 * - gl_ClipDistance    => gl_ClipDistanceMESA    (if gl_ClipDistance is 1D)
 * - gl_ClipDistance[i] => gl_ClipDistanceMESA[i] (if gl_ClipDistance is 2D)
 *
 * Otherwise return NULL.
 */
ir_rvalue *
lower_distance_visitor::lower_distance_vec8(ir_rvalue *ir)
{
   if (!ir->type->is_array())
      return NULL;
   if (ir->type->fields.array != glsl_type::float_type)
      return NULL;

   ir_variable **new_var = NULL;
   if (this->old_distance_out_var) {
      if (ir->variable_referenced() == this->old_distance_out_var)
         new_var = &this->new_distance_out_var;
   }
   if (this->old_distance_in_var) {
      if (ir->variable_referenced() == this->old_distance_in_var)
         new_var = &this->new_distance_in_var;
   }
   if (new_var == NULL)
      return NULL;

   if (ir->as_dereference_variable()) {
      return new(ralloc_parent(ir)) ir_dereference_variable(*new_var);
   } else {
      ir_dereference_array *array_ref = ir->as_dereference_array();
      assert(array_ref);
      assert(array_ref->array->as_dereference_variable());

      return new(ralloc_parent(ir))
         ir_dereference_array(*new_var, array_ref->array_index);
   }
}


void
lower_distance_visitor::handle_rvalue(ir_rvalue **rv)
{
   if (*rv == NULL)
      return;

   ir_dereference_array *const array_deref = (*rv)->as_dereference_array();
   if (array_deref == NULL)
      return;

   /* Replace any expression that indexes one of the floats in gl_ClipDistance
    * with an expression that indexes into one of the vec4's in
    * gl_ClipDistanceMESA and accesses the appropriate component.
    */
   ir_rvalue *lowered_vec8 =
      this->lower_distance_vec8(array_deref->array);
   if (lowered_vec8 != NULL) {
      this->progress = true;
      ir_rvalue *array_index;
      ir_rvalue *swizzle_index;
      this->create_indices(array_deref->array_index, array_index, swizzle_index);
      void *mem_ctx = ralloc_parent(array_deref);

      ir_dereference_array *const new_array_deref =
         new(mem_ctx) ir_dereference_array(lowered_vec8, array_index);

      ir_expression *const expr =
         new(mem_ctx) ir_expression(ir_binop_vector_extract,
                                    new_array_deref,
                                    swizzle_index);

      *rv = expr;
   }
}

void
lower_distance_visitor::fix_lhs(ir_assignment *ir)
{
   if (ir->lhs->ir_type == ir_type_expression) {
      void *mem_ctx = ralloc_parent(ir);
      ir_expression *const expr = (ir_expression *) ir->lhs;

      /* The expression must be of the form:
       *
       *     (vector_extract gl_ClipDistanceMESA[i], j).
       */
      assert(expr->operation == ir_binop_vector_extract);
      assert(expr->operands[0]->ir_type == ir_type_dereference_array);
      assert(expr->operands[0]->type == glsl_type::vec4_type);

      ir_dereference *const new_lhs = (ir_dereference *) expr->operands[0];
      ir->rhs = new(mem_ctx) ir_expression(ir_triop_vector_insert,
                                           glsl_type::vec4_type,
                                           new_lhs->clone(mem_ctx, NULL),
                                           ir->rhs,
                                           expr->operands[1]);
      ir->set_lhs(new_lhs);
      ir->write_mask = WRITEMASK_XYZW;
   }
}

/**
 * Replace any assignment having the 1D gl_ClipDistance (undereferenced) as
 * its LHS or RHS with a sequence of assignments, one for each component of
 * the array.  Each of these assignments is lowered to refer to
 * gl_ClipDistanceMESA as appropriate.
 *
 * We need to do a similar replacement for 2D gl_ClipDistance, however since
 * it's an input, the only case we need to address is where a 1D slice of it
 * is the entire RHS of an assignment, e.g.:
 *
 *     foo = gl_in[i].gl_ClipDistance
 */
ir_visitor_status
lower_distance_visitor::visit_leave(ir_assignment *ir)
{
   /* First invoke the base class visitor.  This causes handle_rvalue() to be
    * called on ir->rhs and ir->condition.
    */
   ir_rvalue_visitor::visit_leave(ir);

   if (this->is_distance_vec8(ir->lhs) ||
       this->is_distance_vec8(ir->rhs)) {
      /* LHS or RHS of the assignment is the entire 1D gl_ClipDistance array
       * (or a 1D slice of a 2D gl_ClipDistance input array).  Since we are
       * reshaping gl_ClipDistance from an array of floats to an array of
       * vec4's, this isn't going to work as a bulk assignment anymore, so
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
lower_distance_visitor::visit_new_assignment(ir_assignment *ir)
{
   ir_instruction *old_base_ir = this->base_ir;
   this->base_ir = ir;
   ir->accept(this);
   this->base_ir = old_base_ir;
}


/**
 * If a 1D gl_ClipDistance variable appears as an argument in an ir_call
 * expression, replace it with a temporary variable, and make sure the ir_call
 * is preceded and/or followed by assignments that copy the contents of the
 * temporary variable to and/or from gl_ClipDistance.  Each of these
 * assignments is then lowered to refer to gl_ClipDistanceMESA.
 *
 * We need to do a similar replacement for 2D gl_ClipDistance, however since
 * it's an input, the only case we need to address is where a 1D slice of it
 * is passed as an "in" parameter to an ir_call, e.g.:
 *
 *     foo(gl_in[i].gl_ClipDistance)
 */
ir_visitor_status
lower_distance_visitor::visit_leave(ir_call *ir)
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

      if (this->is_distance_vec8(actual_param)) {
         /* User is trying to pass the whole 1D gl_ClipDistance array (or a 1D
          * slice of a 2D gl_ClipDistance array) to a function call.  Since we
          * are reshaping gl_ClipDistance from an array of floats to an array
          * of vec4's, this isn't going to work anymore, so use a temporary
          * array instead.
          */
         ir_variable *temp_clip_distance = new(ctx) ir_variable(
            actual_param->type, "temp_clip_distance", ir_var_temporary);
         this->base_ir->insert_before(temp_clip_distance);
         actual_param->replace_with(
            new(ctx) ir_dereference_variable(temp_clip_distance));
         if (formal_param->data.mode == ir_var_function_in
             || formal_param->data.mode == ir_var_function_inout) {
            /* Copy from gl_ClipDistance to the temporary before the call.
             * Since we are going to insert this copy before the current
             * instruction, we need to visit it afterwards to make sure it
             * gets lowered.
             */
            ir_assignment *new_assignment = new(ctx) ir_assignment(
               new(ctx) ir_dereference_variable(temp_clip_distance),
               actual_param->clone(ctx, NULL));
            this->base_ir->insert_before(new_assignment);
            this->visit_new_assignment(new_assignment);
         }
         if (formal_param->data.mode == ir_var_function_out
             || formal_param->data.mode == ir_var_function_inout) {
            /* Copy from the temporary to gl_ClipDistance after the call.
             * Since visit_list_elements() has already decided which
             * instruction it's going to visit next, we need to visit
             * afterwards to make sure it gets lowered.
             */
            ir_assignment *new_assignment = new(ctx) ir_assignment(
               actual_param->clone(ctx, NULL),
               new(ctx) ir_dereference_variable(temp_clip_distance));
            this->base_ir->insert_after(new_assignment);
            this->visit_new_assignment(new_assignment);
         }
      }
   }

   return rvalue_visit(ir);
}

namespace {
class lower_distance_visitor_counter : public ir_rvalue_visitor {
public:
   explicit lower_distance_visitor_counter(void)
      : in_clip_size(0), in_cull_size(0),
        out_clip_size(0), out_cull_size(0)
   {
   }

   virtual ir_visitor_status visit(ir_variable *);
   virtual void handle_rvalue(ir_rvalue **rvalue);

   int in_clip_size;
   int in_cull_size;
   int out_clip_size;
   int out_cull_size;
};

}
/**
 * Count gl_ClipDistance and gl_CullDistance sizes.
 */
ir_visitor_status
lower_distance_visitor_counter::visit(ir_variable *ir)
{
   int *clip_size, *cull_size;

   if (!ir->name)
      return visit_continue;

   if (ir->data.mode == ir_var_shader_out) {
      clip_size = &out_clip_size;
      cull_size = &out_cull_size;
   } else if (ir->data.mode == ir_var_shader_in) {
      clip_size = &in_clip_size;
      cull_size = &in_cull_size;
   } else
      return visit_continue;

   if (ir->type->is_unsized_array())
      return visit_continue;

   if (*clip_size == 0) {
      if (!strcmp(ir->name, "gl_ClipDistance")) {
         if (!ir->type->fields.array->is_array())
            *clip_size = ir->type->array_size();
         else
            *clip_size = ir->type->fields.array->array_size();
      }
   }

   if (*cull_size == 0) {
      if (!strcmp(ir->name, "gl_CullDistance")) {
         if (!ir->type->fields.array->is_array())
            *cull_size = ir->type->array_size();
         else
            *cull_size = ir->type->fields.array->array_size();
      }
   }
   return visit_continue;
}

void
lower_distance_visitor_counter::handle_rvalue(ir_rvalue **)
{
   return;
}

bool
lower_clip_cull_distance(struct gl_shader_program *prog,
                         struct gl_linked_shader *shader)
{
   int clip_size, cull_size;

   lower_distance_visitor_counter count;
   visit_list_elements(&count, shader->ir);

   clip_size = MAX2(count.in_clip_size, count.out_clip_size);
   cull_size = MAX2(count.in_cull_size, count.out_cull_size);

   if (clip_size == 0 && cull_size == 0)
      return false;

   lower_distance_visitor v(shader->Stage, "gl_ClipDistance", clip_size + cull_size, 0);
   visit_list_elements(&v, shader->ir);

   lower_distance_visitor v2(shader->Stage, "gl_CullDistance", &v, clip_size);
   visit_list_elements(&v2, shader->ir);

   if (v2.new_distance_out_var)
      shader->symbols->add_variable(v2.new_distance_out_var);
   if (v2.new_distance_in_var)
      shader->symbols->add_variable(v2.new_distance_in_var);

   return v2.progress;
}
