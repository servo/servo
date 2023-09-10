/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

static ALWAYS_INLINE HalfRGBA8 packRGBA8(I32 a, I32 b) {
#if USE_SSE2
  return _mm_packs_epi32(a, b);
#elif USE_NEON
  return vcombine_u16(vqmovun_s32(a), vqmovun_s32(b));
#else
  return CONVERT(combine(a, b), HalfRGBA8);
#endif
}

static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8(const vec4& v,
                                                 float scale = 255.0f) {
  ivec4 i = round_pixel(v, scale);
  HalfRGBA8 xz = packRGBA8(i.z, i.x);
  HalfRGBA8 yw = packRGBA8(i.y, i.w);
  HalfRGBA8 xyzwl = zipLow(xz, yw);
  HalfRGBA8 xyzwh = zipHigh(xz, yw);
  HalfRGBA8 lo = zip2Low(xyzwl, xyzwh);
  HalfRGBA8 hi = zip2High(xyzwl, xyzwh);
  return combine(lo, hi);
}

static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8(Float alpha,
                                                 float scale = 255.0f) {
  I32 i = round_pixel(alpha, scale);
  HalfRGBA8 c = packRGBA8(i, i);
  c = zipLow(c, c);
  return zip(c, c);
}

static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8(float alpha,
                                                 float scale = 255.0f) {
  I32 i = round_pixel(alpha, scale);
  return repeat2(packRGBA8(i, i));
}

UNUSED static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8(const vec4_scalar& v,
                                                        float scale = 255.0f) {
  I32 i = round_pixel((Float){v.z, v.y, v.x, v.w}, scale);
  return repeat2(packRGBA8(i, i));
}

static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8() {
  return pack_pixels_RGBA8(fragment_shader->gl_FragColor);
}

static ALWAYS_INLINE WideRGBA8 pack_pixels_RGBA8(WideRGBA32F v,
                                                 float scale = 255.0f) {
  ivec4 i = round_pixel(bit_cast<vec4>(v), scale);
  return combine(packRGBA8(i.x, i.y), packRGBA8(i.z, i.w));
}

static ALWAYS_INLINE WideR8 packR8(I32 a) {
#if USE_SSE2
  return lowHalf(bit_cast<V8<uint16_t>>(_mm_packs_epi32(a, a)));
#elif USE_NEON
  return vqmovun_s32(a);
#else
  return CONVERT(a, WideR8);
#endif
}

static ALWAYS_INLINE WideR8 pack_pixels_R8(Float c, float scale = 255.0f) {
  return packR8(round_pixel(c, scale));
}

static ALWAYS_INLINE WideR8 pack_pixels_R8() {
  return pack_pixels_R8(fragment_shader->gl_FragColor.x);
}

// Load a partial span > 0 and < 4 pixels.
template <typename V, typename P>
static ALWAYS_INLINE V partial_load_span(const P* src, int span) {
  return bit_cast<V>(
      (span >= 2
           ? combine(unaligned_load<V2<P>>(src),
                     V2<P>{span > 2 ? unaligned_load<P>(src + 2) : P(0), 0})
           : V4<P>{unaligned_load<P>(src), 0, 0, 0}));
}

// Store a partial span > 0 and < 4 pixels.
template <typename V, typename P>
static ALWAYS_INLINE void partial_store_span(P* dst, V src, int span) {
  auto pixels = bit_cast<V4<P>>(src);
  if (span >= 2) {
    unaligned_store(dst, lowHalf(pixels));
    if (span > 2) {
      unaligned_store(dst + 2, pixels.z);
    }
  } else {
    unaligned_store(dst, pixels.x);
  }
}

// Dispatcher that chooses when to load a full or partial span
template <typename V, typename P>
static ALWAYS_INLINE V load_span(const P* src, int span) {
  if (span >= 4) {
    return unaligned_load<V, P>(src);
  } else {
    return partial_load_span<V, P>(src, span);
  }
}

// Dispatcher that chooses when to store a full or partial span
template <typename V, typename P>
static ALWAYS_INLINE void store_span(P* dst, V src, int span) {
  if (span >= 4) {
    unaligned_store<V, P>(dst, src);
  } else {
    partial_store_span<V, P>(dst, src, span);
  }
}

template <typename T>
static ALWAYS_INLINE T muldiv256(T x, T y) {
  return (x * y) >> 8;
}

// (x*y + x) >> 8, cheap approximation of (x*y) / 255
template <typename T>
static ALWAYS_INLINE T muldiv255(T x, T y) {
  return (x * y + x) >> 8;
}

template <typename V>
static ALWAYS_INLINE WideRGBA8 pack_span(uint32_t*, const V& v,
                                         float scale = 255.0f) {
  return pack_pixels_RGBA8(v, scale);
}

template <typename C>
static ALWAYS_INLINE WideR8 pack_span(uint8_t*, C c, float scale = 255.0f) {
  return pack_pixels_R8(c, scale);
}

// Helper functions to apply a color modulus when available.
struct NoColor {};

template <typename P>
static ALWAYS_INLINE P applyColor(P src, NoColor) {
  return src;
}

struct InvertColor {};

template <typename P>
static ALWAYS_INLINE P applyColor(P src, InvertColor) {
  return 255 - src;
}

template <typename P>
static ALWAYS_INLINE P applyColor(P src, P color) {
  return muldiv255(color, src);
}

static ALWAYS_INLINE WideRGBA8 applyColor(PackedRGBA8 src, WideRGBA8 color) {
  return applyColor(unpack(src), color);
}

template <typename P, typename C>
static ALWAYS_INLINE auto packColor(P* buf, C color) {
  return pack_span(buf, color, 255.0f);
}

template <typename P>
static ALWAYS_INLINE NoColor packColor(UNUSED P* buf, NoColor noColor) {
  return noColor;
}

template <typename P>
static ALWAYS_INLINE InvertColor packColor(UNUSED P* buf,
                                           InvertColor invertColor) {
  return invertColor;
}

// Single argument variation that takes an explicit destination buffer type.
template <typename P, typename C>
static ALWAYS_INLINE auto packColor(C color) {
  // Just pass in a typed null pointer, as the pack routines never use the
  // pointer's value, just its type.
  return packColor((P*)0, color);
}

// Byte-wise addition for when x or y is a signed 8-bit value stored in the
// low byte of a larger type T only with zeroed-out high bits, where T is
// greater than 8 bits, i.e. uint16_t. This can result when muldiv255 is used
// upon signed operands, using up all the precision in a 16 bit integer, and
// potentially losing the sign bit in the last >> 8 shift. Due to the
// properties of two's complement arithmetic, even though we've discarded the
// sign bit, we can still represent a negative number under addition (without
// requiring any extra sign bits), just that any negative number will behave
// like a large unsigned number under addition, generating a single carry bit
// on overflow that we need to discard. Thus, just doing a byte-wise add will
// overflow without the troublesome carry, giving us only the remaining 8 low
// bits we actually need while keeping the high bits at zero.
template <typename T>
static ALWAYS_INLINE T addlow(T x, T y) {
  typedef VectorType<uint8_t, sizeof(T)> bytes;
  return bit_cast<T>(bit_cast<bytes>(x) + bit_cast<bytes>(y));
}

// Replace color components of each pixel with the pixel's alpha values.
template <typename T>
static ALWAYS_INLINE T alphas(T c) {
  return SHUFFLE(c, c, 3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15);
}

// Replace the alpha values of the first vector with alpha values from the
// second, while leaving the color components unmodified.
template <typename T>
static ALWAYS_INLINE T set_alphas(T c, T a) {
  return SHUFFLE(c, a, 0, 1, 2, 19, 4, 5, 6, 23, 8, 9, 10, 27, 12, 13, 14, 31);
}

// Miscellaneous helper functions for working with packed RGBA8 data.
static ALWAYS_INLINE HalfRGBA8 if_then_else(V8<int16_t> c, HalfRGBA8 t,
                                            HalfRGBA8 e) {
  return bit_cast<HalfRGBA8>((c & t) | (~c & e));
}

template <typename T, typename C, int N>
static ALWAYS_INLINE VectorType<T, N> if_then_else(VectorType<C, N> c,
                                                   VectorType<T, N> t,
                                                   VectorType<T, N> e) {
  return combine(if_then_else(lowHalf(c), lowHalf(t), lowHalf(e)),
                 if_then_else(highHalf(c), highHalf(t), highHalf(e)));
}

static ALWAYS_INLINE HalfRGBA8 min(HalfRGBA8 x, HalfRGBA8 y) {
#if USE_SSE2
  return bit_cast<HalfRGBA8>(
      _mm_min_epi16(bit_cast<V8<int16_t>>(x), bit_cast<V8<int16_t>>(y)));
#elif USE_NEON
  return vminq_u16(x, y);
#else
  return if_then_else(x < y, x, y);
#endif
}

template <typename T, int N>
static ALWAYS_INLINE VectorType<T, N> min(VectorType<T, N> x,
                                          VectorType<T, N> y) {
  return combine(min(lowHalf(x), lowHalf(y)), min(highHalf(x), highHalf(y)));
}

static ALWAYS_INLINE HalfRGBA8 max(HalfRGBA8 x, HalfRGBA8 y) {
#if USE_SSE2
  return bit_cast<HalfRGBA8>(
      _mm_max_epi16(bit_cast<V8<int16_t>>(x), bit_cast<V8<int16_t>>(y)));
#elif USE_NEON
  return vmaxq_u16(x, y);
#else
  return if_then_else(x > y, x, y);
#endif
}

template <typename T, int N>
static ALWAYS_INLINE VectorType<T, N> max(VectorType<T, N> x,
                                          VectorType<T, N> y) {
  return combine(max(lowHalf(x), lowHalf(y)), max(highHalf(x), highHalf(y)));
}

template <typename T, int N>
static ALWAYS_INLINE VectorType<T, N> recip(VectorType<T, N> v) {
  return combine(recip(lowHalf(v)), recip(highHalf(v)));
}

// Helper to get the reciprocal if the value is non-zero, or otherwise default
// to the supplied fallback value.
template <typename V>
static ALWAYS_INLINE V recip_or(V v, float f) {
  return if_then_else(v != V(0.0f), recip(v), V(f));
}

template <typename T, int N>
static ALWAYS_INLINE VectorType<T, N> inversesqrt(VectorType<T, N> v) {
  return combine(inversesqrt(lowHalf(v)), inversesqrt(highHalf(v)));
}

// Extract the alpha components so that we can cheaply calculate the reciprocal
// on a single SIMD register. Then multiply the duplicated alpha reciprocal with
// the pixel data. 0 alpha is treated as transparent black.
static ALWAYS_INLINE WideRGBA32F unpremultiply(WideRGBA32F v) {
  Float a = recip_or((Float){v[3], v[7], v[11], v[15]}, 0.0f);
  return v * a.xxxxyyyyzzzzwwww;
}

// Packed RGBA32F data is AoS in BGRA order. Transpose it to SoA and swizzle to
// RGBA to unpack.
static ALWAYS_INLINE vec4 unpack(PackedRGBA32F c) {
  return bit_cast<vec4>(
      SHUFFLE(c, c, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12, 3, 7, 11, 15));
}

// The following lum/sat functions mostly follow the KHR_blend_equation_advanced
// specification but are rearranged to work on premultiplied data.
static ALWAYS_INLINE Float lumv3(vec3 v) {
  return v.x * 0.30f + v.y * 0.59f + v.z * 0.11f;
}

static ALWAYS_INLINE Float minv3(vec3 v) { return min(min(v.x, v.y), v.z); }

static ALWAYS_INLINE Float maxv3(vec3 v) { return max(max(v.x, v.y), v.z); }

static inline vec3 clip_color(vec3 v, Float lum, Float alpha) {
  Float mincol = max(-minv3(v), lum);
  Float maxcol = max(maxv3(v), alpha - lum);
  return lum + v * (lum * (alpha - lum) * recip_or(mincol * maxcol, 0.0f));
}

static inline vec3 set_lum(vec3 base, vec3 ref, Float alpha) {
  return clip_color(base - lumv3(base), lumv3(ref), alpha);
}

static inline vec3 set_lum_sat(vec3 base, vec3 sref, vec3 lref, Float alpha) {
  vec3 diff = base - minv3(base);
  Float sbase = maxv3(diff);
  Float ssat = maxv3(sref) - minv3(sref);
  // The sbase range is rescaled to ssat. If sbase has 0 extent, then rescale
  // to black, as per specification.
  return set_lum(diff * ssat * recip_or(sbase, 0.0f), lref, alpha);
}

// Flags the reflect the current blend-stage clipping to be applied.
enum SWGLClipFlag {
  SWGL_CLIP_FLAG_MASK = 1 << 0,
  SWGL_CLIP_FLAG_AA = 1 << 1,
  SWGL_CLIP_FLAG_BLEND_OVERRIDE = 1 << 2,
};
static int swgl_ClipFlags = 0;
static BlendKey swgl_BlendOverride = BLEND_KEY_NONE;
static WideRGBA8 swgl_BlendColorRGBA8 = {0};
static WideRGBA8 swgl_BlendAlphaRGBA8 = {0};

// A pointer into the color buffer for the start of the span.
static void* swgl_SpanBuf = nullptr;
// A pointer into the clip mask for the start of the span.
static uint8_t* swgl_ClipMaskBuf = nullptr;

static ALWAYS_INLINE WideR8 expand_mask(UNUSED uint8_t* buf, WideR8 mask) {
  return mask;
}
static ALWAYS_INLINE WideRGBA8 expand_mask(UNUSED uint32_t* buf, WideR8 mask) {
  WideRG8 maskRG = zip(mask, mask);
  return zip(maskRG, maskRG);
}

// Loads a chunk of clip masks. The current pointer into the color buffer is
// used to reconstruct the relative position within the span. From there, the
// pointer into the clip mask can be generated from the start of the clip mask
// span.
template <typename P>
static ALWAYS_INLINE uint8_t* get_clip_mask(P* buf) {
  return &swgl_ClipMaskBuf[buf - (P*)swgl_SpanBuf];
}

template <typename P>
static ALWAYS_INLINE auto load_clip_mask(P* buf, int span)
    -> decltype(expand_mask(buf, 0)) {
  return expand_mask(buf,
                     unpack(load_span<PackedR8>(get_clip_mask(buf), span)));
}

// Temporarily removes masking from the blend stage, assuming the caller will
// handle it.
static ALWAYS_INLINE void override_clip_mask() {
  blend_key = BlendKey(blend_key - MASK_BLEND_KEY_NONE);
}

// Restores masking to the blend stage, assuming it was previously overridden.
static ALWAYS_INLINE void restore_clip_mask() {
  blend_key = BlendKey(MASK_BLEND_KEY_NONE + blend_key);
}

// A pointer to the start of the opaque destination region of the span for AA.
static const uint8_t* swgl_OpaqueStart = nullptr;
// The size, in bytes, of the opaque region.
static uint32_t swgl_OpaqueSize = 0;
// AA coverage distance offsets for the left and right edges.
static Float swgl_LeftAADist = 0.0f;
static Float swgl_RightAADist = 0.0f;
// AA coverage slope values used for accumulating coverage for each step.
static Float swgl_AASlope = 0.0f;

// Get the amount of pixels we need to process before the start of the opaque
// region.
template <typename P>
static ALWAYS_INLINE int get_aa_opaque_start(P* buf) {
  return max(int((P*)swgl_OpaqueStart - buf), 0);
}

// Assuming we are already in the opaque part of the span, return the remaining
// size of the opaque part.
template <typename P>
static ALWAYS_INLINE int get_aa_opaque_size(P* buf) {
  return max(int((P*)&swgl_OpaqueStart[swgl_OpaqueSize] - buf), 0);
}

// Temporarily removes anti-aliasing from the blend stage, assuming the caller
// will handle it.
static ALWAYS_INLINE void override_aa() {
  blend_key = BlendKey(blend_key - AA_BLEND_KEY_NONE);
}

// Restores anti-aliasing to the blend stage, assuming it was previously
// overridden.
static ALWAYS_INLINE void restore_aa() {
  blend_key = BlendKey(AA_BLEND_KEY_NONE + blend_key);
}

static PREFER_INLINE WideRGBA8 blend_pixels(uint32_t* buf, PackedRGBA8 pdst,
                                            WideRGBA8 src, int span = 4) {
  WideRGBA8 dst = unpack(pdst);
  const WideRGBA8 RGB_MASK = {0xFFFF, 0xFFFF, 0xFFFF, 0,      0xFFFF, 0xFFFF,
                              0xFFFF, 0,      0xFFFF, 0xFFFF, 0xFFFF, 0,
                              0xFFFF, 0xFFFF, 0xFFFF, 0};
  const WideRGBA8 ALPHA_MASK = {0, 0, 0, 0xFFFF, 0, 0, 0, 0xFFFF,
                                0, 0, 0, 0xFFFF, 0, 0, 0, 0xFFFF};
  const WideRGBA8 ALPHA_OPAQUE = {0, 0, 0, 255, 0, 0, 0, 255,
                                  0, 0, 0, 255, 0, 0, 0, 255};

// clang-format off
  // Computes AA for the given pixel based on the offset of the pixel within
  // destination row. Given the initial coverage offsets for the left and right
  // edges, the offset is scaled by the slope and accumulated to find the
  // minimum coverage value for the pixel. A final weight is generated that
  // can be used to scale the source pixel.
#define DO_AA(format, body)                                   \
  do {                                                        \
    int offset = int((const uint8_t*)buf - swgl_OpaqueStart); \
    if (uint32_t(offset) >= swgl_OpaqueSize) {                \
      Float delta = swgl_AASlope * float(offset);             \
      Float dist = clamp(min(swgl_LeftAADist + delta.x,       \
                             swgl_RightAADist + delta.y),     \
                         0.0f, 256.0f);                       \
      auto aa = pack_pixels_##format(dist, 1.0f);             \
      body;                                                   \
    }                                                         \
  } while (0)

  // Each blend case is preceded by the MASK_ variant. The MASK_ case first
  // loads the mask values and multiplies the source value by them. After, it
  // falls through to the normal blending case using the masked source. The
  // AA_ variations may further precede the blend cases, in which case the
  // source value is further modified before use.
#define BLEND_CASE_KEY(key)                          \
  case AA_##key:                                     \
    DO_AA(RGBA8, src = muldiv256(src, aa));          \
    goto key;                                        \
  case AA_MASK_##key:                                \
    DO_AA(RGBA8, src = muldiv256(src, aa));          \
    FALLTHROUGH;                                     \
  case MASK_##key:                                   \
    src = muldiv255(src, load_clip_mask(buf, span)); \
    FALLTHROUGH;                                     \
  case key: key

#define BLEND_CASE(...) BLEND_CASE_KEY(BLEND_KEY(__VA_ARGS__))

  switch (blend_key) {
  BLEND_CASE(GL_ONE, GL_ZERO):
    return src;
  BLEND_CASE(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA, GL_ONE,
                  GL_ONE_MINUS_SRC_ALPHA):
    // dst + src.a*(src.rgb1 - dst)
    // use addlow for signed overflow
    return addlow(dst, muldiv255(alphas(src), (src | ALPHA_OPAQUE) - dst));
  BLEND_CASE(GL_ONE, GL_ONE_MINUS_SRC_ALPHA):
    return src + dst - muldiv255(dst, alphas(src));
  BLEND_CASE(GL_ZERO, GL_ONE_MINUS_SRC_COLOR):
    return dst - muldiv255(dst, src);
  BLEND_CASE(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, GL_ZERO, GL_ONE):
    return dst - (muldiv255(dst, src) & RGB_MASK);
  BLEND_CASE(GL_ZERO, GL_ONE_MINUS_SRC_ALPHA):
    return dst - muldiv255(dst, alphas(src));
  BLEND_CASE(GL_ZERO, GL_SRC_COLOR):
    return muldiv255(src, dst);
  BLEND_CASE(GL_ONE, GL_ONE):
    return src + dst;
  BLEND_CASE(GL_ONE, GL_ONE, GL_ONE, GL_ONE_MINUS_SRC_ALPHA):
    return src + dst - (muldiv255(dst, src) & ALPHA_MASK);
  BLEND_CASE(GL_ONE_MINUS_DST_ALPHA, GL_ONE, GL_ZERO, GL_ONE):
    // src*(1-dst.a) + dst*1 = src - src*dst.a + dst
    return dst + ((src - muldiv255(src, alphas(dst))) & RGB_MASK);
  BLEND_CASE(GL_CONSTANT_COLOR, GL_ONE_MINUS_SRC_COLOR):
    // src*k + (1-src)*dst = src*k + dst -
    // src*dst = dst + src*(k - dst) use addlow
    // for signed overflow
    return addlow(
        dst, muldiv255(src, repeat2(ctx->blendcolor) - dst));

  // We must explicitly handle the masked/anti-aliased secondary blend case.
  // The secondary color as well as the source must be multiplied by the
  // weights.
  case BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC1_COLOR): {
    WideRGBA8 secondary =
        applyColor(dst,
            packColor<uint32_t>(fragment_shader->gl_SecondaryFragColor));
    return src + dst - secondary;
  }
  case MASK_BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC1_COLOR): {
    WideRGBA8 secondary =
        applyColor(dst,
            packColor<uint32_t>(fragment_shader->gl_SecondaryFragColor));
    WideRGBA8 mask = load_clip_mask(buf, span);
    return muldiv255(src, mask) + dst - muldiv255(secondary, mask);
  }
  case AA_BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC1_COLOR): {
    WideRGBA8 secondary =
        applyColor(dst,
            packColor<uint32_t>(fragment_shader->gl_SecondaryFragColor));
    DO_AA(RGBA8, {
      src = muldiv256(src, aa);
      secondary = muldiv256(secondary, aa);
    });
    return src + dst - secondary;
  }
  case AA_MASK_BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC1_COLOR): {
    WideRGBA8 secondary =
        applyColor(dst,
            packColor<uint32_t>(fragment_shader->gl_SecondaryFragColor));
    WideRGBA8 mask = load_clip_mask(buf, span);
    DO_AA(RGBA8, mask = muldiv256(mask, aa));
    return muldiv255(src, mask) + dst - muldiv255(secondary, mask);
  }

  BLEND_CASE(GL_MIN):
    return min(src, dst);
  BLEND_CASE(GL_MAX):
    return max(src, dst);

  // The KHR_blend_equation_advanced spec describes the blend equations such
  // that the unpremultiplied values Cs, Cd, As, Ad and function f combine to
  // the result:
  //     Cr = f(Cs,Cd)*As*Ad + Cs*As*(1-Ad) + Cd*AD*(1-As)
  //     Ar = As*Ad + As*(1-Ad) + Ad*(1-As)
  // However, working with unpremultiplied values requires expensive math to
  // unpremultiply and premultiply again during blending. We can use the fact
  // that premultiplied value P = C*A and simplify the equations such that no
  // unpremultiplied colors are necessary, allowing us to stay with integer
  // math that avoids floating-point conversions in the common case. Some of
  // the blend modes require division or sqrt, in which case we do convert
  // to (possibly transposed/unpacked) floating-point to implement the mode.
  // However, most common modes can still use cheaper premultiplied integer
  // math. As an example, the multiply mode f(Cs,Cd) = Cs*Cd is simplified
  // to:
  //     Cr = Cs*Cd*As*Ad + Cs*As*(1-Ad) + Cd*Ad*(1-As)
  //     .. Pr = Ps*Pd + Ps - Ps*Ad + Pd - Pd*As
  //     Ar = As*Ad + As - As*Ad + Ad - Ad*As
  //     .. Ar = As + Ad - As*Ad
  // Note that the alpha equation is the same for all blend equations, such
  // that so long as the implementation results in As + Ad - As*Ad, we can
  // avoid using separate instructions to compute the alpha result, which is
  // dependent on the math used to implement each blend mode. The exact
  // reductions used to get the final math for every blend mode are too
  // involved to show here in comments, but mostly follows from replacing
  // Cs*As and Cd*Ad with Ps and Ps while factoring out as many common terms
  // as possible.

  BLEND_CASE(GL_MULTIPLY_KHR): {
    WideRGBA8 diff = muldiv255(alphas(src) - (src & RGB_MASK),
                               alphas(dst) - (dst & RGB_MASK));
    return src + dst + (diff & RGB_MASK) - alphas(diff);
  }
  BLEND_CASE(GL_SCREEN_KHR):
    return src + dst - muldiv255(src, dst);
  BLEND_CASE(GL_OVERLAY_KHR): {
    WideRGBA8 srcA = alphas(src);
    WideRGBA8 dstA = alphas(dst);
    WideRGBA8 diff = muldiv255(src, dst) + muldiv255(srcA - src, dstA - dst);
    return src + dst +
           if_then_else(dst * 2 <= dstA, (diff & RGB_MASK) - alphas(diff),
                        -diff);
  }
  BLEND_CASE(GL_DARKEN_KHR):
    return src + dst -
           max(muldiv255(src, alphas(dst)), muldiv255(dst, alphas(src)));
  BLEND_CASE(GL_LIGHTEN_KHR):
    return src + dst -
           min(muldiv255(src, alphas(dst)), muldiv255(dst, alphas(src)));

  BLEND_CASE(GL_COLORDODGE_KHR): {
    // Color-dodge and color-burn require division, so we convert to FP math
    // here, but avoid transposing to a vec4.
    WideRGBA32F srcF = CONVERT(src, WideRGBA32F);
    WideRGBA32F srcA = alphas(srcF);
    WideRGBA32F dstF = CONVERT(dst, WideRGBA32F);
    WideRGBA32F dstA = alphas(dstF);
    return pack_pixels_RGBA8(
        srcA * set_alphas(
                   min(dstA, dstF * srcA * recip_or(srcA - srcF, 255.0f)),
                   dstF) +
            srcF * (255.0f - dstA) + dstF * (255.0f - srcA),
        1.0f / 255.0f);
  }
  BLEND_CASE(GL_COLORBURN_KHR): {
    WideRGBA32F srcF = CONVERT(src, WideRGBA32F);
    WideRGBA32F srcA = alphas(srcF);
    WideRGBA32F dstF = CONVERT(dst, WideRGBA32F);
    WideRGBA32F dstA = alphas(dstF);
    return pack_pixels_RGBA8(
        srcA * set_alphas((dstA - min(dstA, (dstA - dstF) * srcA *
                                                recip_or(srcF, 255.0f))),
                          dstF) +
            srcF * (255.0f - dstA) + dstF * (255.0f - srcA),
        1.0f / 255.0f);
  }
  BLEND_CASE(GL_HARDLIGHT_KHR): {
    WideRGBA8 srcA = alphas(src);
    WideRGBA8 dstA = alphas(dst);
    WideRGBA8 diff = muldiv255(src, dst) + muldiv255(srcA - src, dstA - dst);
    return src + dst +
           if_then_else(src * 2 <= srcA, (diff & RGB_MASK) - alphas(diff),
                        -diff);
  }

  BLEND_CASE(GL_SOFTLIGHT_KHR): {
    // Soft-light requires an unpremultiply that can't be factored out as
    // well as a sqrt, so we convert to FP math here, but avoid transposing
    // to a vec4.
    WideRGBA32F srcF = CONVERT(src, WideRGBA32F);
    WideRGBA32F srcA = alphas(srcF);
    WideRGBA32F dstF = CONVERT(dst, WideRGBA32F);
    WideRGBA32F dstA = alphas(dstF);
    WideRGBA32F dstU = unpremultiply(dstF);
    WideRGBA32F scale = srcF + srcF - srcA;
    return pack_pixels_RGBA8(
        dstF * (255.0f +
                set_alphas(
                    scale *
                        if_then_else(scale < 0.0f, 1.0f - dstU,
                                     min((16.0f * dstU - 12.0f) * dstU + 3.0f,
                                         inversesqrt(dstU) - 1.0f)),
                    WideRGBA32F(0.0f))) +
            srcF * (255.0f - dstA),
        1.0f / 255.0f);
  }
  BLEND_CASE(GL_DIFFERENCE_KHR): {
    WideRGBA8 diff =
        min(muldiv255(dst, alphas(src)), muldiv255(src, alphas(dst)));
    return src + dst - diff - (diff & RGB_MASK);
  }
  BLEND_CASE(GL_EXCLUSION_KHR): {
    WideRGBA8 diff = muldiv255(src, dst);
    return src + dst - diff - (diff & RGB_MASK);
  }

  // The HSL blend modes are non-separable and require complicated use of
  // division. It is advantageous to convert to FP and transpose to vec4
  // math to more easily manipulate the individual color components.
#define DO_HSL(rgb)                                                            \
  do {                                                                         \
    vec4 srcV = unpack(CONVERT(src, PackedRGBA32F));                           \
    vec4 dstV = unpack(CONVERT(dst, PackedRGBA32F));                           \
    Float srcA = srcV.w * (1.0f / 255.0f);                                     \
    Float dstA = dstV.w * (1.0f / 255.0f);                                     \
    Float srcDstA = srcV.w * dstA;                                             \
    vec3 srcC = vec3(srcV) * dstA;                                             \
    vec3 dstC = vec3(dstV) * srcA;                                             \
    return pack_pixels_RGBA8(vec4(rgb + vec3(srcV) - srcC + vec3(dstV) - dstC, \
                                  srcV.w + dstV.w - srcDstA),                  \
                             1.0f);                                            \
  } while (0)

  BLEND_CASE(GL_HSL_HUE_KHR):
    DO_HSL(set_lum_sat(srcC, dstC, dstC, srcDstA));
  BLEND_CASE(GL_HSL_SATURATION_KHR):
    DO_HSL(set_lum_sat(dstC, srcC, dstC, srcDstA));
  BLEND_CASE(GL_HSL_COLOR_KHR):
    DO_HSL(set_lum(srcC, dstC, srcDstA));
  BLEND_CASE(GL_HSL_LUMINOSITY_KHR):
    DO_HSL(set_lum(dstC, srcC, srcDstA));

  // SWGL-specific extended blend modes.
  BLEND_CASE(SWGL_BLEND_DROP_SHADOW): {
    // Premultiplied alpha over blend, but with source color set to source alpha
    // modulated with a constant color.
    WideRGBA8 color = applyColor(alphas(src), swgl_BlendColorRGBA8);
    return color + dst - muldiv255(dst, alphas(color));
  }

  BLEND_CASE(SWGL_BLEND_SUBPIXEL_TEXT):
    // Premultiplied alpha over blend, but treats the source as a subpixel mask
    // modulated with a constant color.
    return applyColor(src, swgl_BlendColorRGBA8) + dst -
           muldiv255(dst, applyColor(src, swgl_BlendAlphaRGBA8));

  default:
    UNREACHABLE;
    // return src;
  }

#undef BLEND_CASE
#undef BLEND_CASE_KEY
  // clang-format on
}

static PREFER_INLINE WideR8 blend_pixels(uint8_t* buf, WideR8 dst, WideR8 src,
                                         int span = 4) {
// clang-format off
#define BLEND_CASE_KEY(key)                          \
  case AA_##key:                                     \
    DO_AA(R8, src = muldiv256(src, aa));             \
    goto key;                                        \
  case AA_MASK_##key:                                \
    DO_AA(R8, src = muldiv256(src, aa));             \
    FALLTHROUGH;                                     \
  case MASK_##key:                                   \
    src = muldiv255(src, load_clip_mask(buf, span)); \
    FALLTHROUGH;                                     \
  case key: key

#define BLEND_CASE(...) BLEND_CASE_KEY(BLEND_KEY(__VA_ARGS__))

  switch (blend_key) {
  BLEND_CASE(GL_ONE, GL_ZERO):
    return src;
  BLEND_CASE(GL_ZERO, GL_SRC_COLOR):
    return muldiv255(src, dst);
  BLEND_CASE(GL_ONE, GL_ONE):
    return src + dst;
  default:
    UNREACHABLE;
    // return src;
  }

#undef BLEND_CASE
#undef BLEND_CASE_KEY
  // clang-format on
}

static ALWAYS_INLINE void commit_span(uint32_t* buf, WideRGBA8 r) {
  unaligned_store(buf, pack(r));
}

static ALWAYS_INLINE void commit_span(uint32_t* buf, WideRGBA8 r, int len) {
  partial_store_span(buf, pack(r), len);
}

static ALWAYS_INLINE WideRGBA8 blend_span(uint32_t* buf, WideRGBA8 r) {
  return blend_pixels(buf, unaligned_load<PackedRGBA8>(buf), r);
}

static ALWAYS_INLINE WideRGBA8 blend_span(uint32_t* buf, WideRGBA8 r, int len) {
  return blend_pixels(buf, partial_load_span<PackedRGBA8>(buf, len), r, len);
}

static ALWAYS_INLINE void commit_span(uint32_t* buf, PackedRGBA8 r) {
  unaligned_store(buf, r);
}

static ALWAYS_INLINE void commit_span(uint32_t* buf, PackedRGBA8 r, int len) {
  partial_store_span(buf, r, len);
}

static ALWAYS_INLINE PackedRGBA8 blend_span(uint32_t* buf, PackedRGBA8 r) {
  return pack(blend_span(buf, unpack(r)));
}

static ALWAYS_INLINE PackedRGBA8 blend_span(uint32_t* buf, PackedRGBA8 r,
                                            int len) {
  return pack(blend_span(buf, unpack(r), len));
}

static ALWAYS_INLINE void commit_span(uint8_t* buf, WideR8 r) {
  unaligned_store(buf, pack(r));
}

static ALWAYS_INLINE void commit_span(uint8_t* buf, WideR8 r, int len) {
  partial_store_span(buf, pack(r), len);
}

static ALWAYS_INLINE WideR8 blend_span(uint8_t* buf, WideR8 r) {
  return blend_pixels(buf, unpack(unaligned_load<PackedR8>(buf)), r);
}

static ALWAYS_INLINE WideR8 blend_span(uint8_t* buf, WideR8 r, int len) {
  return blend_pixels(buf, unpack(partial_load_span<PackedR8>(buf, len)), r,
                      len);
}

static ALWAYS_INLINE void commit_span(uint8_t* buf, PackedR8 r) {
  unaligned_store(buf, r);
}

static ALWAYS_INLINE void commit_span(uint8_t* buf, PackedR8 r, int len) {
  partial_store_span(buf, r, len);
}

static ALWAYS_INLINE PackedR8 blend_span(uint8_t* buf, PackedR8 r) {
  return pack(blend_span(buf, unpack(r)));
}

static ALWAYS_INLINE PackedR8 blend_span(uint8_t* buf, PackedR8 r, int len) {
  return pack(blend_span(buf, unpack(r), len));
}

template <bool BLEND, typename P, typename R>
static ALWAYS_INLINE void commit_blend_span(P* buf, R r) {
  if (BLEND) {
    commit_span(buf, blend_span(buf, r));
  } else {
    commit_span(buf, r);
  }
}

template <bool BLEND, typename P, typename R>
static ALWAYS_INLINE void commit_blend_span(P* buf, R r, int len) {
  if (BLEND) {
    commit_span(buf, blend_span(buf, r, len), len);
  } else {
    commit_span(buf, r, len);
  }
}

template <typename P, typename R>
static ALWAYS_INLINE void commit_blend_solid_span(P* buf, R r, int len) {
  for (P* end = &buf[len & ~3]; buf < end; buf += 4) {
    commit_span(buf, blend_span(buf, r));
  }
  len &= 3;
  if (len > 0) {
    partial_store_span(buf, pack(blend_span(buf, r, len)), len);
  }
}

template <bool BLEND>
static void commit_solid_span(uint32_t* buf, WideRGBA8 r, int len) {
  commit_blend_solid_span(buf, r, len);
}

template <>
ALWAYS_INLINE void commit_solid_span<false>(uint32_t* buf, WideRGBA8 r,
                                            int len) {
  fill_n(buf, len, bit_cast<U32>(pack(r)).x);
}

template <bool BLEND>
static void commit_solid_span(uint8_t* buf, WideR8 r, int len) {
  commit_blend_solid_span(buf, r, len);
}

template <>
ALWAYS_INLINE void commit_solid_span<false>(uint8_t* buf, WideR8 r, int len) {
  PackedR8 p = pack(r);
  if (uintptr_t(buf) & 3) {
    int align = 4 - (uintptr_t(buf) & 3);
    align = min(align, len);
    partial_store_span(buf, p, align);
    buf += align;
    len -= align;
  }
  fill_n((uint32_t*)buf, len / 4, bit_cast<uint32_t>(p));
  buf += len & ~3;
  len &= 3;
  if (len > 0) {
    partial_store_span(buf, p, len);
  }
}
