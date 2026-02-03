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
 * \file ir_constant_expression.cpp
 * Evaluate and process constant valued expressions
 *
 * In GLSL, constant valued expressions are used in several places.  These
 * must be processed and evaluated very early in the compilation process.
 *
 *    * Sizes of arrays
 *    * Initializers for uniforms
 *    * Initializers for \c const variables
 */

#include <math.h>
#include "util/rounding.h" /* for _mesa_roundeven */
#include "util/half_float.h"
#include "ir.h"
#include "compiler/glsl_types.h"
#include "util/hash_table.h"
#include "util/u_math.h"

static float
dot_f(ir_constant *op0, ir_constant *op1)
{
   assert(op0->type->is_float() && op1->type->is_float());

   float result = 0;
   for (unsigned c = 0; c < op0->type->components(); c++)
      result += op0->value.f[c] * op1->value.f[c];

   return result;
}

static double
dot_d(ir_constant *op0, ir_constant *op1)
{
   assert(op0->type->is_double() && op1->type->is_double());

   double result = 0;
   for (unsigned c = 0; c < op0->type->components(); c++)
      result += op0->value.d[c] * op1->value.d[c];

   return result;
}

/* This method is the only one supported by gcc.  Unions in particular
 * are iffy, and read-through-converted-pointer is killed by strict
 * aliasing.  OTOH, the compiler sees through the memcpy, so the
 * resulting asm is reasonable.
 */
static float
bitcast_u2f(unsigned int u)
{
   static_assert(sizeof(float) == sizeof(unsigned int),
                 "float and unsigned int size mismatch");
   float f;
   memcpy(&f, &u, sizeof(f));
   return f;
}

static unsigned int
bitcast_f2u(float f)
{
   static_assert(sizeof(float) == sizeof(unsigned int),
                 "float and unsigned int size mismatch");
   unsigned int u;
   memcpy(&u, &f, sizeof(f));
   return u;
}

static double
bitcast_u642d(uint64_t u)
{
   static_assert(sizeof(double) == sizeof(uint64_t),
                 "double and uint64_t size mismatch");
   double d;
   memcpy(&d, &u, sizeof(d));
   return d;
}

static double
bitcast_i642d(int64_t i)
{
   static_assert(sizeof(double) == sizeof(int64_t),
                 "double and int64_t size mismatch");
   double d;
   memcpy(&d, &i, sizeof(d));
   return d;
}

static uint64_t
bitcast_d2u64(double d)
{
   static_assert(sizeof(double) == sizeof(uint64_t),
                 "double and uint64_t size mismatch");
   uint64_t u;
   memcpy(&u, &d, sizeof(d));
   return u;
}

static int64_t
bitcast_d2i64(double d)
{
   static_assert(sizeof(double) == sizeof(int64_t),
                 "double and int64_t size mismatch");
   int64_t i;
   memcpy(&i, &d, sizeof(d));
   return i;
}

/**
 * Evaluate one component of a floating-point 4x8 unpacking function.
 */
typedef uint8_t
(*pack_1x8_func_t)(float);

/**
 * Evaluate one component of a floating-point 2x16 unpacking function.
 */
typedef uint16_t
(*pack_1x16_func_t)(float);

/**
 * Evaluate one component of a floating-point 4x8 unpacking function.
 */
typedef float
(*unpack_1x8_func_t)(uint8_t);

/**
 * Evaluate one component of a floating-point 2x16 unpacking function.
 */
typedef float
(*unpack_1x16_func_t)(uint16_t);

/**
 * Evaluate a 2x16 floating-point packing function.
 */
static uint32_t
pack_2x16(pack_1x16_func_t pack_1x16,
          float x, float y)
{
   /* From section 8.4 of the GLSL ES 3.00 spec:
    *
    *    packSnorm2x16
    *    -------------
    *    The first component of the vector will be written to the least
    *    significant bits of the output; the last component will be written to
    *    the most significant bits.
    *
    * The specifications for the other packing functions contain similar
    * language.
    */
   uint32_t u = 0;
   u |= ((uint32_t) pack_1x16(x) << 0);
   u |= ((uint32_t) pack_1x16(y) << 16);
   return u;
}

/**
 * Evaluate a 4x8 floating-point packing function.
 */
static uint32_t
pack_4x8(pack_1x8_func_t pack_1x8,
         float x, float y, float z, float w)
{
   /* From section 8.4 of the GLSL 4.30 spec:
    *
    *    packSnorm4x8
    *    ------------
    *    The first component of the vector will be written to the least
    *    significant bits of the output; the last component will be written to
    *    the most significant bits.
    *
    * The specifications for the other packing functions contain similar
    * language.
    */
   uint32_t u = 0;
   u |= ((uint32_t) pack_1x8(x) << 0);
   u |= ((uint32_t) pack_1x8(y) << 8);
   u |= ((uint32_t) pack_1x8(z) << 16);
   u |= ((uint32_t) pack_1x8(w) << 24);
   return u;
}

/**
 * Evaluate a 2x16 floating-point unpacking function.
 */
static void
unpack_2x16(unpack_1x16_func_t unpack_1x16,
            uint32_t u,
            float *x, float *y)
{
    /* From section 8.4 of the GLSL ES 3.00 spec:
     *
     *    unpackSnorm2x16
     *    ---------------
     *    The first component of the returned vector will be extracted from
     *    the least significant bits of the input; the last component will be
     *    extracted from the most significant bits.
     *
     * The specifications for the other unpacking functions contain similar
     * language.
     */
   *x = unpack_1x16((uint16_t) (u & 0xffff));
   *y = unpack_1x16((uint16_t) (u >> 16));
}

/**
 * Evaluate a 4x8 floating-point unpacking function.
 */
static void
unpack_4x8(unpack_1x8_func_t unpack_1x8, uint32_t u,
           float *x, float *y, float *z, float *w)
{
    /* From section 8.4 of the GLSL 4.30 spec:
     *
     *    unpackSnorm4x8
     *    --------------
     *    The first component of the returned vector will be extracted from
     *    the least significant bits of the input; the last component will be
     *    extracted from the most significant bits.
     *
     * The specifications for the other unpacking functions contain similar
     * language.
     */
   *x = unpack_1x8((uint8_t) (u & 0xff));
   *y = unpack_1x8((uint8_t) (u >> 8));
   *z = unpack_1x8((uint8_t) (u >> 16));
   *w = unpack_1x8((uint8_t) (u >> 24));
}

/**
 * Evaluate one component of packSnorm4x8.
 */
static uint8_t
pack_snorm_1x8(float x)
{
    /* From section 8.4 of the GLSL 4.30 spec:
     *
     *    packSnorm4x8
     *    ------------
     *    The conversion for component c of v to fixed point is done as
     *    follows:
     *
     *      packSnorm4x8: round(clamp(c, -1, +1) * 127.0)
     */
   return (uint8_t)
          _mesa_lroundevenf(CLAMP(x, -1.0f, +1.0f) * 127.0f);
}

/**
 * Evaluate one component of packSnorm2x16.
 */
static uint16_t
pack_snorm_1x16(float x)
{
    /* From section 8.4 of the GLSL ES 3.00 spec:
     *
     *    packSnorm2x16
     *    -------------
     *    The conversion for component c of v to fixed point is done as
     *    follows:
     *
     *      packSnorm2x16: round(clamp(c, -1, +1) * 32767.0)
     */
   return (uint16_t)
          _mesa_lroundevenf(CLAMP(x, -1.0f, +1.0f) * 32767.0f);
}

/**
 * Evaluate one component of unpackSnorm4x8.
 */
static float
unpack_snorm_1x8(uint8_t u)
{
    /* From section 8.4 of the GLSL 4.30 spec:
     *
     *    unpackSnorm4x8
     *    --------------
     *    The conversion for unpacked fixed-point value f to floating point is
     *    done as follows:
     *
     *       unpackSnorm4x8: clamp(f / 127.0, -1, +1)
     */
   return CLAMP((int8_t) u / 127.0f, -1.0f, +1.0f);
}

/**
 * Evaluate one component of unpackSnorm2x16.
 */
static float
unpack_snorm_1x16(uint16_t u)
{
    /* From section 8.4 of the GLSL ES 3.00 spec:
     *
     *    unpackSnorm2x16
     *    ---------------
     *    The conversion for unpacked fixed-point value f to floating point is
     *    done as follows:
     *
     *       unpackSnorm2x16: clamp(f / 32767.0, -1, +1)
     */
   return CLAMP((int16_t) u / 32767.0f, -1.0f, +1.0f);
}

/**
 * Evaluate one component packUnorm4x8.
 */
static uint8_t
pack_unorm_1x8(float x)
{
    /* From section 8.4 of the GLSL 4.30 spec:
     *
     *    packUnorm4x8
     *    ------------
     *    The conversion for component c of v to fixed point is done as
     *    follows:
     *
     *       packUnorm4x8: round(clamp(c, 0, +1) * 255.0)
     */
   return (uint8_t) (int) _mesa_roundevenf(CLAMP(x, 0.0f, 1.0f) * 255.0f);
}

/**
 * Evaluate one component packUnorm2x16.
 */
static uint16_t
pack_unorm_1x16(float x)
{
    /* From section 8.4 of the GLSL ES 3.00 spec:
     *
     *    packUnorm2x16
     *    -------------
     *    The conversion for component c of v to fixed point is done as
     *    follows:
     *
     *       packUnorm2x16: round(clamp(c, 0, +1) * 65535.0)
     */
   return (uint16_t) (int)
          _mesa_roundevenf(CLAMP(x, 0.0f, 1.0f) * 65535.0f);
}

/**
 * Evaluate one component of unpackUnorm4x8.
 */
static float
unpack_unorm_1x8(uint8_t u)
{
    /* From section 8.4 of the GLSL 4.30 spec:
     *
     *    unpackUnorm4x8
     *    --------------
     *    The conversion for unpacked fixed-point value f to floating point is
     *    done as follows:
     *
     *       unpackUnorm4x8: f / 255.0
     */
   return (float) u / 255.0f;
}

/**
 * Evaluate one component of unpackUnorm2x16.
 */
static float
unpack_unorm_1x16(uint16_t u)
{
    /* From section 8.4 of the GLSL ES 3.00 spec:
     *
     *    unpackUnorm2x16
     *    ---------------
     *    The conversion for unpacked fixed-point value f to floating point is
     *    done as follows:
     *
     *       unpackUnorm2x16: f / 65535.0
     */
   return (float) u / 65535.0f;
}

/**
 * Evaluate one component of packHalf2x16.
 */
static uint16_t
pack_half_1x16(float x)
{
   return _mesa_float_to_half(x);
}

/**
 * Evaluate one component of unpackHalf2x16.
 */
static float
unpack_half_1x16(uint16_t u)
{
   return _mesa_half_to_float(u);
}

static int32_t
iadd_saturate(int32_t a, int32_t b)
{
   return CLAMP(int64_t(a) + int64_t(b), INT32_MIN, INT32_MAX);
}

static int64_t
iadd64_saturate(int64_t a, int64_t b)
{
   if (a < 0 && b < INT64_MIN - a)
      return INT64_MIN;

   if (a > 0 && b > INT64_MAX - a)
      return INT64_MAX;

   return a + b;
}

static int32_t
isub_saturate(int32_t a, int32_t b)
{
   return CLAMP(int64_t(a) - int64_t(b), INT32_MIN, INT32_MAX);
}

static int64_t
isub64_saturate(int64_t a, int64_t b)
{
   if (b > 0 && a < INT64_MIN + b)
      return INT64_MIN;

   if (b < 0 && a > INT64_MAX + b)
      return INT64_MAX;

   return a - b;
}

static uint64_t
pack_2x32(uint32_t a, uint32_t b)
{
   uint64_t v = a;
   v |= (uint64_t)b << 32;
   return v;
}

static void
unpack_2x32(uint64_t p, uint32_t *a, uint32_t *b)
{
   *a = p & 0xffffffff;
   *b = (p >> 32);
}

/**
 * Get the constant that is ultimately referenced by an r-value, in a constant
 * expression evaluation context.
 *
 * The offset is used when the reference is to a specific column of a matrix.
 */
static bool
constant_referenced(const ir_dereference *deref,
                    struct hash_table *variable_context,
                    ir_constant *&store, int &offset)
{
   store = NULL;
   offset = 0;

   if (variable_context == NULL)
      return false;

   switch (deref->ir_type) {
   case ir_type_dereference_array: {
      const ir_dereference_array *const da =
         (const ir_dereference_array *) deref;

      ir_constant *const index_c =
         da->array_index->constant_expression_value(variable_context);

      if (!index_c || !index_c->type->is_scalar() ||
          !index_c->type->is_integer_32())
         break;

      const int index = index_c->type->base_type == GLSL_TYPE_INT ?
         index_c->get_int_component(0) :
         index_c->get_uint_component(0);

      ir_constant *substore;
      int suboffset;

      const ir_dereference *const deref = da->array->as_dereference();
      if (!deref)
         break;

      if (!constant_referenced(deref, variable_context, substore, suboffset))
         break;

      const glsl_type *const vt = da->array->type;
      if (vt->is_array()) {
         store = substore->get_array_element(index);
         offset = 0;
      } else if (vt->is_matrix()) {
         store = substore;
         offset = index * vt->vector_elements;
      } else if (vt->is_vector()) {
         store = substore;
         offset = suboffset + index;
      }

      break;
   }

   case ir_type_dereference_record: {
      const ir_dereference_record *const dr =
         (const ir_dereference_record *) deref;

      const ir_dereference *const deref = dr->record->as_dereference();
      if (!deref)
         break;

      ir_constant *substore;
      int suboffset;

      if (!constant_referenced(deref, variable_context, substore, suboffset))
         break;

      /* Since we're dropping it on the floor...
       */
      assert(suboffset == 0);

      store = substore->get_record_field(dr->field_idx);
      break;
   }

   case ir_type_dereference_variable: {
      const ir_dereference_variable *const dv =
         (const ir_dereference_variable *) deref;

      hash_entry *entry = _mesa_hash_table_search(variable_context, dv->var);
      if (entry)
         store = (ir_constant *) entry->data;
      break;
   }

   default:
      assert(!"Should not get here.");
      break;
   }

   return store != NULL;
}


ir_constant *
ir_rvalue::constant_expression_value(void *, struct hash_table *)
{
   assert(this->type->is_error());
   return NULL;
}

static uint32_t
bitfield_reverse(uint32_t v)
{
   /* http://graphics.stanford.edu/~seander/bithacks.html#BitReverseObvious */
   uint32_t r = v; // r will be reversed bits of v; first get LSB of v
   int s = sizeof(v) * CHAR_BIT - 1; // extra shift needed at end

   for (v >>= 1; v; v >>= 1) {
      r <<= 1;
      r |= v & 1;
      s--;
   }
   r <<= s; // shift when v's highest bits are zero

   return r;
}

static int
find_msb_uint(uint32_t v)
{
   int count = 0;

   /* If v == 0, then the loop will terminate when count == 32.  In that case
    * 31-count will produce the -1 result required by GLSL findMSB().
    */
   while (((v & (1u << 31)) == 0) && count != 32) {
      count++;
      v <<= 1;
   }

   return 31 - count;
}

static int
find_msb_int(int32_t v)
{
   /* If v is signed, findMSB() returns the position of the most significant
    * zero bit.
    */
   return find_msb_uint(v < 0 ? ~v : v);
}

static float
ldexpf_flush_subnormal(float x, int exp)
{
   const float result = ldexpf(x, exp);

   /* Flush subnormal values to zero. */
   return !isnormal(result) ? copysignf(0.0f, x) : result;
}

static double
ldexp_flush_subnormal(double x, int exp)
{
   const double result = ldexp(x, exp);

   /* Flush subnormal values to zero. */
   return !isnormal(result) ? copysign(0.0, x) : result;
}

static uint32_t
bitfield_extract_uint(uint32_t value, int offset, int bits)
{
   if (bits == 0)
      return 0;
   else if (offset < 0 || bits < 0)
      return 0; /* Undefined, per spec. */
   else if (offset + bits > 32)
      return 0; /* Undefined, per spec. */
   else {
      value <<= 32 - bits - offset;
      value >>= 32 - bits;
      return value;
   }
}

static int32_t
bitfield_extract_int(int32_t value, int offset, int bits)
{
   if (bits == 0)
      return 0;
   else if (offset < 0 || bits < 0)
      return 0; /* Undefined, per spec. */
   else if (offset + bits > 32)
      return 0; /* Undefined, per spec. */
   else {
      value <<= 32 - bits - offset;
      value >>= 32 - bits;
      return value;
   }
}

static uint32_t
bitfield_insert(uint32_t base, uint32_t insert, int offset, int bits)
{
   if (bits == 0)
      return base;
   else if (offset < 0 || bits < 0)
      return 0; /* Undefined, per spec. */
   else if (offset + bits > 32)
      return 0; /* Undefined, per spec. */
   else {
      unsigned insert_mask = ((1ull << bits) - 1) << offset;

      insert <<= offset;
      insert &= insert_mask;
      base &= ~insert_mask;

      return base | insert;
   }
}

ir_constant *
ir_expression::constant_expression_value(void *mem_ctx,
                                         struct hash_table *variable_context)
{
   assert(mem_ctx);

   if (this->type->is_error())
      return NULL;

   ir_constant *op[ARRAY_SIZE(this->operands)] = { NULL, };
   ir_constant_data data;

   memset(&data, 0, sizeof(data));

   for (unsigned operand = 0; operand < this->num_operands; operand++) {
      op[operand] =
         this->operands[operand]->constant_expression_value(mem_ctx,
                                                            variable_context);
      if (!op[operand])
         return NULL;
   }

   for (unsigned operand = 0; operand < this->num_operands; operand++) {
      if (op[operand]->type->base_type == GLSL_TYPE_FLOAT16) {
         const struct glsl_type *float_type =
            glsl_type::get_instance(GLSL_TYPE_FLOAT,
                                    op[operand]->type->vector_elements,
                                    op[operand]->type->matrix_columns,
                                    op[operand]->type->explicit_stride,
                                    op[operand]->type->interface_row_major);

         ir_constant_data f;
         for (unsigned i = 0; i < ARRAY_SIZE(f.f); i++)
            f.f[i] = _mesa_half_to_float(op[operand]->value.f16[i]);

         op[operand] = new(mem_ctx) ir_constant(float_type, &f);
      }
   }

   if (op[1] != NULL)
      switch (this->operation) {
      case ir_binop_lshift:
      case ir_binop_rshift:
      case ir_binop_ldexp:
      case ir_binop_interpolate_at_offset:
      case ir_binop_interpolate_at_sample:
      case ir_binop_vector_extract:
      case ir_triop_csel:
      case ir_triop_bitfield_extract:
         break;

      default:
         assert(op[0]->type->base_type == op[1]->type->base_type);
         break;
      }

   bool op0_scalar = op[0]->type->is_scalar();
   bool op1_scalar = op[1] != NULL && op[1]->type->is_scalar();

   /* When iterating over a vector or matrix's components, we want to increase
    * the loop counter.  However, for scalars, we want to stay at 0.
    */
   unsigned c0_inc = op0_scalar ? 0 : 1;
   unsigned c1_inc = op1_scalar ? 0 : 1;
   unsigned components;
   if (op1_scalar || !op[1]) {
      components = op[0]->type->components();
   } else {
      components = op[1]->type->components();
   }

   /* Handle array operations here, rather than below. */
   if (op[0]->type->is_array()) {
      assert(op[1] != NULL && op[1]->type->is_array());
      switch (this->operation) {
      case ir_binop_all_equal:
         return new(mem_ctx) ir_constant(op[0]->has_value(op[1]));
      case ir_binop_any_nequal:
         return new(mem_ctx) ir_constant(!op[0]->has_value(op[1]));
      default:
         break;
      }
      return NULL;
   }

#include "ir_expression_operation_constant.h"

   if (this->type->base_type == GLSL_TYPE_FLOAT16) {
      ir_constant_data f;
      for (unsigned i = 0; i < ARRAY_SIZE(f.f16); i++)
         f.f16[i] = _mesa_float_to_half(data.f[i]);

      return new(mem_ctx) ir_constant(this->type, &f);
   }


   return new(mem_ctx) ir_constant(this->type, &data);
}


ir_constant *
ir_texture::constant_expression_value(void *, struct hash_table *)
{
   /* texture lookups aren't constant expressions */
   return NULL;
}


ir_constant *
ir_swizzle::constant_expression_value(void *mem_ctx,
                                      struct hash_table *variable_context)
{
   assert(mem_ctx);

   ir_constant *v = this->val->constant_expression_value(mem_ctx,
                                                         variable_context);

   if (v != NULL) {
      ir_constant_data data = { { 0 } };

      const unsigned swiz_idx[4] = {
         this->mask.x, this->mask.y, this->mask.z, this->mask.w
      };

      for (unsigned i = 0; i < this->mask.num_components; i++) {
         switch (v->type->base_type) {
         case GLSL_TYPE_UINT:
         case GLSL_TYPE_INT:   data.u[i] = v->value.u[swiz_idx[i]]; break;
         case GLSL_TYPE_FLOAT: data.f[i] = v->value.f[swiz_idx[i]]; break;
         case GLSL_TYPE_FLOAT16: data.f16[i] = v->value.f16[swiz_idx[i]]; break;
         case GLSL_TYPE_BOOL:  data.b[i] = v->value.b[swiz_idx[i]]; break;
         case GLSL_TYPE_DOUBLE:data.d[i] = v->value.d[swiz_idx[i]]; break;
         case GLSL_TYPE_UINT64:data.u64[i] = v->value.u64[swiz_idx[i]]; break;
         case GLSL_TYPE_INT64: data.i64[i] = v->value.i64[swiz_idx[i]]; break;
         default:              assert(!"Should not get here."); break;
         }
      }

      return new(mem_ctx) ir_constant(this->type, &data);
   }
   return NULL;
}


ir_constant *
ir_dereference_variable::constant_expression_value(void *mem_ctx,
                                                   struct hash_table *variable_context)
{
   assert(var);
   assert(mem_ctx);

   /* Give priority to the context hashtable, if it exists */
   if (variable_context) {
      hash_entry *entry = _mesa_hash_table_search(variable_context, var);

      if(entry)
         return (ir_constant *) entry->data;
   }

   /* The constant_value of a uniform variable is its initializer,
    * not the lifetime constant value of the uniform.
    */
   if (var->data.mode == ir_var_uniform)
      return NULL;

   if (!var->constant_value)
      return NULL;

   return var->constant_value->clone(mem_ctx, NULL);
}


ir_constant *
ir_dereference_array::constant_expression_value(void *mem_ctx,
                                                struct hash_table *variable_context)
{
   assert(mem_ctx);

   ir_constant *array = this->array->constant_expression_value(mem_ctx, variable_context);
   ir_constant *idx = this->array_index->constant_expression_value(mem_ctx, variable_context);

   if ((array != NULL) && (idx != NULL)) {
      if (array->type->is_matrix()) {
         /* Array access of a matrix results in a vector.
          */
         const unsigned column = idx->value.u[0];

         const glsl_type *const column_type = array->type->column_type();

         /* Offset in the constant matrix to the first element of the column
          * to be extracted.
          */
         const unsigned mat_idx = column * column_type->vector_elements;

         ir_constant_data data = { { 0 } };

         switch (column_type->base_type) {
         case GLSL_TYPE_UINT:
         case GLSL_TYPE_INT:
            for (unsigned i = 0; i < column_type->vector_elements; i++)
               data.u[i] = array->value.u[mat_idx + i];

            break;

         case GLSL_TYPE_FLOAT:
            for (unsigned i = 0; i < column_type->vector_elements; i++)
               data.f[i] = array->value.f[mat_idx + i];

            break;

         case GLSL_TYPE_DOUBLE:
            for (unsigned i = 0; i < column_type->vector_elements; i++)
               data.d[i] = array->value.d[mat_idx + i];

            break;

         default:
            assert(!"Should not get here.");
            break;
         }

         return new(mem_ctx) ir_constant(column_type, &data);
      } else if (array->type->is_vector()) {
         const unsigned component = idx->value.u[0];

         return new(mem_ctx) ir_constant(array, component);
      } else if (array->type->is_array()) {
         const unsigned index = idx->value.u[0];
         return array->get_array_element(index)->clone(mem_ctx, NULL);
      }
   }
   return NULL;
}


ir_constant *
ir_dereference_record::constant_expression_value(void *mem_ctx,
                                                 struct hash_table *)
{
   assert(mem_ctx);

   ir_constant *v = this->record->constant_expression_value(mem_ctx);

   return (v != NULL) ? v->get_record_field(this->field_idx) : NULL;
}


ir_constant *
ir_assignment::constant_expression_value(void *, struct hash_table *)
{
   /* FINISHME: Handle CEs involving assignment (return RHS) */
   return NULL;
}


ir_constant *
ir_constant::constant_expression_value(void *, struct hash_table *)
{
   return this;
}


ir_constant *
ir_call::constant_expression_value(void *mem_ctx, struct hash_table *variable_context)
{
   assert(mem_ctx);

   return this->callee->constant_expression_value(mem_ctx,
                                                  &this->actual_parameters,
                                                  variable_context);
}


bool ir_function_signature::constant_expression_evaluate_expression_list(void *mem_ctx,
                                                                        const struct exec_list &body,
                                                                         struct hash_table *variable_context,
                                                                         ir_constant **result)
{
   assert(mem_ctx);

   foreach_in_list(ir_instruction, inst, &body) {
      switch(inst->ir_type) {

         /* (declare () type symbol) */
      case ir_type_variable: {
         ir_variable *var = inst->as_variable();
         _mesa_hash_table_insert(variable_context, var, ir_constant::zero(this, var->type));
         break;
      }

         /* (assign [condition] (write-mask) (ref) (value)) */
      case ir_type_assignment: {
         ir_assignment *asg = inst->as_assignment();
         if (asg->condition) {
            ir_constant *cond =
               asg->condition->constant_expression_value(mem_ctx,
                                                         variable_context);
            if (!cond)
               return false;
            if (!cond->get_bool_component(0))
               break;
         }

         ir_constant *store = NULL;
         int offset = 0;

         if (!constant_referenced(asg->lhs, variable_context, store, offset))
            return false;

         ir_constant *value =
            asg->rhs->constant_expression_value(mem_ctx, variable_context);

         if (!value)
            return false;

         store->copy_masked_offset(value, offset, asg->write_mask);
         break;
      }

         /* (return (expression)) */
      case ir_type_return:
         assert (result);
         *result =
            inst->as_return()->value->constant_expression_value(mem_ctx,
                                                                variable_context);
         return *result != NULL;

         /* (call name (ref) (params))*/
      case ir_type_call: {
         ir_call *call = inst->as_call();

         /* Just say no to void functions in constant expressions.  We
          * don't need them at that point.
          */

         if (!call->return_deref)
            return false;

         ir_constant *store = NULL;
         int offset = 0;

         if (!constant_referenced(call->return_deref, variable_context,
                                  store, offset))
            return false;

         ir_constant *value =
            call->constant_expression_value(mem_ctx, variable_context);

         if(!value)
            return false;

         store->copy_offset(value, offset);
         break;
      }

         /* (if condition (then-instructions) (else-instructions)) */
      case ir_type_if: {
         ir_if *iif = inst->as_if();

         ir_constant *cond =
            iif->condition->constant_expression_value(mem_ctx,
                                                      variable_context);
         if (!cond || !cond->type->is_boolean())
            return false;

         exec_list &branch = cond->get_bool_component(0) ? iif->then_instructions : iif->else_instructions;

         *result = NULL;
         if (!constant_expression_evaluate_expression_list(mem_ctx, branch,
                                                           variable_context,
                                                           result))
            return false;

         /* If there was a return in the branch chosen, drop out now. */
         if (*result)
            return true;

         break;
      }

         /* Every other expression type, we drop out. */
      default:
         return false;
      }
   }

   /* Reaching the end of the block is not an error condition */
   if (result)
      *result = NULL;

   return true;
}

ir_constant *
ir_function_signature::constant_expression_value(void *mem_ctx,
                                                 exec_list *actual_parameters,
                                                 struct hash_table *variable_context)
{
   assert(mem_ctx);

   const glsl_type *type = this->return_type;
   if (type == glsl_type::void_type)
      return NULL;

   /* From the GLSL 1.20 spec, page 23:
    * "Function calls to user-defined functions (non-built-in functions)
    *  cannot be used to form constant expressions."
    */
   if (!this->is_builtin())
      return NULL;

   /*
    * Of the builtin functions, only the texture lookups and the noise
    * ones must not be used in constant expressions.  Texture instructions
    * include special ir_texture opcodes which can't be constant-folded (see
    * ir_texture::constant_expression_value).  Noise functions, however, we
    * have to special case here.
    */
   if (strcmp(this->function_name(), "noise1") == 0 ||
       strcmp(this->function_name(), "noise2") == 0 ||
       strcmp(this->function_name(), "noise3") == 0 ||
       strcmp(this->function_name(), "noise4") == 0)
      return NULL;

   /* Initialize the table of dereferencable names with the function
    * parameters.  Verify their const-ness on the way.
    *
    * We expect the correctness of the number of parameters to have
    * been checked earlier.
    */
   hash_table *deref_hash = _mesa_pointer_hash_table_create(NULL);

   /* If "origin" is non-NULL, then the function body is there.  So we
    * have to use the variable objects from the object with the body,
    * but the parameter instanciation on the current object.
    */
   const exec_node *parameter_info = origin ? origin->parameters.get_head_raw() : parameters.get_head_raw();

   foreach_in_list(ir_rvalue, n, actual_parameters) {
      ir_constant *constant =
         n->constant_expression_value(mem_ctx, variable_context);
      if (constant == NULL) {
         _mesa_hash_table_destroy(deref_hash, NULL);
         return NULL;
      }


      ir_variable *var = (ir_variable *)parameter_info;
      _mesa_hash_table_insert(deref_hash, var, constant);

      parameter_info = parameter_info->next;
   }

   ir_constant *result = NULL;

   /* Now run the builtin function until something non-constant
    * happens or we get the result.
    */
   if (constant_expression_evaluate_expression_list(mem_ctx, origin ? origin->body : body, deref_hash, &result) &&
       result)
      result = result->clone(mem_ctx, NULL);

   _mesa_hash_table_destroy(deref_hash, NULL);

   return result;
}
