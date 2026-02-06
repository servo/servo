/*
 * Copyright © 2012 Intel Corporation
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

#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "ir_rvalue_visitor.h"

namespace {

using namespace ir_builder;

/**
 * A visitor that lowers built-in floating-point pack/unpack expressions
 * such packSnorm2x16.
 */
class lower_packing_builtins_visitor : public ir_rvalue_visitor {
public:
   /**
    * \param op_mask is a bitmask of `enum lower_packing_builtins_op`
    */
   explicit lower_packing_builtins_visitor(int op_mask)
      : op_mask(op_mask),
        progress(false)
   {
      factory.instructions = &factory_instructions;
   }

   virtual ~lower_packing_builtins_visitor()
   {
      assert(factory_instructions.is_empty());
   }

   bool get_progress() { return progress; }

   void handle_rvalue(ir_rvalue **rvalue)
   {
      if (!*rvalue)
	 return;

      ir_expression *expr = (*rvalue)->as_expression();
      if (!expr)
	 return;

      enum lower_packing_builtins_op lowering_op =
         choose_lowering_op(expr->operation);

      if (lowering_op == LOWER_PACK_UNPACK_NONE)
         return;

      setup_factory(ralloc_parent(expr));

      ir_rvalue *op0 = expr->operands[0];
      ralloc_steal(factory.mem_ctx, op0);

      switch (lowering_op) {
      case LOWER_PACK_SNORM_2x16:
         *rvalue = lower_pack_snorm_2x16(op0);
         break;
      case LOWER_PACK_SNORM_4x8:
         *rvalue = lower_pack_snorm_4x8(op0);
         break;
      case LOWER_PACK_UNORM_2x16:
         *rvalue = lower_pack_unorm_2x16(op0);
         break;
      case LOWER_PACK_UNORM_4x8:
         *rvalue = lower_pack_unorm_4x8(op0);
         break;
      case LOWER_PACK_HALF_2x16:
         *rvalue = lower_pack_half_2x16(op0);
         break;
      case LOWER_UNPACK_SNORM_2x16:
         *rvalue = lower_unpack_snorm_2x16(op0);
         break;
      case LOWER_UNPACK_SNORM_4x8:
         *rvalue = lower_unpack_snorm_4x8(op0);
         break;
      case LOWER_UNPACK_UNORM_2x16:
         *rvalue = lower_unpack_unorm_2x16(op0);
         break;
      case LOWER_UNPACK_UNORM_4x8:
         *rvalue = lower_unpack_unorm_4x8(op0);
         break;
      case LOWER_UNPACK_HALF_2x16:
         *rvalue = lower_unpack_half_2x16(op0);
         break;
      case LOWER_PACK_UNPACK_NONE:
      case LOWER_PACK_USE_BFI:
      case LOWER_PACK_USE_BFE:
         assert(!"not reached");
         break;
      }

      teardown_factory();
      progress = true;
   }

private:
   const int op_mask;
   bool progress;
   ir_factory factory;
   exec_list factory_instructions;

   /**
    * Determine the needed lowering operation by filtering \a expr_op
    * through \ref op_mask.
    */
   enum lower_packing_builtins_op
   choose_lowering_op(ir_expression_operation expr_op)
   {
      /* C++ regards int and enum as fundamentally different types.
       * So, we can't simply return from each case; we must cast the return
       * value.
       */
      int result;

      switch (expr_op) {
      case ir_unop_pack_snorm_2x16:
         result = op_mask & LOWER_PACK_SNORM_2x16;
         break;
      case ir_unop_pack_snorm_4x8:
         result = op_mask & LOWER_PACK_SNORM_4x8;
         break;
      case ir_unop_pack_unorm_2x16:
         result = op_mask & LOWER_PACK_UNORM_2x16;
         break;
      case ir_unop_pack_unorm_4x8:
         result = op_mask & LOWER_PACK_UNORM_4x8;
         break;
      case ir_unop_pack_half_2x16:
         result = op_mask & LOWER_PACK_HALF_2x16;
         break;
      case ir_unop_unpack_snorm_2x16:
         result = op_mask & LOWER_UNPACK_SNORM_2x16;
         break;
      case ir_unop_unpack_snorm_4x8:
         result = op_mask & LOWER_UNPACK_SNORM_4x8;
         break;
      case ir_unop_unpack_unorm_2x16:
         result = op_mask & LOWER_UNPACK_UNORM_2x16;
         break;
      case ir_unop_unpack_unorm_4x8:
         result = op_mask & LOWER_UNPACK_UNORM_4x8;
         break;
      case ir_unop_unpack_half_2x16:
         result = op_mask & LOWER_UNPACK_HALF_2x16;
         break;
      default:
         result = LOWER_PACK_UNPACK_NONE;
         break;
      }

      return static_cast<enum lower_packing_builtins_op>(result);
   }

   void
   setup_factory(void *mem_ctx)
   {
      assert(factory.mem_ctx == NULL);
      assert(factory.instructions->is_empty());

      factory.mem_ctx = mem_ctx;
   }

   void
   teardown_factory()
   {
      base_ir->insert_before(factory.instructions);
      assert(factory.instructions->is_empty());
      factory.mem_ctx = NULL;
   }

   template <typename T>
   ir_constant*
   constant(T x)
   {
      return factory.constant(x);
   }

   /**
    * \brief Pack two uint16's into a single uint32.
    *
    * Interpret the given uvec2 as a uint16 pair. Pack the pair into a uint32
    * where the least significant bits specify the first element of the pair.
    * Return the uint32.
    */
   ir_rvalue*
   pack_uvec2_to_uint(ir_rvalue *uvec2_rval)
   {
      assert(uvec2_rval->type == glsl_type::uvec2_type);

      /* uvec2 u = UVEC2_RVAL; */
      ir_variable *u = factory.make_temp(glsl_type::uvec2_type,
                                         "tmp_pack_uvec2_to_uint");
      factory.emit(assign(u, uvec2_rval));

      if (op_mask & LOWER_PACK_USE_BFI) {
         return bitfield_insert(bit_and(swizzle_x(u), constant(0xffffu)),
                                swizzle_y(u),
                                constant(16u),
                                constant(16u));
      }

      /* return (u.y << 16) | (u.x & 0xffff); */
      return bit_or(lshift(swizzle_y(u), constant(16u)),
                    bit_and(swizzle_x(u), constant(0xffffu)));
   }

   /**
    * \brief Pack four uint8's into a single uint32.
    *
    * Interpret the given uvec4 as a uint32 4-typle. Pack the 4-tuple into a
    * uint32 where the least significant bits specify the first element of the
    * 4-tuple. Return the uint32.
    */
   ir_rvalue*
   pack_uvec4_to_uint(ir_rvalue *uvec4_rval)
   {
      assert(uvec4_rval->type == glsl_type::uvec4_type);

      ir_variable *u = factory.make_temp(glsl_type::uvec4_type,
                                         "tmp_pack_uvec4_to_uint");

      if (op_mask & LOWER_PACK_USE_BFI) {
         /* uvec4 u = UVEC4_RVAL; */
         factory.emit(assign(u, uvec4_rval));

         return bitfield_insert(bitfield_insert(
                                   bitfield_insert(
                                      bit_and(swizzle_x(u), constant(0xffu)),
                                      swizzle_y(u), constant(8u), constant(8u)),
                                   swizzle_z(u), constant(16u), constant(8u)),
                                swizzle_w(u), constant(24u), constant(8u));
      }

      /* uvec4 u = UVEC4_RVAL & 0xff */
      factory.emit(assign(u, bit_and(uvec4_rval, constant(0xffu))));

      /* return (u.w << 24) | (u.z << 16) | (u.y << 8) | u.x; */
      return bit_or(bit_or(lshift(swizzle_w(u), constant(24u)),
                           lshift(swizzle_z(u), constant(16u))),
                    bit_or(lshift(swizzle_y(u), constant(8u)),
                           swizzle_x(u)));
   }

   /**
    * \brief Unpack a uint32 into two uint16's.
    *
    * Interpret the given uint32 as a uint16 pair where the uint32's least
    * significant bits specify the pair's first element. Return the uint16
    * pair as a uvec2.
    */
   ir_rvalue*
   unpack_uint_to_uvec2(ir_rvalue *uint_rval)
   {
      assert(uint_rval->type == glsl_type::uint_type);

      /* uint u = UINT_RVAL; */
      ir_variable *u = factory.make_temp(glsl_type::uint_type,
                                          "tmp_unpack_uint_to_uvec2_u");
      factory.emit(assign(u, uint_rval));

      /* uvec2 u2; */
      ir_variable *u2 = factory.make_temp(glsl_type::uvec2_type,
                                           "tmp_unpack_uint_to_uvec2_u2");

      /* u2.x = u & 0xffffu; */
      factory.emit(assign(u2, bit_and(u, constant(0xffffu)), WRITEMASK_X));

      /* u2.y = u >> 16u; */
      factory.emit(assign(u2, rshift(u, constant(16u)), WRITEMASK_Y));

      return deref(u2).val;
   }

   /**
    * \brief Unpack a uint32 into two int16's.
    *
    * Specifically each 16-bit value is sign-extended to the full width of an
    * int32 on return.
    */
   ir_rvalue *
   unpack_uint_to_ivec2(ir_rvalue *uint_rval)
   {
      assert(uint_rval->type == glsl_type::uint_type);

      if (!(op_mask & LOWER_PACK_USE_BFE)) {
         return rshift(lshift(u2i(unpack_uint_to_uvec2(uint_rval)),
                              constant(16u)),
                       constant(16u));
      }

      ir_variable *i = factory.make_temp(glsl_type::int_type,
                                         "tmp_unpack_uint_to_ivec2_i");
      factory.emit(assign(i, u2i(uint_rval)));

      /* ivec2 i2; */
      ir_variable *i2 = factory.make_temp(glsl_type::ivec2_type,
                                          "tmp_unpack_uint_to_ivec2_i2");

      factory.emit(assign(i2, bitfield_extract(i, constant(0), constant(16)),
                          WRITEMASK_X));
      factory.emit(assign(i2, bitfield_extract(i, constant(16), constant(16)),
                          WRITEMASK_Y));

      return deref(i2).val;
   }

   /**
    * \brief Unpack a uint32 into four uint8's.
    *
    * Interpret the given uint32 as a uint8 4-tuple where the uint32's least
    * significant bits specify the 4-tuple's first element. Return the uint8
    * 4-tuple as a uvec4.
    */
   ir_rvalue*
   unpack_uint_to_uvec4(ir_rvalue *uint_rval)
   {
      assert(uint_rval->type == glsl_type::uint_type);

      /* uint u = UINT_RVAL; */
      ir_variable *u = factory.make_temp(glsl_type::uint_type,
                                          "tmp_unpack_uint_to_uvec4_u");
      factory.emit(assign(u, uint_rval));

      /* uvec4 u4; */
      ir_variable *u4 = factory.make_temp(glsl_type::uvec4_type,
                                           "tmp_unpack_uint_to_uvec4_u4");

      /* u4.x = u & 0xffu; */
      factory.emit(assign(u4, bit_and(u, constant(0xffu)), WRITEMASK_X));

      if (op_mask & LOWER_PACK_USE_BFE) {
         /* u4.y = bitfield_extract(u, 8, 8); */
         factory.emit(assign(u4, bitfield_extract(u, constant(8u), constant(8u)),
                             WRITEMASK_Y));

         /* u4.z = bitfield_extract(u, 16, 8); */
         factory.emit(assign(u4, bitfield_extract(u, constant(16u), constant(8u)),
                             WRITEMASK_Z));
      } else {
         /* u4.y = (u >> 8u) & 0xffu; */
         factory.emit(assign(u4, bit_and(rshift(u, constant(8u)),
                                         constant(0xffu)), WRITEMASK_Y));

         /* u4.z = (u >> 16u) & 0xffu; */
         factory.emit(assign(u4, bit_and(rshift(u, constant(16u)),
                                         constant(0xffu)), WRITEMASK_Z));
      }

      /* u4.w = (u >> 24u) */
      factory.emit(assign(u4, rshift(u, constant(24u)), WRITEMASK_W));

      return deref(u4).val;
   }

   /**
    * \brief Unpack a uint32 into four int8's.
    *
    * Specifically each 8-bit value is sign-extended to the full width of an
    * int32 on return.
    */
   ir_rvalue *
   unpack_uint_to_ivec4(ir_rvalue *uint_rval)
   {
      assert(uint_rval->type == glsl_type::uint_type);

      if (!(op_mask & LOWER_PACK_USE_BFE)) {
         return rshift(lshift(u2i(unpack_uint_to_uvec4(uint_rval)),
                              constant(24u)),
                       constant(24u));
      }

      ir_variable *i = factory.make_temp(glsl_type::int_type,
                                         "tmp_unpack_uint_to_ivec4_i");
      factory.emit(assign(i, u2i(uint_rval)));

      /* ivec4 i4; */
      ir_variable *i4 = factory.make_temp(glsl_type::ivec4_type,
                                          "tmp_unpack_uint_to_ivec4_i4");

      factory.emit(assign(i4, bitfield_extract(i, constant(0), constant(8)),
                          WRITEMASK_X));
      factory.emit(assign(i4, bitfield_extract(i, constant(8), constant(8)),
                          WRITEMASK_Y));
      factory.emit(assign(i4, bitfield_extract(i, constant(16), constant(8)),
                          WRITEMASK_Z));
      factory.emit(assign(i4, bitfield_extract(i, constant(24), constant(8)),
                          WRITEMASK_W));

      return deref(i4).val;
   }

   /**
    * \brief Lower a packSnorm2x16 expression.
    *
    * \param vec2_rval is packSnorm2x16's input
    * \return packSnorm2x16's output as a uint rvalue
    */
   ir_rvalue*
   lower_pack_snorm_2x16(ir_rvalue *vec2_rval)
   {
      /* From page 88 (94 of pdf) of the GLSL ES 3.00 spec:
       *
       *    highp uint packSnorm2x16(vec2 v)
       *    --------------------------------
       *    First, converts each component of the normalized floating-point value
       *    v into 16-bit integer values. Then, the results are packed into the
       *    returned 32-bit unsigned integer.
       *
       *    The conversion for component c of v to fixed point is done as
       *    follows:
       *
       *       packSnorm2x16: round(clamp(c, -1, +1) * 32767.0)
       *
       *    The first component of the vector will be written to the least
       *    significant bits of the output; the last component will be written to
       *    the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return pack_uvec2_to_uint(
       *         uvec2(ivec2(
       *           round(clamp(VEC2_RVALUE, -1.0f, 1.0f) * 32767.0f))));
       *
       * It is necessary to first convert the vec2 to ivec2 rather than directly
       * converting vec2 to uvec2 because the latter conversion is undefined.
       * From page 56 (62 of pdf) of the GLSL ES 3.00 spec: "It is undefined to
       * convert a negative floating point value to an uint".
       */
      assert(vec2_rval->type == glsl_type::vec2_type);

      ir_rvalue *result = pack_uvec2_to_uint(
            i2u(f2i(round_even(mul(clamp(vec2_rval,
                                         constant(-1.0f),
                                         constant(1.0f)),
                                   constant(32767.0f))))));

      assert(result->type == glsl_type::uint_type);
      return result;
   }

   /**
    * \brief Lower a packSnorm4x8 expression.
    *
    * \param vec4_rval is packSnorm4x8's input
    * \return packSnorm4x8's output as a uint rvalue
    */
   ir_rvalue*
   lower_pack_snorm_4x8(ir_rvalue *vec4_rval)
   {
      /* From page 137 (143 of pdf) of the GLSL 4.30 spec:
       *
       *    highp uint packSnorm4x8(vec4 v)
       *    -------------------------------
       *    First, converts each component of the normalized floating-point value
       *    v into 8-bit integer values. Then, the results are packed into the
       *    returned 32-bit unsigned integer.
       *
       *    The conversion for component c of v to fixed point is done as
       *    follows:
       *
       *       packSnorm4x8: round(clamp(c, -1, +1) * 127.0)
       *
       *    The first component of the vector will be written to the least
       *    significant bits of the output; the last component will be written to
       *    the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return pack_uvec4_to_uint(
       *         uvec4(ivec4(
       *           round(clamp(VEC4_RVALUE, -1.0f, 1.0f) * 127.0f))));
       *
       * It is necessary to first convert the vec4 to ivec4 rather than directly
       * converting vec4 to uvec4 because the latter conversion is undefined.
       * From page 87 (93 of pdf) of the GLSL 4.30 spec: "It is undefined to
       * convert a negative floating point value to an uint".
       */
      assert(vec4_rval->type == glsl_type::vec4_type);

      ir_rvalue *result = pack_uvec4_to_uint(
            i2u(f2i(round_even(mul(clamp(vec4_rval,
                                         constant(-1.0f),
                                         constant(1.0f)),
                                   constant(127.0f))))));

      assert(result->type == glsl_type::uint_type);
      return result;
   }

   /**
    * \brief Lower an unpackSnorm2x16 expression.
    *
    * \param uint_rval is unpackSnorm2x16's input
    * \return unpackSnorm2x16's output as a vec2 rvalue
    */
   ir_rvalue*
   lower_unpack_snorm_2x16(ir_rvalue *uint_rval)
   {
      /* From page 88 (94 of pdf) of the GLSL ES 3.00 spec:
       *
       *    highp vec2 unpackSnorm2x16 (highp uint p)
       *    -----------------------------------------
       *    First, unpacks a single 32-bit unsigned integer p into a pair of
       *    16-bit unsigned integers. Then, each component is converted to
       *    a normalized floating-point value to generate the returned
       *    two-component vector.
       *
       *    The conversion for unpacked fixed-point value f to floating point is
       *    done as follows:
       *
       *       unpackSnorm2x16: clamp(f / 32767.0, -1,+1)
       *
       *    The first component of the returned vector will be extracted from the
       *    least significant bits of the input; the last component will be
       *    extracted from the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *    return clamp(
       *       ((ivec2(unpack_uint_to_uvec2(UINT_RVALUE)) << 16) >> 16) / 32767.0f,
       *       -1.0f, 1.0f);
       *
       * The above IR may appear unnecessarily complex, but the intermediate
       * conversion to ivec2 and the bit shifts are necessary to correctly unpack
       * negative floats.
       *
       * To see why, consider packing and then unpacking vec2(-1.0, 0.0).
       * packSnorm2x16 encodes -1.0 as the int16 0xffff. During unpacking, we
       * place that int16 into an int32, which results in the *positive* integer
       * 0x0000ffff.  The int16's sign bit becomes, in the int32, the rather
       * unimportant bit 16. We must now extend the int16's sign bit into bits
       * 17-32, which is accomplished by left-shifting then right-shifting.
       */

      assert(uint_rval->type == glsl_type::uint_type);

      ir_rvalue *result =
        clamp(div(i2f(unpack_uint_to_ivec2(uint_rval)),
                  constant(32767.0f)),
              constant(-1.0f),
              constant(1.0f));

      assert(result->type == glsl_type::vec2_type);
      return result;
   }

   /**
    * \brief Lower an unpackSnorm4x8 expression.
    *
    * \param uint_rval is unpackSnorm4x8's input
    * \return unpackSnorm4x8's output as a vec4 rvalue
    */
   ir_rvalue*
   lower_unpack_snorm_4x8(ir_rvalue *uint_rval)
   {
      /* From page 137 (143 of pdf) of the GLSL 4.30 spec:
       *
       *    highp vec4 unpackSnorm4x8 (highp uint p)
       *    ----------------------------------------
       *    First, unpacks a single 32-bit unsigned integer p into four
       *    8-bit unsigned integers. Then, each component is converted to
       *    a normalized floating-point value to generate the returned
       *    four-component vector.
       *
       *    The conversion for unpacked fixed-point value f to floating point is
       *    done as follows:
       *
       *       unpackSnorm4x8: clamp(f / 127.0, -1, +1)
       *
       *    The first component of the returned vector will be extracted from the
       *    least significant bits of the input; the last component will be
       *    extracted from the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *    return clamp(
       *       ((ivec4(unpack_uint_to_uvec4(UINT_RVALUE)) << 24) >> 24) / 127.0f,
       *       -1.0f, 1.0f);
       *
       * The above IR may appear unnecessarily complex, but the intermediate
       * conversion to ivec4 and the bit shifts are necessary to correctly unpack
       * negative floats.
       *
       * To see why, consider packing and then unpacking vec4(-1.0, 0.0, 0.0,
       * 0.0). packSnorm4x8 encodes -1.0 as the int8 0xff. During unpacking, we
       * place that int8 into an int32, which results in the *positive* integer
       * 0x000000ff.  The int8's sign bit becomes, in the int32, the rather
       * unimportant bit 8. We must now extend the int8's sign bit into bits
       * 9-32, which is accomplished by left-shifting then right-shifting.
       */

      assert(uint_rval->type == glsl_type::uint_type);

      ir_rvalue *result =
        clamp(div(i2f(unpack_uint_to_ivec4(uint_rval)),
                  constant(127.0f)),
              constant(-1.0f),
              constant(1.0f));

      assert(result->type == glsl_type::vec4_type);
      return result;
   }

   /**
    * \brief Lower a packUnorm2x16 expression.
    *
    * \param vec2_rval is packUnorm2x16's input
    * \return packUnorm2x16's output as a uint rvalue
    */
   ir_rvalue*
   lower_pack_unorm_2x16(ir_rvalue *vec2_rval)
   {
      /* From page 88 (94 of pdf) of the GLSL ES 3.00 spec:
       *
       *    highp uint packUnorm2x16 (vec2 v)
       *    ---------------------------------
       *    First, converts each component of the normalized floating-point value
       *    v into 16-bit integer values. Then, the results are packed into the
       *    returned 32-bit unsigned integer.
       *
       *    The conversion for component c of v to fixed point is done as
       *    follows:
       *
       *       packUnorm2x16: round(clamp(c, 0, +1) * 65535.0)
       *
       *    The first component of the vector will be written to the least
       *    significant bits of the output; the last component will be written to
       *    the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return pack_uvec2_to_uint(uvec2(
       *                round(clamp(VEC2_RVALUE, 0.0f, 1.0f) * 65535.0f)));
       *
       * Here it is safe to directly convert the vec2 to uvec2 because the vec2
       * has been clamped to a non-negative range.
       */

      assert(vec2_rval->type == glsl_type::vec2_type);

      ir_rvalue *result = pack_uvec2_to_uint(
         f2u(round_even(mul(saturate(vec2_rval), constant(65535.0f)))));

      assert(result->type == glsl_type::uint_type);
      return result;
   }

   /**
    * \brief Lower a packUnorm4x8 expression.
    *
    * \param vec4_rval is packUnorm4x8's input
    * \return packUnorm4x8's output as a uint rvalue
    */
   ir_rvalue*
   lower_pack_unorm_4x8(ir_rvalue *vec4_rval)
   {
      /* From page 137 (143 of pdf) of the GLSL 4.30 spec:
       *
       *    highp uint packUnorm4x8 (vec4 v)
       *    --------------------------------
       *    First, converts each component of the normalized floating-point value
       *    v into 8-bit integer values. Then, the results are packed into the
       *    returned 32-bit unsigned integer.
       *
       *    The conversion for component c of v to fixed point is done as
       *    follows:
       *
       *       packUnorm4x8: round(clamp(c, 0, +1) * 255.0)
       *
       *    The first component of the vector will be written to the least
       *    significant bits of the output; the last component will be written to
       *    the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return pack_uvec4_to_uint(uvec4(
       *                round(clamp(VEC2_RVALUE, 0.0f, 1.0f) * 255.0f)));
       *
       * Here it is safe to directly convert the vec4 to uvec4 because the vec4
       * has been clamped to a non-negative range.
       */

      assert(vec4_rval->type == glsl_type::vec4_type);

      ir_rvalue *result = pack_uvec4_to_uint(
         f2u(round_even(mul(saturate(vec4_rval), constant(255.0f)))));

      assert(result->type == glsl_type::uint_type);
      return result;
   }

   /**
    * \brief Lower an unpackUnorm2x16 expression.
    *
    * \param uint_rval is unpackUnorm2x16's input
    * \return unpackUnorm2x16's output as a vec2 rvalue
    */
   ir_rvalue*
   lower_unpack_unorm_2x16(ir_rvalue *uint_rval)
   {
      /* From page 89 (95 of pdf) of the GLSL ES 3.00 spec:
       *
       *    highp vec2 unpackUnorm2x16 (highp uint p)
       *    -----------------------------------------
       *    First, unpacks a single 32-bit unsigned integer p into a pair of
       *    16-bit unsigned integers. Then, each component is converted to
       *    a normalized floating-point value to generate the returned
       *    two-component vector.
       *
       *    The conversion for unpacked fixed-point value f to floating point is
       *    done as follows:
       *
       *       unpackUnorm2x16: f / 65535.0
       *
       *    The first component of the returned vector will be extracted from the
       *    least significant bits of the input; the last component will be
       *    extracted from the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return vec2(unpack_uint_to_uvec2(UINT_RVALUE)) / 65535.0;
       */

      assert(uint_rval->type == glsl_type::uint_type);

      ir_rvalue *result = div(u2f(unpack_uint_to_uvec2(uint_rval)),
                              constant(65535.0f));

      assert(result->type == glsl_type::vec2_type);
      return result;
   }

   /**
    * \brief Lower an unpackUnorm4x8 expression.
    *
    * \param uint_rval is unpackUnorm4x8's input
    * \return unpackUnorm4x8's output as a vec4 rvalue
    */
   ir_rvalue*
   lower_unpack_unorm_4x8(ir_rvalue *uint_rval)
   {
      /* From page 137 (143 of pdf) of the GLSL 4.30 spec:
       *
       *    highp vec4 unpackUnorm4x8 (highp uint p)
       *    ----------------------------------------
       *    First, unpacks a single 32-bit unsigned integer p into four
       *    8-bit unsigned integers. Then, each component is converted to
       *    a normalized floating-point value to generate the returned
       *    two-component vector.
       *
       *    The conversion for unpacked fixed-point value f to floating point is
       *    done as follows:
       *
       *       unpackUnorm4x8: f / 255.0
       *
       *    The first component of the returned vector will be extracted from the
       *    least significant bits of the input; the last component will be
       *    extracted from the most significant bits.
       *
       * This function generates IR that approximates the following pseudo-GLSL:
       *
       *     return vec4(unpack_uint_to_uvec4(UINT_RVALUE)) / 255.0;
       */

      assert(uint_rval->type == glsl_type::uint_type);

      ir_rvalue *result = div(u2f(unpack_uint_to_uvec4(uint_rval)),
                              constant(255.0f));

      assert(result->type == glsl_type::vec4_type);
      return result;
   }

   /**
    * \brief Lower the component-wise calculation of packHalf2x16.
    *
    * \param f_rval is one component of packHafl2x16's input
    * \param e_rval is the unshifted exponent bits of f_rval
    * \param m_rval is the unshifted mantissa bits of f_rval
    *
    * \return a uint rvalue that encodes a float16 in its lower 16 bits
    */
   ir_rvalue*
   pack_half_1x16_nosign(ir_rvalue *f_rval,
                         ir_rvalue *e_rval,
                         ir_rvalue *m_rval)
   {
      assert(e_rval->type == glsl_type::uint_type);
      assert(m_rval->type == glsl_type::uint_type);

      /* uint u16; */
      ir_variable *u16 = factory.make_temp(glsl_type::uint_type,
                                           "tmp_pack_half_1x16_u16");

      /* float f = FLOAT_RVAL; */
      ir_variable *f = factory.make_temp(glsl_type::float_type,
                                          "tmp_pack_half_1x16_f");
      factory.emit(assign(f, f_rval));

      /* uint e = E_RVAL; */
      ir_variable *e = factory.make_temp(glsl_type::uint_type,
                                          "tmp_pack_half_1x16_e");
      factory.emit(assign(e, e_rval));

      /* uint m = M_RVAL; */
      ir_variable *m = factory.make_temp(glsl_type::uint_type,
                                          "tmp_pack_half_1x16_m");
      factory.emit(assign(m, m_rval));

      /* Preliminaries
       * -------------
       *
       * For a float16, the bit layout is:
       *
       *   sign:     15
       *   exponent: 10:14
       *   mantissa: 0:9
       *
       * Let f16 be a float16 value. The sign, exponent, and mantissa
       * determine its value thus:
       *
       *   if e16 = 0 and m16 = 0, then zero:       (-1)^s16 * 0                               (1)
       *   if e16 = 0 and m16!= 0, then subnormal:  (-1)^s16 * 2^(e16 - 14) * (m16 / 2^10)     (2)
       *   if 0 < e16 < 31, then normal:            (-1)^s16 * 2^(e16 - 15) * (1 + m16 / 2^10) (3)
       *   if e16 = 31 and m16 = 0, then infinite:  (-1)^s16 * inf                             (4)
       *   if e16 = 31 and m16 != 0, then           NaN                                        (5)
       *
       * where 0 <= m16 < 2^10.
       *
       * For a float32, the bit layout is:
       *
       *   sign:     31
       *   exponent: 23:30
       *   mantissa: 0:22
       *
       * Let f32 be a float32 value. The sign, exponent, and mantissa
       * determine its value thus:
       *
       *   if e32 = 0 and m32 = 0, then zero:        (-1)^s * 0                                (10)
       *   if e32 = 0 and m32 != 0, then subnormal:  (-1)^s * 2^(e32 - 126) * (m32 / 2^23)     (11)
       *   if 0 < e32 < 255, then normal:            (-1)^s * 2^(e32 - 127) * (1 + m32 / 2^23) (12)
       *   if e32 = 255 and m32 = 0, then infinite:  (-1)^s * inf                              (13)
       *   if e32 = 255 and m32 != 0, then           NaN                                       (14)
       *
       * where 0 <= m32 < 2^23.
       *
       * The minimum and maximum normal float16 values are
       *
       *   min_norm16 = 2^(1 - 15) * (1 + 0 / 2^10) = 2^(-14)   (20)
       *   max_norm16 = 2^(30 - 15) * (1 + 1023 / 2^10)         (21)
       *
       * The step at max_norm16 is
       *
       *   max_step16 = 2^5                                     (22)
       *
       * Observe that the float16 boundary values in equations 20-21 lie in the
       * range of normal float32 values.
       *
       *
       * Rounding Behavior
       * -----------------
       * Not all float32 values can be exactly represented as a float16. We
       * round all such intermediate float32 values to the nearest float16; if
       * the float32 is exactly between to float16 values, we round to the one
       * with an even mantissa. This rounding behavior has several benefits:
       *
       *   - It has no sign bias.
       *
       *   - It reproduces the behavior of real hardware: opcode F32TO16 in Intel's
       *     GPU ISA.
       *
       *   - By reproducing the behavior of the GPU (at least on Intel hardware),
       *     compile-time evaluation of constant packHalf2x16 GLSL expressions will
       *     result in the same value as if the expression were executed on the
       *     GPU.
       *
       * Calculation
       * -----------
       * Our task is to compute s16, e16, m16 given f32.  Since this function
       * ignores the sign bit, assume that s32 = s16 = 0.  There are several
       * cases consider.
       */

      factory.emit(

         /* Case 1) f32 is NaN
          *
          *   The resultant f16 will also be NaN.
          */

         /* if (e32 == 255 && m32 != 0) { */
         if_tree(logic_and(equal(e, constant(0xffu << 23u)),
                           logic_not(equal(m, constant(0u)))),

            assign(u16, constant(0x7fffu)),

         /* Case 2) f32 lies in the range [0, min_norm16).
          *
          *   The resultant float16 will be either zero, subnormal, or normal.
          *
          *   Solving
          *
          *     f32 = min_norm16       (30)
          *
          *   gives
          *
          *     e32 = 113 and m32 = 0  (31)
          *
          *   Therefore this case occurs if and only if
          *
          *     e32 < 113              (32)
          */

         /* } else if (e32 < 113) { */
         if_tree(less(e, constant(113u << 23u)),

            /* u16 = uint(round_to_even(abs(f32) * float(1u << 24u))); */
            assign(u16, f2u(round_even(mul(expr(ir_unop_abs, f),
                                           constant((float) (1 << 24)))))),

         /* Case 3) f32 lies in the range
          *         [min_norm16, max_norm16 + max_step16).
          *
          *   The resultant float16 will be either normal or infinite.
          *
          *   Solving
          *
          *     f32 = max_norm16 + max_step16           (40)
          *         = 2^15 * (1 + 1023 / 2^10) + 2^5    (41)
          *         = 2^16                              (42)
          *   gives
          *
          *     e32 = 143 and m32 = 0                   (43)
          *
          *   We already solved the boundary condition f32 = min_norm16 above
          *   in equation 31. Therefore this case occurs if and only if
          *
          *     113 <= e32 and e32 < 143
          */

         /* } else if (e32 < 143) { */
         if_tree(less(e, constant(143u << 23u)),

            /* The addition below handles the case where the mantissa rounds
             * up to 1024 and bumps the exponent.
             *
             * u16 = ((e - (112u << 23u)) >> 13u)
             *     + round_to_even((float(m) / (1u << 13u));
             */
            assign(u16, add(rshift(sub(e, constant(112u << 23u)),
                                   constant(13u)),
                            f2u(round_even(
                                  div(u2f(m), constant((float) (1 << 13))))))),

         /* Case 4) f32 lies in the range [max_norm16 + max_step16, inf].
          *
          *   The resultant float16 will be infinite.
          *
          *   The cases above caught all float32 values in the range
          *   [0, max_norm16 + max_step16), so this is the fall-through case.
          */

         /* } else { */

            assign(u16, constant(31u << 10u))))));

         /* } */

       return deref(u16).val;
   }

   /**
    * \brief Lower a packHalf2x16 expression.
    *
    * \param vec2_rval is packHalf2x16's input
    * \return packHalf2x16's output as a uint rvalue
    */
   ir_rvalue*
   lower_pack_half_2x16(ir_rvalue *vec2_rval)
   {
      /* From page 89 (95 of pdf) of the GLSL ES 3.00 spec:
       *
       *    highp uint packHalf2x16 (mediump vec2 v)
       *    ----------------------------------------
       *    Returns an unsigned integer obtained by converting the components of
       *    a two-component floating-point vector to the 16-bit floating-point
       *    representation found in the OpenGL ES Specification, and then packing
       *    these two 16-bit integers into a 32-bit unsigned integer.
       *
       *    The first vector component specifies the 16 least- significant bits
       *    of the result; the second component specifies the 16 most-significant
       *    bits.
       */

      assert(vec2_rval->type == glsl_type::vec2_type);

      /* vec2 f = VEC2_RVAL; */
      ir_variable *f = factory.make_temp(glsl_type::vec2_type,
                                         "tmp_pack_half_2x16_f");
      factory.emit(assign(f, vec2_rval));

      /* uvec2 f32 = bitcast_f2u(f); */
      ir_variable *f32 = factory.make_temp(glsl_type::uvec2_type,
                                            "tmp_pack_half_2x16_f32");
      factory.emit(assign(f32, expr(ir_unop_bitcast_f2u, f)));

      /* uvec2 f16; */
      ir_variable *f16 = factory.make_temp(glsl_type::uvec2_type,
                                        "tmp_pack_half_2x16_f16");

      /* Get f32's unshifted exponent bits.
       *
       *   uvec2 e = f32 & 0x7f800000u;
       */
      ir_variable *e = factory.make_temp(glsl_type::uvec2_type,
                                          "tmp_pack_half_2x16_e");
      factory.emit(assign(e, bit_and(f32, constant(0x7f800000u))));

      /* Get f32's unshifted mantissa bits.
       *
       *   uvec2 m = f32 & 0x007fffffu;
       */
      ir_variable *m = factory.make_temp(glsl_type::uvec2_type,
                                          "tmp_pack_half_2x16_m");
      factory.emit(assign(m, bit_and(f32, constant(0x007fffffu))));

      /* Set f16's exponent and mantissa bits.
       *
       *   f16.x = pack_half_1x16_nosign(e.x, m.x);
       *   f16.y = pack_half_1y16_nosign(e.y, m.y);
       */
      factory.emit(assign(f16, pack_half_1x16_nosign(swizzle_x(f),
                                                     swizzle_x(e),
                                                     swizzle_x(m)),
                           WRITEMASK_X));
      factory.emit(assign(f16, pack_half_1x16_nosign(swizzle_y(f),
                                                     swizzle_y(e),
                                                     swizzle_y(m)),
                           WRITEMASK_Y));

      /* Set f16's sign bits.
       *
       *   f16 |= (f32 & (1u << 31u) >> 16u;
       */
      factory.emit(
         assign(f16, bit_or(f16,
                            rshift(bit_and(f32, constant(1u << 31u)),
                                   constant(16u)))));


      /* return (f16.y << 16u) | f16.x; */
      ir_rvalue *result = bit_or(lshift(swizzle_y(f16),
                                        constant(16u)),
                                 swizzle_x(f16));

      assert(result->type == glsl_type::uint_type);
      return result;
   }

   /**
    * \brief Lower the component-wise calculation of unpackHalf2x16.
    *
    * Given a uint that encodes a float16 in its lower 16 bits, this function
    * returns a uint that encodes a float32 with the same value. The sign bit
    * of the float16 is ignored.
    *
    * \param e_rval is the unshifted exponent bits of a float16
    * \param m_rval is the unshifted mantissa bits of a float16
    * \param a uint rvalue that encodes a float32
    */
   ir_rvalue*
   unpack_half_1x16_nosign(ir_rvalue *e_rval, ir_rvalue *m_rval)
   {
      assert(e_rval->type == glsl_type::uint_type);
      assert(m_rval->type == glsl_type::uint_type);

      /* uint u32; */
      ir_variable *u32 = factory.make_temp(glsl_type::uint_type,
                                           "tmp_unpack_half_1x16_u32");

      /* uint e = E_RVAL; */
      ir_variable *e = factory.make_temp(glsl_type::uint_type,
                                          "tmp_unpack_half_1x16_e");
      factory.emit(assign(e, e_rval));

      /* uint m = M_RVAL; */
      ir_variable *m = factory.make_temp(glsl_type::uint_type,
                                          "tmp_unpack_half_1x16_m");
      factory.emit(assign(m, m_rval));

      /* Preliminaries
       * -------------
       *
       * For a float16, the bit layout is:
       *
       *   sign:     15
       *   exponent: 10:14
       *   mantissa: 0:9
       *
       * Let f16 be a float16 value. The sign, exponent, and mantissa
       * determine its value thus:
       *
       *   if e16 = 0 and m16 = 0, then zero:       (-1)^s16 * 0                               (1)
       *   if e16 = 0 and m16!= 0, then subnormal:  (-1)^s16 * 2^(e16 - 14) * (m16 / 2^10)     (2)
       *   if 0 < e16 < 31, then normal:            (-1)^s16 * 2^(e16 - 15) * (1 + m16 / 2^10) (3)
       *   if e16 = 31 and m16 = 0, then infinite:  (-1)^s16 * inf                             (4)
       *   if e16 = 31 and m16 != 0, then           NaN                                        (5)
       *
       * where 0 <= m16 < 2^10.
       *
       * For a float32, the bit layout is:
       *
       *   sign: 31
       *   exponent: 23:30
       *   mantissa: 0:22
       *
       * Let f32 be a float32 value. The sign, exponent, and mantissa
       * determine its value thus:
       *
       *   if e32 = 0 and m32 = 0, then zero:        (-1)^s * 0                                (10)
       *   if e32 = 0 and m32 != 0, then subnormal:  (-1)^s * 2^(e32 - 126) * (m32 / 2^23)     (11)
       *   if 0 < e32 < 255, then normal:            (-1)^s * 2^(e32 - 127) * (1 + m32 / 2^23) (12)
       *   if e32 = 255 and m32 = 0, then infinite:  (-1)^s * inf                              (13)
       *   if e32 = 255 and m32 != 0, then           NaN                                       (14)
       *
       * where 0 <= m32 < 2^23.
       *
       * Calculation
       * -----------
       * Our task is to compute s32, e32, m32 given f16.  Since this function
       * ignores the sign bit, assume that s32 = s16 = 0.  There are several
       * cases consider.
       */

      factory.emit(

         /* Case 1) f16 is zero or subnormal.
          *
          *   The simplest method of calcuating f32 in this case is
          *
          *     f32 = f16                       (20)
          *         = 2^(-14) * (m16 / 2^10)    (21)
          *         = m16 / 2^(-24)             (22)
          */

         /* if (e16 == 0) { */
         if_tree(equal(e, constant(0u)),

            /* u32 = bitcast_f2u(float(m) / float(1 << 24)); */
            assign(u32, expr(ir_unop_bitcast_f2u,
                                div(u2f(m), constant((float)(1 << 24))))),

         /* Case 2) f16 is normal.
          *
          *   The equation
          *
          *     f32 = f16                              (30)
          *     2^(e32 - 127) * (1 + m32 / 2^23) =     (31)
          *       2^(e16 - 15) * (1 + m16 / 2^10)
          *
          *   can be decomposed into two
          *
          *     2^(e32 - 127) = 2^(e16 - 15)           (32)
          *     1 + m32 / 2^23 = 1 + m16 / 2^10        (33)
          *
          *   which solve to
          *
          *     e32 = e16 + 112                        (34)
          *     m32 = m16 * 2^13                       (35)
          */

         /* } else if (e16 < 31)) { */
         if_tree(less(e, constant(31u << 10u)),

              /* u32 = ((e + (112 << 10)) | m) << 13;
               */
              assign(u32, lshift(bit_or(add(e, constant(112u << 10u)), m),
                                 constant(13u))),


         /* Case 3) f16 is infinite. */
         if_tree(equal(m, constant(0u)),

                 assign(u32, constant(255u << 23u)),

         /* Case 4) f16 is NaN. */
         /* } else { */

            assign(u32, constant(0x7fffffffu))))));

         /* } */

      return deref(u32).val;
   }

   /**
    * \brief Lower an unpackHalf2x16 expression.
    *
    * \param uint_rval is unpackHalf2x16's input
    * \return unpackHalf2x16's output as a vec2 rvalue
    */
   ir_rvalue*
   lower_unpack_half_2x16(ir_rvalue *uint_rval)
   {
      /* From page 89 (95 of pdf) of the GLSL ES 3.00 spec:
       *
       *    mediump vec2 unpackHalf2x16 (highp uint v)
       *    ------------------------------------------
       *    Returns a two-component floating-point vector with components
       *    obtained by unpacking a 32-bit unsigned integer into a pair of 16-bit
       *    values, interpreting those values as 16-bit floating-point numbers
       *    according to the OpenGL ES Specification, and converting them to
       *    32-bit floating-point values.
       *
       *    The first component of the vector is obtained from the
       *    16 least-significant bits of v; the second component is obtained
       *    from the 16 most-significant bits of v.
       */
      assert(uint_rval->type == glsl_type::uint_type);

      /* uint u = RVALUE;
       * uvec2 f16 = uvec2(u.x & 0xffff, u.y >> 16);
       */
      ir_variable *f16 = factory.make_temp(glsl_type::uvec2_type,
                                            "tmp_unpack_half_2x16_f16");
      factory.emit(assign(f16, unpack_uint_to_uvec2(uint_rval)));

      /* uvec2 f32; */
      ir_variable *f32 = factory.make_temp(glsl_type::uvec2_type,
                                            "tmp_unpack_half_2x16_f32");

      /* Get f16's unshifted exponent bits.
       *
       *    uvec2 e = f16 & 0x7c00u;
       */
      ir_variable *e = factory.make_temp(glsl_type::uvec2_type,
                                          "tmp_unpack_half_2x16_e");
      factory.emit(assign(e, bit_and(f16, constant(0x7c00u))));

      /* Get f16's unshifted mantissa bits.
       *
       *    uvec2 m = f16 & 0x03ffu;
       */
      ir_variable *m = factory.make_temp(glsl_type::uvec2_type,
                                          "tmp_unpack_half_2x16_m");
      factory.emit(assign(m, bit_and(f16, constant(0x03ffu))));

      /* Set f32's exponent and mantissa bits.
       *
       *   f32.x = unpack_half_1x16_nosign(e.x, m.x);
       *   f32.y = unpack_half_1x16_nosign(e.y, m.y);
       */
      factory.emit(assign(f32, unpack_half_1x16_nosign(swizzle_x(e),
                                                       swizzle_x(m)),
                           WRITEMASK_X));
      factory.emit(assign(f32, unpack_half_1x16_nosign(swizzle_y(e),
                                                       swizzle_y(m)),
                           WRITEMASK_Y));

      /* Set f32's sign bit.
       *
       *    f32 |= (f16 & 0x8000u) << 16u;
       */
      factory.emit(assign(f32, bit_or(f32,
                                       lshift(bit_and(f16,
                                                      constant(0x8000u)),
                                              constant(16u)))));

      /* return bitcast_u2f(f32); */
      ir_rvalue *result = expr(ir_unop_bitcast_u2f, f32);
      assert(result->type == glsl_type::vec2_type);
      return result;
   }
};

} // namespace anonymous

/**
 * \brief Lower the builtin packing functions.
 *
 * \param op_mask is a bitmask of `enum lower_packing_builtins_op`.
 */
bool
lower_packing_builtins(exec_list *instructions, int op_mask)
{
   lower_packing_builtins_visitor v(op_mask);
   visit_list_elements(&v, instructions, true);
   return v.get_progress();
}
