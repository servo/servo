/*
 * The implementations contained in this file are heavily based on the
 * implementations found in the Berkeley SoftFloat library. As such, they are
 * licensed under the same 3-clause BSD license:
 *
 * License for Berkeley SoftFloat Release 3e
 *
 * John R. Hauser
 * 2018 January 20
 *
 * The following applies to the whole of SoftFloat Release 3e as well as to
 * each source file individually.
 *
 * Copyright 2011, 2012, 2013, 2014, 2015, 2016, 2017, 2018 The Regents of the
 * University of California.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *  1. Redistributions of source code must retain the above copyright notice,
 *     this list of conditions, and the following disclaimer.
 *
 *  2. Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions, and the following disclaimer in the
 *     documentation and/or other materials provided with the distribution.
 *
 *  3. Neither the name of the University nor the names of its contributors
 *     may be used to endorse or promote products derived from this software
 *     without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS "AS IS", AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE, ARE
 * DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

#version 430
#extension GL_ARB_gpu_shader_int64 : enable
#extension GL_ARB_shader_bit_encoding : enable
#extension GL_EXT_shader_integer_mix : enable
#extension GL_MESA_shader_integer_functions : enable

#pragma warning(off)

/* Software IEEE floating-point rounding mode.
 * GLSL spec section "4.7.1 Range and Precision":
 * The rounding mode cannot be set and is undefined.
 * But here, we are able to define the rounding mode at the compilation time.
 */
#define FLOAT_ROUND_NEAREST_EVEN    0
#define FLOAT_ROUND_TO_ZERO         1
#define FLOAT_ROUND_DOWN            2
#define FLOAT_ROUND_UP              3
#define FLOAT_ROUNDING_MODE         FLOAT_ROUND_NEAREST_EVEN

/* Relax propagation of NaN.  Binary operations with a NaN source will still
 * produce a NaN result, but it won't follow strict IEEE rules.
 */
#define RELAXED_NAN_PROPAGATION

/* Absolute value of a Float64 :
 * Clear the sign bit
 */
uint64_t
__fabs64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   a.y &= 0x7FFFFFFFu;
   return packUint2x32(a);
}

/* Returns 1 if the double-precision floating-point value `a' is a NaN;
 * otherwise returns 0.
 */
bool
__is_nan(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   return (0xFFE00000u <= (a.y<<1)) &&
      ((a.x != 0u) || ((a.y & 0x000FFFFFu) != 0u));
}

/* Negate value of a Float64 :
 * Toggle the sign bit
 */
uint64_t
__fneg64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   a.y ^= (1u << 31);
   return packUint2x32(a);
}

uint64_t
__fsign64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   uvec2 retval;
   retval.x = 0u;
   retval.y = mix((a.y & 0x80000000u) | 0x3FF00000u, 0u, (a.y << 1 | a.x) == 0u);
   return packUint2x32(retval);
}

/* Returns the fraction bits of the double-precision floating-point value `a'.*/
uint
__extractFloat64FracLo(uint64_t a)
{
   return unpackUint2x32(a).x;
}

uint
__extractFloat64FracHi(uint64_t a)
{
   return unpackUint2x32(a).y & 0x000FFFFFu;
}

/* Returns the exponent bits of the double-precision floating-point value `a'.*/
int
__extractFloat64Exp(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   return int((a.y>>20) & 0x7FFu);
}

bool
__feq64_nonnan(uint64_t __a, uint64_t __b)
{
   uvec2 a = unpackUint2x32(__a);
   uvec2 b = unpackUint2x32(__b);
   return (a.x == b.x) &&
          ((a.y == b.y) || ((a.x == 0u) && (((a.y | b.y)<<1) == 0u)));
}

/* Returns true if the double-precision floating-point value `a' is equal to the
 * corresponding value `b', and false otherwise.  The comparison is performed
 * according to the IEEE Standard for Floating-Point Arithmetic.
 */
bool
__feq64(uint64_t a, uint64_t b)
{
   if (__is_nan(a) || __is_nan(b))
      return false;

   return __feq64_nonnan(a, b);
}

/* Returns true if the double-precision floating-point value `a' is not equal
 * to the corresponding value `b', and false otherwise.  The comparison is
 * performed according to the IEEE Standard for Floating-Point Arithmetic.
 */
bool
__fne64(uint64_t a, uint64_t b)
{
   if (__is_nan(a) || __is_nan(b))
      return true;

   return !__feq64_nonnan(a, b);
}

/* Returns the sign bit of the double-precision floating-point value `a'.*/
uint
__extractFloat64Sign(uint64_t a)
{
   return unpackUint2x32(a).y & 0x80000000u;
}

/* Returns true if the signed 64-bit value formed by concatenating `a0' and
 * `a1' is less than the signed 64-bit value formed by concatenating `b0' and
 * `b1'.  Otherwise, returns false.
 */
bool
ilt64(uint a0, uint a1, uint b0, uint b1)
{
   return (int(a0) < int(b0)) || ((a0 == b0) && (a1 < b1));
}

bool
__flt64_nonnan(uint64_t __a, uint64_t __b)
{
   uvec2 a = unpackUint2x32(__a);
   uvec2 b = unpackUint2x32(__b);

   /* IEEE 754 floating point numbers are specifically designed so that, with
    * two exceptions, values can be compared by bit-casting to signed integers
    * with the same number of bits.
    *
    * From https://en.wikipedia.org/wiki/IEEE_754-1985#Comparing_floating-point_numbers:
    *
    *    When comparing as 2's-complement integers: If the sign bits differ,
    *    the negative number precedes the positive number, so 2's complement
    *    gives the correct result (except that negative zero and positive zero
    *    should be considered equal). If both values are positive, the 2's
    *    complement comparison again gives the correct result. Otherwise (two
    *    negative numbers), the correct FP ordering is the opposite of the 2's
    *    complement ordering.
    *
    * The logic implied by the above quotation is:
    *
    *    !both_are_zero(a, b) && (both_negative(a, b) ? a > b : a < b)
    *
    * This is equivalent to
    *
    *    fne(a, b) && (both_negative(a, b) ? a >= b : a < b)
    *
    *    fne(a, b) && (both_negative(a, b) ? !(a < b) : a < b)
    *
    *    fne(a, b) && ((both_negative(a, b) && !(a < b)) ||
    *                  (!both_negative(a, b) && (a < b)))
    *
    * (A!|B)&(A|!B) is (A xor B) which is implemented here using !=.
    *
    *    fne(a, b) && (both_negative(a, b) != (a < b))
    */
   bool lt = ilt64(a.y, a.x, b.y, b.x);
   bool both_negative = (a.y & b.y & 0x80000000u) != 0;

   return !__feq64_nonnan(__a, __b) && (lt != both_negative);
}

/* Returns true if the double-precision floating-point value `a' is less than
 * the corresponding value `b', and false otherwise.  The comparison is performed
 * according to the IEEE Standard for Floating-Point Arithmetic.
 */
bool
__flt64(uint64_t a, uint64_t b)
{
   /* This weird layout matters.  Doing the "obvious" thing results in extra
    * flow control being inserted to implement the short-circuit evaluation
    * rules.  Flow control is bad!
    */
   bool x = !__is_nan(a);
   bool y = !__is_nan(b);
   bool z = __flt64_nonnan(a, b);

   return (x && y && z);
}

/* Returns true if the double-precision floating-point value `a' is greater
 * than or equal to * the corresponding value `b', and false otherwise.  The
 * comparison is performed * according to the IEEE Standard for Floating-Point
 * Arithmetic.
 */
bool
__fge64(uint64_t a, uint64_t b)
{
   /* This weird layout matters.  Doing the "obvious" thing results in extra
    * flow control being inserted to implement the short-circuit evaluation
    * rules.  Flow control is bad!
    */
   bool x = !__is_nan(a);
   bool y = !__is_nan(b);
   bool z = !__flt64_nonnan(a, b);

   return (x && y && z);
}

uint64_t
__fsat64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);

   /* fsat(NaN) should be zero. */
   if (__is_nan(__a) || int(a.y) < 0)
      return 0ul;

   /* IEEE 754 floating point numbers are specifically designed so that, with
    * two exceptions, values can be compared by bit-casting to signed integers
    * with the same number of bits.
    *
    * From https://en.wikipedia.org/wiki/IEEE_754-1985#Comparing_floating-point_numbers:
    *
    *    When comparing as 2's-complement integers: If the sign bits differ,
    *    the negative number precedes the positive number, so 2's complement
    *    gives the correct result (except that negative zero and positive zero
    *    should be considered equal). If both values are positive, the 2's
    *    complement comparison again gives the correct result. Otherwise (two
    *    negative numbers), the correct FP ordering is the opposite of the 2's
    *    complement ordering.
    *
    * We know that both values are not negative, and we know that at least one
    * value is not zero.  Therefore, we can just use the 2's complement
    * comparison ordering.
    */
   if (ilt64(0x3FF00000, 0x00000000, a.y, a.x))
      return 0x3FF0000000000000ul;

   return __a;
}

/* Adds the 64-bit value formed by concatenating `a0' and `a1' to the 64-bit
 * value formed by concatenating `b0' and `b1'.  Addition is modulo 2^64, so
 * any carry out is lost.  The result is broken into two 32-bit pieces which
 * are stored at the locations pointed to by `z0Ptr' and `z1Ptr'.
 */
void
__add64(uint a0, uint a1, uint b0, uint b1,
        out uint z0Ptr,
        out uint z1Ptr)
{
   uint z1 = a1 + b1;
   z1Ptr = z1;
   z0Ptr = a0 + b0 + uint(z1 < a1);
}


/* Subtracts the 64-bit value formed by concatenating `b0' and `b1' from the
 * 64-bit value formed by concatenating `a0' and `a1'.  Subtraction is modulo
 * 2^64, so any borrow out (carry out) is lost.  The result is broken into two
 * 32-bit pieces which are stored at the locations pointed to by `z0Ptr' and
 * `z1Ptr'.
 */
void
__sub64(uint a0, uint a1, uint b0, uint b1,
        out uint z0Ptr,
        out uint z1Ptr)
{
   z1Ptr = a1 - b1;
   z0Ptr = a0 - b0 - uint(a1 < b1);
}

/* Shifts the 64-bit value formed by concatenating `a0' and `a1' right by the
 * number of bits given in `count'.  If any nonzero bits are shifted off, they
 * are "jammed" into the least significant bit of the result by setting the
 * least significant bit to 1.  The value of `count' can be arbitrarily large;
 * in particular, if `count' is greater than 64, the result will be either 0
 * or 1, depending on whether the concatenation of `a0' and `a1' is zero or
 * nonzero.  The result is broken into two 32-bit pieces which are stored at
 * the locations pointed to by `z0Ptr' and `z1Ptr'.
 */
void
__shift64RightJamming(uint a0,
                      uint a1,
                      int count,
                      out uint z0Ptr,
                      out uint z1Ptr)
{
   uint z0;
   uint z1;
   int negCount = (-count) & 31;

   z0 = mix(0u, a0, count == 0);
   z0 = mix(z0, (a0 >> count), count < 32);

   z1 = uint((a0 | a1) != 0u); /* count >= 64 */
   uint z1_lt64 = (a0>>(count & 31)) | uint(((a0<<negCount) | a1) != 0u);
   z1 = mix(z1, z1_lt64, count < 64);
   z1 = mix(z1, (a0 | uint(a1 != 0u)), count == 32);
   uint z1_lt32 = (a0<<negCount) | (a1>>count) | uint ((a1<<negCount) != 0u);
   z1 = mix(z1, z1_lt32, count < 32);
   z1 = mix(z1, a1, count == 0);
   z1Ptr = z1;
   z0Ptr = z0;
}

/* Shifts the 96-bit value formed by concatenating `a0', `a1', and `a2' right
 * by 32 _plus_ the number of bits given in `count'.  The shifted result is
 * at most 64 nonzero bits; these are broken into two 32-bit pieces which are
 * stored at the locations pointed to by `z0Ptr' and `z1Ptr'.  The bits shifted
 * off form a third 32-bit result as follows:  The _last_ bit shifted off is
 * the most-significant bit of the extra result, and the other 31 bits of the
 * extra result are all zero if and only if _all_but_the_last_ bits shifted off
 * were all zero.  This extra result is stored in the location pointed to by
 * `z2Ptr'.  The value of `count' can be arbitrarily large.
 *     (This routine makes more sense if `a0', `a1', and `a2' are considered
 * to form a fixed-point value with binary point between `a1' and `a2'.  This
 * fixed-point value is shifted right by the number of bits given in `count',
 * and the integer part of the result is returned at the locations pointed to
 * by `z0Ptr' and `z1Ptr'.  The fractional part of the result may be slightly
 * corrupted as described above, and is returned at the location pointed to by
 * `z2Ptr'.)
 */
void
__shift64ExtraRightJamming(uint a0, uint a1, uint a2,
                           int count,
                           out uint z0Ptr,
                           out uint z1Ptr,
                           out uint z2Ptr)
{
   uint z0 = 0u;
   uint z1;
   uint z2;
   int negCount = (-count) & 31;

   z2 = mix(uint(a0 != 0u), a0, count == 64);
   z2 = mix(z2, a0 << negCount, count < 64);
   z2 = mix(z2, a1 << negCount, count < 32);

   z1 = mix(0u, (a0 >> (count & 31)), count < 64);
   z1 = mix(z1, (a0<<negCount) | (a1>>count), count < 32);

   a2 = mix(a2 | a1, a2, count < 32);
   z0 = mix(z0, a0 >> count, count < 32);
   z2 |= uint(a2 != 0u);

   z0 = mix(z0, 0u, (count == 32));
   z1 = mix(z1, a0, (count == 32));
   z2 = mix(z2, a1, (count == 32));
   z0 = mix(z0, a0, (count == 0));
   z1 = mix(z1, a1, (count == 0));
   z2 = mix(z2, a2, (count == 0));
   z2Ptr = z2;
   z1Ptr = z1;
   z0Ptr = z0;
}

/* Shifts the 64-bit value formed by concatenating `a0' and `a1' left by the
 * number of bits given in `count'.  Any bits shifted off are lost.  The value
 * of `count' must be less than 32.  The result is broken into two 32-bit
 * pieces which are stored at the locations pointed to by `z0Ptr' and `z1Ptr'.
 */
void
__shortShift64Left(uint a0, uint a1,
                   int count,
                   out uint z0Ptr,
                   out uint z1Ptr)
{
   z1Ptr = a1<<count;
   z0Ptr = mix((a0 << count | (a1 >> ((-count) & 31))), a0, count == 0);
}

/* Packs the sign `zSign', the exponent `zExp', and the significand formed by
 * the concatenation of `zFrac0' and `zFrac1' into a double-precision floating-
 * point value, returning the result.  After being shifted into the proper
 * positions, the three fields `zSign', `zExp', and `zFrac0' are simply added
 * together to form the most significant 32 bits of the result.  This means
 * that any integer portion of `zFrac0' will be added into the exponent.  Since
 * a properly normalized significand will have an integer portion equal to 1,
 * the `zExp' input should be 1 less than the desired result exponent whenever
 * `zFrac0' and `zFrac1' concatenated form a complete, normalized significand.
 */
uint64_t
__packFloat64(uint zSign, int zExp, uint zFrac0, uint zFrac1)
{
   uvec2 z;

   z.y = zSign + (uint(zExp) << 20) + zFrac0;
   z.x = zFrac1;
   return packUint2x32(z);
}

/* Takes an abstract floating-point value having sign `zSign', exponent `zExp',
 * and extended significand formed by the concatenation of `zFrac0', `zFrac1',
 * and `zFrac2', and returns the proper double-precision floating-point value
 * corresponding to the abstract input.  Ordinarily, the abstract value is
 * simply rounded and packed into the double-precision format, with the inexact
 * exception raised if the abstract input cannot be represented exactly.
 * However, if the abstract value is too large, the overflow and inexact
 * exceptions are raised and an infinity or maximal finite value is returned.
 * If the abstract value is too small, the input value is rounded to a
 * subnormal number, and the underflow and inexact exceptions are raised if the
 * abstract input cannot be represented exactly as a subnormal double-precision
 * floating-point number.
 *     The input significand must be normalized or smaller.  If the input
 * significand is not normalized, `zExp' must be 0; in that case, the result
 * returned is a subnormal number, and it must not require rounding.  In the
 * usual case that the input significand is normalized, `zExp' must be 1 less
 * than the "true" floating-point exponent.  The handling of underflow and
 * overflow follows the IEEE Standard for Floating-Point Arithmetic.
 */
uint64_t
__roundAndPackFloat64(uint zSign,
                      int zExp,
                      uint zFrac0,
                      uint zFrac1,
                      uint zFrac2)
{
   bool roundNearestEven;
   bool increment;

   roundNearestEven = FLOAT_ROUNDING_MODE == FLOAT_ROUND_NEAREST_EVEN;
   increment = int(zFrac2) < 0;
   if (!roundNearestEven) {
      if (FLOAT_ROUNDING_MODE == FLOAT_ROUND_TO_ZERO) {
         increment = false;
      } else {
         if (zSign != 0u) {
            increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN) &&
               (zFrac2 != 0u);
         } else {
            increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP) &&
               (zFrac2 != 0u);
         }
      }
   }
   if (0x7FD <= zExp) {
      if ((0x7FD < zExp) ||
         ((zExp == 0x7FD) &&
            (0x001FFFFFu == zFrac0 && 0xFFFFFFFFu == zFrac1) &&
               increment)) {
         if ((FLOAT_ROUNDING_MODE == FLOAT_ROUND_TO_ZERO) ||
            ((zSign != 0u) && (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP)) ||
               ((zSign == 0u) && (FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN))) {
            return __packFloat64(zSign, 0x7FE, 0x000FFFFFu, 0xFFFFFFFFu);
         }
         return __packFloat64(zSign, 0x7FF, 0u, 0u);
      }
   }

   if (zExp < 0) {
      __shift64ExtraRightJamming(
         zFrac0, zFrac1, zFrac2, -zExp, zFrac0, zFrac1, zFrac2);
      zExp = 0;
      if (roundNearestEven) {
         increment = zFrac2 < 0u;
      } else {
         if (zSign != 0u) {
            increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN) &&
               (zFrac2 != 0u);
         } else {
            increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP) &&
               (zFrac2 != 0u);
         }
      }
   }

   if (increment) {
      __add64(zFrac0, zFrac1, 0u, 1u, zFrac0, zFrac1);
      zFrac1 &= ~((zFrac2 + uint(zFrac2 == 0u)) & uint(roundNearestEven));
   } else {
      zExp = mix(zExp, 0, (zFrac0 | zFrac1) == 0u);
   }
   return __packFloat64(zSign, zExp, zFrac0, zFrac1);
}

uint64_t
__roundAndPackUInt64(uint zSign, uint zFrac0, uint zFrac1, uint zFrac2)
{
   bool roundNearestEven;
   bool increment;
   uint64_t default_nan = 0xFFFFFFFFFFFFFFFFUL;

   roundNearestEven = FLOAT_ROUNDING_MODE == FLOAT_ROUND_NEAREST_EVEN;

   if (zFrac2 >= 0x80000000u)
      increment = false;

   if (!roundNearestEven) {
      if (zSign != 0u) {
         if ((FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN) && (zFrac2 != 0u)) {
            increment = false;
         }
      } else {
         increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP) &&
            (zFrac2 != 0u);
      }
   }

   if (increment) {
      __add64(zFrac0, zFrac1, 0u, 1u, zFrac0, zFrac1);
      if ((zFrac0 | zFrac1) != 0u)
         zFrac1 &= ~(1u) + uint(zFrac2 == 0u) & uint(roundNearestEven);
   }
   return mix(packUint2x32(uvec2(zFrac1, zFrac0)), default_nan,
              (zSign != 0u && (zFrac0 | zFrac1) != 0u));
}

int64_t
__roundAndPackInt64(uint zSign, uint zFrac0, uint zFrac1, uint zFrac2)
{
   bool roundNearestEven;
   bool increment;
   int64_t default_NegNaN = -0x7FFFFFFFFFFFFFFEL;
   int64_t default_PosNaN = 0xFFFFFFFFFFFFFFFFL;

   roundNearestEven = FLOAT_ROUNDING_MODE == FLOAT_ROUND_NEAREST_EVEN;

   if (zFrac2 >= 0x80000000u)
      increment = false;

   if (!roundNearestEven) {
      if (zSign != 0u) {
         increment = ((FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN) &&
            (zFrac2 != 0u));
      } else {
         increment = (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP) &&
            (zFrac2 != 0u);
      }
   }

   if (increment) {
      __add64(zFrac0, zFrac1, 0u, 1u, zFrac0, zFrac1);
      if ((zFrac0 | zFrac1) != 0u)
         zFrac1 &= ~(1u) + uint(zFrac2 == 0u) & uint(roundNearestEven);
   }

   int64_t absZ = mix(int64_t(packUint2x32(uvec2(zFrac1, zFrac0))),
                      -int64_t(packUint2x32(uvec2(zFrac1, zFrac0))),
                      zSign != 0u);
   int64_t nan = mix(default_PosNaN, default_NegNaN, zSign != 0u);
   return mix(absZ, nan, ((zSign != 0u) != (absZ < 0)) && bool(absZ));
}

/* Returns the number of leading 0 bits before the most-significant 1 bit of
 * `a'.  If `a' is zero, 32 is returned.
 */
int
__countLeadingZeros32(uint a)
{
   return 31 - findMSB(a);
}

/* Takes an abstract floating-point value having sign `zSign', exponent `zExp',
 * and significand formed by the concatenation of `zSig0' and `zSig1', and
 * returns the proper double-precision floating-point value corresponding
 * to the abstract input.  This routine is just like `__roundAndPackFloat64'
 * except that the input significand has fewer bits and does not have to be
 * normalized.  In all cases, `zExp' must be 1 less than the "true" floating-
 * point exponent.
 */
uint64_t
__normalizeRoundAndPackFloat64(uint zSign,
                               int zExp,
                               uint zFrac0,
                               uint zFrac1)
{
   int shiftCount;
   uint zFrac2;

   if (zFrac0 == 0u) {
      zExp -= 32;
      zFrac0 = zFrac1;
      zFrac1 = 0u;
   }

   shiftCount = __countLeadingZeros32(zFrac0) - 11;
   if (0 <= shiftCount) {
      zFrac2 = 0u;
      __shortShift64Left(zFrac0, zFrac1, shiftCount, zFrac0, zFrac1);
   } else {
      __shift64ExtraRightJamming(
         zFrac0, zFrac1, 0u, -shiftCount, zFrac0, zFrac1, zFrac2);
   }
   zExp -= shiftCount;
   return __roundAndPackFloat64(zSign, zExp, zFrac0, zFrac1, zFrac2);
}

/* Takes two double-precision floating-point values `a' and `b', one of which
 * is a NaN, and returns the appropriate NaN result.
 */
uint64_t
__propagateFloat64NaN(uint64_t __a, uint64_t __b)
{
#if defined RELAXED_NAN_PROPAGATION
   uvec2 a = unpackUint2x32(__a);
   uvec2 b = unpackUint2x32(__b);

   return packUint2x32(uvec2(a.x | b.x, a.y | b.y));
#else
   bool aIsNaN = __is_nan(__a);
   bool bIsNaN = __is_nan(__b);
   uvec2 a = unpackUint2x32(__a);
   uvec2 b = unpackUint2x32(__b);
   a.y |= 0x00080000u;
   b.y |= 0x00080000u;

   return packUint2x32(mix(b, mix(a, b, bvec2(bIsNaN, bIsNaN)), bvec2(aIsNaN, aIsNaN)));
#endif
}

/* If a shader is in the soft-fp64 path, it almost certainly has register
 * pressure problems.  Choose a method to exchange two values that does not
 * require a temporary.
 */
#define EXCHANGE(a, b) \
   do {                \
       a ^= b;         \
       b ^= a;         \
       a ^= b;         \
   } while (false)

/* Returns the result of adding the double-precision floating-point values
 * `a' and `b'.  The operation is performed according to the IEEE Standard for
 * Floating-Point Arithmetic.
 */
uint64_t
__fadd64(uint64_t a, uint64_t b)
{
   uint aSign = __extractFloat64Sign(a);
   uint bSign = __extractFloat64Sign(b);
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   uint bFracLo = __extractFloat64FracLo(b);
   uint bFracHi = __extractFloat64FracHi(b);
   int aExp = __extractFloat64Exp(a);
   int bExp = __extractFloat64Exp(b);
   int expDiff = aExp - bExp;
   if (aSign == bSign) {
      uint zFrac0;
      uint zFrac1;
      uint zFrac2;
      int zExp;

      if (expDiff == 0) {
         if (aExp == 0x7FF) {
            bool propagate = ((aFracHi | bFracHi) | (aFracLo| bFracLo)) != 0u;
            return mix(a, __propagateFloat64NaN(a, b), propagate);
         }
         __add64(aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1);
         if (aExp == 0)
            return __packFloat64(aSign, 0, zFrac0, zFrac1);
         zFrac2 = 0u;
         zFrac0 |= 0x00200000u;
         zExp = aExp;
         __shift64ExtraRightJamming(
            zFrac0, zFrac1, zFrac2, 1, zFrac0, zFrac1, zFrac2);
      } else {
         if (expDiff < 0) {
            EXCHANGE(aFracHi, bFracHi);
            EXCHANGE(aFracLo, bFracLo);
            EXCHANGE(aExp, bExp);
         }

         if (aExp == 0x7FF) {
            bool propagate = (aFracHi | aFracLo) != 0u;
            return mix(__packFloat64(aSign, 0x7ff, 0u, 0u), __propagateFloat64NaN(a, b), propagate);
         }

         expDiff = mix(abs(expDiff), abs(expDiff) - 1, bExp == 0);
         bFracHi = mix(bFracHi | 0x00100000u, bFracHi, bExp == 0);
         __shift64ExtraRightJamming(
            bFracHi, bFracLo, 0u, expDiff, bFracHi, bFracLo, zFrac2);
         zExp = aExp;

         aFracHi |= 0x00100000u;
         __add64(aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1);
         --zExp;
         if (!(zFrac0 < 0x00200000u)) {
            __shift64ExtraRightJamming(zFrac0, zFrac1, zFrac2, 1, zFrac0, zFrac1, zFrac2);
            ++zExp;
         }
      }
      return __roundAndPackFloat64(aSign, zExp, zFrac0, zFrac1, zFrac2);

   } else {
      int zExp;

      __shortShift64Left(aFracHi, aFracLo, 10, aFracHi, aFracLo);
      __shortShift64Left(bFracHi, bFracLo, 10, bFracHi, bFracLo);
      if (expDiff != 0) {
         uint zFrac0;
         uint zFrac1;

         if (expDiff < 0) {
            EXCHANGE(aFracHi, bFracHi);
            EXCHANGE(aFracLo, bFracLo);
            EXCHANGE(aExp, bExp);
            aSign ^= 0x80000000u;
         }

         if (aExp == 0x7FF) {
            bool propagate = (aFracHi | aFracLo) != 0u;
            return mix(__packFloat64(aSign, 0x7ff, 0u, 0u), __propagateFloat64NaN(a, b), propagate);
         }

         expDiff = mix(abs(expDiff), abs(expDiff) - 1, bExp == 0);
         bFracHi = mix(bFracHi | 0x40000000u, bFracHi, bExp == 0);
         __shift64RightJamming(bFracHi, bFracLo, expDiff, bFracHi, bFracLo);
         aFracHi |= 0x40000000u;
         __sub64(aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1);
         zExp = aExp;
         --zExp;
         return __normalizeRoundAndPackFloat64(aSign, zExp - 10, zFrac0, zFrac1);
      }
      if (aExp == 0x7FF) {
         bool propagate = ((aFracHi | bFracHi) | (aFracLo | bFracLo)) != 0u;
         return mix(0xFFFFFFFFFFFFFFFFUL, __propagateFloat64NaN(a, b), propagate);
      }
      bExp = mix(bExp, 1, aExp == 0);
      aExp = mix(aExp, 1, aExp == 0);

      uint zFrac0;
      uint zFrac1;
      uint sign_of_difference = 0;
      if (bFracHi < aFracHi) {
         __sub64(aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1);
      }
      else if (aFracHi < bFracHi) {
         __sub64(bFracHi, bFracLo, aFracHi, aFracLo, zFrac0, zFrac1);
         sign_of_difference = 0x80000000;
      }
      else if (bFracLo <= aFracLo) {
         /* It is possible that zFrac0 and zFrac1 may be zero after this. */
         __sub64(aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1);
      }
      else {
         __sub64(bFracHi, bFracLo, aFracHi, aFracLo, zFrac0, zFrac1);
         sign_of_difference = 0x80000000;
      }
      zExp = mix(bExp, aExp, sign_of_difference == 0u);
      aSign ^= sign_of_difference;
      uint64_t retval_0 = __packFloat64(uint(FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN) << 31, 0, 0u, 0u);
      uint64_t retval_1 = __normalizeRoundAndPackFloat64(aSign, zExp - 11, zFrac0, zFrac1);
      return mix(retval_0, retval_1, zFrac0 != 0u || zFrac1 != 0u);
   }
}

/* Multiplies the 64-bit value formed by concatenating `a0' and `a1' to the
 * 64-bit value formed by concatenating `b0' and `b1' to obtain a 128-bit
 * product.  The product is broken into four 32-bit pieces which are stored at
 * the locations pointed to by `z0Ptr', `z1Ptr', `z2Ptr', and `z3Ptr'.
 */
void
__mul64To128(uint a0, uint a1, uint b0, uint b1,
             out uint z0Ptr,
             out uint z1Ptr,
             out uint z2Ptr,
             out uint z3Ptr)
{
   uint z0 = 0u;
   uint z1 = 0u;
   uint z2 = 0u;
   uint z3 = 0u;
   uint more1 = 0u;
   uint more2 = 0u;

   umulExtended(a1, b1, z2, z3);
   umulExtended(a1, b0, z1, more2);
   __add64(z1, more2, 0u, z2, z1, z2);
   umulExtended(a0, b0, z0, more1);
   __add64(z0, more1, 0u, z1, z0, z1);
   umulExtended(a0, b1, more1, more2);
   __add64(more1, more2, 0u, z2, more1, z2);
   __add64(z0, z1, 0u, more1, z0, z1);
   z3Ptr = z3;
   z2Ptr = z2;
   z1Ptr = z1;
   z0Ptr = z0;
}

/* Normalizes the subnormal double-precision floating-point value represented
 * by the denormalized significand formed by the concatenation of `aFrac0' and
 * `aFrac1'.  The normalized exponent is stored at the location pointed to by
 * `zExpPtr'.  The most significant 21 bits of the normalized significand are
 * stored at the location pointed to by `zFrac0Ptr', and the least significant
 * 32 bits of the normalized significand are stored at the location pointed to
 * by `zFrac1Ptr'.
 */
void
__normalizeFloat64Subnormal(uint aFrac0, uint aFrac1,
                            out int zExpPtr,
                            out uint zFrac0Ptr,
                            out uint zFrac1Ptr)
{
   int shiftCount;
   uint temp_zfrac0, temp_zfrac1;
   shiftCount = __countLeadingZeros32(mix(aFrac0, aFrac1, aFrac0 == 0u)) - 11;
   zExpPtr = mix(1 - shiftCount, -shiftCount - 31, aFrac0 == 0u);

   temp_zfrac0 = mix(aFrac1<<shiftCount, aFrac1>>(-shiftCount), shiftCount < 0);
   temp_zfrac1 = mix(0u, aFrac1<<(shiftCount & 31), shiftCount < 0);

   __shortShift64Left(aFrac0, aFrac1, shiftCount, zFrac0Ptr, zFrac1Ptr);

   zFrac0Ptr = mix(zFrac0Ptr, temp_zfrac0, aFrac0 == 0);
   zFrac1Ptr = mix(zFrac1Ptr, temp_zfrac1, aFrac0 == 0);
}

/* Returns the result of multiplying the double-precision floating-point values
 * `a' and `b'.  The operation is performed according to the IEEE Standard for
 * Floating-Point Arithmetic.
 */
uint64_t
__fmul64(uint64_t a, uint64_t b)
{
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;
   uint zFrac2 = 0u;
   uint zFrac3 = 0u;
   int zExp;

   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   uint bFracLo = __extractFloat64FracLo(b);
   uint bFracHi = __extractFloat64FracHi(b);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);
   int bExp = __extractFloat64Exp(b);
   uint bSign = __extractFloat64Sign(b);
   uint zSign = aSign ^ bSign;
   if (aExp == 0x7FF) {
      if (((aFracHi | aFracLo) != 0u) ||
         ((bExp == 0x7FF) && ((bFracHi | bFracLo) != 0u))) {
         return __propagateFloat64NaN(a, b);
      }
      if ((uint(bExp) | bFracHi | bFracLo) == 0u)
            return 0xFFFFFFFFFFFFFFFFUL;
      return __packFloat64(zSign, 0x7FF, 0u, 0u);
   }
   if (bExp == 0x7FF) {
      /* a cannot be NaN, but is b NaN? */
      if ((bFracHi | bFracLo) != 0u)
#if defined RELAXED_NAN_PROPAGATION
         return b;
#else
         return __propagateFloat64NaN(a, b);
#endif
      if ((uint(aExp) | aFracHi | aFracLo) == 0u)
         return 0xFFFFFFFFFFFFFFFFUL;
      return __packFloat64(zSign, 0x7FF, 0u, 0u);
   }
   if (aExp == 0) {
      if ((aFracHi | aFracLo) == 0u)
         return __packFloat64(zSign, 0, 0u, 0u);
      __normalizeFloat64Subnormal(aFracHi, aFracLo, aExp, aFracHi, aFracLo);
   }
   if (bExp == 0) {
      if ((bFracHi | bFracLo) == 0u)
         return __packFloat64(zSign, 0, 0u, 0u);
      __normalizeFloat64Subnormal(bFracHi, bFracLo, bExp, bFracHi, bFracLo);
   }
   zExp = aExp + bExp - 0x400;
   aFracHi |= 0x00100000u;
   __shortShift64Left(bFracHi, bFracLo, 12, bFracHi, bFracLo);
   __mul64To128(
      aFracHi, aFracLo, bFracHi, bFracLo, zFrac0, zFrac1, zFrac2, zFrac3);
   __add64(zFrac0, zFrac1, aFracHi, aFracLo, zFrac0, zFrac1);
   zFrac2 |= uint(zFrac3 != 0u);
   if (0x00200000u <= zFrac0) {
      __shift64ExtraRightJamming(
         zFrac0, zFrac1, zFrac2, 1, zFrac0, zFrac1, zFrac2);
      ++zExp;
   }
   return __roundAndPackFloat64(zSign, zExp, zFrac0, zFrac1, zFrac2);
}

uint64_t
__ffma64(uint64_t a, uint64_t b, uint64_t c)
{
   return __fadd64(__fmul64(a, b), c);
}

/* Shifts the 64-bit value formed by concatenating `a0' and `a1' right by the
 * number of bits given in `count'.  Any bits shifted off are lost.  The value
 * of `count' can be arbitrarily large; in particular, if `count' is greater
 * than 64, the result will be 0.  The result is broken into two 32-bit pieces
 * which are stored at the locations pointed to by `z0Ptr' and `z1Ptr'.
 */
void
__shift64Right(uint a0, uint a1,
               int count,
               out uint z0Ptr,
               out uint z1Ptr)
{
   uint z0;
   uint z1;
   int negCount = (-count) & 31;

   z0 = 0u;
   z0 = mix(z0, (a0 >> count), count < 32);
   z0 = mix(z0, a0, count == 0);

   z1 = mix(0u, (a0 >> (count & 31)), count < 64);
   z1 = mix(z1, (a0<<negCount) | (a1>>count), count < 32);
   z1 = mix(z1, a0, count == 0);

   z1Ptr = z1;
   z0Ptr = z0;
}

/* Returns the result of converting the double-precision floating-point value
 * `a' to the unsigned integer format.  The conversion is performed according
 * to the IEEE Standard for Floating-Point Arithmetic.
 */
uint
__fp64_to_uint(uint64_t a)
{
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);

   if ((aExp == 0x7FF) && ((aFracHi | aFracLo) != 0u))
      return 0xFFFFFFFFu;

   aFracHi |= mix(0u, 0x00100000u, aExp != 0);

   int shiftDist = 0x427 - aExp;
   if (0 < shiftDist)
      __shift64RightJamming(aFracHi, aFracLo, shiftDist, aFracHi, aFracLo);

   if ((aFracHi & 0xFFFFF000u) != 0u)
      return mix(~0u, 0u, aSign != 0u);

   uint z = 0u;
   uint zero = 0u;
   __shift64Right(aFracHi, aFracLo, 12, zero, z);

   uint expt = mix(~0u, 0u, aSign != 0u);

   return mix(z, expt, (aSign != 0u) && (z != 0u));
}

uint64_t
__uint_to_fp64(uint a)
{
   if (a == 0u)
      return 0ul;

   int shiftDist = __countLeadingZeros32(a) + 21;

   uint aHigh = 0u;
   uint aLow = 0u;
   int negCount = (- shiftDist) & 31;

   aHigh = mix(0u, a<< shiftDist - 32, shiftDist < 64);
   aLow = 0u;
   aHigh = mix(aHigh, 0u, shiftDist == 0);
   aLow = mix(aLow, a, shiftDist ==0);
   aHigh = mix(aHigh, a >> negCount, shiftDist < 32);
   aLow = mix(aLow, a << shiftDist, shiftDist < 32);

   return __packFloat64(0u, 0x432 - shiftDist, aHigh, aLow);
}

uint64_t
__uint64_to_fp64(uint64_t a)
{
   if (a == 0u)
      return 0ul;

   uvec2 aFrac = unpackUint2x32(a);
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);

   if ((aFracHi & 0x80000000u) != 0u) {
      __shift64RightJamming(aFracHi, aFracLo, 1, aFracHi, aFracLo);
      return __roundAndPackFloat64(0, 0x433, aFracHi, aFracLo, 0u);
   } else {
      return __normalizeRoundAndPackFloat64(0, 0x432, aFrac.y, aFrac.x);
   }
}

uint64_t
__fp64_to_uint64(uint64_t a)
{
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);
   uint zFrac2 = 0u;
   uint64_t default_nan = 0xFFFFFFFFFFFFFFFFUL;

   aFracHi = mix(aFracHi, aFracHi | 0x00100000u, aExp != 0);
   int shiftCount = 0x433 - aExp;

   if ( shiftCount <= 0 ) {
      if (shiftCount < -11 && aExp == 0x7FF) {
         if ((aFracHi | aFracLo) != 0u)
            return __propagateFloat64NaN(a, a);
         return mix(default_nan, a, aSign == 0u);
      }
      __shortShift64Left(aFracHi, aFracLo, -shiftCount, aFracHi, aFracLo);
   } else {
      __shift64ExtraRightJamming(aFracHi, aFracLo, zFrac2, shiftCount,
                                 aFracHi, aFracLo, zFrac2);
   }
   return __roundAndPackUInt64(aSign, aFracHi, aFracLo, zFrac2);
}

int64_t
__fp64_to_int64(uint64_t a)
{
   uint zFrac2 = 0u;
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);
   int64_t default_NegNaN = -0x7FFFFFFFFFFFFFFEL;
   int64_t default_PosNaN = 0xFFFFFFFFFFFFFFFFL;

   aFracHi = mix(aFracHi, aFracHi | 0x00100000u, aExp != 0);
   int shiftCount = 0x433 - aExp;

   if (shiftCount <= 0) {
      if (shiftCount < -11 && aExp == 0x7FF) {
         if ((aFracHi | aFracLo) != 0u)
            return default_NegNaN;
         return mix(default_NegNaN, default_PosNaN, aSign == 0u);
      }
      __shortShift64Left(aFracHi, aFracLo, -shiftCount, aFracHi, aFracLo);
   } else {
      __shift64ExtraRightJamming(aFracHi, aFracLo, zFrac2, shiftCount,
                                 aFracHi, aFracLo, zFrac2);
   }

   return __roundAndPackInt64(aSign, aFracHi, aFracLo, zFrac2);
}

uint64_t
__fp32_to_uint64(float f)
{
   uint a = floatBitsToUint(f);
   uint aFrac = a & 0x007FFFFFu;
   int aExp = int((a>>23) & 0xFFu);
   uint aSign = a & 0x80000000u;
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;
   uint zFrac2 = 0u;
   uint64_t default_nan = 0xFFFFFFFFFFFFFFFFUL;
   int shiftCount = 0xBE - aExp;

   if (shiftCount <0) {
      if (aExp == 0xFF)
         return default_nan;
   }

   aFrac = mix(aFrac, aFrac | 0x00800000u, aExp != 0);
   __shortShift64Left(aFrac, 0, 40, zFrac0, zFrac1);

   if (shiftCount != 0) {
      __shift64ExtraRightJamming(zFrac0, zFrac1, zFrac2, shiftCount,
                                 zFrac0, zFrac1, zFrac2);
   }

   return __roundAndPackUInt64(aSign, zFrac0, zFrac1, zFrac2);
}

int64_t
__fp32_to_int64(float f)
{
   uint a = floatBitsToUint(f);
   uint aFrac = a & 0x007FFFFFu;
   int aExp = int((a>>23) & 0xFFu);
   uint aSign = a & 0x80000000u;
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;
   uint zFrac2 = 0u;
   int64_t default_NegNaN = -0x7FFFFFFFFFFFFFFEL;
   int64_t default_PosNaN = 0xFFFFFFFFFFFFFFFFL;
   int shiftCount = 0xBE - aExp;

   if (shiftCount <0) {
      if (aExp == 0xFF && aFrac != 0u)
         return default_NegNaN;
      return mix(default_NegNaN, default_PosNaN, aSign == 0u);
   }

   aFrac = mix(aFrac, aFrac | 0x00800000u, aExp != 0);
   __shortShift64Left(aFrac, 0, 40, zFrac0, zFrac1);

   if (shiftCount != 0) {
      __shift64ExtraRightJamming(zFrac0, zFrac1, zFrac2, shiftCount,
                                 zFrac0, zFrac1, zFrac2);
   }

   return __roundAndPackInt64(aSign, zFrac0, zFrac1, zFrac2);
}

uint64_t
__int64_to_fp64(int64_t a)
{
   if (a==0)
      return 0ul;

   uint64_t absA = mix(uint64_t(a), uint64_t(-a), a < 0);
   uint aFracHi = __extractFloat64FracHi(absA);
   uvec2 aFrac = unpackUint2x32(absA);
   uint zSign = uint(unpackInt2x32(a).y) & 0x80000000u;

   if ((aFracHi & 0x80000000u) != 0u) {
      return mix(0ul, __packFloat64(0x80000000u, 0x434, 0u, 0u), a < 0);
   }

   return __normalizeRoundAndPackFloat64(zSign, 0x432, aFrac.y, aFrac.x);
}

/* Returns the result of converting the double-precision floating-point value
 * `a' to the 32-bit two's complement integer format.  The conversion is
 * performed according to the IEEE Standard for Floating-Point Arithmetic---
 * which means in particular that the conversion is rounded according to the
 * current rounding mode.  If `a' is a NaN, the largest positive integer is
 * returned.  Otherwise, if the conversion overflows, the largest integer with
 * the same sign as `a' is returned.
 */
int
__fp64_to_int(uint64_t a)
{
   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);

   uint absZ = 0u;
   uint aFracExtra = 0u;
   int shiftCount = aExp - 0x413;

   if (0 <= shiftCount) {
      if (0x41E < aExp) {
         if ((aExp == 0x7FF) && bool(aFracHi | aFracLo))
            aSign = 0u;
         return mix(0x7FFFFFFF, 0x80000000, aSign != 0u);
      }
      __shortShift64Left(aFracHi | 0x00100000u, aFracLo, shiftCount, absZ, aFracExtra);
   } else {
      if (aExp < 0x3FF)
         return 0;

      aFracHi |= 0x00100000u;
      aFracExtra = ( aFracHi << (shiftCount & 31)) | aFracLo;
      absZ = aFracHi >> (- shiftCount);
   }

   int z = mix(int(absZ), -int(absZ), aSign != 0u);
   int nan = mix(0x7FFFFFFF, 0x80000000, aSign != 0u);
   return mix(z, nan, ((aSign != 0u) != (z < 0)) && bool(z));
}

/* Returns the result of converting the 32-bit two's complement integer `a'
 * to the double-precision floating-point format.  The conversion is performed
 * according to the IEEE Standard for Floating-Point Arithmetic.
 */
uint64_t
__int_to_fp64(int a)
{
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;
   if (a==0)
      return __packFloat64(0u, 0, 0u, 0u);
   uint zSign = uint(a) & 0x80000000u;
   uint absA = mix(uint(a), uint(-a), a < 0);
   int shiftCount = __countLeadingZeros32(absA) - 11;
   if (0 <= shiftCount) {
      zFrac0 = absA << shiftCount;
      zFrac1 = 0u;
   } else {
      __shift64Right(absA, 0u, -shiftCount, zFrac0, zFrac1);
   }
   return __packFloat64(zSign, 0x412 - shiftCount, zFrac0, zFrac1);
}

bool
__fp64_to_bool(uint64_t a)
{
   return !__feq64_nonnan(__fabs64(a), 0ul);
}

uint64_t
__bool_to_fp64(bool a)
{
   return packUint2x32(uvec2(0x00000000u, uint(-int(a) & 0x3ff00000)));
}

/* Packs the sign `zSign', exponent `zExp', and significand `zFrac' into a
 * single-precision floating-point value, returning the result.  After being
 * shifted into the proper positions, the three fields are simply added
 * together to form the result.  This means that any integer portion of `zSig'
 * will be added into the exponent.  Since a properly normalized significand
 * will have an integer portion equal to 1, the `zExp' input should be 1 less
 * than the desired result exponent whenever `zFrac' is a complete, normalized
 * significand.
 */
float
__packFloat32(uint zSign, int zExp, uint zFrac)
{
   return uintBitsToFloat(zSign + (uint(zExp)<<23) + zFrac);
}

/* Takes an abstract floating-point value having sign `zSign', exponent `zExp',
 * and significand `zFrac', and returns the proper single-precision floating-
 * point value corresponding to the abstract input.  Ordinarily, the abstract
 * value is simply rounded and packed into the single-precision format, with
 * the inexact exception raised if the abstract input cannot be represented
 * exactly.  However, if the abstract value is too large, the overflow and
 * inexact exceptions are raised and an infinity or maximal finite value is
 * returned.  If the abstract value is too small, the input value is rounded to
 * a subnormal number, and the underflow and inexact exceptions are raised if
 * the abstract input cannot be represented exactly as a subnormal single-
 * precision floating-point number.
 *     The input significand `zFrac' has its binary point between bits 30
 * and 29, which is 7 bits to the left of the usual location.  This shifted
 * significand must be normalized or smaller.  If `zFrac' is not normalized,
 * `zExp' must be 0; in that case, the result returned is a subnormal number,
 * and it must not require rounding.  In the usual case that `zFrac' is
 * normalized, `zExp' must be 1 less than the "true" floating-point exponent.
 * The handling of underflow and overflow follows the IEEE Standard for
 * Floating-Point Arithmetic.
 */
float
__roundAndPackFloat32(uint zSign, int zExp, uint zFrac)
{
   bool roundNearestEven;
   int roundIncrement;
   int roundBits;

   roundNearestEven = FLOAT_ROUNDING_MODE == FLOAT_ROUND_NEAREST_EVEN;
   roundIncrement = 0x40;
   if (!roundNearestEven) {
      if (FLOAT_ROUNDING_MODE == FLOAT_ROUND_TO_ZERO) {
         roundIncrement = 0;
      } else {
         roundIncrement = 0x7F;
         if (zSign != 0u) {
            if (FLOAT_ROUNDING_MODE == FLOAT_ROUND_UP)
               roundIncrement = 0;
         } else {
            if (FLOAT_ROUNDING_MODE == FLOAT_ROUND_DOWN)
               roundIncrement = 0;
         }
      }
   }
   roundBits = int(zFrac & 0x7Fu);
   if (0xFDu <= uint(zExp)) {
      if ((0xFD < zExp) || ((zExp == 0xFD) && (int(zFrac) + roundIncrement) < 0))
         return __packFloat32(zSign, 0xFF, 0u) - float(roundIncrement == 0);
      int count = -zExp;
      bool zexp_lt0 = zExp < 0;
      uint zFrac_lt0 = mix(uint(zFrac != 0u), (zFrac>>count) | uint((zFrac<<((-count) & 31)) != 0u), (-zExp) < 32);
      zFrac = mix(zFrac, zFrac_lt0, zexp_lt0);
      roundBits = mix(roundBits, int(zFrac) & 0x7f, zexp_lt0);
      zExp = mix(zExp, 0, zexp_lt0);
   }
   zFrac = (zFrac + uint(roundIncrement))>>7;
   zFrac &= ~uint(((roundBits ^ 0x40) == 0) && roundNearestEven);

   return __packFloat32(zSign, mix(zExp, 0, zFrac == 0u), zFrac);
}

/* Returns the result of converting the double-precision floating-point value
 * `a' to the single-precision floating-point format.  The conversion is
 * performed according to the IEEE Standard for Floating-Point Arithmetic.
 */
float
__fp64_to_fp32(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   uint zFrac = 0u;
   uint allZero = 0u;

   uint aFracLo = __extractFloat64FracLo(__a);
   uint aFracHi = __extractFloat64FracHi(__a);
   int aExp = __extractFloat64Exp(__a);
   uint aSign = __extractFloat64Sign(__a);
   if (aExp == 0x7FF) {
      __shortShift64Left(a.y, a.x, 12, a.y, a.x);
      float rval = uintBitsToFloat(aSign | 0x7FC00000u | (a.y>>9));
      rval = mix(__packFloat32(aSign, 0xFF, 0u), rval, (aFracHi | aFracLo) != 0u);
      return rval;
   }
   __shift64RightJamming(aFracHi, aFracLo, 22, allZero, zFrac);
   zFrac = mix(zFrac, zFrac | 0x40000000u, aExp != 0);
   return __roundAndPackFloat32(aSign, aExp - 0x381, zFrac);
}

float
__uint64_to_fp32(uint64_t __a)
{
   uvec2 aFrac = unpackUint2x32(__a);
   int shiftCount = mix(__countLeadingZeros32(aFrac.y) - 33,
                        __countLeadingZeros32(aFrac.x) - 1,
                        aFrac.y == 0u);

   if (0 <= shiftCount)
      __shortShift64Left(aFrac.y, aFrac.x, shiftCount, aFrac.y, aFrac.x);
   else
      __shift64RightJamming(aFrac.y, aFrac.x, -shiftCount, aFrac.y, aFrac.x);

   return __roundAndPackFloat32(0u, 0x9C - shiftCount, aFrac.x);
}

float
__int64_to_fp32(int64_t __a)
{
   uint aSign = uint(unpackInt2x32(__a).y) & 0x80000000u;
   uint64_t absA = mix(uint64_t(__a), uint64_t(-__a), __a < 0);
   uvec2 aFrac = unpackUint2x32(absA);
   int shiftCount = mix(__countLeadingZeros32(aFrac.y) - 33,
                        __countLeadingZeros32(aFrac.x) - 1,
                        aFrac.y == 0u);

   if (0 <= shiftCount)
      __shortShift64Left(aFrac.y, aFrac.x, shiftCount, aFrac.y, aFrac.x);
   else
      __shift64RightJamming(aFrac.y, aFrac.x, -shiftCount, aFrac.y, aFrac.x);

   return __roundAndPackFloat32(aSign, 0x9C - shiftCount, aFrac.x);
}

/* Returns the result of converting the single-precision floating-point value
 * `a' to the double-precision floating-point format.
 */
uint64_t
__fp32_to_fp64(float f)
{
   uint a = floatBitsToUint(f);
   uint aFrac = a & 0x007FFFFFu;
   int aExp = int((a>>23) & 0xFFu);
   uint aSign = a & 0x80000000u;
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;

   if (aExp == 0xFF) {
      if (aFrac != 0u) {
         uint nanLo = 0u;
         uint nanHi = a<<9;
         __shift64Right(nanHi, nanLo, 12, nanHi, nanLo);
         nanHi |= aSign | 0x7FF80000u;
         return packUint2x32(uvec2(nanLo, nanHi));
      }
      return __packFloat64(aSign, 0x7FF, 0u, 0u);
    }

   if (aExp == 0) {
      if (aFrac == 0u)
         return __packFloat64(aSign, 0, 0u, 0u);
      /* Normalize subnormal */
      int shiftCount = __countLeadingZeros32(aFrac) - 8;
      aFrac <<= shiftCount;
      aExp = 1 - shiftCount;
      --aExp;
   }

   __shift64Right(aFrac, 0u, 3, zFrac0, zFrac1);
   return __packFloat64(aSign, aExp + 0x380, zFrac0, zFrac1);
}

/* Adds the 96-bit value formed by concatenating `a0', `a1', and `a2' to the
 * 96-bit value formed by concatenating `b0', `b1', and `b2'.  Addition is
 * modulo 2^96, so any carry out is lost.  The result is broken into three
 * 32-bit pieces which are stored at the locations pointed to by `z0Ptr',
 * `z1Ptr', and `z2Ptr'.
 */
void
__add96(uint a0, uint a1, uint a2,
        uint b0, uint b1, uint b2,
        out uint z0Ptr,
        out uint z1Ptr,
        out uint z2Ptr)
{
   uint z2 = a2 + b2;
   uint carry1 = uint(z2 < a2);
   uint z1 = a1 + b1;
   uint carry0 = uint(z1 < a1);
   uint z0 = a0 + b0;
   z1 += carry1;
   z0 += uint(z1 < carry1);
   z0 += carry0;
   z2Ptr = z2;
   z1Ptr = z1;
   z0Ptr = z0;
}

/* Subtracts the 96-bit value formed by concatenating `b0', `b1', and `b2' from
 * the 96-bit value formed by concatenating `a0', `a1', and `a2'.  Subtraction
 * is modulo 2^96, so any borrow out (carry out) is lost.  The result is broken
 * into three 32-bit pieces which are stored at the locations pointed to by
 * `z0Ptr', `z1Ptr', and `z2Ptr'.
 */
void
__sub96(uint a0, uint a1, uint a2,
        uint b0, uint b1, uint b2,
        out uint z0Ptr,
        out uint z1Ptr,
        out uint z2Ptr)
{
   uint z2 = a2 - b2;
   uint borrow1 = uint(a2 < b2);
   uint z1 = a1 - b1;
   uint borrow0 = uint(a1 < b1);
   uint z0 = a0 - b0;
   z0 -= uint(z1 < borrow1);
   z1 -= borrow1;
   z0 -= borrow0;
   z2Ptr = z2;
   z1Ptr = z1;
   z0Ptr = z0;
}

/* Returns an approximation to the 32-bit integer quotient obtained by dividing
 * `b' into the 64-bit value formed by concatenating `a0' and `a1'.  The
 * divisor `b' must be at least 2^31.  If q is the exact quotient truncated
 * toward zero, the approximation returned lies between q and q + 2 inclusive.
 * If the exact quotient q is larger than 32 bits, the maximum positive 32-bit
 * unsigned integer is returned.
 */
uint
__estimateDiv64To32(uint a0, uint a1, uint b)
{
   uint b0;
   uint b1;
   uint rem0 = 0u;
   uint rem1 = 0u;
   uint term0 = 0u;
   uint term1 = 0u;
   uint z;

   if (b <= a0)
      return 0xFFFFFFFFu;
   b0 = b>>16;
   z = (b0<<16 <= a0) ? 0xFFFF0000u : (a0 / b0)<<16;
   umulExtended(b, z, term0, term1);
   __sub64(a0, a1, term0, term1, rem0, rem1);
   while (int(rem0) < 0) {
      z -= 0x10000u;
      b1 = b<<16;
      __add64(rem0, rem1, b0, b1, rem0, rem1);
   }
   rem0 = (rem0<<16) | (rem1>>16);
   z |= (b0<<16 <= rem0) ? 0xFFFFu : rem0 / b0;
   return z;
}

uint
__sqrtOddAdjustments(int index)
{
   uint res = 0u;
   if (index == 0)
      res = 0x0004u;
   if (index == 1)
      res = 0x0022u;
   if (index == 2)
      res = 0x005Du;
   if (index == 3)
      res = 0x00B1u;
   if (index == 4)
      res = 0x011Du;
   if (index == 5)
      res = 0x019Fu;
   if (index == 6)
      res = 0x0236u;
   if (index == 7)
      res = 0x02E0u;
   if (index == 8)
      res = 0x039Cu;
   if (index == 9)
      res = 0x0468u;
   if (index == 10)
      res = 0x0545u;
   if (index == 11)
      res = 0x631u;
   if (index == 12)
      res = 0x072Bu;
   if (index == 13)
      res = 0x0832u;
   if (index == 14)
      res = 0x0946u;
   if (index == 15)
      res = 0x0A67u;

   return res;
}

uint
__sqrtEvenAdjustments(int index)
{
   uint res = 0u;
   if (index == 0)
      res = 0x0A2Du;
   if (index == 1)
      res = 0x08AFu;
   if (index == 2)
      res = 0x075Au;
   if (index == 3)
      res = 0x0629u;
   if (index == 4)
      res = 0x051Au;
   if (index == 5)
      res = 0x0429u;
   if (index == 6)
      res = 0x0356u;
   if (index == 7)
      res = 0x029Eu;
   if (index == 8)
      res = 0x0200u;
   if (index == 9)
      res = 0x0179u;
   if (index == 10)
      res = 0x0109u;
   if (index == 11)
      res = 0x00AFu;
   if (index == 12)
      res = 0x0068u;
   if (index == 13)
      res = 0x0034u;
   if (index == 14)
      res = 0x0012u;
   if (index == 15)
      res = 0x0002u;

   return res;
}

/* Returns an approximation to the square root of the 32-bit significand given
 * by `a'.  Considered as an integer, `a' must be at least 2^31.  If bit 0 of
 * `aExp' (the least significant bit) is 1, the integer returned approximates
 * 2^31*sqrt(`a'/2^31), where `a' is considered an integer.  If bit 0 of `aExp'
 * is 0, the integer returned approximates 2^31*sqrt(`a'/2^30).  In either
 * case, the approximation returned lies strictly within +/-2 of the exact
 * value.
 */
uint
__estimateSqrt32(int aExp, uint a)
{
   uint z;

   int index = int(a>>27 & 15u);
   if ((aExp & 1) != 0) {
      z = 0x4000u + (a>>17) - __sqrtOddAdjustments(index);
      z = ((a / z)<<14) + (z<<15);
      a >>= 1;
   } else {
      z = 0x8000u + (a>>17) - __sqrtEvenAdjustments(index);
      z = a / z + z;
      z = (0x20000u <= z) ? 0xFFFF8000u : (z<<15);
      if (z <= a)
         return uint(int(a)>>1);
   }
   return ((__estimateDiv64To32(a, 0u, z))>>1) + (z>>1);
}

/* Returns the square root of the double-precision floating-point value `a'.
 * The operation is performed according to the IEEE Standard for Floating-Point
 * Arithmetic.
 */
uint64_t
__fsqrt64(uint64_t a)
{
   uint zFrac0 = 0u;
   uint zFrac1 = 0u;
   uint zFrac2 = 0u;
   uint doubleZFrac0 = 0u;
   uint rem0 = 0u;
   uint rem1 = 0u;
   uint rem2 = 0u;
   uint rem3 = 0u;
   uint term0 = 0u;
   uint term1 = 0u;
   uint term2 = 0u;
   uint term3 = 0u;
   uint64_t default_nan = 0xFFFFFFFFFFFFFFFFUL;

   uint aFracLo = __extractFloat64FracLo(a);
   uint aFracHi = __extractFloat64FracHi(a);
   int aExp = __extractFloat64Exp(a);
   uint aSign = __extractFloat64Sign(a);
   if (aExp == 0x7FF) {
      if ((aFracHi | aFracLo) != 0u)
         return __propagateFloat64NaN(a, a);
      if (aSign == 0u)
         return a;
      return default_nan;
   }
   if (aSign != 0u) {
      if ((uint(aExp) | aFracHi | aFracLo) == 0u)
         return a;
      return default_nan;
   }
   if (aExp == 0) {
      if ((aFracHi | aFracLo) == 0u)
         return __packFloat64(0u, 0, 0u, 0u);
      __normalizeFloat64Subnormal(aFracHi, aFracLo, aExp, aFracHi, aFracLo);
   }
   int zExp = ((aExp - 0x3FF)>>1) + 0x3FE;
   aFracHi |= 0x00100000u;
   __shortShift64Left(aFracHi, aFracLo, 11, term0, term1);
   zFrac0 = (__estimateSqrt32(aExp, term0)>>1) + 1u;
   if (zFrac0 == 0u)
      zFrac0 = 0x7FFFFFFFu;
   doubleZFrac0 = zFrac0 + zFrac0;
   __shortShift64Left(aFracHi, aFracLo, 9 - (aExp & 1), aFracHi, aFracLo);
   umulExtended(zFrac0, zFrac0, term0, term1);
   __sub64(aFracHi, aFracLo, term0, term1, rem0, rem1);
   while (int(rem0) < 0) {
      --zFrac0;
      doubleZFrac0 -= 2u;
      __add64(rem0, rem1, 0u, doubleZFrac0 | 1u, rem0, rem1);
   }
   zFrac1 = __estimateDiv64To32(rem1, 0u, doubleZFrac0);
   if ((zFrac1 & 0x1FFu) <= 5u) {
      if (zFrac1 == 0u)
         zFrac1 = 1u;
      umulExtended(doubleZFrac0, zFrac1, term1, term2);
      __sub64(rem1, 0u, term1, term2, rem1, rem2);
      umulExtended(zFrac1, zFrac1, term2, term3);
      __sub96(rem1, rem2, 0u, 0u, term2, term3, rem1, rem2, rem3);
      while (int(rem1) < 0) {
         --zFrac1;
         __shortShift64Left(0u, zFrac1, 1, term2, term3);
         term3 |= 1u;
         term2 |= doubleZFrac0;
         __add96(rem1, rem2, rem3, 0u, term2, term3, rem1, rem2, rem3);
      }
      zFrac1 |= uint((rem1 | rem2 | rem3) != 0u);
   }
   __shift64ExtraRightJamming(zFrac0, zFrac1, 0u, 10, zFrac0, zFrac1, zFrac2);
   return __roundAndPackFloat64(0u, zExp, zFrac0, zFrac1, zFrac2);
}

uint64_t
__ftrunc64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   int aExp = __extractFloat64Exp(__a);
   uint zLo;
   uint zHi;

   int unbiasedExp = aExp - 1023;
   int fracBits = 52 - unbiasedExp;
   uint maskLo = mix(~0u << fracBits, 0u, fracBits >= 32);
   uint maskHi = mix(~0u << (fracBits - 32), ~0u, fracBits < 33);
   zLo = maskLo & a.x;
   zHi = maskHi & a.y;

   zLo = mix(zLo, 0u, unbiasedExp < 0);
   zHi = mix(zHi, 0u, unbiasedExp < 0);
   zLo = mix(zLo, a.x, unbiasedExp > 52);
   zHi = mix(zHi, a.y, unbiasedExp > 52);
   return packUint2x32(uvec2(zLo, zHi));
}

uint64_t
__ffloor64(uint64_t a)
{
   /* The big assumtion is that when 'a' is NaN, __ftrunc(a) returns a.  Based
    * on that assumption, NaN values that don't have the sign bit will safely
    * return NaN (identity).  This is guarded by RELAXED_NAN_PROPAGATION
    * because otherwise the NaN should have the "signal" bit set.  The
    * __fadd64 will ensure that occurs.
    */
   bool is_positive =
#if defined RELAXED_NAN_PROPAGATION
      int(unpackUint2x32(a).y) >= 0
#else
      __fge64(a, 0ul)
#endif
      ;
   uint64_t tr = __ftrunc64(a);

   if (is_positive || __feq64(tr, a)) {
      return tr;
   } else {
      return __fadd64(tr, 0xbff0000000000000ul /* -1.0 */);
   }
}

uint64_t
__fround64(uint64_t __a)
{
   uvec2 a = unpackUint2x32(__a);
   int unbiasedExp = __extractFloat64Exp(__a) - 1023;
   uint aHi = a.y;
   uint aLo = a.x;

   if (unbiasedExp < 20) {
      if (unbiasedExp < 0) {
         if ((aHi & 0x80000000u) != 0u && aLo == 0u) {
            return 0;
         }
         aHi &= 0x80000000u;
         if ((a.y & 0x000FFFFFu) == 0u && a.x == 0u) {
            aLo = 0u;
            return packUint2x32(uvec2(aLo, aHi));
         }
         aHi = mix(aHi, (aHi | 0x3FF00000u), unbiasedExp == -1);
         aLo = 0u;
      } else {
         uint maskExp = 0x000FFFFFu >> unbiasedExp;
         uint lastBit = maskExp + 1;
         aHi += 0x00080000u >> unbiasedExp;
         if ((aHi & maskExp) == 0u)
            aHi &= ~lastBit;
         aHi &= ~maskExp;
         aLo = 0u;
      }
   } else if (unbiasedExp > 51 || unbiasedExp == 1024) {
      return __a;
   } else {
      uint maskExp = 0xFFFFFFFFu >> (unbiasedExp - 20);
      if ((aLo & maskExp) == 0u)
         return __a;
      uint tmp = aLo + (1u << (51 - unbiasedExp));
      if(tmp < aLo)
         aHi += 1u;
      aLo = tmp;
      aLo &= ~maskExp;
   }

   return packUint2x32(uvec2(aLo, aHi));
}

uint64_t
__fmin64(uint64_t a, uint64_t b)
{
   /* This weird layout matters.  Doing the "obvious" thing results in extra
    * flow control being inserted to implement the short-circuit evaluation
    * rules.  Flow control is bad!
    */
   bool b_nan = __is_nan(b);
   bool a_lt_b = __flt64_nonnan(a, b);
   bool a_nan = __is_nan(a);

   return (b_nan || a_lt_b) && !a_nan ? a : b;
}

uint64_t
__fmax64(uint64_t a, uint64_t b)
{
   /* This weird layout matters.  Doing the "obvious" thing results in extra
    * flow control being inserted to implement the short-circuit evaluation
    * rules.  Flow control is bad!
    */
   bool b_nan = __is_nan(b);
   bool a_lt_b = __flt64_nonnan(a, b);
   bool a_nan = __is_nan(a);

   return (b_nan || a_lt_b) && !a_nan ? b : a;
}

uint64_t
__ffract64(uint64_t a)
{
   return __fadd64(a, __fneg64(__ffloor64(a)));
}
