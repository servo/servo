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
 * \file lower_instructions.cpp
 *
 * Many GPUs lack native instructions for certain expression operations, and
 * must replace them with some other expression tree.  This pass lowers some
 * of the most common cases, allowing the lowering code to be implemented once
 * rather than in each driver backend.
 *
 * Currently supported transformations:
 * - SUB_TO_ADD_NEG
 * - DIV_TO_MUL_RCP
 * - INT_DIV_TO_MUL_RCP
 * - EXP_TO_EXP2
 * - POW_TO_EXP2
 * - LOG_TO_LOG2
 * - MOD_TO_FLOOR
 * - LDEXP_TO_ARITH
 * - DFREXP_TO_ARITH
 * - CARRY_TO_ARITH
 * - BORROW_TO_ARITH
 * - SAT_TO_CLAMP
 * - DOPS_TO_DFRAC
 *
 * SUB_TO_ADD_NEG:
 * ---------------
 * Breaks an ir_binop_sub expression down to add(op0, neg(op1))
 *
 * This simplifies expression reassociation, and for many backends
 * there is no subtract operation separate from adding the negation.
 * For backends with native subtract operations, they will probably
 * want to recognize add(op0, neg(op1)) or the other way around to
 * produce a subtract anyway.
 *
 * FDIV_TO_MUL_RCP, DDIV_TO_MUL_RCP, and INT_DIV_TO_MUL_RCP:
 * ---------------------------------------------------------
 * Breaks an ir_binop_div expression down to op0 * (rcp(op1)).
 *
 * Many GPUs don't have a divide instruction (945 and 965 included),
 * but they do have an RCP instruction to compute an approximate
 * reciprocal.  By breaking the operation down, constant reciprocals
 * can get constant folded.
 *
 * FDIV_TO_MUL_RCP lowers single-precision and half-precision
 * floating point division;
 * DDIV_TO_MUL_RCP only lowers double-precision floating point division.
 * DIV_TO_MUL_RCP is a convenience macro that sets both flags.
 * INT_DIV_TO_MUL_RCP handles the integer case, converting to and from floating
 * point so that RCP is possible.
 *
 * EXP_TO_EXP2 and LOG_TO_LOG2:
 * ----------------------------
 * Many GPUs don't have a base e log or exponent instruction, but they
 * do have base 2 versions, so this pass converts exp and log to exp2
 * and log2 operations.
 *
 * POW_TO_EXP2:
 * -----------
 * Many older GPUs don't have an x**y instruction.  For these GPUs, convert
 * x**y to 2**(y * log2(x)).
 *
 * MOD_TO_FLOOR:
 * -------------
 * Breaks an ir_binop_mod expression down to (op0 - op1 * floor(op0 / op1))
 *
 * Many GPUs don't have a MOD instruction (945 and 965 included), and
 * if we have to break it down like this anyway, it gives an
 * opportunity to do things like constant fold the (1.0 / op1) easily.
 *
 * Note: before we used to implement this as op1 * fract(op / op1) but this
 * implementation had significant precision errors.
 *
 * LDEXP_TO_ARITH:
 * -------------
 * Converts ir_binop_ldexp to arithmetic and bit operations for float sources.
 *
 * DFREXP_DLDEXP_TO_ARITH:
 * ---------------
 * Converts ir_binop_ldexp, ir_unop_frexp_sig, and ir_unop_frexp_exp to
 * arithmetic and bit ops for double arguments.
 *
 * CARRY_TO_ARITH:
 * ---------------
 * Converts ir_carry into (x + y) < x.
 *
 * BORROW_TO_ARITH:
 * ----------------
 * Converts ir_borrow into (x < y).
 *
 * SAT_TO_CLAMP:
 * -------------
 * Converts ir_unop_saturate into min(max(x, 0.0), 1.0)
 *
 * DOPS_TO_DFRAC:
 * --------------
 * Converts double trunc, ceil, floor, round to fract
 */

#include <math.h>
#include "program/prog_instruction.h" /* for swizzle */
#include "compiler/glsl_types.h"
#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "util/half_float.h"

using namespace ir_builder;

namespace {

class lower_instructions_visitor : public ir_hierarchical_visitor {
public:
   lower_instructions_visitor(unsigned lower)
      : progress(false), lower(lower) { }

   ir_visitor_status visit_leave(ir_expression *);

   bool progress;

private:
   unsigned lower; /** Bitfield of which operations to lower */

   void sub_to_add_neg(ir_expression *);
   void div_to_mul_rcp(ir_expression *);
   void int_div_to_mul_rcp(ir_expression *);
   void mod_to_floor(ir_expression *);
   void exp_to_exp2(ir_expression *);
   void pow_to_exp2(ir_expression *);
   void log_to_log2(ir_expression *);
   void ldexp_to_arith(ir_expression *);
   void dldexp_to_arith(ir_expression *);
   void dfrexp_sig_to_arith(ir_expression *);
   void dfrexp_exp_to_arith(ir_expression *);
   void carry_to_arith(ir_expression *);
   void borrow_to_arith(ir_expression *);
   void sat_to_clamp(ir_expression *);
   void double_dot_to_fma(ir_expression *);
   void double_lrp(ir_expression *);
   void dceil_to_dfrac(ir_expression *);
   void dfloor_to_dfrac(ir_expression *);
   void dround_even_to_dfrac(ir_expression *);
   void dtrunc_to_dfrac(ir_expression *);
   void dsign_to_csel(ir_expression *);
   void bit_count_to_math(ir_expression *);
   void extract_to_shifts(ir_expression *);
   void insert_to_shifts(ir_expression *);
   void reverse_to_shifts(ir_expression *ir);
   void find_lsb_to_float_cast(ir_expression *ir);
   void find_msb_to_float_cast(ir_expression *ir);
   void imul_high_to_mul(ir_expression *ir);
   void sqrt_to_abs_sqrt(ir_expression *ir);
   void mul64_to_mul_and_mul_high(ir_expression *ir);

   ir_expression *_carry(operand a, operand b);

   static ir_constant *_imm_fp(void *mem_ctx,
                               const glsl_type *type,
                               double f,
                               unsigned vector_elements=1);
};

} /* anonymous namespace */

/**
 * Determine if a particular type of lowering should occur
 */
#define lowering(x) (this->lower & x)

bool
lower_instructions(exec_list *instructions, unsigned what_to_lower)
{
   lower_instructions_visitor v(what_to_lower);

   visit_list_elements(&v, instructions);
   return v.progress;
}

void
lower_instructions_visitor::sub_to_add_neg(ir_expression *ir)
{
   ir->operation = ir_binop_add;
   ir->init_num_operands();
   ir->operands[1] = new(ir) ir_expression(ir_unop_neg, ir->operands[1]->type,
					   ir->operands[1], NULL);
   this->progress = true;
}

void
lower_instructions_visitor::div_to_mul_rcp(ir_expression *ir)
{
   assert(ir->operands[1]->type->is_float_16_32_64());

   /* New expression for the 1.0 / op1 */
   ir_rvalue *expr;
   expr = new(ir) ir_expression(ir_unop_rcp,
				ir->operands[1]->type,
				ir->operands[1]);

   /* op0 / op1 -> op0 * (1.0 / op1) */
   ir->operation = ir_binop_mul;
   ir->init_num_operands();
   ir->operands[1] = expr;

   this->progress = true;
}

void
lower_instructions_visitor::int_div_to_mul_rcp(ir_expression *ir)
{
   assert(ir->operands[1]->type->is_integer_32());

   /* Be careful with integer division -- we need to do it as a
    * float and re-truncate, since rcp(n > 1) of an integer would
    * just be 0.
    */
   ir_rvalue *op0, *op1;
   const struct glsl_type *vec_type;

   vec_type = glsl_type::get_instance(GLSL_TYPE_FLOAT,
				      ir->operands[1]->type->vector_elements,
				      ir->operands[1]->type->matrix_columns);

   if (ir->operands[1]->type->base_type == GLSL_TYPE_INT)
      op1 = new(ir) ir_expression(ir_unop_i2f, vec_type, ir->operands[1], NULL);
   else
      op1 = new(ir) ir_expression(ir_unop_u2f, vec_type, ir->operands[1], NULL);

   op1 = new(ir) ir_expression(ir_unop_rcp, op1->type, op1, NULL);

   vec_type = glsl_type::get_instance(GLSL_TYPE_FLOAT,
				      ir->operands[0]->type->vector_elements,
				      ir->operands[0]->type->matrix_columns);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_INT)
      op0 = new(ir) ir_expression(ir_unop_i2f, vec_type, ir->operands[0], NULL);
   else
      op0 = new(ir) ir_expression(ir_unop_u2f, vec_type, ir->operands[0], NULL);

   vec_type = glsl_type::get_instance(GLSL_TYPE_FLOAT,
				      ir->type->vector_elements,
				      ir->type->matrix_columns);

   op0 = new(ir) ir_expression(ir_binop_mul, vec_type, op0, op1);

   if (ir->operands[1]->type->base_type == GLSL_TYPE_INT) {
      ir->operation = ir_unop_f2i;
      ir->operands[0] = op0;
   } else {
      ir->operation = ir_unop_i2u;
      ir->operands[0] = new(ir) ir_expression(ir_unop_f2i, op0);
   }
   ir->init_num_operands();
   ir->operands[1] = NULL;

   this->progress = true;
}

void
lower_instructions_visitor::exp_to_exp2(ir_expression *ir)
{
   ir_constant *log2_e = _imm_fp(ir, ir->type, M_LOG2E);

   ir->operation = ir_unop_exp2;
   ir->init_num_operands();
   ir->operands[0] = new(ir) ir_expression(ir_binop_mul, ir->operands[0]->type,
					   ir->operands[0], log2_e);
   this->progress = true;
}

void
lower_instructions_visitor::pow_to_exp2(ir_expression *ir)
{
   ir_expression *const log2_x =
      new(ir) ir_expression(ir_unop_log2, ir->operands[0]->type,
			    ir->operands[0]);

   ir->operation = ir_unop_exp2;
   ir->init_num_operands();
   ir->operands[0] = new(ir) ir_expression(ir_binop_mul, ir->operands[1]->type,
					   ir->operands[1], log2_x);
   ir->operands[1] = NULL;
   this->progress = true;
}

void
lower_instructions_visitor::log_to_log2(ir_expression *ir)
{
   ir->operation = ir_binop_mul;
   ir->init_num_operands();
   ir->operands[0] = new(ir) ir_expression(ir_unop_log2, ir->operands[0]->type,
					   ir->operands[0], NULL);
   ir->operands[1] = _imm_fp(ir, ir->operands[0]->type, 1.0 / M_LOG2E);
   this->progress = true;
}

void
lower_instructions_visitor::mod_to_floor(ir_expression *ir)
{
   ir_variable *x = new(ir) ir_variable(ir->operands[0]->type, "mod_x",
                                         ir_var_temporary);
   ir_variable *y = new(ir) ir_variable(ir->operands[1]->type, "mod_y",
                                         ir_var_temporary);
   this->base_ir->insert_before(x);
   this->base_ir->insert_before(y);

   ir_assignment *const assign_x =
      new(ir) ir_assignment(new(ir) ir_dereference_variable(x),
                            ir->operands[0]);
   ir_assignment *const assign_y =
      new(ir) ir_assignment(new(ir) ir_dereference_variable(y),
                            ir->operands[1]);

   this->base_ir->insert_before(assign_x);
   this->base_ir->insert_before(assign_y);

   ir_expression *const div_expr =
      new(ir) ir_expression(ir_binop_div, x->type,
                            new(ir) ir_dereference_variable(x),
                            new(ir) ir_dereference_variable(y));

   /* Don't generate new IR that would need to be lowered in an additional
    * pass.
    */
   if ((lowering(FDIV_TO_MUL_RCP) && ir->type->is_float_16_32()) ||
       (lowering(DDIV_TO_MUL_RCP) && ir->type->is_double()))
      div_to_mul_rcp(div_expr);

   ir_expression *const floor_expr =
      new(ir) ir_expression(ir_unop_floor, x->type, div_expr);

   if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
      dfloor_to_dfrac(floor_expr);

   ir_expression *const mul_expr =
      new(ir) ir_expression(ir_binop_mul,
                            new(ir) ir_dereference_variable(y),
                            floor_expr);

   ir->operation = ir_binop_sub;
   ir->init_num_operands();
   ir->operands[0] = new(ir) ir_dereference_variable(x);
   ir->operands[1] = mul_expr;
   this->progress = true;
}

void
lower_instructions_visitor::ldexp_to_arith(ir_expression *ir)
{
   /* Translates
    *    ir_binop_ldexp x exp
    * into
    *
    *    extracted_biased_exp = rshift(bitcast_f2i(abs(x)), exp_shift);
    *    resulting_biased_exp = min(extracted_biased_exp + exp, 255);
    *
    *    if (extracted_biased_exp >= 255)
    *       return x; // +/-inf, NaN
    *
    *    sign_mantissa = bitcast_f2u(x) & sign_mantissa_mask;
    *
    *    if (min(resulting_biased_exp, extracted_biased_exp) < 1)
    *       resulting_biased_exp = 0;
    *    if (resulting_biased_exp >= 255 ||
    *        min(resulting_biased_exp, extracted_biased_exp) < 1) {
    *       sign_mantissa &= sign_mask;
    *    }
    *
    *    return bitcast_u2f(sign_mantissa |
    *                       lshift(i2u(resulting_biased_exp), exp_shift));
    *
    * which we can't actually implement as such, since the GLSL IR doesn't
    * have vectorized if-statements. We actually implement it without branches
    * using conditional-select:
    *
    *    extracted_biased_exp = rshift(bitcast_f2i(abs(x)), exp_shift);
    *    resulting_biased_exp = min(extracted_biased_exp + exp, 255);
    *
    *    sign_mantissa = bitcast_f2u(x) & sign_mantissa_mask;
    *
    *    flush_to_zero = lequal(min(resulting_biased_exp, extracted_biased_exp), 0);
    *    resulting_biased_exp = csel(flush_to_zero, 0, resulting_biased_exp)
    *    zero_mantissa = logic_or(flush_to_zero,
    *                             gequal(resulting_biased_exp, 255));
    *    sign_mantissa = csel(zero_mantissa, sign_mantissa & sign_mask, sign_mantissa);
    *
    *    result = sign_mantissa |
    *             lshift(i2u(resulting_biased_exp), exp_shift));
    *
    *    return csel(extracted_biased_exp >= 255, x, bitcast_u2f(result));
    *
    * The definition of ldexp in the GLSL spec says:
    *
    *    "If this product is too large to be represented in the
    *     floating-point type, the result is undefined."
    *
    * However, the definition of ldexp in the GLSL ES spec does not contain
    * this sentence, so we do need to handle overflow correctly.
    *
    * There is additional language limiting the defined range of exp, but this
    * is merely to allow implementations that store 2^exp in a temporary
    * variable.
    */

   const unsigned vec_elem = ir->type->vector_elements;

   /* Types */
   const glsl_type *ivec = glsl_type::get_instance(GLSL_TYPE_INT, vec_elem, 1);
   const glsl_type *uvec = glsl_type::get_instance(GLSL_TYPE_UINT, vec_elem, 1);
   const glsl_type *bvec = glsl_type::get_instance(GLSL_TYPE_BOOL, vec_elem, 1);

   /* Temporary variables */
   ir_variable *x = new(ir) ir_variable(ir->type, "x", ir_var_temporary);
   ir_variable *exp = new(ir) ir_variable(ivec, "exp", ir_var_temporary);
   ir_variable *result = new(ir) ir_variable(uvec, "result", ir_var_temporary);

   ir_variable *extracted_biased_exp =
      new(ir) ir_variable(ivec, "extracted_biased_exp", ir_var_temporary);
   ir_variable *resulting_biased_exp =
      new(ir) ir_variable(ivec, "resulting_biased_exp", ir_var_temporary);

   ir_variable *sign_mantissa =
      new(ir) ir_variable(uvec, "sign_mantissa", ir_var_temporary);

   ir_variable *flush_to_zero =
      new(ir) ir_variable(bvec, "flush_to_zero", ir_var_temporary);
   ir_variable *zero_mantissa =
      new(ir) ir_variable(bvec, "zero_mantissa", ir_var_temporary);

   ir_instruction &i = *base_ir;

   /* Copy <x> and <exp> arguments. */
   i.insert_before(x);
   i.insert_before(assign(x, ir->operands[0]));
   i.insert_before(exp);
   i.insert_before(assign(exp, ir->operands[1]));

   /* Extract the biased exponent from <x>. */
   i.insert_before(extracted_biased_exp);
   i.insert_before(assign(extracted_biased_exp,
                          rshift(bitcast_f2i(abs(x)),
                                 new(ir) ir_constant(23, vec_elem))));

   /* The definition of ldexp in the GLSL 4.60 spec says:
    *
    *    "If exp is greater than +128 (single-precision) or +1024
    *     (double-precision), the value returned is undefined. If exp is less
    *     than -126 (single-precision) or -1022 (double-precision), the value
    *     returned may be flushed to zero."
    *
    * So we do not have to guard against the possibility of addition overflow,
    * which could happen when exp is close to INT_MAX. Addition underflow
    * cannot happen (the worst case is 0 + (-INT_MAX)).
    */
   i.insert_before(resulting_biased_exp);
   i.insert_before(assign(resulting_biased_exp,
                          min2(add(extracted_biased_exp, exp),
                               new(ir) ir_constant(255, vec_elem))));

   i.insert_before(sign_mantissa);
   i.insert_before(assign(sign_mantissa,
                          bit_and(bitcast_f2u(x),
                                  new(ir) ir_constant(0x807fffffu, vec_elem))));

   /* We flush to zero if the original or resulting biased exponent is 0,
    * indicating a +/-0.0 or subnormal input or output.
    *
    * The mantissa is set to 0 if the resulting biased exponent is 255, since
    * an overflow should produce a +/-inf result.
    *
    * Note that NaN inputs are handled separately.
    */
   i.insert_before(flush_to_zero);
   i.insert_before(assign(flush_to_zero,
                          lequal(min2(resulting_biased_exp,
                                      extracted_biased_exp),
                                 ir_constant::zero(ir, ivec))));
   i.insert_before(assign(resulting_biased_exp,
                          csel(flush_to_zero,
                               ir_constant::zero(ir, ivec),
                               resulting_biased_exp)));

   i.insert_before(zero_mantissa);
   i.insert_before(assign(zero_mantissa,
                          logic_or(flush_to_zero,
                                   equal(resulting_biased_exp,
                                         new(ir) ir_constant(255, vec_elem)))));
   i.insert_before(assign(sign_mantissa,
                          csel(zero_mantissa,
                               bit_and(sign_mantissa,
                                       new(ir) ir_constant(0x80000000u, vec_elem)),
                               sign_mantissa)));

   /* Don't generate new IR that would need to be lowered in an additional
    * pass.
    */
   i.insert_before(result);
   if (!lowering(INSERT_TO_SHIFTS)) {
      i.insert_before(assign(result,
                             bitfield_insert(sign_mantissa,
                                             i2u(resulting_biased_exp),
                                             new(ir) ir_constant(23u, vec_elem),
                                             new(ir) ir_constant(8u, vec_elem))));
   } else {
      i.insert_before(assign(result,
                             bit_or(sign_mantissa,
                                    lshift(i2u(resulting_biased_exp),
                                           new(ir) ir_constant(23, vec_elem)))));
   }

   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = gequal(extracted_biased_exp,
                            new(ir) ir_constant(255, vec_elem));
   ir->operands[1] = new(ir) ir_dereference_variable(x);
   ir->operands[2] = bitcast_u2f(result);

   this->progress = true;
}

void
lower_instructions_visitor::dldexp_to_arith(ir_expression *ir)
{
   /* See ldexp_to_arith for structure. Uses frexp_exp to extract the exponent
    * from the significand.
    */

   const unsigned vec_elem = ir->type->vector_elements;

   /* Types */
   const glsl_type *ivec = glsl_type::get_instance(GLSL_TYPE_INT, vec_elem, 1);
   const glsl_type *bvec = glsl_type::get_instance(GLSL_TYPE_BOOL, vec_elem, 1);

   /* Constants */
   ir_constant *zeroi = ir_constant::zero(ir, ivec);

   ir_constant *sign_mask = new(ir) ir_constant(0x80000000u);

   ir_constant *exp_shift = new(ir) ir_constant(20u);
   ir_constant *exp_width = new(ir) ir_constant(11u);
   ir_constant *exp_bias = new(ir) ir_constant(1022, vec_elem);

   /* Temporary variables */
   ir_variable *x = new(ir) ir_variable(ir->type, "x", ir_var_temporary);
   ir_variable *exp = new(ir) ir_variable(ivec, "exp", ir_var_temporary);

   ir_variable *zero_sign_x = new(ir) ir_variable(ir->type, "zero_sign_x",
                                                  ir_var_temporary);

   ir_variable *extracted_biased_exp =
      new(ir) ir_variable(ivec, "extracted_biased_exp", ir_var_temporary);
   ir_variable *resulting_biased_exp =
      new(ir) ir_variable(ivec, "resulting_biased_exp", ir_var_temporary);

   ir_variable *is_not_zero_or_underflow =
      new(ir) ir_variable(bvec, "is_not_zero_or_underflow", ir_var_temporary);

   ir_instruction &i = *base_ir;

   /* Copy <x> and <exp> arguments. */
   i.insert_before(x);
   i.insert_before(assign(x, ir->operands[0]));
   i.insert_before(exp);
   i.insert_before(assign(exp, ir->operands[1]));

   ir_expression *frexp_exp = expr(ir_unop_frexp_exp, x);
   if (lowering(DFREXP_DLDEXP_TO_ARITH))
      dfrexp_exp_to_arith(frexp_exp);

   /* Extract the biased exponent from <x>. */
   i.insert_before(extracted_biased_exp);
   i.insert_before(assign(extracted_biased_exp, add(frexp_exp, exp_bias)));

   i.insert_before(resulting_biased_exp);
   i.insert_before(assign(resulting_biased_exp,
                          add(extracted_biased_exp, exp)));

   /* Test if result is ±0.0, subnormal, or underflow by checking if the
    * resulting biased exponent would be less than 0x1. If so, the result is
    * 0.0 with the sign of x. (Actually, invert the conditions so that
    * immediate values are the second arguments, which is better for i965)
    * TODO: Implement in a vector fashion.
    */
   i.insert_before(zero_sign_x);
   for (unsigned elem = 0; elem < vec_elem; elem++) {
      ir_variable *unpacked =
         new(ir) ir_variable(glsl_type::uvec2_type, "unpacked", ir_var_temporary);
      i.insert_before(unpacked);
      i.insert_before(
            assign(unpacked,
                   expr(ir_unop_unpack_double_2x32, swizzle(x, elem, 1))));
      i.insert_before(assign(unpacked, bit_and(swizzle_y(unpacked), sign_mask->clone(ir, NULL)),
                             WRITEMASK_Y));
      i.insert_before(assign(unpacked, ir_constant::zero(ir, glsl_type::uint_type), WRITEMASK_X));
      i.insert_before(assign(zero_sign_x,
                             expr(ir_unop_pack_double_2x32, unpacked),
                             1 << elem));
   }
   i.insert_before(is_not_zero_or_underflow);
   i.insert_before(assign(is_not_zero_or_underflow,
                          gequal(resulting_biased_exp,
                                  new(ir) ir_constant(0x1, vec_elem))));
   i.insert_before(assign(x, csel(is_not_zero_or_underflow,
                                  x, zero_sign_x)));
   i.insert_before(assign(resulting_biased_exp,
                          csel(is_not_zero_or_underflow,
                               resulting_biased_exp, zeroi)));

   /* We could test for overflows by checking if the resulting biased exponent
    * would be greater than 0xFE. Turns out we don't need to because the GLSL
    * spec says:
    *
    *    "If this product is too large to be represented in the
    *     floating-point type, the result is undefined."
    */

   ir_rvalue *results[4] = {NULL};
   for (unsigned elem = 0; elem < vec_elem; elem++) {
      ir_variable *unpacked =
         new(ir) ir_variable(glsl_type::uvec2_type, "unpacked", ir_var_temporary);
      i.insert_before(unpacked);
      i.insert_before(
            assign(unpacked,
                   expr(ir_unop_unpack_double_2x32, swizzle(x, elem, 1))));

      ir_expression *bfi = bitfield_insert(
            swizzle_y(unpacked),
            i2u(swizzle(resulting_biased_exp, elem, 1)),
            exp_shift->clone(ir, NULL),
            exp_width->clone(ir, NULL));

      i.insert_before(assign(unpacked, bfi, WRITEMASK_Y));

      results[elem] = expr(ir_unop_pack_double_2x32, unpacked);
   }

   ir->operation = ir_quadop_vector;
   ir->init_num_operands();
   ir->operands[0] = results[0];
   ir->operands[1] = results[1];
   ir->operands[2] = results[2];
   ir->operands[3] = results[3];

   /* Don't generate new IR that would need to be lowered in an additional
    * pass.
    */

   this->progress = true;
}

void
lower_instructions_visitor::dfrexp_sig_to_arith(ir_expression *ir)
{
   const unsigned vec_elem = ir->type->vector_elements;
   const glsl_type *bvec = glsl_type::get_instance(GLSL_TYPE_BOOL, vec_elem, 1);

   /* Double-precision floating-point values are stored as
    *   1 sign bit;
    *   11 exponent bits;
    *   52 mantissa bits.
    *
    * We're just extracting the significand here, so we only need to modify
    * the upper 32-bit uint. Unfortunately we must extract each double
    * independently as there is no vector version of unpackDouble.
    */

   ir_instruction &i = *base_ir;

   ir_variable *is_not_zero =
      new(ir) ir_variable(bvec, "is_not_zero", ir_var_temporary);
   ir_rvalue *results[4] = {NULL};

   ir_constant *dzero = new(ir) ir_constant(0.0, vec_elem);
   i.insert_before(is_not_zero);
   i.insert_before(
         assign(is_not_zero,
                nequal(abs(ir->operands[0]->clone(ir, NULL)), dzero)));

   /* TODO: Remake this as more vector-friendly when int64 support is
    * available.
    */
   for (unsigned elem = 0; elem < vec_elem; elem++) {
      ir_constant *zero = new(ir) ir_constant(0u, 1);
      ir_constant *sign_mantissa_mask = new(ir) ir_constant(0x800fffffu, 1);

      /* Exponent of double floating-point values in the range [0.5, 1.0). */
      ir_constant *exponent_value = new(ir) ir_constant(0x3fe00000u, 1);

      ir_variable *bits =
         new(ir) ir_variable(glsl_type::uint_type, "bits", ir_var_temporary);
      ir_variable *unpacked =
         new(ir) ir_variable(glsl_type::uvec2_type, "unpacked", ir_var_temporary);

      ir_rvalue *x = swizzle(ir->operands[0]->clone(ir, NULL), elem, 1);

      i.insert_before(bits);
      i.insert_before(unpacked);
      i.insert_before(assign(unpacked, expr(ir_unop_unpack_double_2x32, x)));

      /* Manipulate the high uint to remove the exponent and replace it with
       * either the default exponent or zero.
       */
      i.insert_before(assign(bits, swizzle_y(unpacked)));
      i.insert_before(assign(bits, bit_and(bits, sign_mantissa_mask)));
      i.insert_before(assign(bits, bit_or(bits,
                                          csel(swizzle(is_not_zero, elem, 1),
                                               exponent_value,
                                               zero))));
      i.insert_before(assign(unpacked, bits, WRITEMASK_Y));
      results[elem] = expr(ir_unop_pack_double_2x32, unpacked);
   }

   /* Put the dvec back together */
   ir->operation = ir_quadop_vector;
   ir->init_num_operands();
   ir->operands[0] = results[0];
   ir->operands[1] = results[1];
   ir->operands[2] = results[2];
   ir->operands[3] = results[3];

   this->progress = true;
}

void
lower_instructions_visitor::dfrexp_exp_to_arith(ir_expression *ir)
{
   const unsigned vec_elem = ir->type->vector_elements;
   const glsl_type *bvec = glsl_type::get_instance(GLSL_TYPE_BOOL, vec_elem, 1);
   const glsl_type *uvec = glsl_type::get_instance(GLSL_TYPE_UINT, vec_elem, 1);

   /* Double-precision floating-point values are stored as
    *   1 sign bit;
    *   11 exponent bits;
    *   52 mantissa bits.
    *
    * We're just extracting the exponent here, so we only care about the upper
    * 32-bit uint.
    */

   ir_instruction &i = *base_ir;

   ir_variable *is_not_zero =
      new(ir) ir_variable(bvec, "is_not_zero", ir_var_temporary);
   ir_variable *high_words =
      new(ir) ir_variable(uvec, "high_words", ir_var_temporary);
   ir_constant *dzero = new(ir) ir_constant(0.0, vec_elem);
   ir_constant *izero = new(ir) ir_constant(0, vec_elem);

   ir_rvalue *absval = abs(ir->operands[0]);

   i.insert_before(is_not_zero);
   i.insert_before(high_words);
   i.insert_before(assign(is_not_zero, nequal(absval->clone(ir, NULL), dzero)));

   /* Extract all of the upper uints. */
   for (unsigned elem = 0; elem < vec_elem; elem++) {
      ir_rvalue *x = swizzle(absval->clone(ir, NULL), elem, 1);

      i.insert_before(assign(high_words,
                             swizzle_y(expr(ir_unop_unpack_double_2x32, x)),
                             1 << elem));

   }
   ir_constant *exponent_shift = new(ir) ir_constant(20, vec_elem);
   ir_constant *exponent_bias = new(ir) ir_constant(-1022, vec_elem);

   /* For non-zero inputs, shift the exponent down and apply bias. */
   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = new(ir) ir_dereference_variable(is_not_zero);
   ir->operands[1] = add(exponent_bias, u2i(rshift(high_words, exponent_shift)));
   ir->operands[2] = izero;

   this->progress = true;
}

void
lower_instructions_visitor::carry_to_arith(ir_expression *ir)
{
   /* Translates
    *   ir_binop_carry x y
    * into
    *   sum = ir_binop_add x y
    *   bcarry = ir_binop_less sum x
    *   carry = ir_unop_b2i bcarry
    */

   ir_rvalue *x_clone = ir->operands[0]->clone(ir, NULL);
   ir->operation = ir_unop_i2u;
   ir->init_num_operands();
   ir->operands[0] = b2i(less(add(ir->operands[0], ir->operands[1]), x_clone));
   ir->operands[1] = NULL;

   this->progress = true;
}

void
lower_instructions_visitor::borrow_to_arith(ir_expression *ir)
{
   /* Translates
    *   ir_binop_borrow x y
    * into
    *   bcarry = ir_binop_less x y
    *   carry = ir_unop_b2i bcarry
    */

   ir->operation = ir_unop_i2u;
   ir->init_num_operands();
   ir->operands[0] = b2i(less(ir->operands[0], ir->operands[1]));
   ir->operands[1] = NULL;

   this->progress = true;
}

void
lower_instructions_visitor::sat_to_clamp(ir_expression *ir)
{
   /* Translates
    *   ir_unop_saturate x
    * into
    *   ir_binop_min (ir_binop_max(x, 0.0), 1.0)
    */

   ir->operation = ir_binop_min;
   ir->init_num_operands();

   ir_constant *zero = _imm_fp(ir, ir->operands[0]->type, 0.0);
   ir->operands[0] = new(ir) ir_expression(ir_binop_max, ir->operands[0]->type,
                                           ir->operands[0], zero);
   ir->operands[1] = _imm_fp(ir, ir->operands[0]->type, 1.0);

   this->progress = true;
}

void
lower_instructions_visitor::double_dot_to_fma(ir_expression *ir)
{
   ir_variable *temp = new(ir) ir_variable(ir->operands[0]->type->get_base_type(), "dot_res",
					   ir_var_temporary);
   this->base_ir->insert_before(temp);

   int nc = ir->operands[0]->type->components();
   for (int i = nc - 1; i >= 1; i--) {
      ir_assignment *assig;
      if (i == (nc - 1)) {
         assig = assign(temp, mul(swizzle(ir->operands[0]->clone(ir, NULL), i, 1),
                                  swizzle(ir->operands[1]->clone(ir, NULL), i, 1)));
      } else {
         assig = assign(temp, fma(swizzle(ir->operands[0]->clone(ir, NULL), i, 1),
                                  swizzle(ir->operands[1]->clone(ir, NULL), i, 1),
                                  temp));
      }
      this->base_ir->insert_before(assig);
   }

   ir->operation = ir_triop_fma;
   ir->init_num_operands();
   ir->operands[0] = swizzle(ir->operands[0], 0, 1);
   ir->operands[1] = swizzle(ir->operands[1], 0, 1);
   ir->operands[2] = new(ir) ir_dereference_variable(temp);

   this->progress = true;

}

void
lower_instructions_visitor::double_lrp(ir_expression *ir)
{
   int swizval;
   ir_rvalue *op0 = ir->operands[0], *op2 = ir->operands[2];
   ir_constant *one = new(ir) ir_constant(1.0, op2->type->vector_elements);

   switch (op2->type->vector_elements) {
   case 1:
      swizval = SWIZZLE_XXXX;
      break;
   default:
      assert(op0->type->vector_elements == op2->type->vector_elements);
      swizval = SWIZZLE_XYZW;
      break;
   }

   ir->operation = ir_triop_fma;
   ir->init_num_operands();
   ir->operands[0] = swizzle(op2, swizval, op0->type->vector_elements);
   ir->operands[2] = mul(sub(one, op2->clone(ir, NULL)), op0);

   this->progress = true;
}

void
lower_instructions_visitor::dceil_to_dfrac(ir_expression *ir)
{
   /*
    * frtemp = frac(x);
    * temp = sub(x, frtemp);
    * result = temp + ((frtemp != 0.0) ? 1.0 : 0.0);
    */
   ir_instruction &i = *base_ir;
   ir_constant *zero = new(ir) ir_constant(0.0, ir->operands[0]->type->vector_elements);
   ir_constant *one = new(ir) ir_constant(1.0, ir->operands[0]->type->vector_elements);
   ir_variable *frtemp = new(ir) ir_variable(ir->operands[0]->type, "frtemp",
                                             ir_var_temporary);

   i.insert_before(frtemp);
   i.insert_before(assign(frtemp, fract(ir->operands[0])));

   ir->operation = ir_binop_add;
   ir->init_num_operands();
   ir->operands[0] = sub(ir->operands[0]->clone(ir, NULL), frtemp);
   ir->operands[1] = csel(nequal(frtemp, zero), one, zero->clone(ir, NULL));

   this->progress = true;
}

void
lower_instructions_visitor::dfloor_to_dfrac(ir_expression *ir)
{
   /*
    * frtemp = frac(x);
    * result = sub(x, frtemp);
    */
   ir->operation = ir_binop_sub;
   ir->init_num_operands();
   ir->operands[1] = fract(ir->operands[0]->clone(ir, NULL));

   this->progress = true;
}
void
lower_instructions_visitor::dround_even_to_dfrac(ir_expression *ir)
{
   /*
    * insane but works
    * temp = x + 0.5;
    * frtemp = frac(temp);
    * t2 = sub(temp, frtemp);
    * if (frac(x) == 0.5)
    *     result = frac(t2 * 0.5) == 0 ? t2 : t2 - 1;
    *  else
    *     result = t2;

    */
   ir_instruction &i = *base_ir;
   ir_variable *frtemp = new(ir) ir_variable(ir->operands[0]->type, "frtemp",
                                             ir_var_temporary);
   ir_variable *temp = new(ir) ir_variable(ir->operands[0]->type, "temp",
                                           ir_var_temporary);
   ir_variable *t2 = new(ir) ir_variable(ir->operands[0]->type, "t2",
                                           ir_var_temporary);
   ir_constant *p5 = new(ir) ir_constant(0.5, ir->operands[0]->type->vector_elements);
   ir_constant *one = new(ir) ir_constant(1.0, ir->operands[0]->type->vector_elements);
   ir_constant *zero = new(ir) ir_constant(0.0, ir->operands[0]->type->vector_elements);

   i.insert_before(temp);
   i.insert_before(assign(temp, add(ir->operands[0], p5)));

   i.insert_before(frtemp);
   i.insert_before(assign(frtemp, fract(temp)));

   i.insert_before(t2);
   i.insert_before(assign(t2, sub(temp, frtemp)));

   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = equal(fract(ir->operands[0]->clone(ir, NULL)),
                           p5->clone(ir, NULL));
   ir->operands[1] = csel(equal(fract(mul(t2, p5->clone(ir, NULL))),
                                zero),
                          t2,
                          sub(t2, one));
   ir->operands[2] = new(ir) ir_dereference_variable(t2);

   this->progress = true;
}

void
lower_instructions_visitor::dtrunc_to_dfrac(ir_expression *ir)
{
   /*
    * frtemp = frac(x);
    * temp = sub(x, frtemp);
    * result = x >= 0 ? temp : temp + (frtemp == 0.0) ? 0 : 1;
    */
   ir_rvalue *arg = ir->operands[0];
   ir_instruction &i = *base_ir;

   ir_constant *zero = new(ir) ir_constant(0.0, arg->type->vector_elements);
   ir_constant *one = new(ir) ir_constant(1.0, arg->type->vector_elements);
   ir_variable *frtemp = new(ir) ir_variable(arg->type, "frtemp",
                                             ir_var_temporary);
   ir_variable *temp = new(ir) ir_variable(ir->operands[0]->type, "temp",
                                           ir_var_temporary);

   i.insert_before(frtemp);
   i.insert_before(assign(frtemp, fract(arg)));
   i.insert_before(temp);
   i.insert_before(assign(temp, sub(arg->clone(ir, NULL), frtemp)));

   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = gequal(arg->clone(ir, NULL), zero);
   ir->operands[1] = new (ir) ir_dereference_variable(temp);
   ir->operands[2] = add(temp,
                         csel(equal(frtemp, zero->clone(ir, NULL)),
                              zero->clone(ir, NULL),
                              one));

   this->progress = true;
}

void
lower_instructions_visitor::dsign_to_csel(ir_expression *ir)
{
   /*
    * temp = x > 0.0 ? 1.0 : 0.0;
    * result = x < 0.0 ? -1.0 : temp;
    */
   ir_rvalue *arg = ir->operands[0];
   ir_constant *zero = new(ir) ir_constant(0.0, arg->type->vector_elements);
   ir_constant *one = new(ir) ir_constant(1.0, arg->type->vector_elements);
   ir_constant *neg_one = new(ir) ir_constant(-1.0, arg->type->vector_elements);

   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = less(arg->clone(ir, NULL),
                          zero->clone(ir, NULL));
   ir->operands[1] = neg_one;
   ir->operands[2] = csel(greater(arg, zero),
                          one,
                          zero->clone(ir, NULL));

   this->progress = true;
}

void
lower_instructions_visitor::bit_count_to_math(ir_expression *ir)
{
   /* For more details, see:
    *
    * http://graphics.stanford.edu/~seander/bithacks.html#CountBitsSetPaallel
    */
   const unsigned elements = ir->operands[0]->type->vector_elements;
   ir_variable *temp = new(ir) ir_variable(glsl_type::uvec(elements), "temp",
                                           ir_var_temporary);
   ir_constant *c55555555 = new(ir) ir_constant(0x55555555u);
   ir_constant *c33333333 = new(ir) ir_constant(0x33333333u);
   ir_constant *c0F0F0F0F = new(ir) ir_constant(0x0F0F0F0Fu);
   ir_constant *c01010101 = new(ir) ir_constant(0x01010101u);
   ir_constant *c1 = new(ir) ir_constant(1u);
   ir_constant *c2 = new(ir) ir_constant(2u);
   ir_constant *c4 = new(ir) ir_constant(4u);
   ir_constant *c24 = new(ir) ir_constant(24u);

   base_ir->insert_before(temp);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      base_ir->insert_before(assign(temp, ir->operands[0]));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_INT);
      base_ir->insert_before(assign(temp, i2u(ir->operands[0])));
   }

   /* temp = temp - ((temp >> 1) & 0x55555555u); */
   base_ir->insert_before(assign(temp, sub(temp, bit_and(rshift(temp, c1),
                                                         c55555555))));

   /* temp = (temp & 0x33333333u) + ((temp >> 2) & 0x33333333u); */
   base_ir->insert_before(assign(temp, add(bit_and(temp, c33333333),
                                           bit_and(rshift(temp, c2),
                                                   c33333333->clone(ir, NULL)))));

   /* int(((temp + (temp >> 4) & 0xF0F0F0Fu) * 0x1010101u) >> 24); */
   ir->operation = ir_unop_u2i;
   ir->init_num_operands();
   ir->operands[0] = rshift(mul(bit_and(add(temp, rshift(temp, c4)), c0F0F0F0F),
                                c01010101),
                            c24);

   this->progress = true;
}

void
lower_instructions_visitor::extract_to_shifts(ir_expression *ir)
{
   ir_variable *bits =
      new(ir) ir_variable(ir->operands[0]->type, "bits", ir_var_temporary);

   base_ir->insert_before(bits);
   base_ir->insert_before(assign(bits, ir->operands[2]));

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      ir_constant *c1 =
         new(ir) ir_constant(1u, ir->operands[0]->type->vector_elements);
      ir_constant *c32 =
         new(ir) ir_constant(32u, ir->operands[0]->type->vector_elements);
      ir_constant *cFFFFFFFF =
         new(ir) ir_constant(0xFFFFFFFFu, ir->operands[0]->type->vector_elements);

      /* At least some hardware treats (x << y) as (x << (y%32)).  This means
       * we'd get a mask of 0 when bits is 32.  Special case it.
       *
       * mask = bits == 32 ? 0xffffffff : (1u << bits) - 1u;
       */
      ir_expression *mask = csel(equal(bits, c32),
                                 cFFFFFFFF,
                                 sub(lshift(c1, bits), c1->clone(ir, NULL)));

      /* Section 8.8 (Integer Functions) of the GLSL 4.50 spec says:
       *
       *    If bits is zero, the result will be zero.
       *
       * Since (1 << 0) - 1 == 0, we don't need to bother with the conditional
       * select as in the signed integer case.
       *
       * (value >> offset) & mask;
       */
      ir->operation = ir_binop_bit_and;
      ir->init_num_operands();
      ir->operands[0] = rshift(ir->operands[0], ir->operands[1]);
      ir->operands[1] = mask;
      ir->operands[2] = NULL;
   } else {
      ir_constant *c0 =
         new(ir) ir_constant(int(0), ir->operands[0]->type->vector_elements);
      ir_constant *c32 =
         new(ir) ir_constant(int(32), ir->operands[0]->type->vector_elements);
      ir_variable *temp =
         new(ir) ir_variable(ir->operands[0]->type, "temp", ir_var_temporary);

      /* temp = 32 - bits; */
      base_ir->insert_before(temp);
      base_ir->insert_before(assign(temp, sub(c32, bits)));

      /* expr = value << (temp - offset)) >> temp; */
      ir_expression *expr =
         rshift(lshift(ir->operands[0], sub(temp, ir->operands[1])), temp);

      /* Section 8.8 (Integer Functions) of the GLSL 4.50 spec says:
       *
       *    If bits is zero, the result will be zero.
       *
       * Due to the (x << (y%32)) behavior mentioned before, the (value <<
       * (32-0)) doesn't "erase" all of the data as we would like, so finish
       * up with:
       *
       * (bits == 0) ? 0 : e;
       */
      ir->operation = ir_triop_csel;
      ir->init_num_operands();
      ir->operands[0] = equal(c0, bits);
      ir->operands[1] = c0->clone(ir, NULL);
      ir->operands[2] = expr;
   }

   this->progress = true;
}

void
lower_instructions_visitor::insert_to_shifts(ir_expression *ir)
{
   ir_constant *c1;
   ir_constant *c32;
   ir_constant *cFFFFFFFF;
   ir_variable *offset =
      new(ir) ir_variable(ir->operands[0]->type, "offset", ir_var_temporary);
   ir_variable *bits =
      new(ir) ir_variable(ir->operands[0]->type, "bits", ir_var_temporary);
   ir_variable *mask =
      new(ir) ir_variable(ir->operands[0]->type, "mask", ir_var_temporary);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_INT) {
      c1 = new(ir) ir_constant(int(1), ir->operands[0]->type->vector_elements);
      c32 = new(ir) ir_constant(int(32), ir->operands[0]->type->vector_elements);
      cFFFFFFFF = new(ir) ir_constant(int(0xFFFFFFFF), ir->operands[0]->type->vector_elements);
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_UINT);

      c1 = new(ir) ir_constant(1u, ir->operands[0]->type->vector_elements);
      c32 = new(ir) ir_constant(32u, ir->operands[0]->type->vector_elements);
      cFFFFFFFF = new(ir) ir_constant(0xFFFFFFFFu, ir->operands[0]->type->vector_elements);
   }

   base_ir->insert_before(offset);
   base_ir->insert_before(assign(offset, ir->operands[2]));

   base_ir->insert_before(bits);
   base_ir->insert_before(assign(bits, ir->operands[3]));

   /* At least some hardware treats (x << y) as (x << (y%32)).  This means
    * we'd get a mask of 0 when bits is 32.  Special case it.
    *
    * mask = (bits == 32 ? 0xffffffff : (1u << bits) - 1u) << offset;
    *
    * Section 8.8 (Integer Functions) of the GLSL 4.50 spec says:
    *
    *    The result will be undefined if offset or bits is negative, or if the
    *    sum of offset and bits is greater than the number of bits used to
    *    store the operand.
    *
    * Since it's undefined, there are a couple other ways this could be
    * implemented.  The other way that was considered was to put the csel
    * around the whole thing:
    *
    *    final_result = bits == 32 ? insert : ... ;
    */
   base_ir->insert_before(mask);

   base_ir->insert_before(assign(mask, csel(equal(bits, c32),
                                            cFFFFFFFF,
                                            lshift(sub(lshift(c1, bits),
                                                       c1->clone(ir, NULL)),
                                                   offset))));

   /* (base & ~mask) | ((insert << offset) & mask) */
   ir->operation = ir_binop_bit_or;
   ir->init_num_operands();
   ir->operands[0] = bit_and(ir->operands[0], bit_not(mask));
   ir->operands[1] = bit_and(lshift(ir->operands[1], offset), mask);
   ir->operands[2] = NULL;
   ir->operands[3] = NULL;

   this->progress = true;
}

void
lower_instructions_visitor::reverse_to_shifts(ir_expression *ir)
{
   /* For more details, see:
    *
    * http://graphics.stanford.edu/~seander/bithacks.html#ReverseParallel
    */
   ir_constant *c1 =
      new(ir) ir_constant(1u, ir->operands[0]->type->vector_elements);
   ir_constant *c2 =
      new(ir) ir_constant(2u, ir->operands[0]->type->vector_elements);
   ir_constant *c4 =
      new(ir) ir_constant(4u, ir->operands[0]->type->vector_elements);
   ir_constant *c8 =
      new(ir) ir_constant(8u, ir->operands[0]->type->vector_elements);
   ir_constant *c16 =
      new(ir) ir_constant(16u, ir->operands[0]->type->vector_elements);
   ir_constant *c33333333 =
      new(ir) ir_constant(0x33333333u, ir->operands[0]->type->vector_elements);
   ir_constant *c55555555 =
      new(ir) ir_constant(0x55555555u, ir->operands[0]->type->vector_elements);
   ir_constant *c0F0F0F0F =
      new(ir) ir_constant(0x0F0F0F0Fu, ir->operands[0]->type->vector_elements);
   ir_constant *c00FF00FF =
      new(ir) ir_constant(0x00FF00FFu, ir->operands[0]->type->vector_elements);
   ir_variable *temp =
      new(ir) ir_variable(glsl_type::uvec(ir->operands[0]->type->vector_elements),
                          "temp", ir_var_temporary);
   ir_instruction &i = *base_ir;

   i.insert_before(temp);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      i.insert_before(assign(temp, ir->operands[0]));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_INT);
      i.insert_before(assign(temp, i2u(ir->operands[0])));
   }

   /* Swap odd and even bits.
    *
    * temp = ((temp >> 1) & 0x55555555u) | ((temp & 0x55555555u) << 1);
    */
   i.insert_before(assign(temp, bit_or(bit_and(rshift(temp, c1), c55555555),
                                       lshift(bit_and(temp, c55555555->clone(ir, NULL)),
                                              c1->clone(ir, NULL)))));
   /* Swap consecutive pairs.
    *
    * temp = ((temp >> 2) & 0x33333333u) | ((temp & 0x33333333u) << 2);
    */
   i.insert_before(assign(temp, bit_or(bit_and(rshift(temp, c2), c33333333),
                                       lshift(bit_and(temp, c33333333->clone(ir, NULL)),
                                              c2->clone(ir, NULL)))));

   /* Swap nibbles.
    *
    * temp = ((temp >> 4) & 0x0F0F0F0Fu) | ((temp & 0x0F0F0F0Fu) << 4);
    */
   i.insert_before(assign(temp, bit_or(bit_and(rshift(temp, c4), c0F0F0F0F),
                                       lshift(bit_and(temp, c0F0F0F0F->clone(ir, NULL)),
                                              c4->clone(ir, NULL)))));

   /* The last step is, basically, bswap.  Swap the bytes, then swap the
    * words.  When this code is run through GCC on x86, it does generate a
    * bswap instruction.
    *
    * temp = ((temp >> 8) & 0x00FF00FFu) | ((temp & 0x00FF00FFu) << 8);
    * temp = ( temp >> 16              ) | ( temp                << 16);
    */
   i.insert_before(assign(temp, bit_or(bit_and(rshift(temp, c8), c00FF00FF),
                                       lshift(bit_and(temp, c00FF00FF->clone(ir, NULL)),
                                              c8->clone(ir, NULL)))));

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      ir->operation = ir_binop_bit_or;
      ir->init_num_operands();
      ir->operands[0] = rshift(temp, c16);
      ir->operands[1] = lshift(temp, c16->clone(ir, NULL));
   } else {
      ir->operation = ir_unop_u2i;
      ir->init_num_operands();
      ir->operands[0] = bit_or(rshift(temp, c16),
                               lshift(temp, c16->clone(ir, NULL)));
   }

   this->progress = true;
}

void
lower_instructions_visitor::find_lsb_to_float_cast(ir_expression *ir)
{
   /* For more details, see:
    *
    * http://graphics.stanford.edu/~seander/bithacks.html#ZerosOnRightFloatCast
    */
   const unsigned elements = ir->operands[0]->type->vector_elements;
   ir_constant *c0 = new(ir) ir_constant(unsigned(0), elements);
   ir_constant *cminus1 = new(ir) ir_constant(int(-1), elements);
   ir_constant *c23 = new(ir) ir_constant(int(23), elements);
   ir_constant *c7F = new(ir) ir_constant(int(0x7F), elements);
   ir_variable *temp =
      new(ir) ir_variable(glsl_type::ivec(elements), "temp", ir_var_temporary);
   ir_variable *lsb_only =
      new(ir) ir_variable(glsl_type::uvec(elements), "lsb_only", ir_var_temporary);
   ir_variable *as_float =
      new(ir) ir_variable(glsl_type::vec(elements), "as_float", ir_var_temporary);
   ir_variable *lsb =
      new(ir) ir_variable(glsl_type::ivec(elements), "lsb", ir_var_temporary);

   ir_instruction &i = *base_ir;

   i.insert_before(temp);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_INT) {
      i.insert_before(assign(temp, ir->operands[0]));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_UINT);
      i.insert_before(assign(temp, u2i(ir->operands[0])));
   }

   /* The int-to-float conversion is lossless because (value & -value) is
    * either a power of two or zero.  We don't use the result in the zero
    * case.  The uint() cast is necessary so that 0x80000000 does not
    * generate a negative value.
    *
    * uint lsb_only = uint(value & -value);
    * float as_float = float(lsb_only);
    */
   i.insert_before(lsb_only);
   i.insert_before(assign(lsb_only, i2u(bit_and(temp, neg(temp)))));

   i.insert_before(as_float);
   i.insert_before(assign(as_float, u2f(lsb_only)));

   /* This is basically an open-coded frexp.  Implementations that have a
    * native frexp instruction would be better served by that.  This is
    * optimized versus a full-featured open-coded implementation in two ways:
    *
    * - We don't care about a correct result from subnormal numbers (including
    *   0.0), so the raw exponent can always be safely unbiased.
    *
    * - The value cannot be negative, so it does not need to be masked off to
    *   extract the exponent.
    *
    * int lsb = (floatBitsToInt(as_float) >> 23) - 0x7f;
    */
   i.insert_before(lsb);
   i.insert_before(assign(lsb, sub(rshift(bitcast_f2i(as_float), c23), c7F)));

   /* Use lsb_only in the comparison instead of temp so that the & (far above)
    * can possibly generate the result without an explicit comparison.
    *
    * (lsb_only == 0) ? -1 : lsb;
    *
    * Since our input values are all integers, the unbiased exponent must not
    * be negative.  It will only be negative (-0x7f, in fact) if lsb_only is
    * 0.  Instead of using (lsb_only == 0), we could use (lsb >= 0).  Which is
    * better is likely GPU dependent.  Either way, the difference should be
    * small.
    */
   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = equal(lsb_only, c0);
   ir->operands[1] = cminus1;
   ir->operands[2] = new(ir) ir_dereference_variable(lsb);

   this->progress = true;
}

void
lower_instructions_visitor::find_msb_to_float_cast(ir_expression *ir)
{
   /* For more details, see:
    *
    * http://graphics.stanford.edu/~seander/bithacks.html#ZerosOnRightFloatCast
    */
   const unsigned elements = ir->operands[0]->type->vector_elements;
   ir_constant *c0 = new(ir) ir_constant(int(0), elements);
   ir_constant *cminus1 = new(ir) ir_constant(int(-1), elements);
   ir_constant *c23 = new(ir) ir_constant(int(23), elements);
   ir_constant *c7F = new(ir) ir_constant(int(0x7F), elements);
   ir_constant *c000000FF = new(ir) ir_constant(0x000000FFu, elements);
   ir_constant *cFFFFFF00 = new(ir) ir_constant(0xFFFFFF00u, elements);
   ir_variable *temp =
      new(ir) ir_variable(glsl_type::uvec(elements), "temp", ir_var_temporary);
   ir_variable *as_float =
      new(ir) ir_variable(glsl_type::vec(elements), "as_float", ir_var_temporary);
   ir_variable *msb =
      new(ir) ir_variable(glsl_type::ivec(elements), "msb", ir_var_temporary);

   ir_instruction &i = *base_ir;

   i.insert_before(temp);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      i.insert_before(assign(temp, ir->operands[0]));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_INT);

      /* findMSB(uint(abs(some_int))) almost always does the right thing.
       * There are two problem values:
       *
       * * 0x80000000.  Since abs(0x80000000) == 0x80000000, findMSB returns
       *   31.  However, findMSB(int(0x80000000)) == 30.
       *
       * * 0xffffffff.  Since abs(0xffffffff) == 1, findMSB returns
       *   31.  Section 8.8 (Integer Functions) of the GLSL 4.50 spec says:
       *
       *    For a value of zero or negative one, -1 will be returned.
       *
       * For all negative number cases, including 0x80000000 and 0xffffffff,
       * the correct value is obtained from findMSB if instead of negating the
       * (already negative) value the logical-not is used.  A conditonal
       * logical-not can be achieved in two instructions.
       */
      ir_variable *as_int =
         new(ir) ir_variable(glsl_type::ivec(elements), "as_int", ir_var_temporary);
      ir_constant *c31 = new(ir) ir_constant(int(31), elements);

      i.insert_before(as_int);
      i.insert_before(assign(as_int, ir->operands[0]));
      i.insert_before(assign(temp, i2u(expr(ir_binop_bit_xor,
                                            as_int,
                                            rshift(as_int, c31)))));
   }

   /* The int-to-float conversion is lossless because bits are conditionally
    * masked off the bottom of temp to ensure the value has at most 24 bits of
    * data or is zero.  We don't use the result in the zero case.  The uint()
    * cast is necessary so that 0x80000000 does not generate a negative value.
    *
    * float as_float = float(temp > 255 ? temp & ~255 : temp);
    */
   i.insert_before(as_float);
   i.insert_before(assign(as_float, u2f(csel(greater(temp, c000000FF),
                                             bit_and(temp, cFFFFFF00),
                                             temp))));

   /* This is basically an open-coded frexp.  Implementations that have a
    * native frexp instruction would be better served by that.  This is
    * optimized versus a full-featured open-coded implementation in two ways:
    *
    * - We don't care about a correct result from subnormal numbers (including
    *   0.0), so the raw exponent can always be safely unbiased.
    *
    * - The value cannot be negative, so it does not need to be masked off to
    *   extract the exponent.
    *
    * int msb = (floatBitsToInt(as_float) >> 23) - 0x7f;
    */
   i.insert_before(msb);
   i.insert_before(assign(msb, sub(rshift(bitcast_f2i(as_float), c23), c7F)));

   /* Use msb in the comparison instead of temp so that the subtract can
    * possibly generate the result without an explicit comparison.
    *
    * (msb < 0) ? -1 : msb;
    *
    * Since our input values are all integers, the unbiased exponent must not
    * be negative.  It will only be negative (-0x7f, in fact) if temp is 0.
    */
   ir->operation = ir_triop_csel;
   ir->init_num_operands();
   ir->operands[0] = less(msb, c0);
   ir->operands[1] = cminus1;
   ir->operands[2] = new(ir) ir_dereference_variable(msb);

   this->progress = true;
}

ir_expression *
lower_instructions_visitor::_carry(operand a, operand b)
{
   if (lowering(CARRY_TO_ARITH))
      return i2u(b2i(less(add(a, b),
                          a.val->clone(ralloc_parent(a.val), NULL))));
   else
      return carry(a, b);
}

ir_constant *
lower_instructions_visitor::_imm_fp(void *mem_ctx,
                                    const glsl_type *type,
                                    double f,
                                    unsigned vector_elements)
{
   switch (type->base_type) {
   case GLSL_TYPE_FLOAT:
      return new(mem_ctx) ir_constant((float) f, vector_elements);
   case GLSL_TYPE_DOUBLE:
      return new(mem_ctx) ir_constant((double) f, vector_elements);
   case GLSL_TYPE_FLOAT16:
      return new(mem_ctx) ir_constant(mesa_float16_t(f), vector_elements);
   default:
      assert(!"unknown float type for immediate");
      return NULL;
   }
}

void
lower_instructions_visitor::imul_high_to_mul(ir_expression *ir)
{
   /*   ABCD
    * * EFGH
    * ======
    * (GH * CD) + (GH * AB) << 16 + (EF * CD) << 16 + (EF * AB) << 32
    *
    * In GLSL, (a * b) becomes
    *
    * uint m1 = (a & 0x0000ffffu) * (b & 0x0000ffffu);
    * uint m2 = (a & 0x0000ffffu) * (b >> 16);
    * uint m3 = (a >> 16)         * (b & 0x0000ffffu);
    * uint m4 = (a >> 16)         * (b >> 16);
    *
    * uint c1;
    * uint c2;
    * uint lo_result;
    * uint hi_result;
    *
    * lo_result = uaddCarry(m1, m2 << 16, c1);
    * hi_result = m4 + c1;
    * lo_result = uaddCarry(lo_result, m3 << 16, c2);
    * hi_result = hi_result + c2;
    * hi_result = hi_result + (m2 >> 16) + (m3 >> 16);
    */
   const unsigned elements = ir->operands[0]->type->vector_elements;
   ir_variable *src1 =
      new(ir) ir_variable(glsl_type::uvec(elements), "src1", ir_var_temporary);
   ir_variable *src1h =
      new(ir) ir_variable(glsl_type::uvec(elements), "src1h", ir_var_temporary);
   ir_variable *src1l =
      new(ir) ir_variable(glsl_type::uvec(elements), "src1l", ir_var_temporary);
   ir_variable *src2 =
      new(ir) ir_variable(glsl_type::uvec(elements), "src2", ir_var_temporary);
   ir_variable *src2h =
      new(ir) ir_variable(glsl_type::uvec(elements), "src2h", ir_var_temporary);
   ir_variable *src2l =
      new(ir) ir_variable(glsl_type::uvec(elements), "src2l", ir_var_temporary);
   ir_variable *t1 =
      new(ir) ir_variable(glsl_type::uvec(elements), "t1", ir_var_temporary);
   ir_variable *t2 =
      new(ir) ir_variable(glsl_type::uvec(elements), "t2", ir_var_temporary);
   ir_variable *lo =
      new(ir) ir_variable(glsl_type::uvec(elements), "lo", ir_var_temporary);
   ir_variable *hi =
      new(ir) ir_variable(glsl_type::uvec(elements), "hi", ir_var_temporary);
   ir_variable *different_signs = NULL;
   ir_constant *c0000FFFF = new(ir) ir_constant(0x0000FFFFu, elements);
   ir_constant *c16 = new(ir) ir_constant(16u, elements);

   ir_instruction &i = *base_ir;

   i.insert_before(src1);
   i.insert_before(src2);
   i.insert_before(src1h);
   i.insert_before(src2h);
   i.insert_before(src1l);
   i.insert_before(src2l);

   if (ir->operands[0]->type->base_type == GLSL_TYPE_UINT) {
      i.insert_before(assign(src1, ir->operands[0]));
      i.insert_before(assign(src2, ir->operands[1]));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_INT);

      ir_variable *itmp1 =
         new(ir) ir_variable(glsl_type::ivec(elements), "itmp1", ir_var_temporary);
      ir_variable *itmp2 =
         new(ir) ir_variable(glsl_type::ivec(elements), "itmp2", ir_var_temporary);
      ir_constant *c0 = new(ir) ir_constant(int(0), elements);

      i.insert_before(itmp1);
      i.insert_before(itmp2);
      i.insert_before(assign(itmp1, ir->operands[0]));
      i.insert_before(assign(itmp2, ir->operands[1]));

      different_signs =
         new(ir) ir_variable(glsl_type::bvec(elements), "different_signs",
                             ir_var_temporary);

      i.insert_before(different_signs);
      i.insert_before(assign(different_signs, expr(ir_binop_logic_xor,
                                                   less(itmp1, c0),
                                                   less(itmp2, c0->clone(ir, NULL)))));

      i.insert_before(assign(src1, i2u(abs(itmp1))));
      i.insert_before(assign(src2, i2u(abs(itmp2))));
   }

   i.insert_before(assign(src1l, bit_and(src1, c0000FFFF)));
   i.insert_before(assign(src2l, bit_and(src2, c0000FFFF->clone(ir, NULL))));
   i.insert_before(assign(src1h, rshift(src1, c16)));
   i.insert_before(assign(src2h, rshift(src2, c16->clone(ir, NULL))));

   i.insert_before(lo);
   i.insert_before(hi);
   i.insert_before(t1);
   i.insert_before(t2);

   i.insert_before(assign(lo, mul(src1l, src2l)));
   i.insert_before(assign(t1, mul(src1l, src2h)));
   i.insert_before(assign(t2, mul(src1h, src2l)));
   i.insert_before(assign(hi, mul(src1h, src2h)));

   i.insert_before(assign(hi, add(hi, _carry(lo, lshift(t1, c16->clone(ir, NULL))))));
   i.insert_before(assign(lo,            add(lo, lshift(t1, c16->clone(ir, NULL)))));

   i.insert_before(assign(hi, add(hi, _carry(lo, lshift(t2, c16->clone(ir, NULL))))));
   i.insert_before(assign(lo,            add(lo, lshift(t2, c16->clone(ir, NULL)))));

   if (different_signs == NULL) {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_UINT);

      ir->operation = ir_binop_add;
      ir->init_num_operands();
      ir->operands[0] = add(hi, rshift(t1, c16->clone(ir, NULL)));
      ir->operands[1] = rshift(t2, c16->clone(ir, NULL));
   } else {
      assert(ir->operands[0]->type->base_type == GLSL_TYPE_INT);

      i.insert_before(assign(hi, add(add(hi, rshift(t1, c16->clone(ir, NULL))),
                                     rshift(t2, c16->clone(ir, NULL)))));

      /* For channels where different_signs is set we have to perform a 64-bit
       * negation.  This is *not* the same as just negating the high 32-bits.
       * Consider -3 * 2.  The high 32-bits is 0, but the desired result is
       * -1, not -0!  Recall -x == ~x + 1.
       */
      ir_variable *neg_hi =
         new(ir) ir_variable(glsl_type::ivec(elements), "neg_hi", ir_var_temporary);
      ir_constant *c1 = new(ir) ir_constant(1u, elements);

      i.insert_before(neg_hi);
      i.insert_before(assign(neg_hi, add(bit_not(u2i(hi)),
                                         u2i(_carry(bit_not(lo), c1)))));

      ir->operation = ir_triop_csel;
      ir->init_num_operands();
      ir->operands[0] = new(ir) ir_dereference_variable(different_signs);
      ir->operands[1] = new(ir) ir_dereference_variable(neg_hi);
      ir->operands[2] = u2i(hi);
   }
}

void
lower_instructions_visitor::sqrt_to_abs_sqrt(ir_expression *ir)
{
   ir->operands[0] = new(ir) ir_expression(ir_unop_abs, ir->operands[0]);
   this->progress = true;
}

void
lower_instructions_visitor::mul64_to_mul_and_mul_high(ir_expression *ir)
{
   /* Lower 32x32-> 64 to
    *    msb = imul_high(x_lo, y_lo)
    *    lsb = mul(x_lo, y_lo)
    */
   const unsigned elements = ir->operands[0]->type->vector_elements;

   const ir_expression_operation operation =
      ir->type->base_type == GLSL_TYPE_UINT64 ? ir_unop_pack_uint_2x32
                                              : ir_unop_pack_int_2x32;

   const glsl_type *var_type = ir->type->base_type == GLSL_TYPE_UINT64
                               ? glsl_type::uvec(elements)
                               : glsl_type::ivec(elements);

   const glsl_type *ret_type = ir->type->base_type == GLSL_TYPE_UINT64
                               ? glsl_type::uvec2_type
                               : glsl_type::ivec2_type;

   ir_instruction &i = *base_ir;

   ir_variable *msb =
      new(ir) ir_variable(var_type, "msb", ir_var_temporary);
   ir_variable *lsb =
      new(ir) ir_variable(var_type, "lsb", ir_var_temporary);
   ir_variable *x =
      new(ir) ir_variable(var_type, "x", ir_var_temporary);
   ir_variable *y =
      new(ir) ir_variable(var_type, "y", ir_var_temporary);

   i.insert_before(x);
   i.insert_before(assign(x, ir->operands[0]));
   i.insert_before(y);
   i.insert_before(assign(y, ir->operands[1]));
   i.insert_before(msb);
   i.insert_before(lsb);

   i.insert_before(assign(msb, imul_high(x, y)));
   i.insert_before(assign(lsb, mul(x, y)));

   ir_rvalue *result[4] = {NULL};
   for (unsigned elem = 0; elem < elements; elem++) {
      ir_rvalue *val = new(ir) ir_expression(ir_quadop_vector, ret_type,
                                             swizzle(lsb, elem, 1),
                                             swizzle(msb, elem, 1), NULL, NULL);
      result[elem] = expr(operation, val);
   }

   ir->operation = ir_quadop_vector;
   ir->init_num_operands();
   ir->operands[0] = result[0];
   ir->operands[1] = result[1];
   ir->operands[2] = result[2];
   ir->operands[3] = result[3];

   this->progress = true;
}

ir_visitor_status
lower_instructions_visitor::visit_leave(ir_expression *ir)
{
   switch (ir->operation) {
   case ir_binop_dot:
      if (ir->operands[0]->type->is_double())
         double_dot_to_fma(ir);
      break;
   case ir_triop_lrp:
      if (ir->operands[0]->type->is_double())
         double_lrp(ir);
      break;
   case ir_binop_sub:
      if (lowering(SUB_TO_ADD_NEG))
	 sub_to_add_neg(ir);
      break;

   case ir_binop_div:
      if (ir->operands[1]->type->is_integer_32() && lowering(INT_DIV_TO_MUL_RCP))
	 int_div_to_mul_rcp(ir);
      else if ((ir->operands[1]->type->is_float_16_32() && lowering(FDIV_TO_MUL_RCP)) ||
               (ir->operands[1]->type->is_double() && lowering(DDIV_TO_MUL_RCP)))
	 div_to_mul_rcp(ir);
      break;

   case ir_unop_exp:
      if (lowering(EXP_TO_EXP2))
	 exp_to_exp2(ir);
      break;

   case ir_unop_log:
      if (lowering(LOG_TO_LOG2))
	 log_to_log2(ir);
      break;

   case ir_binop_mod:
      if (lowering(MOD_TO_FLOOR) && ir->type->is_float_16_32_64())
	 mod_to_floor(ir);
      break;

   case ir_binop_pow:
      if (lowering(POW_TO_EXP2))
	 pow_to_exp2(ir);
      break;

   case ir_binop_ldexp:
      if (lowering(LDEXP_TO_ARITH) && ir->type->is_float())
         ldexp_to_arith(ir);
      if (lowering(DFREXP_DLDEXP_TO_ARITH) && ir->type->is_double())
         dldexp_to_arith(ir);
      break;

   case ir_unop_frexp_exp:
      if (lowering(DFREXP_DLDEXP_TO_ARITH) && ir->operands[0]->type->is_double())
         dfrexp_exp_to_arith(ir);
      break;

   case ir_unop_frexp_sig:
      if (lowering(DFREXP_DLDEXP_TO_ARITH) && ir->operands[0]->type->is_double())
         dfrexp_sig_to_arith(ir);
      break;

   case ir_binop_carry:
      if (lowering(CARRY_TO_ARITH))
         carry_to_arith(ir);
      break;

   case ir_binop_borrow:
      if (lowering(BORROW_TO_ARITH))
         borrow_to_arith(ir);
      break;

   case ir_unop_saturate:
      if (lowering(SAT_TO_CLAMP))
         sat_to_clamp(ir);
      break;

   case ir_unop_trunc:
      if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
         dtrunc_to_dfrac(ir);
      break;

   case ir_unop_ceil:
      if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
         dceil_to_dfrac(ir);
      break;

   case ir_unop_floor:
      if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
         dfloor_to_dfrac(ir);
      break;

   case ir_unop_round_even:
      if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
         dround_even_to_dfrac(ir);
      break;

   case ir_unop_sign:
      if (lowering(DOPS_TO_DFRAC) && ir->type->is_double())
         dsign_to_csel(ir);
      break;

   case ir_unop_bit_count:
      if (lowering(BIT_COUNT_TO_MATH))
         bit_count_to_math(ir);
      break;

   case ir_triop_bitfield_extract:
      if (lowering(EXTRACT_TO_SHIFTS))
         extract_to_shifts(ir);
      break;

   case ir_quadop_bitfield_insert:
      if (lowering(INSERT_TO_SHIFTS))
         insert_to_shifts(ir);
      break;

   case ir_unop_bitfield_reverse:
      if (lowering(REVERSE_TO_SHIFTS))
         reverse_to_shifts(ir);
      break;

   case ir_unop_find_lsb:
      if (lowering(FIND_LSB_TO_FLOAT_CAST))
         find_lsb_to_float_cast(ir);
      break;

   case ir_unop_find_msb:
      if (lowering(FIND_MSB_TO_FLOAT_CAST))
         find_msb_to_float_cast(ir);
      break;

   case ir_binop_imul_high:
      if (lowering(IMUL_HIGH_TO_MUL))
         imul_high_to_mul(ir);
      break;

   case ir_binop_mul:
      if (lowering(MUL64_TO_MUL_AND_MUL_HIGH) &&
          (ir->type->base_type == GLSL_TYPE_INT64 ||
           ir->type->base_type == GLSL_TYPE_UINT64) &&
          (ir->operands[0]->type->base_type == GLSL_TYPE_INT ||
           ir->operands[1]->type->base_type == GLSL_TYPE_UINT))
         mul64_to_mul_and_mul_high(ir);
      break;

   case ir_unop_rsq:
   case ir_unop_sqrt:
      if (lowering(SQRT_TO_ABS_SQRT))
         sqrt_to_abs_sqrt(ir);
      break;

   default:
      return visit_continue;
   }

   return visit_continue;
}
