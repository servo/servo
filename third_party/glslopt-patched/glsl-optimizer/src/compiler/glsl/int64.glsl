/* Compile with:
 *
 * glsl_compiler --version 400 --dump-builder int64.glsl > builtin_int64.h
 *
 * Version 4.00+ is required for umulExtended.
 */
#version 400
#extension GL_ARB_gpu_shader_int64: require
#extension GL_ARB_shading_language_420pack: require

uvec2
umul64(uvec2 a, uvec2 b)
{
   uvec2 result;

   umulExtended(a.x, b.x, result.y, result.x);
   result.y += a.x * b.y + a.y * b.x;

   return result;
}

ivec2
sign64(ivec2 a)
{
   ivec2 result;

   result.y = a.y >> 31;
   result.x = result.y | int((a.x | a.y) != 0);

   return result;
}

uvec4
udivmod64(uvec2 n, uvec2 d)
{
   uvec2 quot = uvec2(0U, 0U);
   int log2_denom = findMSB(d.y) + 32;

   /* If the upper 32 bits of denom are non-zero, it is impossible for shifts
    * greater than 32 bits to occur.  If the upper 32 bits of the numerator
    * are zero, it is impossible for (denom << [63, 32]) <= numer unless
    * denom == 0.
    */
   if (d.y == 0 && n.y >= d.x) {
      log2_denom = findMSB(d.x);

      /* Since the upper 32 bits of denom are zero, log2_denom <= 31 and we
       * don't have to compare log2_denom inside the loop as is done in the
       * general case (below).
       */
      for (int i = 31; i >= 1; i--) {
	 if (log2_denom <= 31 - i && (d.x << i) <= n.y) {
	    n.y -= d.x << i;
	    quot.y |= 1U << i;
	 }
      }

      /* log2_denom is always <= 31, so manually peel the last loop
       * iteration.
       */
      if (d.x <= n.y) {
	 n.y -= d.x;
	 quot.y |= 1U;
      }
   }

   uint64_t d64 = packUint2x32(d);
   uint64_t n64 = packUint2x32(n);
   for (int i = 31; i >= 1; i--) {
      if (log2_denom <= 63 - i && (d64 << i) <= n64) {
	 n64 -= d64 << i;
	 quot.x |= 1U << i;
      }
   }

   /* log2_denom is always <= 63, so manually peel the last loop
    * iteration.
    */
   if (d64 <= n64) {
      n64 -= d64;
      quot.x |= 1U;
   }

   return uvec4(quot, unpackUint2x32(n64));
}

uvec2
udiv64(uvec2 n, uvec2 d)
{
   return udivmod64(n, d).xy;
}

ivec2
idiv64(ivec2 _n, ivec2 _d)
{
   const bool negate = (_n.y < 0) != (_d.y < 0);
   uvec2 n = unpackUint2x32(uint64_t(abs(packInt2x32(_n))));
   uvec2 d = unpackUint2x32(uint64_t(abs(packInt2x32(_d))));

   uvec2 quot = udivmod64(n, d).xy;

   return negate ? unpackInt2x32(-int64_t(packUint2x32(quot))) : ivec2(quot);
}

uvec2
umod64(uvec2 n, uvec2 d)
{
   return udivmod64(n, d).zw;
}

ivec2
imod64(ivec2 _n, ivec2 _d)
{
   const bool negate = (_n.y < 0) != (_d.y < 0);
   uvec2 n = unpackUint2x32(uint64_t(abs(packInt2x32(_n))));
   uvec2 d = unpackUint2x32(uint64_t(abs(packInt2x32(_d))));

   uvec2 rem = udivmod64(n, d).zw;

   return negate ? unpackInt2x32(-int64_t(packUint2x32(rem))) : ivec2(rem);
}
