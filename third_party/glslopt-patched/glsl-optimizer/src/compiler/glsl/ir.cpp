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
#include <string.h>
#include "ir.h"
#include "util/half_float.h"
#include "compiler/glsl_types.h"
#include "glsl_parser_extras.h"


ir_rvalue::ir_rvalue(enum ir_node_type t)
   : ir_instruction(t)
{
   this->type = glsl_type::error_type;
}

bool ir_rvalue::is_zero() const
{
   return false;
}

bool ir_rvalue::is_one() const
{
   return false;
}

bool ir_rvalue::is_negative_one() const
{
   return false;
}

/**
 * Modify the swizzle make to move one component to another
 *
 * \param m    IR swizzle to be modified
 * \param from Component in the RHS that is to be swizzled
 * \param to   Desired swizzle location of \c from
 */
static void
update_rhs_swizzle(ir_swizzle_mask &m, unsigned from, unsigned to)
{
   switch (to) {
   case 0: m.x = from; break;
   case 1: m.y = from; break;
   case 2: m.z = from; break;
   case 3: m.w = from; break;
   default: assert(!"Should not get here.");
   }
}

void
ir_assignment::set_lhs(ir_rvalue *lhs)
{
   void *mem_ctx = this;
   bool swizzled = false;

   while (lhs != NULL) {
      ir_swizzle *swiz = lhs->as_swizzle();

      if (swiz == NULL)
	 break;

      unsigned write_mask = 0;
      ir_swizzle_mask rhs_swiz = { 0, 0, 0, 0, 0, 0 };

      for (unsigned i = 0; i < swiz->mask.num_components; i++) {
	 unsigned c = 0;

	 switch (i) {
	 case 0: c = swiz->mask.x; break;
	 case 1: c = swiz->mask.y; break;
	 case 2: c = swiz->mask.z; break;
	 case 3: c = swiz->mask.w; break;
	 default: assert(!"Should not get here.");
	 }

	 write_mask |= (((this->write_mask >> i) & 1) << c);
	 update_rhs_swizzle(rhs_swiz, i, c);
         rhs_swiz.num_components = swiz->val->type->vector_elements;
      }

      this->write_mask = write_mask;
      lhs = swiz->val;

      this->rhs = new(mem_ctx) ir_swizzle(this->rhs, rhs_swiz);
      swizzled = true;
   }

   if (swizzled) {
      /* Now, RHS channels line up with the LHS writemask.  Collapse it
       * to just the channels that will be written.
       */
      ir_swizzle_mask rhs_swiz = { 0, 0, 0, 0, 0, 0 };
      int rhs_chan = 0;
      for (int i = 0; i < 4; i++) {
	 if (write_mask & (1 << i))
	    update_rhs_swizzle(rhs_swiz, i, rhs_chan++);
      }
      rhs_swiz.num_components = rhs_chan;
      this->rhs = new(mem_ctx) ir_swizzle(this->rhs, rhs_swiz);
   }

   assert((lhs == NULL) || lhs->as_dereference());

   this->lhs = (ir_dereference *) lhs;
}

ir_variable *
ir_assignment::whole_variable_written()
{
   ir_variable *v = this->lhs->whole_variable_referenced();

   if (v == NULL)
      return NULL;

   if (v->type->is_scalar())
      return v;

   if (v->type->is_vector()) {
      const unsigned mask = (1U << v->type->vector_elements) - 1;

      if (mask != this->write_mask)
	 return NULL;
   }

   /* Either all the vector components are assigned or the variable is some
    * composite type (and the whole thing is assigned.
    */
   return v;
}

ir_assignment::ir_assignment(ir_dereference *lhs, ir_rvalue *rhs,
			     ir_rvalue *condition, unsigned write_mask)
   : ir_instruction(ir_type_assignment)
{
   this->condition = condition;
   this->rhs = rhs;
   this->lhs = lhs;
   this->write_mask = write_mask;

   if (lhs->type->is_scalar() || lhs->type->is_vector()) {
      int lhs_components = 0;
      for (int i = 0; i < 4; i++) {
	 if (write_mask & (1 << i))
	    lhs_components++;
      }

      assert(lhs_components == this->rhs->type->vector_elements);
   }
}

ir_assignment::ir_assignment(ir_rvalue *lhs, ir_rvalue *rhs,
			     ir_rvalue *condition)
   : ir_instruction(ir_type_assignment)
{
   this->condition = condition;
   this->rhs = rhs;

   /* If the RHS is a vector type, assume that all components of the vector
    * type are being written to the LHS.  The write mask comes from the RHS
    * because we can have a case where the LHS is a vec4 and the RHS is a
    * vec3.  In that case, the assignment is:
    *
    *     (assign (...) (xyz) (var_ref lhs) (var_ref rhs))
    */
   if (rhs->type->is_vector())
      this->write_mask = (1U << rhs->type->vector_elements) - 1;
   else if (rhs->type->is_scalar())
      this->write_mask = 1;
   else
      this->write_mask = 0;

   this->set_lhs(lhs);
}

ir_expression::ir_expression(int op, const struct glsl_type *type,
			     ir_rvalue *op0, ir_rvalue *op1,
			     ir_rvalue *op2, ir_rvalue *op3)
   : ir_rvalue(ir_type_expression)
{
   this->type = type;
   this->operation = ir_expression_operation(op);
   this->operands[0] = op0;
   this->operands[1] = op1;
   this->operands[2] = op2;
   this->operands[3] = op3;
   init_num_operands();

#ifndef NDEBUG
   for (unsigned i = num_operands; i < 4; i++) {
      assert(this->operands[i] == NULL);
   }

   for (unsigned i = 0; i < num_operands; i++) {
      assert(this->operands[i] != NULL);
   }
#endif
}

ir_expression::ir_expression(int op, ir_rvalue *op0)
   : ir_rvalue(ir_type_expression)
{
   this->operation = ir_expression_operation(op);
   this->operands[0] = op0;
   this->operands[1] = NULL;
   this->operands[2] = NULL;
   this->operands[3] = NULL;

   assert(op <= ir_last_unop);
   init_num_operands();
   assert(num_operands == 1);
   assert(this->operands[0]);

   switch (this->operation) {
   case ir_unop_bit_not:
   case ir_unop_logic_not:
   case ir_unop_neg:
   case ir_unop_abs:
   case ir_unop_sign:
   case ir_unop_rcp:
   case ir_unop_rsq:
   case ir_unop_sqrt:
   case ir_unop_exp:
   case ir_unop_log:
   case ir_unop_exp2:
   case ir_unop_log2:
   case ir_unop_trunc:
   case ir_unop_ceil:
   case ir_unop_floor:
   case ir_unop_fract:
   case ir_unop_round_even:
   case ir_unop_sin:
   case ir_unop_cos:
   case ir_unop_dFdx:
   case ir_unop_dFdx_coarse:
   case ir_unop_dFdx_fine:
   case ir_unop_dFdy:
   case ir_unop_dFdy_coarse:
   case ir_unop_dFdy_fine:
   case ir_unop_bitfield_reverse:
   case ir_unop_interpolate_at_centroid:
   case ir_unop_clz:
   case ir_unop_saturate:
   case ir_unop_atan:
      this->type = op0->type;
      break;

   case ir_unop_f2i:
   case ir_unop_b2i:
   case ir_unop_u2i:
   case ir_unop_d2i:
   case ir_unop_bitcast_f2i:
   case ir_unop_bit_count:
   case ir_unop_find_msb:
   case ir_unop_find_lsb:
   case ir_unop_subroutine_to_int:
   case ir_unop_i642i:
   case ir_unop_u642i:
      this->type = glsl_type::get_instance(GLSL_TYPE_INT,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_b2f:
   case ir_unop_i2f:
   case ir_unop_u2f:
   case ir_unop_d2f:
   case ir_unop_f162f:
   case ir_unop_bitcast_i2f:
   case ir_unop_bitcast_u2f:
   case ir_unop_i642f:
   case ir_unop_u642f:
      this->type = glsl_type::get_instance(GLSL_TYPE_FLOAT,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_f2f16:
   case ir_unop_f2fmp:
   case ir_unop_b2f16:
      this->type = glsl_type::get_instance(GLSL_TYPE_FLOAT16,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_f2b:
   case ir_unop_i2b:
   case ir_unop_d2b:
   case ir_unop_f162b:
   case ir_unop_i642b:
      this->type = glsl_type::get_instance(GLSL_TYPE_BOOL,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_f2d:
   case ir_unop_i2d:
   case ir_unop_u2d:
   case ir_unop_i642d:
   case ir_unop_u642d:
      this->type = glsl_type::get_instance(GLSL_TYPE_DOUBLE,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_i2u:
   case ir_unop_f2u:
   case ir_unop_d2u:
   case ir_unop_bitcast_f2u:
   case ir_unop_i642u:
   case ir_unop_u642u:
      this->type = glsl_type::get_instance(GLSL_TYPE_UINT,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_i2i64:
   case ir_unop_u2i64:
   case ir_unop_b2i64:
   case ir_unop_f2i64:
   case ir_unop_d2i64:
   case ir_unop_u642i64:
      this->type = glsl_type::get_instance(GLSL_TYPE_INT64,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_i2u64:
   case ir_unop_u2u64:
   case ir_unop_f2u64:
   case ir_unop_d2u64:
   case ir_unop_i642u64:
      this->type = glsl_type::get_instance(GLSL_TYPE_UINT64,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_unpack_double_2x32:
   case ir_unop_unpack_uint_2x32:
      this->type = glsl_type::uvec2_type;
      break;

   case ir_unop_unpack_int_2x32:
      this->type = glsl_type::ivec2_type;
      break;

   case ir_unop_pack_snorm_2x16:
   case ir_unop_pack_snorm_4x8:
   case ir_unop_pack_unorm_2x16:
   case ir_unop_pack_unorm_4x8:
   case ir_unop_pack_half_2x16:
      this->type = glsl_type::uint_type;
      break;

   case ir_unop_pack_double_2x32:
      this->type = glsl_type::double_type;
      break;

   case ir_unop_pack_int_2x32:
      this->type = glsl_type::int64_t_type;
      break;

   case ir_unop_pack_uint_2x32:
      this->type = glsl_type::uint64_t_type;
      break;

   case ir_unop_unpack_snorm_2x16:
   case ir_unop_unpack_unorm_2x16:
   case ir_unop_unpack_half_2x16:
      this->type = glsl_type::vec2_type;
      break;

   case ir_unop_unpack_snorm_4x8:
   case ir_unop_unpack_unorm_4x8:
      this->type = glsl_type::vec4_type;
      break;

   case ir_unop_unpack_sampler_2x32:
   case ir_unop_unpack_image_2x32:
      this->type = glsl_type::uvec2_type;
      break;

   case ir_unop_pack_sampler_2x32:
   case ir_unop_pack_image_2x32:
      this->type = op0->type;
      break;

   case ir_unop_frexp_sig:
      this->type = op0->type;
      break;
   case ir_unop_frexp_exp:
      this->type = glsl_type::get_instance(GLSL_TYPE_INT,
					   op0->type->vector_elements, 1);
      break;

   case ir_unop_get_buffer_size:
   case ir_unop_ssbo_unsized_array_length:
      this->type = glsl_type::int_type;
      break;

   case ir_unop_bitcast_i642d:
   case ir_unop_bitcast_u642d:
      this->type = glsl_type::get_instance(GLSL_TYPE_DOUBLE,
                                           op0->type->vector_elements, 1);
      break;

   case ir_unop_bitcast_d2i64:
      this->type = glsl_type::get_instance(GLSL_TYPE_INT64,
                                           op0->type->vector_elements, 1);
      break;
   case ir_unop_bitcast_d2u64:
      this->type = glsl_type::get_instance(GLSL_TYPE_UINT64,
                                           op0->type->vector_elements, 1);
      break;

   default:
      assert(!"not reached: missing automatic type setup for ir_expression");
      this->type = op0->type;
      break;
   }
}

ir_expression::ir_expression(int op, ir_rvalue *op0, ir_rvalue *op1)
   : ir_rvalue(ir_type_expression)
{
   this->operation = ir_expression_operation(op);
   this->operands[0] = op0;
   this->operands[1] = op1;
   this->operands[2] = NULL;
   this->operands[3] = NULL;

   assert(op > ir_last_unop);
   init_num_operands();
   assert(num_operands == 2);
   for (unsigned i = 0; i < num_operands; i++) {
      assert(this->operands[i] != NULL);
   }

   switch (this->operation) {
   case ir_binop_all_equal:
   case ir_binop_any_nequal:
      this->type = glsl_type::bool_type;
      break;

   case ir_binop_add:
   case ir_binop_sub:
   case ir_binop_min:
   case ir_binop_max:
   case ir_binop_pow:
   case ir_binop_mul:
   case ir_binop_div:
   case ir_binop_mod:
   case ir_binop_atan2:
      if (op0->type->is_scalar()) {
	 this->type = op1->type;
      } else if (op1->type->is_scalar()) {
	 this->type = op0->type;
      } else {
         if (this->operation == ir_binop_mul) {
            this->type = glsl_type::get_mul_type(op0->type, op1->type);
         } else {
            assert(op0->type == op1->type);
            this->type = op0->type;
         }
      }
      break;

   case ir_binop_logic_and:
   case ir_binop_logic_xor:
   case ir_binop_logic_or:
   case ir_binop_bit_and:
   case ir_binop_bit_xor:
   case ir_binop_bit_or:
       assert(!op0->type->is_matrix());
       assert(!op1->type->is_matrix());
      if (op0->type->is_scalar()) {
         this->type = op1->type;
      } else if (op1->type->is_scalar()) {
         this->type = op0->type;
      } else {
          assert(op0->type->vector_elements == op1->type->vector_elements);
          this->type = op0->type;
      }
      break;

   case ir_binop_equal:
   case ir_binop_nequal:
   case ir_binop_gequal:
   case ir_binop_less:
      assert(op0->type == op1->type);
      this->type = glsl_type::get_instance(GLSL_TYPE_BOOL,
					   op0->type->vector_elements, 1);
      break;

   case ir_binop_dot:
      this->type = op0->type->get_base_type();
      break;

   case ir_binop_imul_high:
   case ir_binop_mul_32x16:
   case ir_binop_carry:
   case ir_binop_borrow:
   case ir_binop_lshift:
   case ir_binop_rshift:
   case ir_binop_ldexp:
   case ir_binop_interpolate_at_offset:
   case ir_binop_interpolate_at_sample:
      this->type = op0->type;
      break;

   case ir_binop_add_sat:
   case ir_binop_sub_sat:
   case ir_binop_avg:
   case ir_binop_avg_round:
      assert(op0->type == op1->type);
      this->type = op0->type;
      break;

   case ir_binop_abs_sub: {
      enum glsl_base_type base;

      assert(op0->type == op1->type);

      switch (op0->type->base_type) {
      case GLSL_TYPE_UINT:
      case GLSL_TYPE_INT:
         base = GLSL_TYPE_UINT;
         break;
      case GLSL_TYPE_UINT8:
      case GLSL_TYPE_INT8:
         base = GLSL_TYPE_UINT8;
         break;
      case GLSL_TYPE_UINT16:
      case GLSL_TYPE_INT16:
         base = GLSL_TYPE_UINT16;
         break;
      case GLSL_TYPE_UINT64:
      case GLSL_TYPE_INT64:
         base = GLSL_TYPE_UINT64;
         break;
      default:
         unreachable(!"Invalid base type.");
      }

      this->type = glsl_type::get_instance(base, op0->type->vector_elements, 1);
      break;
   }

   case ir_binop_vector_extract:
      this->type = op0->type->get_scalar_type();
      break;

   default:
      assert(!"not reached: missing automatic type setup for ir_expression");
      this->type = glsl_type::float_type;
   }
}

ir_expression::ir_expression(int op, ir_rvalue *op0, ir_rvalue *op1,
                             ir_rvalue *op2)
   : ir_rvalue(ir_type_expression)
{
   this->operation = ir_expression_operation(op);
   this->operands[0] = op0;
   this->operands[1] = op1;
   this->operands[2] = op2;
   this->operands[3] = NULL;

   assert(op > ir_last_binop && op <= ir_last_triop);
   init_num_operands();
   assert(num_operands == 3);
   for (unsigned i = 0; i < num_operands; i++) {
      assert(this->operands[i] != NULL);
   }

   switch (this->operation) {
   case ir_triop_fma:
   case ir_triop_lrp:
   case ir_triop_bitfield_extract:
   case ir_triop_vector_insert:
      this->type = op0->type;
      break;

   case ir_triop_csel:
      this->type = op1->type;
      break;

   default:
      assert(!"not reached: missing automatic type setup for ir_expression");
      this->type = glsl_type::float_type;
   }
}

/**
 * This is only here for ir_reader to used for testing purposes. Please use
 * the precomputed num_operands field if you need the number of operands.
 */
unsigned
ir_expression::get_num_operands(ir_expression_operation op)
{
   assert(op <= ir_last_opcode);

   if (op <= ir_last_unop)
      return 1;

   if (op <= ir_last_binop)
      return 2;

   if (op <= ir_last_triop)
      return 3;

   if (op <= ir_last_quadop)
      return 4;

   unreachable("Could not calculate number of operands");
}

#include "ir_expression_operation_strings.h"

const char*
depth_layout_string(ir_depth_layout layout)
{
   switch(layout) {
   case ir_depth_layout_none:      return "";
   case ir_depth_layout_any:       return "depth_any";
   case ir_depth_layout_greater:   return "depth_greater";
   case ir_depth_layout_less:      return "depth_less";
   case ir_depth_layout_unchanged: return "depth_unchanged";

   default:
      assert(0);
      return "";
   }
}

ir_expression_operation
ir_expression::get_operator(const char *str)
{
   for (int op = 0; op <= int(ir_last_opcode); op++) {
      if (strcmp(str, ir_expression_operation_strings[op]) == 0)
	 return (ir_expression_operation) op;
   }
   return (ir_expression_operation) -1;
}

ir_variable *
ir_expression::variable_referenced() const
{
   switch (operation) {
      case ir_binop_vector_extract:
      case ir_triop_vector_insert:
         /* We get these for things like a[0] where a is a vector type. In these
          * cases we want variable_referenced() to return the actual vector
          * variable this is wrapping.
          */
         return operands[0]->variable_referenced();
      default:
         return ir_rvalue::variable_referenced();
   }
}

ir_constant::ir_constant()
   : ir_rvalue(ir_type_constant)
{
   this->const_elements = NULL;
}

ir_constant::ir_constant(const struct glsl_type *type,
			 const ir_constant_data *data)
   : ir_rvalue(ir_type_constant)
{
   this->const_elements = NULL;

   assert((type->base_type >= GLSL_TYPE_UINT)
	  && (type->base_type <= GLSL_TYPE_IMAGE));

   this->type = type;
   memcpy(& this->value, data, sizeof(this->value));
}

ir_constant::ir_constant(mesa_float16_t f16, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_FLOAT16, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.f16[i] = f16.bits;
   }
   for (unsigned i = vector_elements; i < 16; i++)  {
      this->value.f[i] = 0;
   }
}

ir_constant::ir_constant(float f, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_FLOAT, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.f[i] = f;
   }
   for (unsigned i = vector_elements; i < 16; i++)  {
      this->value.f[i] = 0;
   }
}

ir_constant::ir_constant(double d, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_DOUBLE, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.d[i] = d;
   }
   for (unsigned i = vector_elements; i < 16; i++)  {
      this->value.d[i] = 0.0;
   }
}

ir_constant::ir_constant(unsigned int u, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_UINT, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.u[i] = u;
   }
   for (unsigned i = vector_elements; i < 16; i++) {
      this->value.u[i] = 0;
   }
}

ir_constant::ir_constant(int integer, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_INT, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.i[i] = integer;
   }
   for (unsigned i = vector_elements; i < 16; i++) {
      this->value.i[i] = 0;
   }
}

ir_constant::ir_constant(uint64_t u64, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_UINT64, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.u64[i] = u64;
   }
   for (unsigned i = vector_elements; i < 16; i++) {
      this->value.u64[i] = 0;
   }
}

ir_constant::ir_constant(int64_t int64, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_INT64, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.i64[i] = int64;
   }
   for (unsigned i = vector_elements; i < 16; i++) {
      this->value.i64[i] = 0;
   }
}

ir_constant::ir_constant(bool b, unsigned vector_elements)
   : ir_rvalue(ir_type_constant)
{
   assert(vector_elements <= 4);
   this->type = glsl_type::get_instance(GLSL_TYPE_BOOL, vector_elements, 1);
   for (unsigned i = 0; i < vector_elements; i++) {
      this->value.b[i] = b;
   }
   for (unsigned i = vector_elements; i < 16; i++) {
      this->value.b[i] = false;
   }
}

ir_constant::ir_constant(const ir_constant *c, unsigned i)
   : ir_rvalue(ir_type_constant)
{
   this->const_elements = NULL;
   this->type = c->type->get_base_type();

   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  this->value.u[0] = c->value.u[i]; break;
   case GLSL_TYPE_INT:   this->value.i[0] = c->value.i[i]; break;
   case GLSL_TYPE_FLOAT: this->value.f[0] = c->value.f[i]; break;
   case GLSL_TYPE_FLOAT16: this->value.f16[0] = c->value.f16[i]; break;
   case GLSL_TYPE_BOOL:  this->value.b[0] = c->value.b[i]; break;
   case GLSL_TYPE_DOUBLE: this->value.d[0] = c->value.d[i]; break;
   default:              assert(!"Should not get here."); break;
   }
}

ir_constant::ir_constant(const struct glsl_type *type, exec_list *value_list)
   : ir_rvalue(ir_type_constant)
{
   this->const_elements = NULL;
   this->type = type;

   assert(type->is_scalar() || type->is_vector() || type->is_matrix()
	  || type->is_struct() || type->is_array());

   /* If the constant is a record, the types of each of the entries in
    * value_list must be a 1-for-1 match with the structure components.  Each
    * entry must also be a constant.  Just move the nodes from the value_list
    * to the list in the ir_constant.
    */
   if (type->is_array() || type->is_struct()) {
      this->const_elements = ralloc_array(this, ir_constant *, type->length);
      unsigned i = 0;
      foreach_in_list(ir_constant, value, value_list) {
	 assert(value->as_constant() != NULL);

	 this->const_elements[i++] = value;
      }
      return;
   }

   for (unsigned i = 0; i < 16; i++) {
      this->value.u[i] = 0;
   }

   ir_constant *value = (ir_constant *) (value_list->get_head_raw());

   /* Constructors with exactly one scalar argument are special for vectors
    * and matrices.  For vectors, the scalar value is replicated to fill all
    * the components.  For matrices, the scalar fills the components of the
    * diagonal while the rest is filled with 0.
    */
   if (value->type->is_scalar() && value->next->is_tail_sentinel()) {
      if (type->is_matrix()) {
	 /* Matrix - fill diagonal (rest is already set to 0) */
         for (unsigned i = 0; i < type->matrix_columns; i++) {
            switch (type->base_type) {
            case GLSL_TYPE_FLOAT:
               this->value.f[i * type->vector_elements + i] =
                  value->value.f[0];
               break;
            case GLSL_TYPE_DOUBLE:
               this->value.d[i * type->vector_elements + i] =
                  value->value.d[0];
               break;
            case GLSL_TYPE_FLOAT16:
               this->value.f16[i * type->vector_elements + i] =
                  value->value.f16[0];
               break;
            default:
               assert(!"unexpected matrix base type");
            }
         }
      } else {
	 /* Vector or scalar - fill all components */
	 switch (type->base_type) {
	 case GLSL_TYPE_UINT:
	 case GLSL_TYPE_INT:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.u[i] = value->value.u[0];
	    break;
	 case GLSL_TYPE_FLOAT:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.f[i] = value->value.f[0];
	    break;
	 case GLSL_TYPE_FLOAT16:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.f16[i] = value->value.f16[0];
	    break;
	 case GLSL_TYPE_DOUBLE:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.d[i] = value->value.d[0];
	    break;
	 case GLSL_TYPE_UINT64:
	 case GLSL_TYPE_INT64:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.u64[i] = value->value.u64[0];
	    break;
	 case GLSL_TYPE_BOOL:
	    for (unsigned i = 0; i < type->components(); i++)
	       this->value.b[i] = value->value.b[0];
	    break;
	 case GLSL_TYPE_SAMPLER:
	 case GLSL_TYPE_IMAGE:
	    this->value.u64[0] = value->value.u64[0];
	    break;
	 default:
	    assert(!"Should not get here.");
	    break;
	 }
      }
      return;
   }

   if (type->is_matrix() && value->type->is_matrix()) {
      assert(value->next->is_tail_sentinel());

      /* From section 5.4.2 of the GLSL 1.20 spec:
       * "If a matrix is constructed from a matrix, then each component
       *  (column i, row j) in the result that has a corresponding component
       *  (column i, row j) in the argument will be initialized from there."
       */
      unsigned cols = MIN2(type->matrix_columns, value->type->matrix_columns);
      unsigned rows = MIN2(type->vector_elements, value->type->vector_elements);
      for (unsigned i = 0; i < cols; i++) {
	 for (unsigned j = 0; j < rows; j++) {
	    const unsigned src = i * value->type->vector_elements + j;
	    const unsigned dst = i * type->vector_elements + j;
	    this->value.f[dst] = value->value.f[src];
	 }
      }

      /* "All other components will be initialized to the identity matrix." */
      for (unsigned i = cols; i < type->matrix_columns; i++)
	 this->value.f[i * type->vector_elements + i] = 1.0;

      return;
   }

   /* Use each component from each entry in the value_list to initialize one
    * component of the constant being constructed.
    */
   unsigned i = 0;
   for (;;) {
      assert(value->as_constant() != NULL);
      assert(!value->is_tail_sentinel());

      for (unsigned j = 0; j < value->type->components(); j++) {
	 switch (type->base_type) {
	 case GLSL_TYPE_UINT:
	    this->value.u[i] = value->get_uint_component(j);
	    break;
	 case GLSL_TYPE_INT:
	    this->value.i[i] = value->get_int_component(j);
	    break;
	 case GLSL_TYPE_FLOAT:
	    this->value.f[i] = value->get_float_component(j);
	    break;
	 case GLSL_TYPE_FLOAT16:
	    this->value.f16[i] = value->get_float16_component(j);
	    break;
	 case GLSL_TYPE_BOOL:
	    this->value.b[i] = value->get_bool_component(j);
	    break;
	 case GLSL_TYPE_DOUBLE:
	    this->value.d[i] = value->get_double_component(j);
	    break;
         case GLSL_TYPE_UINT64:
	    this->value.u64[i] = value->get_uint64_component(j);
	    break;
	 case GLSL_TYPE_INT64:
	    this->value.i64[i] = value->get_int64_component(j);
	    break;
	 default:
	    /* FINISHME: What to do?  Exceptions are not the answer.
	     */
	    break;
	 }

	 i++;
	 if (i >= type->components())
	    break;
      }

      if (i >= type->components())
	 break; /* avoid downcasting a list sentinel */
      value = (ir_constant *) value->next;
   }
}

ir_constant *
ir_constant::zero(void *mem_ctx, const glsl_type *type)
{
   assert(type->is_scalar() || type->is_vector() || type->is_matrix()
	  || type->is_struct() || type->is_array());

   ir_constant *c = new(mem_ctx) ir_constant;
   c->type = type;
   memset(&c->value, 0, sizeof(c->value));

   if (type->is_array()) {
      c->const_elements = ralloc_array(c, ir_constant *, type->length);

      for (unsigned i = 0; i < type->length; i++)
	 c->const_elements[i] = ir_constant::zero(c, type->fields.array);
   }

   if (type->is_struct()) {
      c->const_elements = ralloc_array(c, ir_constant *, type->length);

      for (unsigned i = 0; i < type->length; i++) {
         c->const_elements[i] =
            ir_constant::zero(mem_ctx, type->fields.structure[i].type);
      }
   }

   return c;
}

bool
ir_constant::get_bool_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return this->value.u[i] != 0;
   case GLSL_TYPE_INT:   return this->value.i[i] != 0;
   case GLSL_TYPE_FLOAT: return ((int)this->value.f[i]) != 0;
   case GLSL_TYPE_FLOAT16: return ((int)_mesa_half_to_float(this->value.f16[i])) != 0;
   case GLSL_TYPE_BOOL:  return this->value.b[i];
   case GLSL_TYPE_DOUBLE: return this->value.d[i] != 0.0;
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return this->value.u64[i] != 0;
   case GLSL_TYPE_INT64:  return this->value.i64[i] != 0;
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return false;
}

float
ir_constant::get_float_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return (float) this->value.u[i];
   case GLSL_TYPE_INT:   return (float) this->value.i[i];
   case GLSL_TYPE_FLOAT: return this->value.f[i];
   case GLSL_TYPE_FLOAT16: return _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1.0f : 0.0f;
   case GLSL_TYPE_DOUBLE: return (float) this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return (float) this->value.u64[i];
   case GLSL_TYPE_INT64:  return (float) this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0.0;
}

uint16_t
ir_constant::get_float16_component(unsigned i) const
{
   if (this->type->base_type == GLSL_TYPE_FLOAT16)
      return this->value.f16[i];
   else
      return _mesa_float_to_half(get_float_component(i));
}

double
ir_constant::get_double_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return (double) this->value.u[i];
   case GLSL_TYPE_INT:   return (double) this->value.i[i];
   case GLSL_TYPE_FLOAT: return (double) this->value.f[i];
   case GLSL_TYPE_FLOAT16: return (double) _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1.0 : 0.0;
   case GLSL_TYPE_DOUBLE: return this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return (double) this->value.u64[i];
   case GLSL_TYPE_INT64:  return (double) this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0.0;
}

int
ir_constant::get_int_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return this->value.u[i];
   case GLSL_TYPE_INT:   return this->value.i[i];
   case GLSL_TYPE_FLOAT: return (int) this->value.f[i];
   case GLSL_TYPE_FLOAT16: return (int) _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1 : 0;
   case GLSL_TYPE_DOUBLE: return (int) this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return (int) this->value.u64[i];
   case GLSL_TYPE_INT64:  return (int) this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0;
}

unsigned
ir_constant::get_uint_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return this->value.u[i];
   case GLSL_TYPE_INT:   return this->value.i[i];
   case GLSL_TYPE_FLOAT: return (unsigned) this->value.f[i];
   case GLSL_TYPE_FLOAT16: return (unsigned) _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1 : 0;
   case GLSL_TYPE_DOUBLE: return (unsigned) this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return (unsigned) this->value.u64[i];
   case GLSL_TYPE_INT64:  return (unsigned) this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0;
}

int64_t
ir_constant::get_int64_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return this->value.u[i];
   case GLSL_TYPE_INT:   return this->value.i[i];
   case GLSL_TYPE_FLOAT: return (int64_t) this->value.f[i];
   case GLSL_TYPE_FLOAT16: return (int64_t) _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1 : 0;
   case GLSL_TYPE_DOUBLE: return (int64_t) this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return (int64_t) this->value.u64[i];
   case GLSL_TYPE_INT64:  return this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0;
}

uint64_t
ir_constant::get_uint64_component(unsigned i) const
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:  return this->value.u[i];
   case GLSL_TYPE_INT:   return this->value.i[i];
   case GLSL_TYPE_FLOAT: return (uint64_t) this->value.f[i];
   case GLSL_TYPE_FLOAT16: return (uint64_t) _mesa_half_to_float(this->value.f16[i]);
   case GLSL_TYPE_BOOL:  return this->value.b[i] ? 1 : 0;
   case GLSL_TYPE_DOUBLE: return (uint64_t) this->value.d[i];
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64: return this->value.u64[i];
   case GLSL_TYPE_INT64:  return (uint64_t) this->value.i64[i];
   default:              assert(!"Should not get here."); break;
   }

   /* Must return something to make the compiler happy.  This is clearly an
    * error case.
    */
   return 0;
}

ir_constant *
ir_constant::get_array_element(unsigned i) const
{
   assert(this->type->is_array());

   /* From page 35 (page 41 of the PDF) of the GLSL 1.20 spec:
    *
    *     "Behavior is undefined if a shader subscripts an array with an index
    *     less than 0 or greater than or equal to the size the array was
    *     declared with."
    *
    * Most out-of-bounds accesses are removed before things could get this far.
    * There are cases where non-constant array index values can get constant
    * folded.
    */
   if (int(i) < 0)
      i = 0;
   else if (i >= this->type->length)
      i = this->type->length - 1;

   return const_elements[i];
}

ir_constant *
ir_constant::get_record_field(int idx)
{
   assert(this->type->is_struct());
   assert(idx >= 0 && (unsigned) idx < this->type->length);

   return const_elements[idx];
}

void
ir_constant::copy_offset(ir_constant *src, int offset)
{
   switch (this->type->base_type) {
   case GLSL_TYPE_UINT:
   case GLSL_TYPE_INT:
   case GLSL_TYPE_FLOAT:
   case GLSL_TYPE_FLOAT16:
   case GLSL_TYPE_DOUBLE:
   case GLSL_TYPE_SAMPLER:
   case GLSL_TYPE_IMAGE:
   case GLSL_TYPE_UINT64:
   case GLSL_TYPE_INT64:
   case GLSL_TYPE_BOOL: {
      unsigned int size = src->type->components();
      assert (size <= this->type->components() - offset);
      for (unsigned int i=0; i<size; i++) {
	 switch (this->type->base_type) {
	 case GLSL_TYPE_UINT:
	    value.u[i+offset] = src->get_uint_component(i);
	    break;
	 case GLSL_TYPE_INT:
	    value.i[i+offset] = src->get_int_component(i);
	    break;
	 case GLSL_TYPE_FLOAT:
	    value.f[i+offset] = src->get_float_component(i);
	    break;
	 case GLSL_TYPE_FLOAT16:
	    value.f16[i+offset] = src->get_float16_component(i);
	    break;
	 case GLSL_TYPE_BOOL:
	    value.b[i+offset] = src->get_bool_component(i);
	    break;
	 case GLSL_TYPE_DOUBLE:
	    value.d[i+offset] = src->get_double_component(i);
	    break;
	 case GLSL_TYPE_SAMPLER:
	 case GLSL_TYPE_IMAGE:
	 case GLSL_TYPE_UINT64:
	    value.u64[i+offset] = src->get_uint64_component(i);
	    break;
	 case GLSL_TYPE_INT64:
	    value.i64[i+offset] = src->get_int64_component(i);
	    break;
	 default: // Shut up the compiler
	    break;
	 }
      }
      break;
   }

   case GLSL_TYPE_STRUCT:
   case GLSL_TYPE_ARRAY: {
      assert (src->type == this->type);
      for (unsigned i = 0; i < this->type->length; i++) {
	 this->const_elements[i] = src->const_elements[i]->clone(this, NULL);
      }
      break;
   }

   default:
      assert(!"Should not get here.");
      break;
   }
}

void
ir_constant::copy_masked_offset(ir_constant *src, int offset, unsigned int mask)
{
   assert (!type->is_array() && !type->is_struct());

   if (!type->is_vector() && !type->is_matrix()) {
      offset = 0;
      mask = 1;
   }

   int id = 0;
   for (int i=0; i<4; i++) {
      if (mask & (1 << i)) {
	 switch (this->type->base_type) {
	 case GLSL_TYPE_UINT:
	    value.u[i+offset] = src->get_uint_component(id++);
	    break;
	 case GLSL_TYPE_INT:
	    value.i[i+offset] = src->get_int_component(id++);
	    break;
	 case GLSL_TYPE_FLOAT:
	    value.f[i+offset] = src->get_float_component(id++);
	    break;
	 case GLSL_TYPE_FLOAT16:
	    value.f16[i+offset] = src->get_float16_component(id++);
	    break;
	 case GLSL_TYPE_BOOL:
	    value.b[i+offset] = src->get_bool_component(id++);
	    break;
	 case GLSL_TYPE_DOUBLE:
	    value.d[i+offset] = src->get_double_component(id++);
	    break;
	 case GLSL_TYPE_SAMPLER:
	 case GLSL_TYPE_IMAGE:
	 case GLSL_TYPE_UINT64:
	    value.u64[i+offset] = src->get_uint64_component(id++);
	    break;
	 case GLSL_TYPE_INT64:
	    value.i64[i+offset] = src->get_int64_component(id++);
	    break;
	 default:
	    assert(!"Should not get here.");
	    return;
	 }
      }
   }
}

bool
ir_constant::has_value(const ir_constant *c) const
{
   if (this->type != c->type)
      return false;

   if (this->type->is_array() || this->type->is_struct()) {
      for (unsigned i = 0; i < this->type->length; i++) {
	 if (!this->const_elements[i]->has_value(c->const_elements[i]))
	    return false;
      }
      return true;
   }

   for (unsigned i = 0; i < this->type->components(); i++) {
      switch (this->type->base_type) {
      case GLSL_TYPE_UINT:
	 if (this->value.u[i] != c->value.u[i])
	    return false;
	 break;
      case GLSL_TYPE_INT:
	 if (this->value.i[i] != c->value.i[i])
	    return false;
	 break;
      case GLSL_TYPE_FLOAT:
	 if (this->value.f[i] != c->value.f[i])
	    return false;
	 break;
      case GLSL_TYPE_FLOAT16:
	/* Convert to float to make sure NaN and ±0.0 compares correctly */
	 if (_mesa_half_to_float(this->value.f16[i]) !=
             _mesa_half_to_float(c->value.f16[i]))
	    return false;
	 break;
      case GLSL_TYPE_BOOL:
	 if (this->value.b[i] != c->value.b[i])
	    return false;
	 break;
      case GLSL_TYPE_DOUBLE:
	 if (this->value.d[i] != c->value.d[i])
	    return false;
	 break;
      case GLSL_TYPE_SAMPLER:
      case GLSL_TYPE_IMAGE:
      case GLSL_TYPE_UINT64:
	 if (this->value.u64[i] != c->value.u64[i])
	    return false;
	 break;
      case GLSL_TYPE_INT64:
	 if (this->value.i64[i] != c->value.i64[i])
	    return false;
	 break;
      default:
	 assert(!"Should not get here.");
	 return false;
      }
   }

   return true;
}

bool
ir_constant::is_value(float f, int i) const
{
   if (!this->type->is_scalar() && !this->type->is_vector())
      return false;

   /* Only accept boolean values for 0/1. */
   if (int(bool(i)) != i && this->type->is_boolean())
      return false;

   for (unsigned c = 0; c < this->type->vector_elements; c++) {
      switch (this->type->base_type) {
      case GLSL_TYPE_FLOAT:
	 if (this->value.f[c] != f)
	    return false;
	 break;
      case GLSL_TYPE_FLOAT16:
         if (_mesa_half_to_float(this->value.f16[c]) != f)
            return false;
         break;
      case GLSL_TYPE_INT:
	 if (this->value.i[c] != i)
	    return false;
	 break;
      case GLSL_TYPE_UINT:
	 if (this->value.u[c] != unsigned(i))
	    return false;
	 break;
      case GLSL_TYPE_BOOL:
	 if (this->value.b[c] != bool(i))
	    return false;
	 break;
      case GLSL_TYPE_DOUBLE:
	 if (this->value.d[c] != double(f))
	    return false;
	 break;
      case GLSL_TYPE_SAMPLER:
      case GLSL_TYPE_IMAGE:
      case GLSL_TYPE_UINT64:
	 if (this->value.u64[c] != uint64_t(i))
	    return false;
	 break;
      case GLSL_TYPE_INT64:
	 if (this->value.i64[c] != i)
	    return false;
	 break;
      default:
	 /* The only other base types are structures, arrays, and samplers.
	  * Samplers cannot be constants, and the others should have been
	  * filtered out above.
	  */
	 assert(!"Should not get here.");
	 return false;
      }
   }

   return true;
}

bool
ir_constant::is_zero() const
{
   return is_value(0.0, 0);
}

bool
ir_constant::is_one() const
{
   return is_value(1.0, 1);
}

bool
ir_constant::is_negative_one() const
{
   return is_value(-1.0, -1);
}

bool
ir_constant::is_uint16_constant() const
{
   if (!type->is_integer_32())
      return false;

   return value.u[0] < (1 << 16);
}

ir_loop::ir_loop()
   : ir_instruction(ir_type_loop)
{
}


ir_dereference_variable::ir_dereference_variable(ir_variable *var)
   : ir_dereference(ir_type_dereference_variable)
{
   assert(var != NULL);

   this->var = var;
   this->type = var->type;
}


ir_dereference_array::ir_dereference_array(ir_rvalue *value,
					   ir_rvalue *array_index)
   : ir_dereference(ir_type_dereference_array)
{
   this->array_index = array_index;
   this->set_array(value);
}


ir_dereference_array::ir_dereference_array(ir_variable *var,
					   ir_rvalue *array_index)
   : ir_dereference(ir_type_dereference_array)
{
   void *ctx = ralloc_parent(var);

   this->array_index = array_index;
   this->set_array(new(ctx) ir_dereference_variable(var));
}


void
ir_dereference_array::set_array(ir_rvalue *value)
{
   assert(value != NULL);

   this->array = value;

   const glsl_type *const vt = this->array->type;

   if (vt->is_array()) {
      type = vt->fields.array;
   } else if (vt->is_matrix()) {
      type = vt->column_type();
   } else if (vt->is_vector()) {
      type = vt->get_base_type();
   }
}


ir_dereference_record::ir_dereference_record(ir_rvalue *value,
					     const char *field)
   : ir_dereference(ir_type_dereference_record)
{
   assert(value != NULL);

   this->record = value;
   this->type = this->record->type->field_type(field);
   this->field_idx = this->record->type->field_index(field);
}


ir_dereference_record::ir_dereference_record(ir_variable *var,
					     const char *field)
   : ir_dereference(ir_type_dereference_record)
{
   void *ctx = ralloc_parent(var);

   this->record = new(ctx) ir_dereference_variable(var);
   this->type = this->record->type->field_type(field);
   this->field_idx = this->record->type->field_index(field);
}

bool
ir_dereference::is_lvalue(const struct _mesa_glsl_parse_state *state) const
{
   ir_variable *var = this->variable_referenced();

   /* Every l-value derference chain eventually ends in a variable.
    */
   if ((var == NULL) || var->data.read_only)
      return false;

   /* From section 4.1.7 of the ARB_bindless_texture spec:
    *
    * "Samplers can be used as l-values, so can be assigned into and used as
    *  "out" and "inout" function parameters."
    *
    * From section 4.1.X of the ARB_bindless_texture spec:
    *
    * "Images can be used as l-values, so can be assigned into and used as
    *  "out" and "inout" function parameters."
    */
   if ((!state || state->has_bindless()) &&
       (this->type->contains_sampler() || this->type->contains_image()))
      return true;

   /* From section 4.1.7 of the GLSL 4.40 spec:
    *
    *   "Opaque variables cannot be treated as l-values; hence cannot
    *    be used as out or inout function parameters, nor can they be
    *    assigned into."
    */
   if (this->type->contains_opaque())
      return false;

   return true;
}


static const char * const tex_opcode_strs[] = { "tex", "txb", "txl", "txd", "txf", "txf_ms", "txs", "lod", "tg4", "query_levels", "texture_samples", "samples_identical" };

const char *ir_texture::opcode_string()
{
   assert((unsigned int) op < ARRAY_SIZE(tex_opcode_strs));
   return tex_opcode_strs[op];
}

ir_texture_opcode
ir_texture::get_opcode(const char *str)
{
   const int count = sizeof(tex_opcode_strs) / sizeof(tex_opcode_strs[0]);
   for (int op = 0; op < count; op++) {
      if (strcmp(str, tex_opcode_strs[op]) == 0)
	 return (ir_texture_opcode) op;
   }
   return (ir_texture_opcode) -1;
}


void
ir_texture::set_sampler(ir_dereference *sampler, const glsl_type *type)
{
   assert(sampler != NULL);
   assert(type != NULL);
   this->sampler = sampler;
   this->type = type;

   if (this->op == ir_txs || this->op == ir_query_levels ||
       this->op == ir_texture_samples) {
      assert(type->base_type == GLSL_TYPE_INT);
   } else if (this->op == ir_lod) {
      assert(type->vector_elements == 2);
      assert(type->is_float());
   } else if (this->op == ir_samples_identical) {
      assert(type == glsl_type::bool_type);
      assert(sampler->type->is_sampler());
      assert(sampler->type->sampler_dimensionality == GLSL_SAMPLER_DIM_MS);
   } else {
      assert(sampler->type->sampled_type == (int) type->base_type);
      if (sampler->type->sampler_shadow)
	 assert(type->vector_elements == 4 || type->vector_elements == 1);
      else
	 assert(type->vector_elements == 4);
   }
}

bool
ir_texture::has_lod(const glsl_type *sampler_type)
{
   assert(sampler_type->is_sampler());

   switch (sampler_type->sampler_dimensionality) {
   case GLSL_SAMPLER_DIM_RECT:
   case GLSL_SAMPLER_DIM_BUF:
   case GLSL_SAMPLER_DIM_MS:
      return false;
   default:
      return true;
   }
}

void
ir_swizzle::init_mask(const unsigned *comp, unsigned count)
{
   assert((count >= 1) && (count <= 4));

   memset(&this->mask, 0, sizeof(this->mask));
   this->mask.num_components = count;

   unsigned dup_mask = 0;
   switch (count) {
   case 4:
      assert(comp[3] <= 3);
      dup_mask |= (1U << comp[3])
	 & ((1U << comp[0]) | (1U << comp[1]) | (1U << comp[2]));
      this->mask.w = comp[3];

   case 3:
      assert(comp[2] <= 3);
      dup_mask |= (1U << comp[2])
	 & ((1U << comp[0]) | (1U << comp[1]));
      this->mask.z = comp[2];

   case 2:
      assert(comp[1] <= 3);
      dup_mask |= (1U << comp[1])
	 & ((1U << comp[0]));
      this->mask.y = comp[1];

   case 1:
      assert(comp[0] <= 3);
      this->mask.x = comp[0];
   }

   this->mask.has_duplicates = dup_mask != 0;

   /* Based on the number of elements in the swizzle and the base type
    * (i.e., float, int, unsigned, or bool) of the vector being swizzled,
    * generate the type of the resulting value.
    */
   type = glsl_type::get_instance(val->type->base_type, mask.num_components, 1);
}

ir_swizzle::ir_swizzle(ir_rvalue *val, unsigned x, unsigned y, unsigned z,
		       unsigned w, unsigned count)
   : ir_rvalue(ir_type_swizzle), val(val)
{
   const unsigned components[4] = { x, y, z, w };
   this->init_mask(components, count);
}

ir_swizzle::ir_swizzle(ir_rvalue *val, const unsigned *comp,
		       unsigned count)
   : ir_rvalue(ir_type_swizzle), val(val)
{
   this->init_mask(comp, count);
}

ir_swizzle::ir_swizzle(ir_rvalue *val, ir_swizzle_mask mask)
   : ir_rvalue(ir_type_swizzle), val(val), mask(mask)
{
   this->type = glsl_type::get_instance(val->type->base_type,
					mask.num_components, 1);
}

#define X 1
#define R 5
#define S 9
#define I 13

ir_swizzle *
ir_swizzle::create(ir_rvalue *val, const char *str, unsigned vector_length)
{
   void *ctx = ralloc_parent(val);

   /* For each possible swizzle character, this table encodes the value in
    * \c idx_map that represents the 0th element of the vector.  For invalid
    * swizzle characters (e.g., 'k'), a special value is used that will allow
    * detection of errors.
    */
   static const unsigned char base_idx[26] = {
   /* a  b  c  d  e  f  g  h  i  j  k  l  m */
      R, R, I, I, I, I, R, I, I, I, I, I, I,
   /* n  o  p  q  r  s  t  u  v  w  x  y  z */
      I, I, S, S, R, S, S, I, I, X, X, X, X
   };

   /* Each valid swizzle character has an entry in the previous table.  This
    * table encodes the base index encoded in the previous table plus the actual
    * index of the swizzle character.  When processing swizzles, the first
    * character in the string is indexed in the previous table.  Each character
    * in the string is indexed in this table, and the value found there has the
    * value form the first table subtracted.  The result must be on the range
    * [0,3].
    *
    * For example, the string "wzyx" will get X from the first table.  Each of
    * the charcaters will get X+3, X+2, X+1, and X+0 from this table.  After
    * subtraction, the swizzle values are { 3, 2, 1, 0 }.
    *
    * The string "wzrg" will get X from the first table.  Each of the characters
    * will get X+3, X+2, R+0, and R+1 from this table.  After subtraction, the
    * swizzle values are { 3, 2, 4, 5 }.  Since 4 and 5 are outside the range
    * [0,3], the error is detected.
    */
   static const unsigned char idx_map[26] = {
   /* a    b    c    d    e    f    g    h    i    j    k    l    m */
      R+3, R+2, 0,   0,   0,   0,   R+1, 0,   0,   0,   0,   0,   0,
   /* n    o    p    q    r    s    t    u    v    w    x    y    z */
      0,   0,   S+2, S+3, R+0, S+0, S+1, 0,   0,   X+3, X+0, X+1, X+2
   };

   int swiz_idx[4] = { 0, 0, 0, 0 };
   unsigned i;


   /* Validate the first character in the swizzle string and look up the base
    * index value as described above.
    */
   if ((str[0] < 'a') || (str[0] > 'z'))
      return NULL;

   const unsigned base = base_idx[str[0] - 'a'];


   for (i = 0; (i < 4) && (str[i] != '\0'); i++) {
      /* Validate the next character, and, as described above, convert it to a
       * swizzle index.
       */
      if ((str[i] < 'a') || (str[i] > 'z'))
	 return NULL;

      swiz_idx[i] = idx_map[str[i] - 'a'] - base;
      if ((swiz_idx[i] < 0) || (swiz_idx[i] >= (int) vector_length))
	 return NULL;
   }

   if (str[i] != '\0')
	 return NULL;

   return new(ctx) ir_swizzle(val, swiz_idx[0], swiz_idx[1], swiz_idx[2],
			      swiz_idx[3], i);
}

#undef X
#undef R
#undef S
#undef I

ir_variable *
ir_swizzle::variable_referenced() const
{
   return this->val->variable_referenced();
}


bool ir_variable::temporaries_allocate_names = false;

const char ir_variable::tmp_name[] = "compiler_temp";

ir_variable::ir_variable(const struct glsl_type *type, const char *name,
			 ir_variable_mode mode)
   : ir_instruction(ir_type_variable)
{
   this->type = type;

   if (mode == ir_var_temporary && !ir_variable::temporaries_allocate_names)
      name = NULL;

   /* The ir_variable clone method may call this constructor with name set to
    * tmp_name.
    */
   assert(name != NULL
          || mode == ir_var_temporary
          || mode == ir_var_function_in
          || mode == ir_var_function_out
          || mode == ir_var_function_inout);
   assert(name != ir_variable::tmp_name
          || mode == ir_var_temporary);
   if (mode == ir_var_temporary
       && (name == NULL || name == ir_variable::tmp_name)) {
      this->name = ir_variable::tmp_name;
   } else if (name == NULL ||
              strlen(name) < ARRAY_SIZE(this->name_storage)) {
      strcpy(this->name_storage, name ? name : "");
      this->name = this->name_storage;
   } else {
      this->name = ralloc_strdup(this, name);
   }

   this->u.max_ifc_array_access = NULL;

   this->data.explicit_location = false;
   this->data.explicit_index = false;
   this->data.explicit_binding = false;
   this->data.explicit_component = false;
   this->data.has_initializer = false;
   this->data.is_unmatched_generic_inout = false;
   this->data.is_xfb_only = false;
   this->data.explicit_xfb_buffer = false;
   this->data.explicit_xfb_offset = false;
   this->data.explicit_xfb_stride = false;
   this->data.location = -1;
   this->data.location_frac = 0;
   this->data.matrix_layout = GLSL_MATRIX_LAYOUT_INHERITED;
   this->data.from_named_ifc_block = false;
   this->data.must_be_shader_input = false;
   this->data.index = 0;
   this->data.binding = 0;
   this->data.warn_extension_index = 0;
   this->constant_value = NULL;
   this->constant_initializer = NULL;
   this->data.depth_layout = ir_depth_layout_none;
   this->data.used = false;
   this->data.assigned = false;
   this->data.always_active_io = false;
   this->data.read_only = false;
   this->data.centroid = false;
   this->data.sample = false;
   this->data.patch = false;
   this->data.explicit_invariant = false;
   this->data.invariant = false;
   this->data.precise = false;
   this->data.how_declared = ir_var_declared_normally;
   this->data.mode = mode;
   this->data.interpolation = INTERP_MODE_NONE;
   this->data.max_array_access = -1;
   this->data.offset = 0;
   this->data.precision = GLSL_PRECISION_NONE;
   this->data.memory_read_only = false;
   this->data.memory_write_only = false;
   this->data.memory_coherent = false;
   this->data.memory_volatile = false;
   this->data.memory_restrict = false;
   this->data.from_ssbo_unsized_array = false;
   this->data.implicit_sized_array = false;
   this->data.fb_fetch_output = false;
   this->data.bindless = false;
   this->data.bound = false;
   this->data.image_format = PIPE_FORMAT_NONE;
   this->data._num_state_slots = 0;
   this->data.param_index = 0;
   this->data.stream = 0;
   this->data.xfb_buffer = -1;
   this->data.xfb_stride = -1;

   this->interface_type = NULL;

   if (type != NULL) {
      if (type->is_interface())
         this->init_interface_type(type);
      else if (type->without_array()->is_interface())
         this->init_interface_type(type->without_array());
   }
}


const char *
interpolation_string(unsigned interpolation)
{
   switch (interpolation) {
   case INTERP_MODE_NONE:          return "no";
   case INTERP_MODE_SMOOTH:        return "smooth";
   case INTERP_MODE_FLAT:          return "flat";
   case INTERP_MODE_NOPERSPECTIVE: return "noperspective";
   }

   assert(!"Should not get here.");
   return "";
}

const char *const ir_variable::warn_extension_table[] = {
   "",
   "GL_ARB_shader_stencil_export",
   "GL_AMD_shader_stencil_export",
};

void
ir_variable::enable_extension_warning(const char *extension)
{
   for (unsigned i = 0; i < ARRAY_SIZE(warn_extension_table); i++) {
      if (strcmp(warn_extension_table[i], extension) == 0) {
         this->data.warn_extension_index = i;
         return;
      }
   }

   assert(!"Should not get here.");
   this->data.warn_extension_index = 0;
}

const char *
ir_variable::get_extension_warning() const
{
   return this->data.warn_extension_index == 0
      ? NULL : warn_extension_table[this->data.warn_extension_index];
}

ir_function_signature::ir_function_signature(const glsl_type *return_type,
                                             builtin_available_predicate b)
   : ir_instruction(ir_type_function_signature),
     return_type(return_type), is_defined(false),
     return_precision(GLSL_PRECISION_NONE),
     intrinsic_id(ir_intrinsic_invalid), builtin_avail(b), _function(NULL)
{
   this->origin = NULL;
}


bool
ir_function_signature::is_builtin() const
{
   return builtin_avail != NULL;
}


bool
ir_function_signature::is_builtin_available(const _mesa_glsl_parse_state *state) const
{
   /* We can't call the predicate without a state pointer, so just say that
    * the signature is available.  At compile time, we need the filtering,
    * but also receive a valid state pointer.  At link time, we're resolving
    * imported built-in prototypes to their definitions, which will always
    * be an exact match.  So we can skip the filtering.
    */
   if (state == NULL)
      return true;

   assert(builtin_avail != NULL);
   return builtin_avail(state);
}


static bool
modes_match(unsigned a, unsigned b)
{
   if (a == b)
      return true;

   /* Accept "in" vs. "const in" */
   if ((a == ir_var_const_in && b == ir_var_function_in) ||
       (b == ir_var_const_in && a == ir_var_function_in))
      return true;

   return false;
}


const char *
ir_function_signature::qualifiers_match(exec_list *params)
{
   /* check that the qualifiers match. */
   foreach_two_lists(a_node, &this->parameters, b_node, params) {
      ir_variable *a = (ir_variable *) a_node;
      ir_variable *b = (ir_variable *) b_node;

      if (a->data.read_only != b->data.read_only ||
	  !modes_match(a->data.mode, b->data.mode) ||
	  a->data.interpolation != b->data.interpolation ||
	  a->data.centroid != b->data.centroid ||
          a->data.sample != b->data.sample ||
          a->data.patch != b->data.patch ||
          a->data.memory_read_only != b->data.memory_read_only ||
          a->data.memory_write_only != b->data.memory_write_only ||
          a->data.memory_coherent != b->data.memory_coherent ||
          a->data.memory_volatile != b->data.memory_volatile ||
          a->data.memory_restrict != b->data.memory_restrict) {

	 /* parameter a's qualifiers don't match */
	 return a->name;
      }
   }
   return NULL;
}


void
ir_function_signature::replace_parameters(exec_list *new_params)
{
   /* Destroy all of the previous parameter information.  If the previous
    * parameter information comes from the function prototype, it may either
    * specify incorrect parameter names or not have names at all.
    */
   new_params->move_nodes_to(&parameters);
}


ir_function::ir_function(const char *name)
   : ir_instruction(ir_type_function)
{
   this->subroutine_index = -1;
   this->name = ralloc_strdup(this, name);
}


bool
ir_function::has_user_signature()
{
   foreach_in_list(ir_function_signature, sig, &this->signatures) {
      if (!sig->is_builtin())
	 return true;
   }
   return false;
}


ir_rvalue *
ir_rvalue::error_value(void *mem_ctx)
{
   ir_rvalue *v = new(mem_ctx) ir_rvalue(ir_type_unset);

   v->type = glsl_type::error_type;
   return v;
}


void
visit_exec_list(exec_list *list, ir_visitor *visitor)
{
   foreach_in_list_safe(ir_instruction, node, list) {
      node->accept(visitor);
   }
}


static void
steal_memory(ir_instruction *ir, void *new_ctx)
{
   ir_variable *var = ir->as_variable();
   ir_function *fn = ir->as_function();
   ir_constant *constant = ir->as_constant();
   if (var != NULL && var->constant_value != NULL)
      steal_memory(var->constant_value, ir);

   if (var != NULL && var->constant_initializer != NULL)
      steal_memory(var->constant_initializer, ir);

   if (fn != NULL && fn->subroutine_types)
      ralloc_steal(new_ctx, fn->subroutine_types);

   /* The components of aggregate constants are not visited by the normal
    * visitor, so steal their values by hand.
    */
   if (constant != NULL &&
       (constant->type->is_array() || constant->type->is_struct())) {
      for (unsigned int i = 0; i < constant->type->length; i++) {
         steal_memory(constant->const_elements[i], ir);
      }
   }

   ralloc_steal(new_ctx, ir);
}


void
reparent_ir(exec_list *list, void *mem_ctx)
{
   foreach_in_list(ir_instruction, node, list) {
      visit_tree(node, steal_memory, mem_ctx);
   }
}


static ir_rvalue *
try_min_one(ir_rvalue *ir)
{
   ir_expression *expr = ir->as_expression();

   if (!expr || expr->operation != ir_binop_min)
      return NULL;

   if (expr->operands[0]->is_one())
      return expr->operands[1];

   if (expr->operands[1]->is_one())
      return expr->operands[0];

   return NULL;
}

static ir_rvalue *
try_max_zero(ir_rvalue *ir)
{
   ir_expression *expr = ir->as_expression();

   if (!expr || expr->operation != ir_binop_max)
      return NULL;

   if (expr->operands[0]->is_zero())
      return expr->operands[1];

   if (expr->operands[1]->is_zero())
      return expr->operands[0];

   return NULL;
}

ir_rvalue *
ir_rvalue::as_rvalue_to_saturate()
{
   ir_expression *expr = this->as_expression();

   if (!expr)
      return NULL;

   ir_rvalue *max_zero = try_max_zero(expr);
   if (max_zero) {
      return try_min_one(max_zero);
   } else {
      ir_rvalue *min_one = try_min_one(expr);
      if (min_one) {
	 return try_max_zero(min_one);
      }
   }

   return NULL;
}


unsigned
vertices_per_prim(GLenum prim)
{
   switch (prim) {
   case GL_POINTS:
      return 1;
   case GL_LINES:
      return 2;
   case GL_TRIANGLES:
      return 3;
   case GL_LINES_ADJACENCY:
      return 4;
   case GL_TRIANGLES_ADJACENCY:
      return 6;
   default:
      assert(!"Bad primitive");
      return 3;
   }
}

/**
 * Generate a string describing the mode of a variable
 */
const char *
mode_string(const ir_variable *var)
{
   switch (var->data.mode) {
   case ir_var_auto:
      return (var->data.read_only) ? "global constant" : "global variable";

   case ir_var_uniform:
      return "uniform";

   case ir_var_shader_storage:
      return "buffer";

   case ir_var_shader_in:
      return "shader input";

   case ir_var_shader_out:
      return "shader output";

   case ir_var_function_in:
   case ir_var_const_in:
      return "function input";

   case ir_var_function_out:
      return "function output";

   case ir_var_function_inout:
      return "function inout";

   case ir_var_system_value:
      return "shader input";

   case ir_var_temporary:
      return "compiler temporary";

   case ir_var_mode_count:
      break;
   }

   assert(!"Should not get here.");
   return "invalid variable";
}
