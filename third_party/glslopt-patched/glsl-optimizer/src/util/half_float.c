/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2007  Brian Paul   All Rights Reserved.
 * Copyright 2015 Philip Taylor <philip@zaynar.co.uk>
 * Copyright 2018 Advanced Micro Devices, Inc.
 * Copyright (C) 2018-2019 Intel Corporation
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
 * OTHER DEALINGS IN THE SOFTWARE.
 */

#include <math.h>
#include <assert.h>
#include "half_float.h"
#include "util/u_half.h"
#include "rounding.h"
#include "softfloat.h"
#include "macros.h"

typedef union { float f; int32_t i; uint32_t u; } fi_type;

/**
 * Convert a 4-byte float to a 2-byte half float.
 *
 * Not all float32 values can be represented exactly as a float16 value. We
 * round such intermediate float32 values to the nearest float16. When the
 * float32 lies exactly between to float16 values, we round to the one with
 * an even mantissa.
 *
 * This rounding behavior has several benefits:
 *   - It has no sign bias.
 *
 *   - It reproduces the behavior of real hardware: opcode F32TO16 in Intel's
 *     GPU ISA.
 *
 *   - By reproducing the behavior of the GPU (at least on Intel hardware),
 *     compile-time evaluation of constant packHalf2x16 GLSL expressions will
 *     result in the same value as if the expression were executed on the GPU.
 */
uint16_t
_mesa_float_to_half(float val)
{
   const fi_type fi = {val};
   const int flt_m = fi.i & 0x7fffff;
   const int flt_e = (fi.i >> 23) & 0xff;
   const int flt_s = (fi.i >> 31) & 0x1;
   int s, e, m = 0;
   uint16_t result;

   /* sign bit */
   s = flt_s;

   /* handle special cases */
   if ((flt_e == 0) && (flt_m == 0)) {
      /* zero */
      /* m = 0; - already set */
      e = 0;
   }
   else if ((flt_e == 0) && (flt_m != 0)) {
      /* denorm -- denorm float maps to 0 half */
      /* m = 0; - already set */
      e = 0;
   }
   else if ((flt_e == 0xff) && (flt_m == 0)) {
      /* infinity */
      /* m = 0; - already set */
      e = 31;
   }
   else if ((flt_e == 0xff) && (flt_m != 0)) {
      /* NaN */
      m = 1;
      e = 31;
   }
   else {
      /* regular number */
      const int new_exp = flt_e - 127;
      if (new_exp < -14) {
         /* The float32 lies in the range (0.0, min_normal16) and is rounded
          * to a nearby float16 value. The result will be either zero, subnormal,
          * or normal.
          */
         e = 0;
         m = _mesa_lroundevenf((1 << 24) * fabsf(fi.f));
      }
      else if (new_exp > 15) {
         /* map this value to infinity */
         /* m = 0; - already set */
         e = 31;
      }
      else {
         /* The float32 lies in the range
          *   [min_normal16, max_normal16 + max_step16)
          * and is rounded to a nearby float16 value. The result will be
          * either normal or infinite.
          */
         e = new_exp + 15;
         m = _mesa_lroundevenf(flt_m / (float) (1 << 13));
      }
   }

   assert(0 <= m && m <= 1024);
   if (m == 1024) {
      /* The float32 was rounded upwards into the range of the next exponent,
       * so bump the exponent. This correctly handles the case where f32
       * should be rounded up to float16 infinity.
       */
      ++e;
      m = 0;
   }

   result = (s << 15) | (e << 10) | m;
   return result;
}

uint16_t
_mesa_float_to_float16_rtz(float val)
{
    return _mesa_float_to_half_rtz(val);
}

/**
 * Convert a 2-byte half float to a 4-byte float.
 * Based on code from:
 * http://www.opengl.org/discussion_boards/ubb/Forum3/HTML/008786.html
 */
float
_mesa_half_to_float(uint16_t val)
{
   return util_half_to_float(val);
}

/**
  * Convert 0.0 to 0x00, 1.0 to 0xff.
  * Values outside the range [0.0, 1.0] will give undefined results.
  */
uint8_t _mesa_half_to_unorm8(uint16_t val)
{
   const int m = val & 0x3ff;
   const int e = (val >> 10) & 0x1f;
   ASSERTED const int s = (val >> 15) & 0x1;

   /* v = round_to_nearest(1.mmmmmmmmmm * 2^(e-15) * 255)
    *   = round_to_nearest((1.mmmmmmmmmm * 255) * 2^(e-15))
    *   = round_to_nearest((1mmmmmmmmmm * 255) * 2^(e-25))
    *   = round_to_zero((1mmmmmmmmmm * 255) * 2^(e-25) + 0.5)
    *   = round_to_zero(((1mmmmmmmmmm * 255) * 2^(e-24) + 1) / 2)
    *
    * This happens to give the correct answer for zero/subnormals too
    */
   assert(s == 0 && val <= FP16_ONE); /* check 0 <= this <= 1 */
   /* (implies e <= 15, which means the bit-shifts below are safe) */

   uint32_t v = ((1 << 10) | m) * 255;
   v = ((v >> (24 - e)) + 1) >> 1;
   return v;
}

/**
  * Takes a uint16_t, divides by 65536, converts the infinite-precision
  * result to fp16 with round-to-zero. Used by the ASTC decoder.
  */
uint16_t _mesa_uint16_div_64k_to_half(uint16_t v)
{
   /* Zero or subnormal. Set the mantissa to (v << 8) and return. */
   if (v < 4)
      return v << 8;

   /* Count the leading 0s in the uint16_t */
#ifdef HAVE___BUILTIN_CLZ
   int n = __builtin_clz(v) - 16;
#else
   int n = 16;
   for (int i = 15; i >= 0; i--) {
      if (v & (1 << i)) {
         n = 15 - i;
         break;
      }
   }
#endif

   /* Shift the mantissa up so bit 16 is the hidden 1 bit,
    * mask it off, then shift back down to 10 bits
    */
   int m = ( ((uint32_t)v << (n + 1)) & 0xffff ) >> 6;

   /*  (0{n} 1 X{15-n}) * 2^-16
    * = 1.X * 2^(15-n-16)
    * = 1.X * 2^(14-n - 15)
    * which is the FP16 form with e = 14 - n
    */
   int e = 14 - n;

   assert(e >= 1 && e <= 30);
   assert(m >= 0 && m < 0x400);

   return (e << 10) | m;
}
