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
 * \file lower_mat_op_to_vec.cpp
 *
 * Breaks matrix operation expressions down to a series of vector operations.
 *
 * Generally this is how we have to codegen matrix operations for a
 * GPU, so this gives us the chance to constant fold operations on a
 * column or row.
 */

#include "ir.h"
#include "ir_expression_flattening.h"
#include "compiler/glsl_types.h"

namespace {

class ir_mat_op_to_vec_visitor : public ir_hierarchical_visitor {
public:
   ir_mat_op_to_vec_visitor()
   {
      this->made_progress = false;
      this->mem_ctx = NULL;
   }

   ir_visitor_status visit_leave(ir_assignment *);

   ir_dereference *get_column(ir_dereference *val, int col);
   ir_rvalue *get_element(ir_dereference *val, int col, int row);

   void do_mul_mat_mat(ir_dereference *result,
		       ir_dereference *a, ir_dereference *b);
   void do_mul_mat_vec(ir_dereference *result,
		       ir_dereference *a, ir_dereference *b);
   void do_mul_vec_mat(ir_dereference *result,
		       ir_dereference *a, ir_dereference *b);
   void do_mul_mat_scalar(ir_dereference *result,
			  ir_dereference *a, ir_dereference *b);
   void do_equal_mat_mat(ir_dereference *result, ir_dereference *a,
			 ir_dereference *b, bool test_equal);

   void *mem_ctx;
   bool made_progress;
};

} /* anonymous namespace */

static bool
mat_op_to_vec_predicate(ir_instruction *ir)
{
   ir_expression *expr = ir->as_expression();
   unsigned int i;

   if (!expr)
      return false;

   for (i = 0; i < expr->num_operands; i++) {
      if (expr->operands[i]->type->is_matrix())
	 return true;
   }

   return false;
}

bool
do_mat_op_to_vec(exec_list *instructions)
{
   ir_mat_op_to_vec_visitor v;

   /* Pull out any matrix expression to a separate assignment to a
    * temp.  This will make our handling of the breakdown to
    * operations on the matrix's vector components much easier.
    */
   do_expression_flattening(instructions, mat_op_to_vec_predicate);

   visit_list_elements(&v, instructions);

   return v.made_progress;
}

ir_rvalue *
ir_mat_op_to_vec_visitor::get_element(ir_dereference *val, int col, int row)
{
   val = get_column(val, col);

   return new(mem_ctx) ir_swizzle(val, row, 0, 0, 0, 1);
}

ir_dereference *
ir_mat_op_to_vec_visitor::get_column(ir_dereference *val, int row)
{
   val = val->clone(mem_ctx, NULL);

   if (val->type->is_matrix()) {
      val = new(mem_ctx) ir_dereference_array(val,
					      new(mem_ctx) ir_constant(row));
   }

   return val;
}

void
ir_mat_op_to_vec_visitor::do_mul_mat_mat(ir_dereference *result,
					 ir_dereference *a,
					 ir_dereference *b)
{
   unsigned b_col, i;
   ir_assignment *assign;
   ir_expression *expr;

   for (b_col = 0; b_col < b->type->matrix_columns; b_col++) {
      /* first column */
      expr = new(mem_ctx) ir_expression(ir_binop_mul,
					get_column(a, 0),
					get_element(b, b_col, 0));

      /* following columns */
      for (i = 1; i < a->type->matrix_columns; i++) {
	 ir_expression *mul_expr;

	 mul_expr = new(mem_ctx) ir_expression(ir_binop_mul,
					       get_column(a, i),
					       get_element(b, b_col, i));
	 expr = new(mem_ctx) ir_expression(ir_binop_add,
					   expr,
					   mul_expr);
      }

      assign = new(mem_ctx) ir_assignment(get_column(result, b_col), expr);
      base_ir->insert_before(assign);
   }
}

void
ir_mat_op_to_vec_visitor::do_mul_mat_vec(ir_dereference *result,
					 ir_dereference *a,
					 ir_dereference *b)
{
   unsigned i;
   ir_assignment *assign;
   ir_expression *expr;

   /* first column */
   expr = new(mem_ctx) ir_expression(ir_binop_mul,
				     get_column(a, 0),
				     get_element(b, 0, 0));

   /* following columns */
   for (i = 1; i < a->type->matrix_columns; i++) {
      ir_expression *mul_expr;

      mul_expr = new(mem_ctx) ir_expression(ir_binop_mul,
					    get_column(a, i),
					    get_element(b, 0, i));
      expr = new(mem_ctx) ir_expression(ir_binop_add, expr, mul_expr);
   }

   result = result->clone(mem_ctx, NULL);
   assign = new(mem_ctx) ir_assignment(result, expr);
   base_ir->insert_before(assign);
}

void
ir_mat_op_to_vec_visitor::do_mul_vec_mat(ir_dereference *result,
					 ir_dereference *a,
					 ir_dereference *b)
{
   unsigned i;

   for (i = 0; i < b->type->matrix_columns; i++) {
      ir_rvalue *column_result;
      ir_expression *column_expr;
      ir_assignment *column_assign;

      column_result = result->clone(mem_ctx, NULL);
      column_result = new(mem_ctx) ir_swizzle(column_result, i, 0, 0, 0, 1);

      column_expr = new(mem_ctx) ir_expression(ir_binop_dot,
					       a->clone(mem_ctx, NULL),
					       get_column(b, i));

      column_assign = new(mem_ctx) ir_assignment(column_result,
						 column_expr);
      base_ir->insert_before(column_assign);
   }
}

void
ir_mat_op_to_vec_visitor::do_mul_mat_scalar(ir_dereference *result,
					    ir_dereference *a,
					    ir_dereference *b)
{
   unsigned i;

   for (i = 0; i < a->type->matrix_columns; i++) {
      ir_expression *column_expr;
      ir_assignment *column_assign;

      column_expr = new(mem_ctx) ir_expression(ir_binop_mul,
					       get_column(a, i),
					       b->clone(mem_ctx, NULL));

      column_assign = new(mem_ctx) ir_assignment(get_column(result, i),
						 column_expr);
      base_ir->insert_before(column_assign);
   }
}

void
ir_mat_op_to_vec_visitor::do_equal_mat_mat(ir_dereference *result,
					   ir_dereference *a,
					   ir_dereference *b,
					   bool test_equal)
{
   /* This essentially implements the following GLSL:
    *
    * bool equal(mat4 a, mat4 b)
    * {
    *   return !any(bvec4(a[0] != b[0],
    *                     a[1] != b[1],
    *                     a[2] != b[2],
    *                     a[3] != b[3]);
    * }
    *
    * bool nequal(mat4 a, mat4 b)
    * {
    *   return any(bvec4(a[0] != b[0],
    *                    a[1] != b[1],
    *                    a[2] != b[2],
    *                    a[3] != b[3]);
    * }
    */
   const unsigned columns = a->type->matrix_columns;
   const glsl_type *const bvec_type =
      glsl_type::get_instance(GLSL_TYPE_BOOL, columns, 1);

   ir_variable *const tmp_bvec =
      new(this->mem_ctx) ir_variable(bvec_type, "mat_cmp_bvec",
				     ir_var_temporary);
   this->base_ir->insert_before(tmp_bvec);

   for (unsigned i = 0; i < columns; i++) {
      ir_expression *const cmp =
	 new(this->mem_ctx) ir_expression(ir_binop_any_nequal,
					  get_column(a, i),
					  get_column(b, i));

      ir_dereference *const lhs =
	 new(this->mem_ctx) ir_dereference_variable(tmp_bvec);

      ir_assignment *const assign =
	 new(this->mem_ctx) ir_assignment(lhs, cmp, NULL, (1U << i));

      this->base_ir->insert_before(assign);
   }

   ir_rvalue *const val = new(this->mem_ctx) ir_dereference_variable(tmp_bvec);
   uint8_t vec_elems = val->type->vector_elements;
   ir_expression *any =
      new(this->mem_ctx) ir_expression(ir_binop_any_nequal, val,
                                       new(this->mem_ctx) ir_constant(false,
                                                                      vec_elems));

   if (test_equal)
      any = new(this->mem_ctx) ir_expression(ir_unop_logic_not, any);

   ir_assignment *const assign =
      new(mem_ctx) ir_assignment(result->clone(mem_ctx, NULL), any);
   base_ir->insert_before(assign);
}

static bool
has_matrix_operand(const ir_expression *expr, unsigned &columns)
{
   for (unsigned i = 0; i < expr->num_operands; i++) {
      if (expr->operands[i]->type->is_matrix()) {
	 columns = expr->operands[i]->type->matrix_columns;
	 return true;
      }
   }

   return false;
}


ir_visitor_status
ir_mat_op_to_vec_visitor::visit_leave(ir_assignment *orig_assign)
{
   ir_expression *orig_expr = orig_assign->rhs->as_expression();
   unsigned int i, matrix_columns = 1;
   ir_dereference *op[2];

   if (!orig_expr)
      return visit_continue;

   if (!has_matrix_operand(orig_expr, matrix_columns))
      return visit_continue;

   assert(orig_expr->num_operands <= 2);

   mem_ctx = ralloc_parent(orig_assign);

   ir_dereference_variable *result =
      orig_assign->lhs->as_dereference_variable();
   assert(result);

   /* Store the expression operands in temps so we can use them
    * multiple times.
    */
   for (i = 0; i < orig_expr->num_operands; i++) {
      ir_assignment *assign;
      ir_dereference *deref = orig_expr->operands[i]->as_dereference();

      /* Avoid making a temporary if we don't need to to avoid aliasing. */
      if (deref &&
	  deref->variable_referenced() != result->variable_referenced()) {
	 op[i] = deref;
	 continue;
      }

      /* Otherwise, store the operand in a temporary generally if it's
       * not a dereference.
       */
      ir_variable *var = new(mem_ctx) ir_variable(orig_expr->operands[i]->type,
						  "mat_op_to_vec",
						  ir_var_temporary);
      base_ir->insert_before(var);

      /* Note that we use this dereference for the assignment.  That means
       * that others that want to use op[i] have to clone the deref.
       */
      op[i] = new(mem_ctx) ir_dereference_variable(var);
      assign = new(mem_ctx) ir_assignment(op[i], orig_expr->operands[i]);
      base_ir->insert_before(assign);
   }

   /* OK, time to break down this matrix operation. */
   switch (orig_expr->operation) {
   case ir_unop_d2f:
   case ir_unop_f2d:
   case ir_unop_f2f16:
   case ir_unop_f2fmp:
   case ir_unop_f162f:
   case ir_unop_neg: {
      /* Apply the operation to each column.*/
      for (i = 0; i < matrix_columns; i++) {
	 ir_expression *column_expr;
	 ir_assignment *column_assign;

	 column_expr = new(mem_ctx) ir_expression(orig_expr->operation,
						  get_column(op[0], i));

	 column_assign = new(mem_ctx) ir_assignment(get_column(result, i),
						    column_expr);
	 assert(column_assign->write_mask != 0);
	 base_ir->insert_before(column_assign);
      }
      break;
   }
   case ir_binop_add:
   case ir_binop_sub:
   case ir_binop_div:
   case ir_binop_mod: {
      /* For most operations, the matrix version is just going
       * column-wise through and applying the operation to each column
       * if available.
       */
      for (i = 0; i < matrix_columns; i++) {
	 ir_expression *column_expr;
	 ir_assignment *column_assign;

	 column_expr = new(mem_ctx) ir_expression(orig_expr->operation,
						  get_column(op[0], i),
						  get_column(op[1], i));

	 column_assign = new(mem_ctx) ir_assignment(get_column(result, i),
						    column_expr);
	 assert(column_assign->write_mask != 0);
	 base_ir->insert_before(column_assign);
      }
      break;
   }
   case ir_binop_mul:
      if (op[0]->type->is_matrix()) {
	 if (op[1]->type->is_matrix()) {
	    do_mul_mat_mat(result, op[0], op[1]);
	 } else if (op[1]->type->is_vector()) {
	    do_mul_mat_vec(result, op[0], op[1]);
	 } else {
	    assert(op[1]->type->is_scalar());
	    do_mul_mat_scalar(result, op[0], op[1]);
	 }
      } else {
	 assert(op[1]->type->is_matrix());
	 if (op[0]->type->is_vector()) {
	    do_mul_vec_mat(result, op[0], op[1]);
	 } else {
	    assert(op[0]->type->is_scalar());
	    do_mul_mat_scalar(result, op[1], op[0]);
	 }
      }
      break;

   case ir_binop_all_equal:
   case ir_binop_any_nequal:
      do_equal_mat_mat(result, op[1], op[0],
		       (orig_expr->operation == ir_binop_all_equal));
      break;

   default:
      printf("FINISHME: Handle matrix operation for %s\n",
	     ir_expression_operation_strings[orig_expr->operation]);
      abort();
   }
   orig_assign->remove();
   this->made_progress = true;

   return visit_continue;
}
