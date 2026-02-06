/**************************************************************************
 *
 * Copyright 2010 Luca Barbieri
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice (including the
 * next paragraph) shall be included in all copies or substantial
 * portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE COPYRIGHT OWNER(S) AND/OR ITS SUPPLIERS BE
 * LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
 * OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
 * WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 **************************************************************************/


#ifndef U_HALF_H
#define U_HALF_H

#include "pipe/p_compiler.h"
#include "util/u_math.h"
#include "util/half_float.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * References for float <-> half conversions
 *
 *  http://fgiesen.wordpress.com/2012/03/28/half-to-float-done-quic/
 *  https://gist.github.com/2156668
 *  https://gist.github.com/2144712
 */

static inline uint16_t
util_float_to_half(float f)
{
   return _mesa_float_to_half(f);
}

static inline uint16_t
util_float_to_half_rtz(float f)
{
   uint32_t sign_mask  = 0x80000000;
   uint32_t round_mask = ~0xfff;
   uint32_t f32inf = 0xff << 23;
   uint32_t f16inf = 0x1f << 23;
   uint32_t sign;
   union fi magic;
   union fi f32;
   uint16_t f16;

   magic.ui = 0xf << 23;

   f32.f = f;

   /* Sign */
   sign = f32.ui & sign_mask;
   f32.ui ^= sign;

   if (f32.ui == f32inf) {
      /* Inf */
      f16 = 0x7c00;
   } else if (f32.ui > f32inf) {
      /* NaN */
      f16 = 0x7e00;
   } else {
      /* Number */
      f32.ui &= round_mask;
      f32.f  *= magic.f;
      f32.ui -= round_mask;
      /*
       * XXX: The magic mul relies on denorms being available, otherwise
       * all f16 denorms get flushed to zero - hence when this is used
       * for tgsi_exec in softpipe we won't get f16 denorms.
       */
      /*
       * Clamp to max finite value if overflowed.
       * OpenGL has completely undefined rounding behavior for float to
       * half-float conversions, and this matches what is mandated for float
       * to fp11/fp10, which recommend round-to-nearest-finite too.
       * (d3d10 is deeply unhappy about flushing such values to infinity, and
       * while it also mandates round-to-zero it doesn't care nearly as much
       * about that.)
       */
      if (f32.ui > f16inf)
         f32.ui = f16inf - 1;

      f16 = f32.ui >> 13;
   }

   /* Sign */
   f16 |= sign >> 16;

   return f16;
}

static inline float
util_half_to_float(uint16_t f16)
{
   union fi infnan;
   union fi magic;
   union fi f32;

   infnan.ui = 0x8f << 23;
   infnan.f = 65536.0f;
   magic.ui  = 0xef << 23;

   /* Exponent / Mantissa */
   f32.ui = (f16 & 0x7fff) << 13;

   /* Adjust */
   f32.f *= magic.f;
   /* XXX: The magic mul relies on denorms being available */

   /* Inf / NaN */
   if (f32.f >= infnan.f)
      f32.ui |= 0xff << 23;

   /* Sign */
   f32.ui |= (uint32_t)(f16 & 0x8000) << 16;

   return f32.f;
}

#ifdef __cplusplus
}
#endif

#endif /* U_HALF_H */

