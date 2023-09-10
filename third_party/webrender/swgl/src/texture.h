/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

namespace glsl {

using PackedRGBA8 = V16<uint8_t>;
using WideRGBA8 = V16<uint16_t>;
using HalfRGBA8 = V8<uint16_t>;

SI WideRGBA8 unpack(PackedRGBA8 p) { return CONVERT(p, WideRGBA8); }

template <int N>
UNUSED SI VectorType<uint8_t, N> genericPackWide(VectorType<uint16_t, N> p) {
  typedef VectorType<uint8_t, N> packed_type;
  // Generic conversions only mask off the low byte without actually clamping
  // like a real pack. First force the word to all 1s if it overflows, and then
  // add on the sign bit to cause it to roll over to 0 if it was negative.
  p = (p | (p > 255)) + (p >> 15);
  return CONVERT(p, packed_type);
}

SI PackedRGBA8 pack(WideRGBA8 p) {
#if USE_SSE2
  return _mm_packus_epi16(lowHalf(p), highHalf(p));
#elif USE_NEON
  return vcombine_u8(vqmovn_u16(lowHalf(p)), vqmovn_u16(highHalf(p)));
#else
  return genericPackWide(p);
#endif
}

using PackedR8 = V4<uint8_t>;
using WideR8 = V4<uint16_t>;

SI WideR8 unpack(PackedR8 p) { return CONVERT(p, WideR8); }

SI PackedR8 pack(WideR8 p) {
#if USE_SSE2
  auto m = expand(p);
  auto r = bit_cast<V16<uint8_t>>(_mm_packus_epi16(m, m));
  return SHUFFLE(r, r, 0, 1, 2, 3);
#elif USE_NEON
  return lowHalf(bit_cast<V8<uint8_t>>(vqmovn_u16(expand(p))));
#else
  return genericPackWide(p);
#endif
}

using PackedRG8 = V8<uint8_t>;
using WideRG8 = V8<uint16_t>;

SI PackedRG8 pack(WideRG8 p) {
#if USE_SSE2
  return lowHalf(bit_cast<V16<uint8_t>>(_mm_packus_epi16(p, p)));
#elif USE_NEON
  return bit_cast<V8<uint8_t>>(vqmovn_u16(p));
#else
  return genericPackWide(p);
#endif
}

SI I32 clampCoord(I32 coord, int limit, int base = 0) {
#if USE_SSE2
  return _mm_min_epi16(_mm_max_epi16(coord, _mm_set1_epi32(base)),
                       _mm_set1_epi32(limit - 1));
#else
  return clamp(coord, base, limit - 1);
#endif
}

SI int clampCoord(int coord, int limit, int base = 0) {
  return min(max(coord, base), limit - 1);
}

template <typename T, typename S>
SI T clamp2D(T P, S sampler) {
  return T{clampCoord(P.x, sampler->width), clampCoord(P.y, sampler->height)};
}

SI float to_float(uint32_t x) { return x * (1.f / 255.f); }

SI vec4 pixel_to_vec4(uint32_t a, uint32_t b, uint32_t c, uint32_t d) {
  U32 pixels = {a, b, c, d};
  return vec4(cast((pixels >> 16) & 0xFF), cast((pixels >> 8) & 0xFF),
              cast(pixels & 0xFF), cast(pixels >> 24)) *
         (1.0f / 255.0f);
}

SI vec4 pixel_float_to_vec4(Float a, Float b, Float c, Float d) {
  return vec4(Float{a.x, b.x, c.x, d.x}, Float{a.y, b.y, c.y, d.y},
              Float{a.z, b.z, c.z, d.z}, Float{a.w, b.w, c.w, d.w});
}

SI ivec4 pixel_int_to_ivec4(I32 a, I32 b, I32 c, I32 d) {
  return ivec4(I32{a.x, b.x, c.x, d.x}, I32{a.y, b.y, c.y, d.y},
               I32{a.z, b.z, c.z, d.z}, I32{a.w, b.w, c.w, d.w});
}

SI vec4_scalar pixel_to_vec4(uint32_t p) {
  U32 i = {(p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF, p >> 24};
  Float f = cast(i) * (1.0f / 255.0f);
  return vec4_scalar(f.x, f.y, f.z, f.w);
}

template <typename S>
SI vec4 fetchOffsetsRGBA8(S sampler, I32 offset) {
  return pixel_to_vec4(sampler->buf[offset.x], sampler->buf[offset.y],
                       sampler->buf[offset.z], sampler->buf[offset.w]);
}

template <typename S>
vec4 texelFetchRGBA8(S sampler, ivec2 P) {
  I32 offset = P.x + P.y * sampler->stride;
  return fetchOffsetsRGBA8(sampler, offset);
}

template <typename S>
SI Float fetchOffsetsR8(S sampler, I32 offset) {
  U32 i = {
      ((uint8_t*)sampler->buf)[offset.x], ((uint8_t*)sampler->buf)[offset.y],
      ((uint8_t*)sampler->buf)[offset.z], ((uint8_t*)sampler->buf)[offset.w]};
  return cast(i) * (1.0f / 255.0f);
}

template <typename S>
vec4 texelFetchR8(S sampler, ivec2 P) {
  I32 offset = P.x + P.y * sampler->stride;
  return vec4(fetchOffsetsR8(sampler, offset), 0.0f, 0.0f, 1.0f);
}

template <typename S>
SI vec4 fetchOffsetsRG8(S sampler, I32 offset) {
  uint16_t* buf = (uint16_t*)sampler->buf;
  U16 pixels = {buf[offset.x], buf[offset.y], buf[offset.z], buf[offset.w]};
  Float r = CONVERT(pixels & 0xFF, Float) * (1.0f / 255.0f);
  Float g = CONVERT(pixels >> 8, Float) * (1.0f / 255.0f);
  return vec4(r, g, 0.0f, 1.0f);
}

template <typename S>
vec4 texelFetchRG8(S sampler, ivec2 P) {
  I32 offset = P.x + P.y * sampler->stride;
  return fetchOffsetsRG8(sampler, offset);
}

template <typename S>
SI Float fetchOffsetsR16(S sampler, I32 offset) {
  U32 i = {
      ((uint16_t*)sampler->buf)[offset.x], ((uint16_t*)sampler->buf)[offset.y],
      ((uint16_t*)sampler->buf)[offset.z], ((uint16_t*)sampler->buf)[offset.w]};
  return cast(i) * (1.0f / 65535.0f);
}

template <typename S>
vec4 texelFetchR16(S sampler, ivec2 P) {
  I32 offset = P.x + P.y * sampler->stride;
  return vec4(fetchOffsetsR16(sampler, offset), 0.0f, 0.0f, 1.0f);
}

template <typename S>
SI vec4 fetchOffsetsFloat(S sampler, I32 offset) {
  return pixel_float_to_vec4(
      *(Float*)&sampler->buf[offset.x], *(Float*)&sampler->buf[offset.y],
      *(Float*)&sampler->buf[offset.z], *(Float*)&sampler->buf[offset.w]);
}

vec4 texelFetchFloat(sampler2D sampler, ivec2 P) {
  I32 offset = P.x * 4 + P.y * sampler->stride;
  return fetchOffsetsFloat(sampler, offset);
}

template <typename S>
SI vec4 fetchOffsetsYUV422(S sampler, I32 offset) {
  // Layout is 2 pixel chunks (occupying 4 bytes) organized as: G0, B, G1, R.
  // Offset is aligned to a chunk rather than a pixel, and selector specifies
  // pixel within the chunk.
  I32 selector = offset & 1;
  offset &= ~1;
  uint16_t* buf = (uint16_t*)sampler->buf;
  U32 pixels = {*(uint32_t*)&buf[offset.x], *(uint32_t*)&buf[offset.y],
                *(uint32_t*)&buf[offset.z], *(uint32_t*)&buf[offset.w]};
  Float b = CONVERT((pixels >> 8) & 0xFF, Float) * (1.0f / 255.0f);
  Float r = CONVERT((pixels >> 24), Float) * (1.0f / 255.0f);
  Float g =
      CONVERT(if_then_else(-selector, pixels >> 16, pixels) & 0xFF, Float) *
      (1.0f / 255.0f);
  return vec4(r, g, b, 1.0f);
}

template <typename S>
vec4 texelFetchYUV422(S sampler, ivec2 P) {
  I32 offset = P.x + P.y * sampler->stride;
  return fetchOffsetsYUV422(sampler, offset);
}

vec4 texelFetch(sampler2D sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  switch (sampler->format) {
    case TextureFormat::RGBA32F:
      return texelFetchFloat(sampler, P);
    case TextureFormat::RGBA8:
      return texelFetchRGBA8(sampler, P);
    case TextureFormat::R8:
      return texelFetchR8(sampler, P);
    case TextureFormat::RG8:
      return texelFetchRG8(sampler, P);
    case TextureFormat::R16:
      return texelFetchR16(sampler, P);
    case TextureFormat::YUV422:
      return texelFetchYUV422(sampler, P);
    default:
      assert(false);
      return vec4();
  }
}

vec4 texelFetch(sampler2DRGBA32F sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA32F);
  return texelFetchFloat(sampler, P);
}

vec4 texelFetch(sampler2DRGBA8 sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA8);
  return texelFetchRGBA8(sampler, P);
}

vec4 texelFetch(sampler2DR8 sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::R8);
  return texelFetchR8(sampler, P);
}

vec4 texelFetch(sampler2DRG8 sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RG8);
  return texelFetchRG8(sampler, P);
}

vec4_scalar texelFetch(sampler2D sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  if (sampler->format == TextureFormat::RGBA32F) {
    return *(vec4_scalar*)&sampler->buf[P.x * 4 + P.y * sampler->stride];
  } else {
    assert(sampler->format == TextureFormat::RGBA8);
    return pixel_to_vec4(sampler->buf[P.x + P.y * sampler->stride]);
  }
}

vec4_scalar texelFetch(sampler2DRGBA32F sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA32F);
  return *(vec4_scalar*)&sampler->buf[P.x * 4 + P.y * sampler->stride];
}

vec4_scalar texelFetch(sampler2DRGBA8 sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA8);
  return pixel_to_vec4(sampler->buf[P.x + P.y * sampler->stride]);
}

vec4_scalar texelFetch(sampler2DR8 sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::R8);
  return vec4_scalar{
      to_float(((uint8_t*)sampler->buf)[P.x + P.y * sampler->stride]), 0.0f,
      0.0f, 1.0f};
}

vec4_scalar texelFetch(sampler2DRG8 sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RG8);
  uint16_t pixel = ((uint16_t*)sampler->buf)[P.x + P.y * sampler->stride];
  return vec4_scalar{to_float(pixel & 0xFF), to_float(pixel >> 8), 0.0f, 1.0f};
}

vec4 texelFetch(sampler2DRect sampler, ivec2 P) {
  P = clamp2D(P, sampler);
  switch (sampler->format) {
    case TextureFormat::RGBA8:
      return texelFetchRGBA8(sampler, P);
    case TextureFormat::R8:
      return texelFetchR8(sampler, P);
    case TextureFormat::RG8:
      return texelFetchRG8(sampler, P);
    case TextureFormat::R16:
      return texelFetchR16(sampler, P);
    case TextureFormat::YUV422:
      return texelFetchYUV422(sampler, P);
    default:
      assert(false);
      return vec4();
  }
}

template <typename S>
SI ivec4 fetchOffsetsInt(S sampler, I32 offset) {
  return pixel_int_to_ivec4(
      *(I32*)&sampler->buf[offset.x], *(I32*)&sampler->buf[offset.y],
      *(I32*)&sampler->buf[offset.z], *(I32*)&sampler->buf[offset.w]);
}

ivec4 texelFetch(isampler2D sampler, ivec2 P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA32I);
  I32 offset = P.x * 4 + P.y * sampler->stride;
  return fetchOffsetsInt(sampler, offset);
}

ivec4_scalar texelFetch(isampler2D sampler, ivec2_scalar P, int lod) {
  assert(lod == 0);
  P = clamp2D(P, sampler);
  assert(sampler->format == TextureFormat::RGBA32I);
  return *(ivec4_scalar*)&sampler->buf[P.x * 4 + P.y * sampler->stride];
}

SI vec4_scalar* texelFetchPtr(sampler2D sampler, ivec2_scalar P, int min_x,
                              int max_x, int min_y, int max_y) {
  P.x = min(max(P.x, -min_x), int(sampler->width) - 1 - max_x);
  P.y = min(max(P.y, -min_y), int(sampler->height) - 1 - max_y);
  assert(sampler->format == TextureFormat::RGBA32F);
  return (vec4_scalar*)&sampler->buf[P.x * 4 + P.y * sampler->stride];
}

SI ivec4_scalar* texelFetchPtr(isampler2D sampler, ivec2_scalar P, int min_x,
                               int max_x, int min_y, int max_y) {
  P.x = min(max(P.x, -min_x), int(sampler->width) - 1 - max_x);
  P.y = min(max(P.y, -min_y), int(sampler->height) - 1 - max_y);
  assert(sampler->format == TextureFormat::RGBA32I);
  return (ivec4_scalar*)&sampler->buf[P.x * 4 + P.y * sampler->stride];
}

template <typename S>
SI I32 texelFetchPtr(S sampler, ivec2 P, int min_x, int max_x, int min_y,
                     int max_y) {
  P.x = clampCoord(P.x, int(sampler->width) - max_x, -min_x);
  P.y = clampCoord(P.y, int(sampler->height) - max_y, -min_y);
  return P.x * 4 + P.y * sampler->stride;
}

template <typename S, typename P>
SI P texelFetchUnchecked(S sampler, P* ptr, int x, int y = 0) {
  return ptr[x + y * (sampler->stride >> 2)];
}

SI vec4 texelFetchUnchecked(sampler2D sampler, I32 offset, int x, int y = 0) {
  assert(sampler->format == TextureFormat::RGBA32F);
  return fetchOffsetsFloat(sampler, offset + (x * 4 + y * sampler->stride));
}

SI ivec4 texelFetchUnchecked(isampler2D sampler, I32 offset, int x, int y = 0) {
  assert(sampler->format == TextureFormat::RGBA32I);
  return fetchOffsetsInt(sampler, offset + (x * 4 + y * sampler->stride));
}

#define texelFetchOffset(sampler, P, lod, offset) \
  texelFetch(sampler, (P) + (offset), lod)

// Scale texture coords for quantization, subtract offset for filtering
// (assuming coords already offset to texel centers), and round to nearest
// 1/scale increment
template <typename T>
SI T linearQuantize(T P, float scale) {
  return P * scale + (0.5f - 0.5f * scale);
}

// Helper version that also scales normalized texture coords for sampler
template <typename T, typename S>
SI T samplerScale(S sampler, T P) {
  P.x *= sampler->width;
  P.y *= sampler->height;
  return P;
}

template <typename T>
SI T samplerScale(UNUSED sampler2DRect sampler, T P) {
  return P;
}

template <typename T, typename S>
SI T linearQuantize(T P, float scale, S sampler) {
  return linearQuantize(samplerScale(sampler, P), scale);
}

// Compute clamped offset of first row for linear interpolation
template <typename S, typename I>
SI auto computeRow(S sampler, I i, size_t margin = 1) -> decltype(i.x) {
  return clampCoord(i.x, sampler->width - margin) +
         clampCoord(i.y, sampler->height) * sampler->stride;
}

// Compute clamped offset of second row for linear interpolation from first row
template <typename S, typename I>
SI auto computeNextRowOffset(S sampler, I i) -> decltype(i.x) {
  return if_then_else(i.y >= 0 && i.y < int32_t(sampler->height) - 1,
                      sampler->stride, 0);
}

// Convert X coordinate to a 2^7 scale fraction for interpolation
template <typename S>
SI I16 computeFracX(S sampler, ivec2 i, ivec2 frac) {
  auto overread = i.x > int32_t(sampler->width) - 2;
  return CONVERT((((frac.x & (i.x >= 0)) | overread) & 0x7F) - overread, I16);
}

// Convert Y coordinate to a 2^7 scale fraction for interpolation
SI I16 computeFracNoClamp(I32 frac) { return CONVERT(frac & 0x7F, I16); }
SI I16 computeFracY(ivec2 frac) { return computeFracNoClamp(frac.y); }

struct WidePlanarRGBA8 {
  V8<uint16_t> rg;
  V8<uint16_t> ba;
};

template <typename S>
SI WidePlanarRGBA8 textureLinearPlanarRGBA8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::RGBA8);

  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  I16 fracx = computeFracX(sampler, i, frac);
  I16 fracy = computeFracY(frac);

  auto a0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.x]), V8<int16_t>);
  auto a1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.x]), V8<int16_t>);
  a0 += ((a1 - a0) * fracy.x) >> 7;

  auto b0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.y]), V8<int16_t>);
  auto b1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.y]), V8<int16_t>);
  b0 += ((b1 - b0) * fracy.y) >> 7;

  auto abl = zipLow(a0, b0);
  auto abh = zipHigh(a0, b0);
  abl += ((abh - abl) * fracx.xyxyxyxy) >> 7;

  auto c0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.z]), V8<int16_t>);
  auto c1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.z]), V8<int16_t>);
  c0 += ((c1 - c0) * fracy.z) >> 7;

  auto d0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.w]), V8<int16_t>);
  auto d1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.w]), V8<int16_t>);
  d0 += ((d1 - d0) * fracy.w) >> 7;

  auto cdl = zipLow(c0, d0);
  auto cdh = zipHigh(c0, d0);
  cdl += ((cdh - cdl) * fracx.zwzwzwzw) >> 7;

  auto rg = V8<uint16_t>(zip2Low(abl, cdl));
  auto ba = V8<uint16_t>(zip2High(abl, cdl));
  return WidePlanarRGBA8{rg, ba};
}

template <typename S>
vec4 textureLinearRGBA8(S sampler, vec2 P) {
  ivec2 i(linearQuantize(P, 128, sampler));
  auto planar = textureLinearPlanarRGBA8(sampler, i);
  auto rg = CONVERT(planar.rg, V8<float>);
  auto ba = CONVERT(planar.ba, V8<float>);
  auto r = lowHalf(rg);
  auto g = highHalf(rg);
  auto b = lowHalf(ba);
  auto a = highHalf(ba);
  return vec4(b, g, r, a) * (1.0f / 255.0f);
}

template <typename S>
static inline U16 textureLinearUnpackedR8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::R8);
  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  I16 fracx = computeFracX(sampler, i, frac);
  I16 fracy = computeFracY(frac);

  uint8_t* buf = (uint8_t*)sampler->buf;
  auto a0 = unaligned_load<V2<uint8_t>>(&buf[row0.x]);
  auto b0 = unaligned_load<V2<uint8_t>>(&buf[row0.y]);
  auto c0 = unaligned_load<V2<uint8_t>>(&buf[row0.z]);
  auto d0 = unaligned_load<V2<uint8_t>>(&buf[row0.w]);
  auto abcd0 = CONVERT(combine(a0, b0, c0, d0), V8<int16_t>);

  auto a1 = unaligned_load<V2<uint8_t>>(&buf[row1.x]);
  auto b1 = unaligned_load<V2<uint8_t>>(&buf[row1.y]);
  auto c1 = unaligned_load<V2<uint8_t>>(&buf[row1.z]);
  auto d1 = unaligned_load<V2<uint8_t>>(&buf[row1.w]);
  auto abcd1 = CONVERT(combine(a1, b1, c1, d1), V8<int16_t>);

  abcd0 += ((abcd1 - abcd0) * fracy.xxyyzzww) >> 7;

  abcd0 = SHUFFLE(abcd0, abcd0, 0, 2, 4, 6, 1, 3, 5, 7);
  auto abcdl = lowHalf(abcd0);
  auto abcdh = highHalf(abcd0);
  abcdl += ((abcdh - abcdl) * fracx) >> 7;

  return U16(abcdl);
}

template <typename S>
vec4 textureLinearR8(S sampler, vec2 P) {
  assert(sampler->format == TextureFormat::R8);

  ivec2 i(linearQuantize(P, 128, sampler));
  Float r = CONVERT(textureLinearUnpackedR8(sampler, i), Float);
  return vec4(r * (1.0f / 255.0f), 0.0f, 0.0f, 1.0f);
}

struct WidePlanarRG8 {
  V8<uint16_t> rg;
};

template <typename S>
SI WidePlanarRG8 textureLinearPlanarRG8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::RG8);

  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  I16 fracx = computeFracX(sampler, i, frac);
  I16 fracy = computeFracY(frac);

  uint16_t* buf = (uint16_t*)sampler->buf;

  // Load RG bytes for two adjacent pixels - rgRG
  auto a0 = unaligned_load<V4<uint8_t>>(&buf[row0.x]);
  auto b0 = unaligned_load<V4<uint8_t>>(&buf[row0.y]);
  auto ab0 = CONVERT(combine(a0, b0), V8<int16_t>);
  // Load two pixels for next row
  auto a1 = unaligned_load<V4<uint8_t>>(&buf[row1.x]);
  auto b1 = unaligned_load<V4<uint8_t>>(&buf[row1.y]);
  auto ab1 = CONVERT(combine(a1, b1), V8<int16_t>);
  // Blend rows
  ab0 += ((ab1 - ab0) * fracy.xxxxyyyy) >> 7;

  auto c0 = unaligned_load<V4<uint8_t>>(&buf[row0.z]);
  auto d0 = unaligned_load<V4<uint8_t>>(&buf[row0.w]);
  auto cd0 = CONVERT(combine(c0, d0), V8<int16_t>);
  auto c1 = unaligned_load<V4<uint8_t>>(&buf[row1.z]);
  auto d1 = unaligned_load<V4<uint8_t>>(&buf[row1.w]);
  auto cd1 = CONVERT(combine(c1, d1), V8<int16_t>);
  // Blend rows
  cd0 += ((cd1 - cd0) * fracy.zzzzwwww) >> 7;

  // ab = a.rgRG,b.rgRG
  // cd = c.rgRG,d.rgRG
  // ... ac = ar,cr,ag,cg,aR,cR,aG,cG
  // ... bd = br,dr,bg,dg,bR,dR,bG,dG
  auto ac = zipLow(ab0, cd0);
  auto bd = zipHigh(ab0, cd0);
  // ar,br,cr,dr,ag,bg,cg,dg
  // aR,bR,cR,dR,aG,bG,cG,dG
  auto abcdl = zipLow(ac, bd);
  auto abcdh = zipHigh(ac, bd);
  // Blend columns
  abcdl += ((abcdh - abcdl) * fracx.xyzwxyzw) >> 7;

  auto rg = V8<uint16_t>(abcdl);
  return WidePlanarRG8{rg};
}

template <typename S>
vec4 textureLinearRG8(S sampler, vec2 P) {
  ivec2 i(linearQuantize(P, 128, sampler));
  auto planar = textureLinearPlanarRG8(sampler, i);
  auto rg = CONVERT(planar.rg, V8<float>) * (1.0f / 255.0f);
  auto r = lowHalf(rg);
  auto g = highHalf(rg);
  return vec4(r, g, 0.0f, 1.0f);
}

// Samples R16 texture with linear filtering and returns results packed as
// signed I16. One bit of precision is shifted away from the bottom end to
// accommodate the sign bit, so only 15 bits of precision is left.
template <typename S>
static inline I16 textureLinearUnpackedR16(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::R16);

  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);

  I16 fracx =
      CONVERT(
          ((frac.x & (i.x >= 0)) | (i.x > int32_t(sampler->width) - 2)) & 0x7F,
          I16)
      << 8;
  I16 fracy = computeFracY(frac) << 8;

  // Sample the 16 bit data for both rows
  uint16_t* buf = (uint16_t*)sampler->buf;
  auto a0 = unaligned_load<V2<uint16_t>>(&buf[row0.x]);
  auto b0 = unaligned_load<V2<uint16_t>>(&buf[row0.y]);
  auto c0 = unaligned_load<V2<uint16_t>>(&buf[row0.z]);
  auto d0 = unaligned_load<V2<uint16_t>>(&buf[row0.w]);
  auto abcd0 = CONVERT(combine(a0, b0, c0, d0) >> 1, V8<int16_t>);

  auto a1 = unaligned_load<V2<uint16_t>>(&buf[row1.x]);
  auto b1 = unaligned_load<V2<uint16_t>>(&buf[row1.y]);
  auto c1 = unaligned_load<V2<uint16_t>>(&buf[row1.z]);
  auto d1 = unaligned_load<V2<uint16_t>>(&buf[row1.w]);
  auto abcd1 = CONVERT(combine(a1, b1, c1, d1) >> 1, V8<int16_t>);

  // The samples occupy 15 bits and the fraction occupies 15 bits, so that when
  // they are multiplied together, the new scaled sample will fit in the high
  // 14 bits of the result. It is left shifted once to make it 15 bits again
  // for the final multiply.
#if USE_SSE2
  abcd0 += bit_cast<V8<int16_t>>(_mm_mulhi_epi16(abcd1 - abcd0, fracy.xxyyzzww))
           << 1;
#elif USE_NEON
  // NEON has a convenient instruction that does both the multiply and the
  // doubling, so doesn't need an extra shift.
  abcd0 += bit_cast<V8<int16_t>>(vqrdmulhq_s16(abcd1 - abcd0, fracy.xxyyzzww));
#else
  abcd0 += CONVERT((CONVERT(abcd1 - abcd0, V8<int32_t>) *
                    CONVERT(fracy.xxyyzzww, V8<int32_t>)) >>
                       16,
                   V8<int16_t>)
           << 1;
#endif

  abcd0 = SHUFFLE(abcd0, abcd0, 0, 2, 4, 6, 1, 3, 5, 7);
  auto abcdl = lowHalf(abcd0);
  auto abcdh = highHalf(abcd0);
#if USE_SSE2
  abcdl += lowHalf(bit_cast<V8<int16_t>>(
               _mm_mulhi_epi16(expand(abcdh - abcdl), expand(fracx))))
           << 1;
#elif USE_NEON
  abcdl += bit_cast<V4<int16_t>>(vqrdmulh_s16(abcdh - abcdl, fracx));
#else
  abcdl += CONVERT((CONVERT(abcdh - abcdl, V4<int32_t>) *
                    CONVERT(fracx, V4<int32_t>)) >>
                       16,
                   V4<int16_t>)
           << 1;
#endif

  return abcdl;
}

template <typename S>
vec4 textureLinearR16(S sampler, vec2 P) {
  assert(sampler->format == TextureFormat::R16);

  ivec2 i(linearQuantize(P, 128, sampler));
  Float r = CONVERT(textureLinearUnpackedR16(sampler, i), Float);
  return vec4(r * (1.0f / 32767.0f), 0.0f, 0.0f, 1.0f);
}

using PackedRGBA32F = V16<float>;
using WideRGBA32F = V16<float>;

template <typename S>
vec4 textureLinearRGBA32F(S sampler, vec2 P) {
  assert(sampler->format == TextureFormat::RGBA32F);
  P = samplerScale(sampler, P);
  P -= 0.5f;
  vec2 f = floor(P);
  vec2 r = P - f;
  ivec2 i(f);
  ivec2 c(clampCoord(i.x, sampler->width - 1),
          clampCoord(i.y, sampler->height));
  r.x = if_then_else(i.x >= 0, if_then_else(i.x < sampler->width - 1, r.x, 1.0),
                     0.0f);
  I32 offset0 = c.x * 4 + c.y * sampler->stride;
  I32 offset1 = offset0 + computeNextRowOffset(sampler, i);

  Float c0 = mix(mix(*(Float*)&sampler->buf[offset0.x],
                     *(Float*)&sampler->buf[offset0.x + 4], r.x),
                 mix(*(Float*)&sampler->buf[offset1.x],
                     *(Float*)&sampler->buf[offset1.x + 4], r.x),
                 r.y);
  Float c1 = mix(mix(*(Float*)&sampler->buf[offset0.y],
                     *(Float*)&sampler->buf[offset0.y + 4], r.x),
                 mix(*(Float*)&sampler->buf[offset1.y],
                     *(Float*)&sampler->buf[offset1.y + 4], r.x),
                 r.y);
  Float c2 = mix(mix(*(Float*)&sampler->buf[offset0.z],
                     *(Float*)&sampler->buf[offset0.z + 4], r.x),
                 mix(*(Float*)&sampler->buf[offset1.z],
                     *(Float*)&sampler->buf[offset1.z + 4], r.x),
                 r.y);
  Float c3 = mix(mix(*(Float*)&sampler->buf[offset0.w],
                     *(Float*)&sampler->buf[offset0.w + 4], r.x),
                 mix(*(Float*)&sampler->buf[offset1.w],
                     *(Float*)&sampler->buf[offset1.w + 4], r.x),
                 r.y);
  return pixel_float_to_vec4(c0, c1, c2, c3);
}

struct WidePlanarYUV8 {
  U16 y;
  U16 u;
  U16 v;
};

template <typename S>
SI WidePlanarYUV8 textureLinearPlanarYUV422(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::YUV422);

  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i, 2);
  // Layout is 2 pixel chunks (occupying 4 bytes) organized as: G0, B, G1, R.
  // Get the selector for the pixel within the chunk.
  I32 selector = row0 & 1;
  // Align the row index to the chunk.
  row0 &= ~1;
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  // G only needs to be clamped to a pixel boundary for safe interpolation,
  // whereas the BR fraction needs to be clamped 1 extra pixel inside to a chunk
  // boundary.
  frac.x &= (i.x >= 0);
  auto fracx =
      CONVERT(combine(frac.x | (i.x > int32_t(sampler->width) - 3),
                      (frac.x >> 1) | (i.x > int32_t(sampler->width) - 3)) &
                  0x7F,
              V8<int16_t>);
  I16 fracy = computeFracY(frac);

  uint16_t* buf = (uint16_t*)sampler->buf;

  // Load bytes for two adjacent chunks - g0,b,g1,r,G0,B,G1,R
  // We always need to interpolate between (b,r) and (B,R).
  // Depending on selector we need to either interpolate between g0 and g1
  // or between g1 and G0. So for now we just interpolate both cases for g
  // and will select the appropriate one on output.
  auto a0 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row0.x]), V8<int16_t>);
  auto a1 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row1.x]), V8<int16_t>);
  // Combine with next row.
  a0 += ((a1 - a0) * fracy.x) >> 7;

  auto b0 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row0.y]), V8<int16_t>);
  auto b1 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row1.y]), V8<int16_t>);
  b0 += ((b1 - b0) * fracy.y) >> 7;

  auto c0 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row0.z]), V8<int16_t>);
  auto c1 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row1.z]), V8<int16_t>);
  c0 += ((c1 - c0) * fracy.z) >> 7;

  auto d0 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row0.w]), V8<int16_t>);
  auto d1 = CONVERT(unaligned_load<V8<uint8_t>>(&buf[row1.w]), V8<int16_t>);
  d0 += ((d1 - d0) * fracy.w) >> 7;

  // Shuffle things around so we end up with g0,g0,g0,g0,b,b,b,b and
  // g1,g1,g1,g1,r,r,r,r.
  auto abl = zipLow(a0, b0);
  auto cdl = zipLow(c0, d0);
  auto g0b = zip2Low(abl, cdl);
  auto g1r = zip2High(abl, cdl);

  // Need to zip g1,B,G0,R. Instead of using a bunch of complicated masks and
  // and shifts, just shuffle here instead... We finally end up with
  // g1,g1,g1,g1,B,B,B,B and G0,G0,G0,G0,R,R,R,R.
  auto abh = SHUFFLE(a0, b0, 2, 10, 5, 13, 4, 12, 7, 15);
  auto cdh = SHUFFLE(c0, d0, 2, 10, 5, 13, 4, 12, 7, 15);
  auto g1B = zip2Low(abh, cdh);
  auto G0R = zip2High(abh, cdh);

  // Finally interpolate between adjacent columns.
  g0b += ((g1B - g0b) * fracx) >> 7;
  g1r += ((G0R - g1r) * fracx) >> 7;

  // Choose either g0 or g1 based on selector.
  return WidePlanarYUV8{
      U16(if_then_else(CONVERT(-selector, I16), lowHalf(g1r), lowHalf(g0b))),
      U16(highHalf(g0b)), U16(highHalf(g1r))};
}

template <typename S>
vec4 textureLinearYUV422(S sampler, vec2 P) {
  ivec2 i(linearQuantize(P, 128, sampler));
  auto planar = textureLinearPlanarYUV422(sampler, i);
  auto y = CONVERT(planar.y, Float) * (1.0f / 255.0f);
  auto u = CONVERT(planar.u, Float) * (1.0f / 255.0f);
  auto v = CONVERT(planar.v, Float) * (1.0f / 255.0f);
  return vec4(v, y, u, 1.0f);
}

SI vec4 texture(sampler2D sampler, vec2 P) {
  if (sampler->filter == TextureFilter::LINEAR) {
    switch (sampler->format) {
      case TextureFormat::RGBA32F:
        return textureLinearRGBA32F(sampler, P);
      case TextureFormat::RGBA8:
        return textureLinearRGBA8(sampler, P);
      case TextureFormat::R8:
        return textureLinearR8(sampler, P);
      case TextureFormat::RG8:
        return textureLinearRG8(sampler, P);
      case TextureFormat::R16:
        return textureLinearR16(sampler, P);
      case TextureFormat::YUV422:
        return textureLinearYUV422(sampler, P);
      default:
        assert(false);
        return vec4();
    }
  } else {
    ivec2 coord(roundzero(P.x, sampler->width),
                roundzero(P.y, sampler->height));
    return texelFetch(sampler, coord, 0);
  }
}

vec4 texture(sampler2DRect sampler, vec2 P) {
  if (sampler->filter == TextureFilter::LINEAR) {
    switch (sampler->format) {
      case TextureFormat::RGBA8:
        return textureLinearRGBA8(sampler, P);
      case TextureFormat::R8:
        return textureLinearR8(sampler, P);
      case TextureFormat::RG8:
        return textureLinearRG8(sampler, P);
      case TextureFormat::R16:
        return textureLinearR16(sampler, P);
      case TextureFormat::YUV422:
        return textureLinearYUV422(sampler, P);
      default:
        assert(false);
        return vec4();
    }
  } else {
    ivec2 coord(roundzero(P.x, 1.0f), roundzero(P.y, 1.0f));
    return texelFetch(sampler, coord);
  }
}

template <typename S>
vec4_scalar texture(S sampler, vec2_scalar P) {
  return force_scalar(texture(sampler, vec2(P)));
}

ivec2_scalar textureSize(sampler2D sampler, int) {
  return ivec2_scalar{int32_t(sampler->width), int32_t(sampler->height)};
}

ivec2_scalar textureSize(sampler2DRect sampler) {
  return ivec2_scalar{int32_t(sampler->width), int32_t(sampler->height)};
}

template <typename S>
static WideRGBA8 textureLinearUnpackedRGBA8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::RGBA8);
  ivec2 frac = i;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  I16 fracx = computeFracX(sampler, i, frac);
  I16 fracy = computeFracY(frac);

  auto a0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.x]), V8<int16_t>);
  auto a1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.x]), V8<int16_t>);
  a0 += ((a1 - a0) * fracy.x) >> 7;

  auto b0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.y]), V8<int16_t>);
  auto b1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.y]), V8<int16_t>);
  b0 += ((b1 - b0) * fracy.y) >> 7;

  auto abl = combine(lowHalf(a0), lowHalf(b0));
  auto abh = combine(highHalf(a0), highHalf(b0));
  abl += ((abh - abl) * fracx.xxxxyyyy) >> 7;

  auto c0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.z]), V8<int16_t>);
  auto c1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.z]), V8<int16_t>);
  c0 += ((c1 - c0) * fracy.z) >> 7;

  auto d0 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row0.w]), V8<int16_t>);
  auto d1 =
      CONVERT(unaligned_load<V8<uint8_t>>(&sampler->buf[row1.w]), V8<int16_t>);
  d0 += ((d1 - d0) * fracy.w) >> 7;

  auto cdl = combine(lowHalf(c0), lowHalf(d0));
  auto cdh = combine(highHalf(c0), highHalf(d0));
  cdl += ((cdh - cdl) * fracx.zzzzwwww) >> 7;

  return combine(HalfRGBA8(abl), HalfRGBA8(cdl));
}

template <typename S>
static PackedRGBA8 textureLinearPackedRGBA8(S sampler, ivec2 i) {
  return pack(textureLinearUnpackedRGBA8(sampler, i));
}

template <typename S>
static PackedRGBA8 textureNearestPackedRGBA8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::RGBA8);
  I32 row = computeRow(sampler, i, 0);
  return combine(unaligned_load<V4<uint8_t>>(&sampler->buf[row.x]),
                 unaligned_load<V4<uint8_t>>(&sampler->buf[row.y]),
                 unaligned_load<V4<uint8_t>>(&sampler->buf[row.z]),
                 unaligned_load<V4<uint8_t>>(&sampler->buf[row.w]));
}

template <typename S>
static PackedR8 textureLinearPackedR8(S sampler, ivec2 i) {
  return pack(textureLinearUnpackedR8(sampler, i));
}

template <typename S>
static WideRG8 textureLinearUnpackedRG8(S sampler, ivec2 i) {
  assert(sampler->format == TextureFormat::RG8);
  ivec2 frac = i & 0x7F;
  i >>= 7;

  I32 row0 = computeRow(sampler, i);
  I32 row1 = row0 + computeNextRowOffset(sampler, i);
  I16 fracx = computeFracX(sampler, i, frac);
  I16 fracy = computeFracY(frac);

  uint16_t* buf = (uint16_t*)sampler->buf;

  // Load RG bytes for two adjacent pixels - rgRG
  auto a0 = unaligned_load<V4<uint8_t>>(&buf[row0.x]);
  auto b0 = unaligned_load<V4<uint8_t>>(&buf[row0.y]);
  auto ab0 = CONVERT(combine(a0, b0), V8<int16_t>);
  // Load two pixels for next row
  auto a1 = unaligned_load<V4<uint8_t>>(&buf[row1.x]);
  auto b1 = unaligned_load<V4<uint8_t>>(&buf[row1.y]);
  auto ab1 = CONVERT(combine(a1, b1), V8<int16_t>);
  // Blend rows
  ab0 += ((ab1 - ab0) * fracy.xxxxyyyy) >> 7;

  auto c0 = unaligned_load<V4<uint8_t>>(&buf[row0.z]);
  auto d0 = unaligned_load<V4<uint8_t>>(&buf[row0.w]);
  auto cd0 = CONVERT(combine(c0, d0), V8<int16_t>);
  auto c1 = unaligned_load<V4<uint8_t>>(&buf[row1.z]);
  auto d1 = unaligned_load<V4<uint8_t>>(&buf[row1.w]);
  auto cd1 = CONVERT(combine(c1, d1), V8<int16_t>);
  // Blend rows
  cd0 += ((cd1 - cd0) * fracy.zzzzwwww) >> 7;

  // ab = a.rgRG,b.rgRG
  // cd = c.rgRG,d.rgRG
  // ... ac = a.rg,c.rg,a.RG,c.RG
  // ... bd = b.rg,d.rg,b.RG,d.RG
  auto ac = zip2Low(ab0, cd0);
  auto bd = zip2High(ab0, cd0);
  // a.rg,b.rg,c.rg,d.rg
  // a.RG,b.RG,c.RG,d.RG
  auto abcdl = zip2Low(ac, bd);
  auto abcdh = zip2High(ac, bd);
  // Blend columns
  abcdl += ((abcdh - abcdl) * fracx.xxyyzzww) >> 7;

  return WideRG8(abcdl);
}

template <typename S>
static PackedRG8 textureLinearPackedRG8(S sampler, ivec2 i) {
  return pack(textureLinearUnpackedRG8(sampler, i));
}

template <int N>
static ALWAYS_INLINE VectorType<uint16_t, N> addsat(VectorType<uint16_t, N> x,
                                                    VectorType<uint16_t, N> y) {
  auto r = x + y;
  return r | (r < x);
}

template <typename P, typename S>
static VectorType<uint16_t, 4 * sizeof(P)> gaussianBlurHorizontal(
    S sampler, const ivec2_scalar& i, int minX, int maxX, int radius,
    float coeff, float coeffStep) {
  // Packed and unpacked vectors for a chunk of the given pixel type.
  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;
  typedef VectorType<uint16_t, 4 * sizeof(P)> unpacked_type;

  // Pre-scale the coefficient by 8 bits of fractional precision, so that when
  // the sample is multiplied by it, it will yield a 16 bit unsigned integer
  // that will use all 16 bits of precision to accumulate the sum.
  coeff *= 1 << 8;
  float coeffStep2 = coeffStep * coeffStep;

  int row = computeRow(sampler, i);
  P* buf = (P*)sampler->buf;
  auto pixelsRight = unaligned_load<V4<P>>(&buf[row]);
  auto pixelsLeft = pixelsRight;
  auto sum = CONVERT(bit_cast<packed_type>(pixelsRight), unpacked_type) *
             uint16_t(coeff + 0.5f);

  // Here we use some trickery to reuse the pixels within a chunk, shifted over
  // by one pixel, to get the next sample for the entire chunk. This allows us
  // to sample only one pixel for each offset across the entire chunk in both
  // the left and right directions. To avoid clamping within the loop to the
  // texture bounds, we compute the valid radius that doesn't require clamping
  // and fall back to a slower clamping loop outside of that valid radius.
  int offset = 1;
  // The left bound is how much we can offset the sample before the start of
  // the row bounds.
  int leftBound = i.x - max(minX, 0);
  // The right bound is how much we can offset the sample before the end of the
  // row bounds.
  int rightBound = min(maxX, sampler->width - 1) - i.x;
  int validRadius = min(radius, min(leftBound, rightBound - (4 - 1)));
  for (; offset <= validRadius; offset++) {
    // Overwrite the pixel that needs to be shifted out with the new pixel, and
    // shift it into the correct location.
    pixelsRight.x = unaligned_load<P>(&buf[row + offset + 4 - 1]);
    pixelsRight = pixelsRight.yzwx;
    pixelsLeft = pixelsLeft.wxyz;
    pixelsLeft.x = unaligned_load<P>(&buf[row - offset]);

    // Accumulate the Gaussian coefficients step-wise.
    coeff *= coeffStep;
    coeffStep *= coeffStep2;

    // Both left and right samples at this offset use the same coefficient.
    sum = addsat(sum,
                 (CONVERT(bit_cast<packed_type>(pixelsRight), unpacked_type) +
                  CONVERT(bit_cast<packed_type>(pixelsLeft), unpacked_type)) *
                     uint16_t(coeff + 0.5f));
  }

  for (; offset <= radius; offset++) {
    pixelsRight.x =
        unaligned_load<P>(&buf[row + min(offset + 4 - 1, rightBound)]);
    pixelsRight = pixelsRight.yzwx;
    pixelsLeft = pixelsLeft.wxyz;
    pixelsLeft.x = unaligned_load<P>(&buf[row - min(offset, leftBound)]);

    coeff *= coeffStep;
    coeffStep *= coeffStep2;

    sum = addsat(sum,
                 (CONVERT(bit_cast<packed_type>(pixelsRight), unpacked_type) +
                  CONVERT(bit_cast<packed_type>(pixelsLeft), unpacked_type)) *
                     uint16_t(coeff + 0.5f));
  }

  // Shift away the intermediate precision.
  return sum >> 8;
}

template <typename P, typename S>
static VectorType<uint16_t, 4 * sizeof(P)> gaussianBlurVertical(
    S sampler, const ivec2_scalar& i, int minY, int maxY, int radius,
    float coeff, float coeffStep) {
  // Packed and unpacked vectors for a chunk of the given pixel type.
  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;
  typedef VectorType<uint16_t, 4 * sizeof(P)> unpacked_type;

  // Pre-scale the coefficient by 8 bits of fractional precision, so that when
  // the sample is multiplied by it, it will yield a 16 bit unsigned integer
  // that will use all 16 bits of precision to accumulate the sum.
  coeff *= 1 << 8;
  float coeffStep2 = coeffStep * coeffStep;

  int rowAbove = computeRow(sampler, i);
  int rowBelow = rowAbove;
  P* buf = (P*)sampler->buf;
  auto pixels = unaligned_load<V4<P>>(&buf[rowAbove]);
  auto sum = CONVERT(bit_cast<packed_type>(pixels), unpacked_type) *
             uint16_t(coeff + 0.5f);

  // For the vertical loop we can't be quite as creative with reusing old values
  // as we were in the horizontal loop. We just do the obvious implementation of
  // loading a chunk from each row in turn and accumulating it into the sum. We
  // compute a valid radius within which we don't need to clamp the sampled row
  // and use that to avoid any clamping in the main inner loop. We fall back to
  // a slower clamping loop outside of that valid radius.
  int offset = 1;
  int belowBound = i.y - max(minY, 0);
  int aboveBound = min(maxY, sampler->height - 1) - i.y;
  int validRadius = min(radius, min(belowBound, aboveBound));
  for (; offset <= validRadius; offset++) {
    rowAbove += sampler->stride;
    rowBelow -= sampler->stride;
    auto pixelsAbove = unaligned_load<V4<P>>(&buf[rowAbove]);
    auto pixelsBelow = unaligned_load<V4<P>>(&buf[rowBelow]);

    // Accumulate the Gaussian coefficients step-wise.
    coeff *= coeffStep;
    coeffStep *= coeffStep2;

    // Both above and below samples at this offset use the same coefficient.
    sum = addsat(sum,
                 (CONVERT(bit_cast<packed_type>(pixelsAbove), unpacked_type) +
                  CONVERT(bit_cast<packed_type>(pixelsBelow), unpacked_type)) *
                     uint16_t(coeff + 0.5f));
  }

  for (; offset <= radius; offset++) {
    if (offset <= aboveBound) {
      rowAbove += sampler->stride;
    }
    if (offset <= belowBound) {
      rowBelow -= sampler->stride;
    }
    auto pixelsAbove = unaligned_load<V4<P>>(&buf[rowAbove]);
    auto pixelsBelow = unaligned_load<V4<P>>(&buf[rowBelow]);

    coeff *= coeffStep;
    coeffStep *= coeffStep2;

    sum = addsat(sum,
                 (CONVERT(bit_cast<packed_type>(pixelsAbove), unpacked_type) +
                  CONVERT(bit_cast<packed_type>(pixelsBelow), unpacked_type)) *
                     uint16_t(coeff + 0.5f));
  }

  // Shift away the intermediate precision.
  return sum >> 8;
}

}  // namespace glsl
