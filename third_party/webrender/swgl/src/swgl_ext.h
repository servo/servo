/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// When using a solid color with clip masking, the cost of loading the clip mask
// in the blend stage exceeds the cost of processing the color. Here we handle
// the entire span of clip mask texture before the blend stage to more
// efficiently process it and modulate it with color without incurring blend
// stage overheads.
template <typename P, typename C>
static void commit_masked_solid_span(P* buf, C color, int len) {
  override_clip_mask();
  uint8_t* mask = get_clip_mask(buf);
  for (P* end = &buf[len]; buf < end; buf += 4, mask += 4) {
    commit_span(
        buf,
        blend_span(
            buf,
            applyColor(expand_mask(buf, unpack(unaligned_load<PackedR8>(mask))),
                       color)));
  }
  restore_clip_mask();
}

// When using a solid color with anti-aliasing, most of the solid span will not
// benefit from anti-aliasing in the opaque region. We only want to apply the AA
// blend stage in the non-opaque start and end of the span where AA is needed.
template <typename P, typename R>
static ALWAYS_INLINE void commit_aa_solid_span(P* buf, R r, int len) {
  if (int start = min((get_aa_opaque_start(buf) + 3) & ~3, len)) {
    commit_solid_span<true>(buf, r, start);
    buf += start;
    len -= start;
  }
  if (int opaque = min((get_aa_opaque_size(buf) + 3) & ~3, len)) {
    override_aa();
    commit_solid_span<true>(buf, r, opaque);
    restore_aa();
    buf += opaque;
    len -= opaque;
  }
  if (len > 0) {
    commit_solid_span<true>(buf, r, len);
  }
}

// Forces a value with vector run-class to have scalar run-class.
template <typename T>
static ALWAYS_INLINE auto swgl_forceScalar(T v) -> decltype(force_scalar(v)) {
  return force_scalar(v);
}

// Advance all varying inperpolants by a single chunk
#define swgl_stepInterp() step_interp_inputs()

// Pseudo-intrinsic that accesses the interpolation step for a given varying
#define swgl_interpStep(v) (interp_step.v)

// Commit an entire span of a solid color. This dispatches to clip-masked and
// anti-aliased fast-paths as appropriate.
#define swgl_commitSolid(format, v, n)                                   \
  do {                                                                   \
    int len = (n);                                                       \
    if (blend_key) {                                                     \
      if (swgl_ClipFlags & SWGL_CLIP_FLAG_MASK) {                        \
        commit_masked_solid_span(swgl_Out##format,                       \
                                 packColor(swgl_Out##format, (v)), len); \
      } else if (swgl_ClipFlags & SWGL_CLIP_FLAG_AA) {                   \
        commit_aa_solid_span(swgl_Out##format,                           \
                             pack_span(swgl_Out##format, (v)), len);     \
      } else {                                                           \
        commit_solid_span<true>(swgl_Out##format,                        \
                                pack_span(swgl_Out##format, (v)), len);  \
      }                                                                  \
    } else {                                                             \
      commit_solid_span<false>(swgl_Out##format,                         \
                               pack_span(swgl_Out##format, (v)), len);   \
    }                                                                    \
    swgl_Out##format += len;                                             \
    swgl_SpanLength -= len;                                              \
  } while (0)
#define swgl_commitSolidRGBA8(v) swgl_commitSolid(RGBA8, v, swgl_SpanLength)
#define swgl_commitSolidR8(v) swgl_commitSolid(R8, v, swgl_SpanLength)
#define swgl_commitPartialSolidRGBA8(len, v) \
  swgl_commitSolid(RGBA8, v, min(int(len), swgl_SpanLength))
#define swgl_commitPartialSolidR8(len, v) \
  swgl_commitSolid(R8, v, min(int(len), swgl_SpanLength))

#define swgl_commitChunk(format, chunk)                 \
  do {                                                  \
    auto r = chunk;                                     \
    if (blend_key) r = blend_span(swgl_Out##format, r); \
    commit_span(swgl_Out##format, r);                   \
    swgl_Out##format += swgl_StepSize;                  \
    swgl_SpanLength -= swgl_StepSize;                   \
  } while (0)

// Commit a single chunk of a color
#define swgl_commitColor(format, color) \
  swgl_commitChunk(format, pack_pixels_##format(color))
#define swgl_commitColorRGBA8(color) swgl_commitColor(RGBA8, color)
#define swgl_commitColorR8(color) swgl_commitColor(R8, color)

template <typename S>
static ALWAYS_INLINE bool swgl_isTextureLinear(S s) {
  return s->filter == TextureFilter::LINEAR;
}

template <typename S>
static ALWAYS_INLINE bool swgl_isTextureRGBA8(S s) {
  return s->format == TextureFormat::RGBA8;
}

template <typename S>
static ALWAYS_INLINE bool swgl_isTextureR8(S s) {
  return s->format == TextureFormat::R8;
}

// Use the default linear quantization scale of 128. This gives 7 bits of
// fractional precision, which when multiplied with a signed 9 bit value
// still fits in a 16 bit integer.
const int swgl_LinearQuantizeScale = 128;

// Quantizes UVs for access into a linear texture.
template <typename S, typename T>
static ALWAYS_INLINE T swgl_linearQuantize(S s, T p) {
  return linearQuantize(p, swgl_LinearQuantizeScale, s);
}

// Quantizes an interpolation step for UVs for access into a linear texture.
template <typename S, typename T>
static ALWAYS_INLINE T swgl_linearQuantizeStep(S s, T p) {
  return samplerScale(s, p) * swgl_LinearQuantizeScale;
}

template <typename S>
static ALWAYS_INLINE WideRGBA8 textureLinearUnpacked(UNUSED uint32_t* buf,
                                                     S sampler, ivec2 i) {
  return textureLinearUnpackedRGBA8(sampler, i);
}

template <typename S>
static ALWAYS_INLINE WideR8 textureLinearUnpacked(UNUSED uint8_t* buf,
                                                  S sampler, ivec2 i) {
  return textureLinearUnpackedR8(sampler, i);
}

template <typename S>
static ALWAYS_INLINE bool matchTextureFormat(S s, UNUSED uint32_t* buf) {
  return swgl_isTextureRGBA8(s);
}

template <typename S>
static ALWAYS_INLINE bool matchTextureFormat(S s, UNUSED uint8_t* buf) {
  return swgl_isTextureR8(s);
}

// Quantizes the UVs to the 2^7 scale needed for calculating fractional offsets
// for linear sampling.
#define LINEAR_QUANTIZE_UV(sampler, uv, uv_step, uv_rect, min_uv, max_uv)     \
  uv = swgl_linearQuantize(sampler, uv);                                      \
  vec2_scalar uv_step =                                                       \
      float(swgl_StepSize) * vec2_scalar{uv.x.y - uv.x.x, uv.y.y - uv.y.x};   \
  vec2_scalar min_uv = max(                                                   \
      swgl_linearQuantize(sampler, vec2_scalar{uv_rect.x, uv_rect.y}), 0.0f); \
  vec2_scalar max_uv =                                                        \
      max(swgl_linearQuantize(sampler, vec2_scalar{uv_rect.z, uv_rect.w}),    \
          min_uv);

// Implements the fallback linear filter that can deal with clamping and
// arbitrary scales.
template <bool BLEND, typename S, typename C, typename P>
static P* blendTextureLinearFallback(S sampler, vec2 uv, int span,
                                     vec2_scalar uv_step, vec2_scalar min_uv,
                                     vec2_scalar max_uv, C color, P* buf) {
  for (P* end = buf + span; buf < end; buf += swgl_StepSize, uv += uv_step) {
    commit_blend_span<BLEND>(
        buf, applyColor(textureLinearUnpacked(buf, sampler,
                                              ivec2(clamp(uv, min_uv, max_uv))),
                        color));
  }
  return buf;
}

static ALWAYS_INLINE U64 castForShuffle(V16<int16_t> r) {
  return bit_cast<U64>(r);
}
static ALWAYS_INLINE U16 castForShuffle(V4<int16_t> r) {
  return bit_cast<U16>(r);
}

static ALWAYS_INLINE V16<int16_t> applyFracX(V16<int16_t> r, I16 fracx) {
  return r * fracx.xxxxyyyyzzzzwwww;
}
static ALWAYS_INLINE V4<int16_t> applyFracX(V4<int16_t> r, I16 fracx) {
  return r * fracx;
}

// Implements a faster linear filter that works with axis-aligned constant Y but
// scales less than 1, i.e. upscaling. In this case we can optimize for the
// constant Y fraction as well as load all chunks from memory in a single tap
// for each row.
template <bool BLEND, typename S, typename C, typename P>
static void blendTextureLinearUpscale(S sampler, vec2 uv, int span,
                                      vec2_scalar uv_step, vec2_scalar min_uv,
                                      vec2_scalar max_uv, C color, P* buf) {
  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;
  typedef VectorType<uint16_t, 4 * sizeof(P)> unpacked_type;
  typedef VectorType<int16_t, 4 * sizeof(P)> signed_unpacked_type;

  ivec2 i(clamp(uv, min_uv, max_uv));
  ivec2 frac = i;
  i >>= 7;
  P* row0 = (P*)sampler->buf + computeRow(sampler, ivec2_scalar(0, i.y.x));
  P* row1 = row0 + computeNextRowOffset(sampler, ivec2_scalar(0, i.y.x));
  I16 fracx = computeFracX(sampler, i, frac);
  int16_t fracy = computeFracY(frac).x;
  auto src0 =
      CONVERT(unaligned_load<packed_type>(&row0[i.x.x]), signed_unpacked_type);
  auto src1 =
      CONVERT(unaligned_load<packed_type>(&row1[i.x.x]), signed_unpacked_type);
  auto src = castForShuffle(src0 + (((src1 - src0) * fracy) >> 7));

  // We attempt to sample ahead by one chunk and interpolate it with the current
  // one. However, due to the complication of upscaling, we may not necessarily
  // shift in all the next set of samples.
  for (P* end = buf + span; buf < end; buf += 4) {
    uv.x += uv_step.x;
    I32 ixn = cast(uv.x);
    I16 fracn = computeFracNoClamp(ixn);
    ixn >>= 7;
    auto src0n = CONVERT(unaligned_load<packed_type>(&row0[ixn.x]),
                         signed_unpacked_type);
    auto src1n = CONVERT(unaligned_load<packed_type>(&row1[ixn.x]),
                         signed_unpacked_type);
    auto srcn = castForShuffle(src0n + (((src1n - src0n) * fracy) >> 7));

    // Since we're upscaling, we know that a source pixel has a larger footprint
    // than the destination pixel, and thus all the source pixels needed for
    // this chunk will fall within a single chunk of texture data. However,
    // since the source pixels don't map 1:1 with destination pixels, we need to
    // shift the source pixels over based on their offset from the start of the
    // chunk. This could conceivably be optimized better with usage of PSHUFB or
    // VTBL instructions However, since PSHUFB requires SSSE3, instead we resort
    // to masking in the correct pixels to avoid having to index into memory.
    // For the last sample to interpolate with, we need to potentially shift in
    // a sample from the next chunk over in the case the samples fill out an
    // entire chunk.
    auto shuf = src;
    auto shufn = SHUFFLE(src, ixn.x == i.x.w ? srcn.yyyy : srcn, 1, 2, 3, 4);
    if (i.x.y == i.x.x) {
      shuf = shuf.xxyz;
      shufn = shufn.xxyz;
    }
    if (i.x.z == i.x.y) {
      shuf = shuf.xyyz;
      shufn = shufn.xyyz;
    }
    if (i.x.w == i.x.z) {
      shuf = shuf.xyzz;
      shufn = shufn.xyzz;
    }

    // Convert back to a signed unpacked type so that we can interpolate the
    // final result.
    auto interp = bit_cast<signed_unpacked_type>(shuf);
    auto interpn = bit_cast<signed_unpacked_type>(shufn);
    interp += applyFracX(interpn - interp, fracx) >> 7;

    commit_blend_span<BLEND>(
        buf, applyColor(bit_cast<unpacked_type>(interp), color));

    i.x = ixn;
    fracx = fracn;
    src = srcn;
  }
}

// This is the fastest variant of the linear filter that still provides
// filtering. In cases where there is no scaling required, but we have a
// subpixel offset that forces us to blend in neighboring pixels, we can
// optimize away most of the memory loads and shuffling that is required by the
// fallback filter.
template <bool BLEND, typename S, typename C, typename P>
static void blendTextureLinearFast(S sampler, vec2 uv, int span,
                                   vec2_scalar min_uv, vec2_scalar max_uv,
                                   C color, P* buf) {
  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;
  typedef VectorType<uint16_t, 4 * sizeof(P)> unpacked_type;
  typedef VectorType<int16_t, 4 * sizeof(P)> signed_unpacked_type;

  ivec2 i(clamp(uv, min_uv, max_uv));
  ivec2 frac = i;
  i >>= 7;
  P* row0 = (P*)sampler->buf + computeRow(sampler, force_scalar(i));
  P* row1 = row0 + computeNextRowOffset(sampler, force_scalar(i));
  int16_t fracx = computeFracX(sampler, i, frac).x;
  int16_t fracy = computeFracY(frac).x;
  auto src0 = CONVERT(unaligned_load<packed_type>(row0), signed_unpacked_type);
  auto src1 = CONVERT(unaligned_load<packed_type>(row1), signed_unpacked_type);
  auto src = castForShuffle(src0 + (((src1 - src0) * fracy) >> 7));

  // Since there is no scaling, we sample ahead by one chunk and interpolate it
  // with the current one. We can then reuse this value on the next iteration.
  for (P* end = buf + span; buf < end; buf += 4) {
    row0 += 4;
    row1 += 4;
    auto src0n =
        CONVERT(unaligned_load<packed_type>(row0), signed_unpacked_type);
    auto src1n =
        CONVERT(unaligned_load<packed_type>(row1), signed_unpacked_type);
    auto srcn = castForShuffle(src0n + (((src1n - src0n) * fracy) >> 7));

    // For the last sample to interpolate with, we need to potentially shift in
    // a sample from the next chunk over since the samples fill out an entire
    // chunk.
    auto interp = bit_cast<signed_unpacked_type>(src);
    auto interpn =
        bit_cast<signed_unpacked_type>(SHUFFLE(src, srcn, 1, 2, 3, 4));
    interp += ((interpn - interp) * fracx) >> 7;

    commit_blend_span<BLEND>(
        buf, applyColor(bit_cast<unpacked_type>(interp), color));

    src = srcn;
  }
}

// Implements a faster linear filter that works with axis-aligned constant Y but
// downscaling the texture by half. In this case we can optimize for the
// constant X/Y fractions and reduction factor while minimizing shuffling.
template <bool BLEND, typename S, typename C, typename P>
static NO_INLINE void blendTextureLinearDownscale(S sampler, vec2 uv, int span,
                                                  vec2_scalar min_uv,
                                                  vec2_scalar max_uv, C color,
                                                  P* buf) {
  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;
  typedef VectorType<uint16_t, 4 * sizeof(P)> unpacked_type;
  typedef VectorType<int16_t, 4 * sizeof(P)> signed_unpacked_type;

  ivec2 i(clamp(uv, min_uv, max_uv));
  ivec2 frac = i;
  i >>= 7;
  P* row0 = (P*)sampler->buf + computeRow(sampler, force_scalar(i));
  P* row1 = row0 + computeNextRowOffset(sampler, force_scalar(i));
  int16_t fracx = computeFracX(sampler, i, frac).x;
  int16_t fracy = computeFracY(frac).x;

  for (P* end = buf + span; buf < end; buf += 4) {
    auto src0 =
        CONVERT(unaligned_load<packed_type>(row0), signed_unpacked_type);
    auto src1 =
        CONVERT(unaligned_load<packed_type>(row1), signed_unpacked_type);
    auto src = castForShuffle(src0 + (((src1 - src0) * fracy) >> 7));
    row0 += 4;
    row1 += 4;
    auto src0n =
        CONVERT(unaligned_load<packed_type>(row0), signed_unpacked_type);
    auto src1n =
        CONVERT(unaligned_load<packed_type>(row1), signed_unpacked_type);
    auto srcn = castForShuffle(src0n + (((src1n - src0n) * fracy) >> 7));
    row0 += 4;
    row1 += 4;

    auto interp =
        bit_cast<signed_unpacked_type>(SHUFFLE(src, srcn, 0, 2, 4, 6));
    auto interpn =
        bit_cast<signed_unpacked_type>(SHUFFLE(src, srcn, 1, 3, 5, 7));
    interp += ((interpn - interp) * fracx) >> 7;

    commit_blend_span<BLEND>(
        buf, applyColor(bit_cast<unpacked_type>(interp), color));
  }
}

enum LinearFilter {
  // No linear filter is needed.
  LINEAR_FILTER_NEAREST = 0,
  // The most general linear filter that handles clamping and varying scales.
  LINEAR_FILTER_FALLBACK,
  // A linear filter optimized for axis-aligned upscaling.
  LINEAR_FILTER_UPSCALE,
  // A linear filter with no scaling but with subpixel offset.
  LINEAR_FILTER_FAST,
  // A linear filter optimized for 2x axis-aligned downscaling.
  LINEAR_FILTER_DOWNSCALE
};

// Dispatches to an appropriate linear filter depending on the selected filter.
template <bool BLEND, typename S, typename C, typename P>
static P* blendTextureLinearDispatch(S sampler, vec2 uv, int span,
                                     vec2_scalar uv_step, vec2_scalar min_uv,
                                     vec2_scalar max_uv, C color, P* buf,
                                     LinearFilter filter) {
  P* end = buf + span;
  if (filter != LINEAR_FILTER_FALLBACK) {
    // If we're not using the fallback, then Y is constant across the entire
    // row. We just need to ensure that we handle any samples that might pull
    // data from before the start of the row and require clamping.
    float beforeDist = max(0.0f, min_uv.x) - uv.x.x;
    if (beforeDist > 0) {
      int before = clamp(int(ceil(beforeDist / uv_step.x)) * swgl_StepSize, 0,
                         int(end - buf));
      buf = blendTextureLinearFallback<BLEND>(sampler, uv, before, uv_step,
                                              min_uv, max_uv, color, buf);
      uv.x += (before / swgl_StepSize) * uv_step.x;
    }
    // We need to check how many samples we can take from inside the row without
    // requiring clamping. In case the filter oversamples the row by a step, we
    // subtract off a step from the width to leave some room.
    float insideDist =
        min(max_uv.x, float((int(sampler->width) - swgl_StepSize) *
                            swgl_LinearQuantizeScale)) -
        uv.x.x;
    if (uv_step.x > 0.0f && insideDist >= uv_step.x) {
      int inside = int(end - buf);
      if (filter == LINEAR_FILTER_DOWNSCALE) {
        inside = clamp(int(insideDist * (0.5f / swgl_LinearQuantizeScale)) &
                           ~(swgl_StepSize - 1),
                       0, inside);
        blendTextureLinearDownscale<BLEND>(sampler, uv, inside, min_uv, max_uv,
                                           color, buf);
      } else if (filter == LINEAR_FILTER_UPSCALE) {
        inside = clamp(int(insideDist / uv_step.x) * swgl_StepSize, 0, inside);
        blendTextureLinearUpscale<BLEND>(sampler, uv, inside, uv_step, min_uv,
                                         max_uv, color, buf);
      } else {
        inside = clamp(int(insideDist * (1.0f / swgl_LinearQuantizeScale)) &
                           ~(swgl_StepSize - 1),
                       0, inside);
        blendTextureLinearFast<BLEND>(sampler, uv, inside, min_uv, max_uv,
                                      color, buf);
      }
      buf += inside;
      uv.x += (inside / swgl_StepSize) * uv_step.x;
    }
  }
  // If the fallback filter was requested, or if there are any samples left that
  // may be outside the row and require clamping, then handle that with here.
  if (buf < end) {
    buf = blendTextureLinearFallback<BLEND>(
        sampler, uv, int(end - buf), uv_step, min_uv, max_uv, color, buf);
  }
  return buf;
}

// Helper function to quantize UVs for linear filtering before dispatch
template <bool BLEND, typename S, typename C, typename P>
static inline int blendTextureLinear(S sampler, vec2 uv, int span,
                                     const vec4_scalar& uv_rect, C color,
                                     P* buf, LinearFilter filter) {
  if (!matchTextureFormat(sampler, buf)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler, uv, uv_step, uv_rect, min_uv, max_uv);
  blendTextureLinearDispatch<BLEND>(sampler, uv, span, uv_step, min_uv, max_uv,
                                    color, buf, filter);
  return span;
}

// Samples an axis-aligned span of on a single row of a texture using 1:1
// nearest filtering. Sampling is constrained to only fall within the given UV
// bounds. This requires a pointer to the destination buffer. An optional color
// modulus can be supplied.
template <bool BLEND, typename S, typename C, typename P>
static int blendTextureNearestFast(S sampler, vec2 uv, int span,
                                   const vec4_scalar& uv_rect, C color,
                                   P* buf) {
  if (!matchTextureFormat(sampler, buf)) {
    return 0;
  }

  typedef VectorType<uint8_t, 4 * sizeof(P)> packed_type;

  ivec2_scalar i = make_ivec2(samplerScale(sampler, force_scalar(uv)));
  ivec2_scalar minUV =
      make_ivec2(samplerScale(sampler, vec2_scalar{uv_rect.x, uv_rect.y}));
  ivec2_scalar maxUV =
      make_ivec2(samplerScale(sampler, vec2_scalar{uv_rect.z, uv_rect.w}));

  // Calculate the row pointer within the buffer, clamping to within valid row
  // bounds.
  P* row =
      &((P*)sampler
            ->buf)[clamp(clampCoord(i.y, sampler->height), minUV.y, maxUV.y) *
                   sampler->stride];
  // Find clamped X bounds within the row.
  int minX = clamp(minUV.x, 0, sampler->width - 1);
  int maxX = clamp(maxUV.x, minX, sampler->width - 1);
  int curX = i.x;
  int endX = i.x + span;
  // If we need to start sampling below the valid sample bounds, then we need to
  // fill this section with a constant clamped sample.
  if (curX < minX) {
    int n = min(minX, endX) - curX;
    auto src =
        applyColor(unpack(bit_cast<packed_type>(V4<P>(row[minX]))), color);
    commit_solid_span<BLEND>(buf, src, n);
    buf += n;
    curX += n;
  }
  // Here we only deal with valid samples within the sample bounds. No clamping
  // should occur here within these inner loops.
  int n = max(min(maxX + 1, endX) - curX, 0);
  // Try to process as many chunks as possible with full loads and stores.
  for (int end = curX + (n & ~3); curX < end; curX += 4, buf += 4) {
    auto src = applyColor(unaligned_load<packed_type>(&row[curX]), color);
    commit_blend_span<BLEND>(buf, src);
  }
  n &= 3;
  // If we have any leftover samples after processing chunks, use partial loads
  // and stores.
  if (n > 0) {
    auto src = applyColor(partial_load_span<packed_type>(&row[curX], n), color);
    commit_blend_span<BLEND>(buf, src, n);
    buf += n;
    curX += n;
  }
  // If we still have samples left above the valid sample bounds, then we again
  // need to fill this section with a constant clamped sample.
  if (curX < endX) {
    auto src =
        applyColor(unpack(bit_cast<packed_type>(V4<P>(row[maxX]))), color);
    commit_solid_span<BLEND>(buf, src, endX - curX);
  }
  return span;
}

// We need to verify that the pixel step reasonably approximates stepping by a
// single texel for every pixel we need to reproduce. Try to ensure that the
// margin of error is no more than approximately 2^-7. Also, we check here if
// the scaling can be quantized for acceleration.
template <typename T>
static ALWAYS_INLINE int spanNeedsScale(int span, T P) {
  span &= ~(128 - 1);
  span += 128;
  int scaled = round((P.x.y - P.x.x) * span);
  return scaled != span ? (scaled == span * 2 ? 2 : 1) : 0;
}

// Helper function to decide whether we can safely apply 1:1 nearest filtering
// without diverging too much from the linear filter.
template <typename S, typename T>
static inline LinearFilter needsTextureLinear(S sampler, T P, int span) {
  // First verify if the row Y doesn't change across samples
  if (P.y.x != P.y.y) {
    return LINEAR_FILTER_FALLBACK;
  }
  P = samplerScale(sampler, P);
  if (int scale = spanNeedsScale(span, P)) {
    // If the source region is not flipped and smaller than the destination,
    // then we can use the upscaling filter since row Y is constant.
    return P.x.x < P.x.y && P.x.y - P.x.x <= 1
               ? LINEAR_FILTER_UPSCALE
               : (scale == 2 ? LINEAR_FILTER_DOWNSCALE
                             : LINEAR_FILTER_FALLBACK);
  }
  // Also verify that we're reasonably close to the center of a texel
  // so that it doesn't look that much different than if a linear filter
  // was used.
  if ((int(P.x.x * 4.0f + 0.5f) & 3) != 2 ||
      (int(P.y.x * 4.0f + 0.5f) & 3) != 2) {
    // The source and destination regions are the same, but there is a
    // significant subpixel offset. We can use a faster linear filter to deal
    // with the offset in this case.
    return LINEAR_FILTER_FAST;
  }
  // Otherwise, we have a constant 1:1 step and we're stepping reasonably close
  // to the center of each pixel, so it's safe to disable the linear filter and
  // use nearest.
  return LINEAR_FILTER_NEAREST;
}

// Commit an entire span with linear filtering
#define swgl_commitTextureLinear(format, s, p, uv_rect, color, n)              \
  do {                                                                         \
    auto packed_color = packColor(swgl_Out##format, color);                    \
    int len = (n);                                                             \
    int drawn = 0;                                                             \
    if (LinearFilter filter = needsTextureLinear(s, p, len)) {                 \
      if (blend_key) {                                                         \
        drawn = blendTextureLinear<true>(s, p, len, uv_rect, packed_color,     \
                                         swgl_Out##format, filter);            \
      } else {                                                                 \
        drawn = blendTextureLinear<false>(s, p, len, uv_rect, packed_color,    \
                                          swgl_Out##format, filter);           \
      }                                                                        \
    } else if (blend_key) {                                                    \
      drawn = blendTextureNearestFast<true>(s, p, len, uv_rect, packed_color,  \
                                            swgl_Out##format);                 \
    } else {                                                                   \
      drawn = blendTextureNearestFast<false>(s, p, len, uv_rect, packed_color, \
                                             swgl_Out##format);                \
    }                                                                          \
    swgl_Out##format += drawn;                                                 \
    swgl_SpanLength -= drawn;                                                  \
  } while (0)
#define swgl_commitTextureLinearRGBA8(s, p, uv_rect) \
  swgl_commitTextureLinear(RGBA8, s, p, uv_rect, NoColor(), swgl_SpanLength)
#define swgl_commitTextureLinearR8(s, p, uv_rect) \
  swgl_commitTextureLinear(R8, s, p, uv_rect, NoColor(), swgl_SpanLength)

// Commit a partial span with linear filtering, optionally inverting the color
#define swgl_commitPartialTextureLinearR8(len, s, p, uv_rect) \
  swgl_commitTextureLinear(R8, s, p, uv_rect, NoColor(),      \
                           min(int(len), swgl_SpanLength))
#define swgl_commitPartialTextureLinearInvertR8(len, s, p, uv_rect) \
  swgl_commitTextureLinear(R8, s, p, uv_rect, InvertColor(),        \
                           min(int(len), swgl_SpanLength))

// Commit an entire span with linear filtering that is scaled by a color
#define swgl_commitTextureLinearColorRGBA8(s, p, uv_rect, color) \
  swgl_commitTextureLinear(RGBA8, s, p, uv_rect, color, swgl_SpanLength)
#define swgl_commitTextureLinearColorR8(s, p, uv_rect, color) \
  swgl_commitTextureLinear(R8, s, p, uv_rect, color, swgl_SpanLength)

// Helper function that samples from an R8 texture while expanding it to support
// a differing framebuffer format.
template <bool BLEND, typename S, typename C, typename P>
static inline int blendTextureLinearR8(S sampler, vec2 uv, int span,
                                       const vec4_scalar& uv_rect, C color,
                                       P* buf) {
  if (!swgl_isTextureR8(sampler)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler, uv, uv_step, uv_rect, min_uv, max_uv);
  for (P* end = buf + span; buf < end; buf += swgl_StepSize, uv += uv_step) {
    commit_blend_span<BLEND>(
        buf, applyColor(expand_mask(buf, textureLinearUnpackedR8(
                                             sampler,
                                             ivec2(clamp(uv, min_uv, max_uv)))),
                        color));
  }
  return span;
}

// Commit an entire span with linear filtering while expanding from R8 to RGBA8
#define swgl_commitTextureLinearColorR8ToRGBA8(s, p, uv_rect, color)      \
  do {                                                                    \
    auto packed_color = packColor(swgl_OutRGBA8, color);                  \
    int drawn = 0;                                                        \
    if (blend_key) {                                                      \
      drawn = blendTextureLinearR8<true>(s, p, swgl_SpanLength, uv_rect,  \
                                         packed_color, swgl_OutRGBA8);    \
    } else {                                                              \
      drawn = blendTextureLinearR8<false>(s, p, swgl_SpanLength, uv_rect, \
                                          packed_color, swgl_OutRGBA8);   \
    }                                                                     \
    swgl_OutRGBA8 += drawn;                                               \
    swgl_SpanLength -= drawn;                                             \
  } while (0)
#define swgl_commitTextureLinearR8ToRGBA8(s, p, uv_rect) \
  swgl_commitTextureLinearColorR8ToRGBA8(s, p, uv_rect, NoColor())

// Compute repeating UVs, possibly constrained by tile repeat limits
static inline vec2 tileRepeatUV(vec2 uv, const vec2_scalar& tile_repeat) {
  if (tile_repeat.x > 0.0f) {
    // Clamp to a number slightly less than the tile repeat limit so that
    // it results in a number close to but not equal to 1 after fract().
    // This avoids fract() yielding 0 if the limit was left as whole integer.
    uv = clamp(uv, vec2_scalar(0.0f), tile_repeat - 1.0e-6f);
  }
  return fract(uv);
}

// Compute the number of non-repeating steps before we need to potentially
// repeat the UVs.
static inline int computeNoRepeatSteps(Float uv, float uv_step,
                                       float tile_repeat, int steps) {
  if (uv.w < uv.x) {
    // Ensure the UV taps are ordered low to high.
    uv = uv.wzyx;
  }
  // Check if the samples cross the boundary of the next whole integer or the
  // tile repeat limit, whichever is lower.
  float limit = floor(uv.x) + 1.0f;
  if (tile_repeat > 0.0f) {
    limit = min(limit, tile_repeat);
  }
  return uv.x >= 0.0f && uv.w < limit
             ? (uv_step != 0.0f
                    ? int(min(float(steps), (limit - uv.x) / uv_step))
                    : steps)
             : 0;
}

// Blends an entire span of texture with linear filtering and repeating UVs.
template <bool BLEND, typename S, typename C, typename P>
static int blendTextureLinearRepeat(S sampler, vec2 uv, int span,
                                    const vec2_scalar& tile_repeat,
                                    const vec4_scalar& uv_repeat,
                                    const vec4_scalar& uv_rect, C color,
                                    P* buf) {
  if (!matchTextureFormat(sampler, buf)) {
    return 0;
  }
  vec2_scalar uv_scale = {uv_repeat.z - uv_repeat.x, uv_repeat.w - uv_repeat.y};
  vec2_scalar uv_offset = {uv_repeat.x, uv_repeat.y};
  // Choose a linear filter to use for no-repeat sub-spans
  LinearFilter filter =
      needsTextureLinear(sampler, uv * uv_scale + uv_offset, span);
  // We need to step UVs unscaled and unquantized so that we can modulo them
  // with fract. We use uv_scale and uv_offset to map them into the correct
  // range.
  vec2_scalar uv_step =
      float(swgl_StepSize) * vec2_scalar{uv.x.y - uv.x.x, uv.y.y - uv.y.x};
  uv_scale = swgl_linearQuantizeStep(sampler, uv_scale);
  uv_offset = swgl_linearQuantize(sampler, uv_offset);
  vec2_scalar min_uv = max(
      swgl_linearQuantize(sampler, vec2_scalar{uv_rect.x, uv_rect.y}), 0.0f);
  vec2_scalar max_uv = max(
      swgl_linearQuantize(sampler, vec2_scalar{uv_rect.z, uv_rect.w}), min_uv);
  for (P* end = buf + span; buf < end; buf += swgl_StepSize, uv += uv_step) {
    int steps = int(end - buf) / swgl_StepSize;
    // Find the sub-span before UVs repeat to avoid expensive repeat math
    steps = computeNoRepeatSteps(uv.x, uv_step.x, tile_repeat.x, steps);
    if (steps > 0) {
      steps = computeNoRepeatSteps(uv.y, uv_step.y, tile_repeat.y, steps);
      if (steps > 0) {
        buf = blendTextureLinearDispatch<BLEND>(
            sampler, fract(uv) * uv_scale + uv_offset, steps * swgl_StepSize,
            uv_step * uv_scale, min_uv, max_uv, color, buf, filter);
        if (buf >= end) {
          break;
        }
        uv += steps * uv_step;
      }
    }
    // UVs might repeat within this step, so explicitly compute repeated UVs
    vec2 repeated_uv = clamp(
        tileRepeatUV(uv, tile_repeat) * uv_scale + uv_offset, min_uv, max_uv);
    commit_blend_span<BLEND>(
        buf, applyColor(textureLinearUnpacked(buf, sampler, ivec2(repeated_uv)),
                        color));
  }
  return span;
}

// Commit an entire span with linear filtering and repeating UVs
#define swgl_commitTextureLinearRepeat(format, s, p, tile_repeat, uv_repeat,   \
                                       uv_rect, color)                         \
  do {                                                                         \
    auto packed_color = packColor(swgl_Out##format, color);                    \
    int drawn = 0;                                                             \
    if (blend_key) {                                                           \
      drawn = blendTextureLinearRepeat<true>(s, p, swgl_SpanLength,            \
                                             tile_repeat, uv_repeat, uv_rect,  \
                                             packed_color, swgl_Out##format);  \
    } else {                                                                   \
      drawn = blendTextureLinearRepeat<false>(s, p, swgl_SpanLength,           \
                                              tile_repeat, uv_repeat, uv_rect, \
                                              packed_color, swgl_Out##format); \
    }                                                                          \
    swgl_Out##format += drawn;                                                 \
    swgl_SpanLength -= drawn;                                                  \
  } while (0)
#define swgl_commitTextureLinearRepeatRGBA8(s, p, tile_repeat, uv_repeat,      \
                                            uv_rect)                           \
  swgl_commitTextureLinearRepeat(RGBA8, s, p, tile_repeat, uv_repeat, uv_rect, \
                                 NoColor())
#define swgl_commitTextureLinearRepeatColorRGBA8(s, p, tile_repeat, uv_repeat, \
                                                 uv_rect, color)               \
  swgl_commitTextureLinearRepeat(RGBA8, s, p, tile_repeat, uv_repeat, uv_rect, \
                                 color)

template <typename S>
static ALWAYS_INLINE PackedRGBA8 textureNearestPacked(UNUSED uint32_t* buf,
                                                      S sampler, ivec2 i) {
  return textureNearestPackedRGBA8(sampler, i);
}

// Blends an entire span of texture with nearest filtering and either
// repeated or clamped UVs.
template <bool BLEND, bool REPEAT, typename S, typename C, typename P>
static int blendTextureNearestRepeat(S sampler, vec2 uv, int span,
                                     const vec2_scalar& tile_repeat,
                                     const vec4_scalar& uv_rect, C color,
                                     P* buf) {
  if (!matchTextureFormat(sampler, buf)) {
    return 0;
  }
  if (!REPEAT) {
    // If clamping, then we step pre-scaled to the sampler. For repeat modes,
    // this will be accomplished via uv_scale instead.
    uv = samplerScale(sampler, uv);
  }
  vec2_scalar uv_step =
      float(swgl_StepSize) * vec2_scalar{uv.x.y - uv.x.x, uv.y.y - uv.y.x};
  vec2_scalar min_uv = samplerScale(sampler, vec2_scalar{uv_rect.x, uv_rect.y});
  vec2_scalar max_uv = samplerScale(sampler, vec2_scalar{uv_rect.z, uv_rect.w});
  vec2_scalar uv_scale = max_uv - min_uv;
  // If the effective sampling area of this texture is only a single pixel, then
  // treat it as a solid span. For repeat modes, the bounds are specified on
  // pixel boundaries, whereas for clamp modes, bounds are on pixel centers, so
  // the test varies depending on which. If the sample range on an axis is
  // greater than one pixel, we can still check if we don't move far enough from
  // the pixel center on that axis to hit the next pixel.
  if ((int(min_uv.x) + (REPEAT ? 1 : 0) >= int(max_uv.x) ||
       (uv_step.x * span * (REPEAT ? uv_scale.x : 1.0f) < 0.5f)) &&
      (int(min_uv.y) + (REPEAT ? 1 : 0) >= int(max_uv.y) ||
       (uv_step.y * span * (REPEAT ? uv_scale.y : 1.0f) < 0.5f))) {
    vec2 repeated_uv = REPEAT
                           ? tileRepeatUV(uv, tile_repeat) * uv_scale + min_uv
                           : clamp(uv, min_uv, max_uv);
    commit_solid_span<BLEND>(buf,
                             applyColor(unpack(textureNearestPacked(
                                            buf, sampler, ivec2(repeated_uv))),
                                        color),
                             span);
  } else {
    for (P* end = buf + span; buf < end; buf += swgl_StepSize, uv += uv_step) {
      if (REPEAT) {
        int steps = int(end - buf) / swgl_StepSize;
        // Find the sub-span before UVs repeat to avoid expensive repeat math
        steps = computeNoRepeatSteps(uv.x, uv_step.x, tile_repeat.x, steps);
        if (steps > 0) {
          steps = computeNoRepeatSteps(uv.y, uv_step.y, tile_repeat.y, steps);
          if (steps > 0) {
            vec2 inside_uv = fract(uv) * uv_scale + min_uv;
            vec2 inside_step = uv_step * uv_scale;
            for (P* outside = &buf[steps * swgl_StepSize]; buf < outside;
                 buf += swgl_StepSize, inside_uv += inside_step) {
              commit_blend_span<BLEND>(
                  buf, applyColor(
                           textureNearestPacked(buf, sampler, ivec2(inside_uv)),
                           color));
            }
            if (buf >= end) {
              break;
            }
            uv += steps * uv_step;
          }
        }
      }

      // UVs might repeat within this step, so explicitly compute repeated UVs
      vec2 repeated_uv = REPEAT
                             ? tileRepeatUV(uv, tile_repeat) * uv_scale + min_uv
                             : clamp(uv, min_uv, max_uv);
      commit_blend_span<BLEND>(
          buf,
          applyColor(textureNearestPacked(buf, sampler, ivec2(repeated_uv)),
                     color));
    }
  }
  return span;
}

// Determine if we can use the fast nearest filter for the given nearest mode.
// If the Y coordinate varies more than half a pixel over
// the span (which might cause the texel to alias to the next one), or the span
// needs X scaling, then we have to use the fallback.
template <typename S, typename T>
static ALWAYS_INLINE bool needsNearestFallback(S sampler, T P, int span) {
  P = samplerScale(sampler, P);
  return (P.y.y - P.y.x) * span >= 0.5f || spanNeedsScale(span, P);
}

// Commit an entire span with nearest filtering and either clamped or repeating
// UVs
#define swgl_commitTextureNearest(format, s, p, uv_rect, color)               \
  do {                                                                        \
    auto packed_color = packColor(swgl_Out##format, color);                   \
    int drawn = 0;                                                            \
    if (needsNearestFallback(s, p, swgl_SpanLength)) {                        \
      if (blend_key) {                                                        \
        drawn = blendTextureNearestRepeat<true, false>(                       \
            s, p, swgl_SpanLength, 0.0f, uv_rect, packed_color,               \
            swgl_Out##format);                                                \
      } else {                                                                \
        drawn = blendTextureNearestRepeat<false, false>(                      \
            s, p, swgl_SpanLength, 0.0f, uv_rect, packed_color,               \
            swgl_Out##format);                                                \
      }                                                                       \
    } else if (blend_key) {                                                   \
      drawn = blendTextureNearestFast<true>(s, p, swgl_SpanLength, uv_rect,   \
                                            packed_color, swgl_Out##format);  \
    } else {                                                                  \
      drawn = blendTextureNearestFast<false>(s, p, swgl_SpanLength, uv_rect,  \
                                             packed_color, swgl_Out##format); \
    }                                                                         \
    swgl_Out##format += drawn;                                                \
    swgl_SpanLength -= drawn;                                                 \
  } while (0)
#define swgl_commitTextureNearestRGBA8(s, p, uv_rect) \
  swgl_commitTextureNearest(RGBA8, s, p, uv_rect, NoColor())
#define swgl_commitTextureNearestColorRGBA8(s, p, uv_rect, color) \
  swgl_commitTextureNearest(RGBA8, s, p, uv_rect, color)

#define swgl_commitTextureNearestRepeat(format, s, p, tile_repeat, uv_rect, \
                                        color)                              \
  do {                                                                      \
    auto packed_color = packColor(swgl_Out##format, color);                 \
    int drawn = 0;                                                          \
    if (blend_key) {                                                        \
      drawn = blendTextureNearestRepeat<true, true>(                        \
          s, p, swgl_SpanLength, tile_repeat, uv_rect, packed_color,        \
          swgl_Out##format);                                                \
    } else {                                                                \
      drawn = blendTextureNearestRepeat<false, true>(                       \
          s, p, swgl_SpanLength, tile_repeat, uv_rect, packed_color,        \
          swgl_Out##format);                                                \
    }                                                                       \
    swgl_Out##format += drawn;                                              \
    swgl_SpanLength -= drawn;                                               \
  } while (0)
#define swgl_commitTextureNearestRepeatRGBA8(s, p, tile_repeat, uv_repeat, \
                                             uv_rect)                      \
  swgl_commitTextureNearestRepeat(RGBA8, s, p, tile_repeat, uv_repeat,     \
                                  NoColor())
#define swgl_commitTextureNearestRepeatColorRGBA8(s, p, tile_repeat,         \
                                                  uv_repeat, uv_rect, color) \
  swgl_commitTextureNearestRepeat(RGBA8, s, p, tile_repeat, uv_repeat, color)

// Commit an entire span of texture with filtering determined by sampler state.
#define swgl_commitTexture(format, s, ...)               \
  do {                                                   \
    if (s->filter == TextureFilter::LINEAR) {            \
      swgl_commitTextureLinear##format(s, __VA_ARGS__);  \
    } else {                                             \
      swgl_commitTextureNearest##format(s, __VA_ARGS__); \
    }                                                    \
  } while (0)
#define swgl_commitTextureRGBA8(...) swgl_commitTexture(RGBA8, __VA_ARGS__)
#define swgl_commitTextureColorRGBA8(...) \
  swgl_commitTexture(ColorRGBA8, __VA_ARGS__)
#define swgl_commitTextureRepeatRGBA8(...) \
  swgl_commitTexture(RepeatRGBA8, __VA_ARGS__)
#define swgl_commitTextureRepeatColorRGBA8(...) \
  swgl_commitTexture(RepeatColorRGBA8, __VA_ARGS__)

// Commit an entire span of a separable pass of a Gaussian blur that falls
// within the given radius scaled by supplied coefficients, clamped to uv_rect
// bounds.
template <bool BLEND, typename S, typename P>
static int blendGaussianBlur(S sampler, vec2 uv, const vec4_scalar& uv_rect,
                             P* buf, int span, bool hori, int radius,
                             vec2_scalar coeffs) {
  if (!matchTextureFormat(sampler, buf)) {
    return 0;
  }
  vec2_scalar size = {float(sampler->width), float(sampler->height)};
  ivec2_scalar curUV = make_ivec2(force_scalar(uv) * size);
  ivec4_scalar bounds = make_ivec4(uv_rect * make_vec4(size, size));
  int startX = curUV.x;
  int endX = min(bounds.z, curUV.x + span);
  if (hori) {
    for (; curUV.x + swgl_StepSize <= endX;
         buf += swgl_StepSize, curUV.x += swgl_StepSize) {
      commit_blend_span<BLEND>(
          buf, gaussianBlurHorizontal<P>(sampler, curUV, bounds.x, bounds.z,
                                         radius, coeffs.x, coeffs.y));
    }
  } else {
    for (; curUV.x + swgl_StepSize <= endX;
         buf += swgl_StepSize, curUV.x += swgl_StepSize) {
      commit_blend_span<BLEND>(
          buf, gaussianBlurVertical<P>(sampler, curUV, bounds.y, bounds.w,
                                       radius, coeffs.x, coeffs.y));
    }
  }
  return curUV.x - startX;
}

#define swgl_commitGaussianBlur(format, s, p, uv_rect, hori, radius, coeffs)   \
  do {                                                                         \
    int drawn = 0;                                                             \
    if (blend_key) {                                                           \
      drawn = blendGaussianBlur<true>(s, p, uv_rect, swgl_Out##format,         \
                                      swgl_SpanLength, hori, radius, coeffs);  \
    } else {                                                                   \
      drawn = blendGaussianBlur<false>(s, p, uv_rect, swgl_Out##format,        \
                                       swgl_SpanLength, hori, radius, coeffs); \
    }                                                                          \
    swgl_Out##format += drawn;                                                 \
    swgl_SpanLength -= drawn;                                                  \
  } while (0)
#define swgl_commitGaussianBlurRGBA8(s, p, uv_rect, hori, radius, coeffs) \
  swgl_commitGaussianBlur(RGBA8, s, p, uv_rect, hori, radius, coeffs)
#define swgl_commitGaussianBlurR8(s, p, uv_rect, hori, radius, coeffs) \
  swgl_commitGaussianBlur(R8, s, p, uv_rect, hori, radius, coeffs)

// Convert and pack planar YUV samples to RGB output using a color space
static ALWAYS_INLINE PackedRGBA8 convertYUV(int colorSpace, U16 y, U16 u,
                                            U16 v) {
  auto yy = V8<int16_t>(zip(y, y));
  auto uv = V8<int16_t>(zip(u, v));
  return yuvMatrix[colorSpace].convert(yy, uv);
}

// Helper functions to sample from planar YUV textures before converting to RGB
template <typename S0>
static ALWAYS_INLINE PackedRGBA8 sampleYUV(S0 sampler0, ivec2 uv0,
                                           int colorSpace,
                                           UNUSED int rescaleFactor) {
  switch (sampler0->format) {
    case TextureFormat::RGBA8: {
      auto planar = textureLinearPlanarRGBA8(sampler0, uv0);
      return convertYUV(colorSpace, highHalf(planar.rg), lowHalf(planar.rg),
                        lowHalf(planar.ba));
    }
    case TextureFormat::YUV422: {
      auto planar = textureLinearPlanarYUV422(sampler0, uv0);
      return convertYUV(colorSpace, planar.y, planar.u, planar.v);
    }
    default:
      assert(false);
      return PackedRGBA8(0);
  }
}

template <bool BLEND, typename S0, typename P, typename C = NoColor>
static int blendYUV(P* buf, int span, S0 sampler0, vec2 uv0,
                    const vec4_scalar& uv_rect0, int colorSpace,
                    int rescaleFactor, C color = C()) {
  if (!swgl_isTextureLinear(sampler0)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler0, uv0, uv_step0, uv_rect0, min_uv0, max_uv0);
  auto c = packColor(buf, color);
  auto* end = buf + span;
  for (; buf < end; buf += swgl_StepSize, uv0 += uv_step0) {
    commit_blend_span<BLEND>(
        buf, applyColor(sampleYUV(sampler0, ivec2(clamp(uv0, min_uv0, max_uv0)),
                                  colorSpace, rescaleFactor),
                        c));
  }
  return span;
}

template <typename S0, typename S1>
static ALWAYS_INLINE PackedRGBA8 sampleYUV(S0 sampler0, ivec2 uv0, S1 sampler1,
                                           ivec2 uv1, int colorSpace,
                                           UNUSED int rescaleFactor) {
  switch (sampler1->format) {
    case TextureFormat::RG8: {
      assert(sampler0->format == TextureFormat::R8);
      auto y = textureLinearUnpackedR8(sampler0, uv0);
      auto planar = textureLinearPlanarRG8(sampler1, uv1);
      return convertYUV(colorSpace, y, lowHalf(planar.rg), highHalf(planar.rg));
    }
    case TextureFormat::RGBA8: {
      assert(sampler0->format == TextureFormat::R8);
      auto y = textureLinearUnpackedR8(sampler0, uv0);
      auto planar = textureLinearPlanarRGBA8(sampler1, uv1);
      return convertYUV(colorSpace, y, lowHalf(planar.ba), highHalf(planar.rg));
    }
    default:
      assert(false);
      return PackedRGBA8(0);
  }
}

template <bool BLEND, typename S0, typename S1, typename P,
          typename C = NoColor>
static int blendYUV(P* buf, int span, S0 sampler0, vec2 uv0,
                    const vec4_scalar& uv_rect0, S1 sampler1, vec2 uv1,
                    const vec4_scalar& uv_rect1, int colorSpace,
                    int rescaleFactor, C color = C()) {
  if (!swgl_isTextureLinear(sampler0) || !swgl_isTextureLinear(sampler1)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler0, uv0, uv_step0, uv_rect0, min_uv0, max_uv0);
  LINEAR_QUANTIZE_UV(sampler1, uv1, uv_step1, uv_rect1, min_uv1, max_uv1);
  auto c = packColor(buf, color);
  auto* end = buf + span;
  for (; buf < end; buf += swgl_StepSize, uv0 += uv_step0, uv1 += uv_step1) {
    commit_blend_span<BLEND>(
        buf, applyColor(sampleYUV(sampler0, ivec2(clamp(uv0, min_uv0, max_uv0)),
                                  sampler1, ivec2(clamp(uv1, min_uv1, max_uv1)),
                                  colorSpace, rescaleFactor),
                        c));
  }
  return span;
}

template <typename S0, typename S1, typename S2>
static ALWAYS_INLINE PackedRGBA8 sampleYUV(S0 sampler0, ivec2 uv0, S1 sampler1,
                                           ivec2 uv1, S2 sampler2, ivec2 uv2,
                                           int colorSpace, int rescaleFactor) {
  assert(sampler0->format == sampler1->format &&
         sampler0->format == sampler2->format);
  switch (sampler0->format) {
    case TextureFormat::R8: {
      auto y = textureLinearUnpackedR8(sampler0, uv0);
      auto u = textureLinearUnpackedR8(sampler1, uv1);
      auto v = textureLinearUnpackedR8(sampler2, uv2);
      return convertYUV(colorSpace, y, u, v);
    }
    case TextureFormat::R16: {
      // The rescaling factor represents how many bits to add to renormalize the
      // texture to 16 bits, and so the color depth is actually 16 minus the
      // rescaling factor.
      // Need to right shift the sample by the amount of bits over 8 it
      // occupies. On output from textureLinearUnpackedR16, we have lost 1 bit
      // of precision at the low end already, hence 1 is subtracted from the
      // color depth.
      int colorDepth = 16 - rescaleFactor;
      int rescaleBits = (colorDepth - 1) - 8;
      auto y = textureLinearUnpackedR16(sampler0, uv0) >> rescaleBits;
      auto u = textureLinearUnpackedR16(sampler1, uv1) >> rescaleBits;
      auto v = textureLinearUnpackedR16(sampler2, uv2) >> rescaleBits;
      return convertYUV(colorSpace, U16(y), U16(u), U16(v));
    }
    default:
      assert(false);
      return PackedRGBA8(0);
  }
}

// Fallback helper for when we can't specifically accelerate YUV with
// composition.
template <bool BLEND, typename S0, typename S1, typename S2, typename P,
          typename C>
static void blendYUVFallback(P* buf, int span, S0 sampler0, vec2 uv0,
                             vec2_scalar uv_step0, vec2_scalar min_uv0,
                             vec2_scalar max_uv0, S1 sampler1, vec2 uv1,
                             vec2_scalar uv_step1, vec2_scalar min_uv1,
                             vec2_scalar max_uv1, S2 sampler2, vec2 uv2,
                             vec2_scalar uv_step2, vec2_scalar min_uv2,
                             vec2_scalar max_uv2, int colorSpace,
                             int rescaleFactor, C color) {
  for (auto* end = buf + span; buf < end; buf += swgl_StepSize, uv0 += uv_step0,
             uv1 += uv_step1, uv2 += uv_step2) {
    commit_blend_span<BLEND>(
        buf, applyColor(sampleYUV(sampler0, ivec2(clamp(uv0, min_uv0, max_uv0)),
                                  sampler1, ivec2(clamp(uv1, min_uv1, max_uv1)),
                                  sampler2, ivec2(clamp(uv2, min_uv2, max_uv2)),
                                  colorSpace, rescaleFactor),
                        color));
  }
}

template <bool BLEND, typename S0, typename S1, typename S2, typename P,
          typename C = NoColor>
static int blendYUV(P* buf, int span, S0 sampler0, vec2 uv0,
                    const vec4_scalar& uv_rect0, S1 sampler1, vec2 uv1,
                    const vec4_scalar& uv_rect1, S2 sampler2, vec2 uv2,
                    const vec4_scalar& uv_rect2, int colorSpace,
                    int rescaleFactor, C color = C()) {
  if (!swgl_isTextureLinear(sampler0) || !swgl_isTextureLinear(sampler1) ||
      !swgl_isTextureLinear(sampler2)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler0, uv0, uv_step0, uv_rect0, min_uv0, max_uv0);
  LINEAR_QUANTIZE_UV(sampler1, uv1, uv_step1, uv_rect1, min_uv1, max_uv1);
  LINEAR_QUANTIZE_UV(sampler2, uv2, uv_step2, uv_rect2, min_uv2, max_uv2);
  auto c = packColor(buf, color);
  blendYUVFallback<BLEND>(buf, span, sampler0, uv0, uv_step0, min_uv0, max_uv0,
                          sampler1, uv1, uv_step1, min_uv1, max_uv1, sampler2,
                          uv2, uv_step2, min_uv2, max_uv2, colorSpace,
                          rescaleFactor, c);
  return span;
}

// A variant of the blendYUV that attempts to reuse the inner loops from the
// CompositeYUV infrastructure. CompositeYUV imposes stricter requirements on
// the source data, which in turn allows it to be much faster than blendYUV.
// At a minimum, we need to ensure that we are outputting to a BGRA8 framebuffer
// and that no color scaling is applied, which we can accomplish via template
// specialization. We need to further validate inside that texture formats
// and dimensions are sane for video and that the video is axis-aligned before
// acceleration can proceed.
template <bool BLEND>
static int blendYUV(uint32_t* buf, int span, sampler2DRect sampler0, vec2 uv0,
                    const vec4_scalar& uv_rect0, sampler2DRect sampler1,
                    vec2 uv1, const vec4_scalar& uv_rect1,
                    sampler2DRect sampler2, vec2 uv2,
                    const vec4_scalar& uv_rect2, int colorSpace,
                    int rescaleFactor, NoColor noColor = NoColor()) {
  if (!swgl_isTextureLinear(sampler0) || !swgl_isTextureLinear(sampler1) ||
      !swgl_isTextureLinear(sampler2)) {
    return 0;
  }
  LINEAR_QUANTIZE_UV(sampler0, uv0, uv_step0, uv_rect0, min_uv0, max_uv0);
  LINEAR_QUANTIZE_UV(sampler1, uv1, uv_step1, uv_rect1, min_uv1, max_uv1);
  LINEAR_QUANTIZE_UV(sampler2, uv2, uv_step2, uv_rect2, min_uv2, max_uv2);
  auto* end = buf + span;
  // CompositeYUV imposes further restrictions on the source textures, such that
  // the the Y/U/V samplers must all have a matching format, the U/V samplers
  // must have matching sizes and sample coordinates, and there must be no
  // change in row across the entire span.
  if (sampler0->format == sampler1->format &&
      sampler1->format == sampler2->format &&
      sampler1->width == sampler2->width &&
      sampler1->height == sampler2->height && uv_step0.y == 0 &&
      uv_step0.x > 0 && uv_step1.y == 0 && uv_step1.x > 0 &&
      uv_step1 == uv_step2 && uv1.x.x == uv2.x.x && uv1.y.x == uv2.y.x) {
    // CompositeYUV does not support a clamp rect, so we must take care to
    // advance till we're inside the bounds of the clamp rect.
    int outside = min(int(ceil(max((min_uv0.x - uv0.x.x) / uv_step0.x,
                                   (min_uv1.x - uv1.x.x) / uv_step1.x))),
                      (end - buf) / swgl_StepSize);
    if (outside > 0) {
      blendYUVFallback<BLEND>(
          buf, outside * swgl_StepSize, sampler0, uv0, uv_step0, min_uv0,
          max_uv0, sampler1, uv1, uv_step1, min_uv1, max_uv1, sampler2, uv2,
          uv_step2, min_uv2, max_uv2, colorSpace, rescaleFactor, noColor);
      buf += outside * swgl_StepSize;
      uv0.x += outside * uv_step0.x;
      uv1.x += outside * uv_step1.x;
      uv2.x += outside * uv_step2.x;
    }
    // Find the amount of chunks inside the clamp rect before we hit the
    // maximum. If there are any chunks inside, we can finally dispatch to
    // CompositeYUV.
    int inside = min(int(min((max_uv0.x - uv0.x.x) / uv_step0.x,
                             (max_uv1.x - uv1.x.x) / uv_step1.x)),
                     (end - buf) / swgl_StepSize);
    if (inside > 0) {
      // We need the color depth, which is relative to the texture format and
      // rescale factor.
      int colorDepth =
          (sampler0->format == TextureFormat::R16 ? 16 : 8) - rescaleFactor;
      // Finally, call the inner loop of CompositeYUV.
      linear_row_yuv<BLEND>(
          buf, inside * swgl_StepSize, sampler0, force_scalar(uv0),
          uv_step0.x / swgl_StepSize, sampler1, sampler2, force_scalar(uv1),
          uv_step1.x / swgl_StepSize, colorDepth, yuvMatrix[colorSpace]);
      // Now that we're done, advance past the processed inside portion.
      buf += inside * swgl_StepSize;
      uv0.x += inside * uv_step0.x;
      uv1.x += inside * uv_step1.x;
      uv2.x += inside * uv_step2.x;
    }
  }
  // We either got here because we have some samples outside the clamp rect, or
  // because some of the preconditions were not satisfied. Process whatever is
  // left of the span.
  blendYUVFallback<BLEND>(buf, end - buf, sampler0, uv0, uv_step0, min_uv0,
                          max_uv0, sampler1, uv1, uv_step1, min_uv1, max_uv1,
                          sampler2, uv2, uv_step2, min_uv2, max_uv2, colorSpace,
                          rescaleFactor, noColor);
  return span;
}

// Commit a single chunk of a YUV surface represented by multiple planar
// textures. This requires a color space specifier selecting how to convert
// from YUV to RGB output. In the case of HDR formats, a rescaling factor
// selects how many bits of precision must be utilized on conversion. See the
// sampleYUV dispatcher functions for the various supported plane
// configurations this intrinsic accepts.
#define swgl_commitTextureLinearYUV(...)                                    \
  do {                                                                      \
    int drawn = 0;                                                          \
    if (blend_key) {                                                        \
      drawn = blendYUV<true>(swgl_OutRGBA8, swgl_SpanLength, __VA_ARGS__);  \
    } else {                                                                \
      drawn = blendYUV<false>(swgl_OutRGBA8, swgl_SpanLength, __VA_ARGS__); \
    }                                                                       \
    swgl_OutRGBA8 += drawn;                                                 \
    swgl_SpanLength -= drawn;                                               \
  } while (0)

// Commit a single chunk of a YUV surface scaled by a color.
#define swgl_commitTextureLinearColorYUV(...) \
  swgl_commitTextureLinearYUV(__VA_ARGS__)

// Each gradient stops entry is a pair of RGBA32F start color and end step.
struct GradientStops {
  Float startColor;
  union {
    Float stepColor;
    vec4_scalar stepData;
  };

  // Whether this gradient entry can be merged with an adjacent entry. The
  // step will be equal with the adjacent step if and only if they can be
  // merged, or rather, that the stops are actually part of a single larger
  // gradient.
  bool can_merge(const GradientStops& next) const {
    return stepData == next.stepData;
  }

  // Get the interpolated color within the entry based on the offset from its
  // start.
  Float interpolate(float offset) const {
    return startColor + stepColor * offset;
  }

  // Get the end color of the entry where interpolation stops.
  Float end_color() const { return startColor + stepColor; }
};

// Checks if a gradient table of the specified size exists at the UV coords of
// the address within an RGBA32F texture. If so, a linear address within the
// texture is returned that may be used to sample the gradient table later. If
// the address doesn't describe a valid gradient, then a negative value is
// returned.
static inline int swgl_validateGradient(sampler2D sampler, ivec2_scalar address,
                                        int entries) {
  return sampler->format == TextureFormat::RGBA32F && address.y >= 0 &&
                 address.y < int(sampler->height) && address.x >= 0 &&
                 address.x < int(sampler->width) && entries > 0 &&
                 address.x +
                         int(sizeof(GradientStops) / sizeof(Float)) * entries <=
                     int(sampler->width)
             ? address.y * sampler->stride + address.x * 4
             : -1;
}

static inline WideRGBA8 sampleGradient(sampler2D sampler, int address,
                                       Float entry) {
  assert(sampler->format == TextureFormat::RGBA32F);
  assert(address >= 0 && address < int(sampler->height * sampler->stride));
  // Get the integer portion of the entry index to find the entry colors.
  I32 index = cast(entry);
  // Use the fractional portion of the entry index to control blending between
  // entry colors.
  Float offset = entry - cast(index);
  // Every entry is a pair of colors blended by the fractional offset.
  assert(test_all(index >= 0 &&
                  index * int(sizeof(GradientStops) / sizeof(Float)) <
                      int(sampler->width)));
  GradientStops* stops = (GradientStops*)&sampler->buf[address];
  // Blend between the colors for each SIMD lane, then pack them to RGBA8
  // result. Since the layout of the RGBA8 framebuffer is actually BGRA while
  // the gradient table has RGBA colors, swizzling is required.
  return combine(
      packRGBA8(round_pixel(stops[index.x].interpolate(offset.x).zyxw),
                round_pixel(stops[index.y].interpolate(offset.y).zyxw)),
      packRGBA8(round_pixel(stops[index.z].interpolate(offset.z).zyxw),
                round_pixel(stops[index.w].interpolate(offset.w).zyxw)));
}

// Samples a gradient entry from the gradient at the provided linearized
// address. The integer portion of the entry index is used to find the entry
// within the table whereas the fractional portion is used to blend between
// adjacent table entries.
#define swgl_commitGradientRGBA8(sampler, address, entry) \
  swgl_commitChunk(RGBA8, sampleGradient(sampler, address, entry))

// Variant that allows specifying a color multiplier of the gradient result.
#define swgl_commitGradientColorRGBA8(sampler, address, entry, color)         \
  swgl_commitChunk(RGBA8, applyColor(sampleGradient(sampler, address, entry), \
                                     packColor(swgl_OutRGBA, color)))

// Samples an entire span of a linear gradient by crawling the gradient table
// and looking for consecutive stops that can be merged into a single larger
// gradient, then interpolating between those larger gradients within the span.
template <bool BLEND>
static bool commitLinearGradient(sampler2D sampler, int address, float size,
                                 bool repeat, Float offset, uint32_t* buf,
                                 int span) {
  assert(sampler->format == TextureFormat::RGBA32F);
  assert(address >= 0 && address < int(sampler->height * sampler->stride));
  GradientStops* stops = (GradientStops*)&sampler->buf[address];
  // Get the chunk delta from the difference in offset steps. This represents
  // how far within the gradient table we advance for every step in output,
  // normalized to gradient table size.
  float delta = (offset.y - offset.x) * 4.0f;
  if (!isfinite(delta)) {
    return false;
  }
  for (; span > 0;) {
    // If repeat is desired, we need to limit the offset to a fractional value.
    if (repeat) {
      offset = fract(offset);
    }
    // Try to process as many chunks as are within the span if possible.
    float chunks = 0.25f * span;
    // To properly handle both clamping and repeating of the table offset, we
    // need to ensure we don't run past the 0 and 1 points. Here we compute the
    // intercept points depending on whether advancing forwards or backwards in
    // the gradient table to ensure the chunk count is limited by the amount
    // before intersection. If there is no delta, then we compute no intercept.
    float startEntry;
    int minIndex, maxIndex;
    if (offset.x < 0) {
      // If we're below the gradient table, use the first color stop. We can
      // only intercept the table if walking forward.
      startEntry = 0;
      minIndex = int(startEntry);
      maxIndex = minIndex;
      if (delta > 0) {
        chunks = min(chunks, -offset.x / delta);
      }
    } else if (offset.x < 1) {
      // Otherwise, we're inside the gradient table. Depending on the direction
      // we're walking the the table, we may intersect either the 0 or 1 offset.
      // Compute the start entry based on our initial offset, and compute the
      // end entry based on the available chunks limited by intercepts. Clamp
      // them into the valid range of the table.
      startEntry = 1.0f + offset.x * size;
      if (delta < 0) {
        chunks = min(chunks, -offset.x / delta);
      } else if (delta > 0) {
        chunks = min(chunks, (1 - offset.x) / delta);
      }
      float endEntry = clamp(1.0f + (offset.x + delta * int(chunks)) * size,
                             0.0f, 1.0f + size);
      // Now that we know the range of entries we need to sample, we want to
      // find the largest possible merged gradient within that range. Depending
      // on which direction we are advancing in the table, we either walk up or
      // down the table trying to merge the current entry with the adjacent
      // entry. We finally limit the chunks to only sample from this merged
      // gradient.
      minIndex = int(startEntry);
      maxIndex = minIndex;
      if (delta > 0) {
        while (maxIndex + 1 < endEntry &&
               stops[maxIndex].can_merge(stops[maxIndex + 1])) {
          maxIndex++;
        }
        chunks = min(chunks, (maxIndex + 1 - startEntry) / (delta * size));
      } else if (delta < 0) {
        while (minIndex - 1 > endEntry &&
               stops[minIndex - 1].can_merge(stops[minIndex])) {
          minIndex--;
        }
        chunks = min(chunks, (minIndex - startEntry) / (delta * size));
      }
    } else {
      // If we're above the gradient table, use the last color stop. We can
      // only intercept the table if walking backward.
      startEntry = 1.0f + size;
      minIndex = int(startEntry);
      maxIndex = minIndex;
      if (delta < 0) {
        chunks = min(chunks, (1 - offset.x) / delta);
      }
    }
    // If there are any amount of whole chunks of a merged gradient found,
    // then we want to process that as a single gradient span with the start
    // and end colors from the min and max entries.
    if (chunks >= 1.0f) {
      int inside = int(chunks);
      // Sample the start color from the min entry and the end color from the
      // max entry of the merged gradient. These are scaled to a range of
      // 0..0xFF00, as that is the largest shifted value that can fit in a U16.
      // Since we are only doing addition with the step value, we can still
      // represent negative step values without having to use an explicit sign
      // bit, as the result will still come out the same, allowing us to gain an
      // extra bit of precision. We will later shift these into 8 bit output
      // range while committing the span, but stepping with higher precision to
      // avoid banding. We convert from RGBA to BGRA here to avoid doing this in
      // the inner loop.
      auto minColorF = stops[minIndex].startColor.zyxw * float(0xFF00);
      auto maxColorF = stops[maxIndex].end_color().zyxw * float(0xFF00);
      // Get the color range of the merged gradient, normalized to its size.
      auto colorRangeF =
          (maxColorF - minColorF) * (1.0f / (maxIndex + 1 - minIndex));
      // Compute the actual starting color of the current start offset within
      // the merged gradient. The value 0.5 is added to the low bits (0x80) so
      // that the color will effective round to the nearest increment below.
      auto colorF =
          minColorF + colorRangeF * (startEntry - minIndex) + float(0x80);
      // Compute the portion of the color range that we advance on each chunk.
      Float deltaColorF = colorRangeF * (delta * size);
      // Quantize the color delta and current color. These have already been
      // scaled to the 0..0xFF00 range, so we just need to round them to U16.
      auto deltaColor = repeat4(CONVERT(round_pixel(deltaColorF, 1), U16));
      auto color =
          combine(CONVERT(round_pixel(colorF, 1), U16),
                  CONVERT(round_pixel(colorF + deltaColorF * 0.25f, 1), U16),
                  CONVERT(round_pixel(colorF + deltaColorF * 0.5f, 1), U16),
                  CONVERT(round_pixel(colorF + deltaColorF * 0.75f, 1), U16));
      // Finally, step the current color through the output chunks, shifting
      // it into 8 bit range and outputting as we go.
      for (auto* end = buf + inside * 4; buf < end; buf += 4) {
        commit_blend_span<BLEND>(buf, bit_cast<WideRGBA8>(color >> 8));
        color += deltaColor;
      }
      // Deduct the number of chunks inside the gradient from the remaining
      // overall span. If we exhausted the span, bail out.
      span -= inside * 4;
      if (span <= 0) {
        break;
      }
      // Otherwise, assume we're in a transitional section of the gradient that
      // will probably require per-sample table lookups, so fall through below.
      offset += inside * delta;
      if (repeat) {
        offset = fract(offset);
      }
    }
    // If we get here, there were no whole chunks of a merged gradient found
    // that we could process, but we still have a non-zero amount of span left.
    // That means we have segments of gradient that begin or end at the current
    // entry we're on. For this case, we just fall back to sampleGradient which
    // will calculate a table entry for each sample, assuming the samples may
    // have different table entries.
    Float entry = clamp(offset * size + 1.0f, 0.0f, 1.0f + size);
    commit_blend_span<BLEND>(buf, sampleGradient(sampler, address, entry));
    span -= 4;
    buf += 4;
    offset += delta;
  }
  return true;
}

// Commits an entire span of a linear gradient, given the address of a table
// previously resolved with swgl_validateGradient. The size of the inner portion
// of the table is given, assuming the table start and ends with a single entry
// each to deal with clamping. Repeating will be handled if necessary. The
// initial offset within the table is used to designate where to start the span
// and how to step through the gradient table.
#define swgl_commitLinearGradientRGBA8(sampler, address, size, repeat, offset) \
  do {                                                                         \
    bool drawn = false;                                                        \
    if (blend_key) {                                                           \
      drawn =                                                                  \
          commitLinearGradient<true>(sampler, address, size, repeat, offset,   \
                                     swgl_OutRGBA8, swgl_SpanLength);          \
    } else {                                                                   \
      drawn =                                                                  \
          commitLinearGradient<false>(sampler, address, size, repeat, offset,  \
                                      swgl_OutRGBA8, swgl_SpanLength);         \
    }                                                                          \
    if (drawn) {                                                               \
      swgl_OutRGBA8 += swgl_SpanLength;                                        \
      swgl_SpanLength = 0;                                                     \
    }                                                                          \
  } while (0)

template <bool CLAMP, typename V>
static ALWAYS_INLINE V fastSqrt(V v) {
#if USE_SSE2 || USE_NEON
  // Clamp to avoid zero in inversesqrt.
  return v * inversesqrt(CLAMP ? max(v, V(1.0e-10f)) : v);
#else
  return sqrt(v);
#endif
}

template <bool CLAMP, typename V>
static ALWAYS_INLINE auto fastLength(V v) {
  return fastSqrt<CLAMP>(dot(v, v));
}

// Samples an entire span of a radial gradient by crawling the gradient table
// and looking for consecutive stops that can be merged into a single larger
// gradient, then interpolating between those larger gradients within the span
// based on the computed position relative to a radius.
template <bool BLEND>
static bool commitRadialGradient(sampler2D sampler, int address, float size,
                                 bool repeat, vec2 pos, float radius,
                                 uint32_t* buf, int span) {
  assert(sampler->format == TextureFormat::RGBA32F);
  assert(address >= 0 && address < int(sampler->height * sampler->stride));
  GradientStops* stops = (GradientStops*)&sampler->buf[address];
  // clang-format off
  // Given position p, delta d, and radius r, we need to repeatedly solve the
  // following quadratic for the pixel offset t:
  //    length(p + t*d) = r
  //    (px + t*dx)^2 + (py + t*dy)^2 = r^2
  // Rearranged into quadratic equation form (t^2*a + t*b + c = 0) this is:
  //    t^2*(dx^2+dy^2) + t*2*(dx*px+dy*py) + (px^2+py^2-r^2) = 0
  //    t^2*d.d + t*2*d.p + (p.p-r^2) = 0
  // The solution of the quadratic formula t=(-b+-sqrt(b^2-4ac))/2a reduces to:
  //    t = -d.p/d.d +- sqrt((d.p/d.d)^2 - (p.p-r^2)/d.d)
  // Note that d.p, d.d, p.p, and r^2 are constant across the gradient, and so
  // we cache them below for faster computation.
  //
  // The quadratic has two solutions, representing the span intersecting the
  // given radius of gradient, which can occur at two offsets. If there is only
  // one solution (where b^2-4ac = 0), this represents the point at which the
  // span runs tangent to the radius. This middle point is significant in that
  // before it, we walk down the gradient ramp, and after it, we walk up the
  // ramp.
  // clang-format on
  vec2_scalar pos0 = {pos.x.x, pos.y.x};
  vec2_scalar delta = {pos.x.y - pos.x.x, pos.y.y - pos.y.x};
  float deltaDelta = dot(delta, delta);
  if (!isfinite(deltaDelta) || !isfinite(radius)) {
    return false;
  }
  float invDelta, middleT, middleB;
  if (deltaDelta > 0) {
    invDelta = 1.0f / deltaDelta;
    middleT = -dot(delta, pos0) * invDelta;
    middleB = middleT * middleT - dot(pos0, pos0) * invDelta;
  } else {
    // If position is invariant, just set the coefficients so the quadratic
    // always reduces to the end of the span.
    invDelta = 0.0f;
    middleT = float(span);
    middleB = 0.0f;
  }
  // We only want search for merged gradients up to the minimum of either the
  // mid-point or the span length. Cache those offsets here as they don't vary
  // in the inner loop.
  Float middleEndRadius = fastLength<true>(
      pos0 + delta * (Float){middleT, float(span), 0.0f, 0.0f});
  float middleRadius = span < middleT ? middleEndRadius.y : middleEndRadius.x;
  float endRadius = middleEndRadius.y;
  // Convert delta to change in position per chunk.
  delta *= 4;
  deltaDelta *= 4 * 4;
  // clang-format off
  // Given current position p and delta d, we reduce:
  //    length(p) = sqrt(dot(p,p)) = dot(p,p) * invsqrt(dot(p,p))
  // where dot(p+d,p+d) can be accumulated as:
  //    (x+dx)^2+(y+dy)^2 = (x^2+y^2) + 2(x*dx+y*dy) + (dx^2+dy^2)
  //                      = p.p + 2p.d + d.d
  // Since p increases by d every loop iteration, p.d increases by d.d, and thus
  // we can accumulate d.d to calculate 2p.d, then allowing us to get the next
  // dot-product by adding it to dot-product p.p of the prior iteration. This
  // saves us some multiplications and an expensive sqrt inside the inner loop.
  // clang-format on
  Float dotPos = dot(pos, pos);
  Float dotPosDelta = 2.0f * dot(pos, delta) + deltaDelta;
  float deltaDelta2 = 2.0f * deltaDelta;
  for (int t = 0; t < span;) {
    // Compute the gradient table offset from the current position.
    Float offset = fastSqrt<true>(dotPos) - radius;
    float startRadius = radius;
    // If repeat is desired, we need to limit the offset to a fractional value.
    if (repeat) {
      // The non-repeating radius at which the gradient table actually starts,
      // radius + floor(offset) = radius + (offset - fract(offset)).
      startRadius += offset.x;
      offset = fract(offset);
      startRadius -= offset.x;
    }
    // We need to find the min/max index in the table of the gradient we want to
    // use as well as the intercept point where we leave this gradient.
    float intercept = -1;
    int minIndex = 0;
    int maxIndex = int(1.0f + size);
    if (offset.x < 0) {
      // If inside the inner radius of the gradient table, then use the first
      // stop. Set the intercept to advance forward to the start of the gradient
      // table.
      maxIndex = minIndex;
      if (t >= middleT) {
        intercept = radius;
      }
    } else if (offset.x < 1) {
      // Otherwise, we're inside the valid part of the gradient table.
      minIndex = int(1.0f + offset.x * size);
      maxIndex = minIndex;
      // Find the offset in the gradient that corresponds to the search limit.
      // We only search up to the minimum of either the mid-point or the span
      // length. Get the table index that corresponds to this offset, clamped so
      // that we avoid hitting the beginning (0) or end (1 + size) of the table.
      float searchOffset =
          (t >= middleT ? endRadius : middleRadius) - startRadius;
      int searchIndex = int(clamp(1.0f + size * searchOffset, 1.0f, size));
      // If we are past the mid-point, walk up the gradient table trying to
      // merge stops. If we're below the mid-point, we need to walk down the
      // table. We note the table index at which we need to look for an
      // intercept to determine a valid span.
      if (t >= middleT) {
        while (maxIndex + 1 <= searchIndex &&
               stops[maxIndex].can_merge(stops[maxIndex + 1])) {
          maxIndex++;
        }
        intercept = maxIndex + 1;
      } else {
        while (minIndex - 1 >= searchIndex &&
               stops[minIndex - 1].can_merge(stops[minIndex])) {
          minIndex--;
        }
        intercept = minIndex;
      }
      // Convert from a table index into units of radius from the center of the
      // gradient.
      intercept = clamp((intercept - 1.0f) / size, 0.0f, 1.0f) + startRadius;
    } else {
      // If outside the outer radius of the gradient table, then use the last
      // stop. Set the intercept to advance toward the valid part of the
      // gradient table if going in, or just run to the end of the span if going
      // away from the gradient.
      minIndex = maxIndex;
      if (t < middleT) {
        intercept = radius + 1;
      }
    }
    // Solve the quadratic for t to find where the merged gradient ends. If no
    // intercept is found, just go to the middle or end of the span.
    float endT = t >= middleT ? span : min(span, int(middleT));
    if (intercept >= 0) {
      float b = middleB + intercept * intercept * invDelta;
      if (b > 0) {
        b = fastSqrt<false>(b);
        endT = min(endT, t >= middleT ? middleT + b : middleT - b);
      }
    }
    // Figure out how many chunks are actually inside the merged gradient.
    if (t + 4.0f <= endT) {
      int inside = int(endT - t) & ~3;
      // Convert start and end colors to BGRA and scale to 0..255 range later.
      auto minColorF = stops[minIndex].startColor.zyxw * 255.0f;
      auto maxColorF = stops[maxIndex].end_color().zyxw * 255.0f;
      // Compute the change in color per change in gradient offset.
      auto deltaColorF =
          (maxColorF - minColorF) * (size / (maxIndex + 1 - minIndex));
      // Subtract off the color difference of the beginning of the current span
      // from the beginning of the gradient.
      Float colorF =
          minColorF - deltaColorF * (startRadius + (minIndex - 1) / size);
      // Finally, walk over the span accumulating the position dot product and
      // getting its sqrt as an offset into the color ramp. Since we're already
      // in BGRA format and scaled to 255, we just need to round to an integer
      // and pack down to pixel format.
      for (auto* end = buf + inside; buf < end; buf += 4) {
        Float offsetG = fastSqrt<false>(dotPos);
        commit_blend_span<BLEND>(
            buf,
            combine(
                packRGBA8(round_pixel(colorF + deltaColorF * offsetG.x, 1),
                          round_pixel(colorF + deltaColorF * offsetG.y, 1)),
                packRGBA8(round_pixel(colorF + deltaColorF * offsetG.z, 1),
                          round_pixel(colorF + deltaColorF * offsetG.w, 1))));
        dotPos += dotPosDelta;
        dotPosDelta += deltaDelta2;
      }
      // Advance past the portion of gradient we just processed.
      t += inside;
      // If we hit the end of the span, exit out now.
      if (t >= span) {
        break;
      }
      // Otherwise, we are most likely in a transitional section of the gradient
      // between stops that will likely require doing per-sample table lookups.
      // Rather than having to redo all the searching above to figure that out,
      // just assume that to be the case and fall through below to doing the
      // table lookups to hopefully avoid an iteration.
      offset = fastSqrt<true>(dotPos) - radius;
      if (repeat) {
        offset = fract(offset);
      }
    }
    // If we got here, that means we still have span left to process but did not
    // have any whole chunks that fell within a merged gradient. Just fall back
    // to doing a table lookup for each sample.
    Float entry = clamp(offset * size + 1.0f, 0.0f, 1.0f + size);
    commit_blend_span<BLEND>(buf, sampleGradient(sampler, address, entry));
    buf += 4;
    t += 4;
    dotPos += dotPosDelta;
    dotPosDelta += deltaDelta2;
  }
  return true;
}

// Commits an entire span of a radial gradient similar to
// swglcommitLinearGradient, but given a varying 2D position scaled to
// gradient-space and a radius at which the distance from the origin maps to the
// start of the gradient table.
#define swgl_commitRadialGradientRGBA8(sampler, address, size, repeat, pos,    \
                                       radius)                                 \
  do {                                                                         \
    bool drawn = false;                                                        \
    if (blend_key) {                                                           \
      drawn =                                                                  \
          commitRadialGradient<true>(sampler, address, size, repeat, pos,      \
                                     radius, swgl_OutRGBA8, swgl_SpanLength);  \
    } else {                                                                   \
      drawn =                                                                  \
          commitRadialGradient<false>(sampler, address, size, repeat, pos,     \
                                      radius, swgl_OutRGBA8, swgl_SpanLength); \
    }                                                                          \
    if (drawn) {                                                               \
      swgl_OutRGBA8 += swgl_SpanLength;                                        \
      swgl_SpanLength = 0;                                                     \
    }                                                                          \
  } while (0)

// Extension to set a clip mask image to be sampled during blending. The offset
// specifies the positioning of the clip mask image relative to the viewport
// origin. The bounding box specifies the rectangle relative to the clip mask's
// origin that constrains sampling within the clip mask. Blending must be
// enabled for this to work.
static sampler2D swgl_ClipMask = nullptr;
static IntPoint swgl_ClipMaskOffset = {0, 0};
static IntRect swgl_ClipMaskBounds = {0, 0, 0, 0};
#define swgl_clipMask(mask, offset, bb_origin, bb_size)        \
  do {                                                         \
    if (bb_size != vec2_scalar(0.0f, 0.0f)) {                  \
      swgl_ClipFlags |= SWGL_CLIP_FLAG_MASK;                   \
      swgl_ClipMask = mask;                                    \
      swgl_ClipMaskOffset = make_ivec2(offset);                \
      swgl_ClipMaskBounds =                                    \
          IntRect(make_ivec2(bb_origin), make_ivec2(bb_size)); \
    }                                                          \
  } while (0)

// Extension to enable anti-aliasing for the given edges of a quad.
// Blending must be enable for this to work.
static int swgl_AAEdgeMask = 0;

static ALWAYS_INLINE int calcAAEdgeMask(bool on) { return on ? 0xF : 0; }
static ALWAYS_INLINE int calcAAEdgeMask(int mask) { return mask; }
static ALWAYS_INLINE int calcAAEdgeMask(bvec4_scalar mask) {
  return (mask.x ? 1 : 0) | (mask.y ? 2 : 0) | (mask.z ? 4 : 0) |
         (mask.w ? 8 : 0);
}

#define swgl_antiAlias(edges)                \
  do {                                       \
    swgl_AAEdgeMask = calcAAEdgeMask(edges); \
    if (swgl_AAEdgeMask) {                   \
      swgl_ClipFlags |= SWGL_CLIP_FLAG_AA;   \
    }                                        \
  } while (0)

#define swgl_blendDropShadow(color)                         \
  do {                                                      \
    swgl_ClipFlags |= SWGL_CLIP_FLAG_BLEND_OVERRIDE;        \
    swgl_BlendOverride = BLEND_KEY(SWGL_BLEND_DROP_SHADOW); \
    swgl_BlendColorRGBA8 = packColor<uint32_t>(color);      \
  } while (0)

#define swgl_blendSubpixelText(color)                         \
  do {                                                        \
    swgl_ClipFlags |= SWGL_CLIP_FLAG_BLEND_OVERRIDE;          \
    swgl_BlendOverride = BLEND_KEY(SWGL_BLEND_SUBPIXEL_TEXT); \
    swgl_BlendColorRGBA8 = packColor<uint32_t>(color);        \
    swgl_BlendAlphaRGBA8 = alphas(swgl_BlendColorRGBA8);      \
  } while (0)

// Dispatch helper used by the GLSL translator to swgl_drawSpan functions.
// The number of pixels committed is tracked by checking for the difference in
// swgl_SpanLength. Any varying interpolants used will be advanced past the
// committed part of the span in case the fragment shader must be executed for
// any remaining pixels that were not committed by the span shader.
#define DISPATCH_DRAW_SPAN(self, format)        \
  do {                                          \
    int total = self->swgl_SpanLength;          \
    self->swgl_drawSpan##format();              \
    int drawn = total - self->swgl_SpanLength;  \
    if (drawn) self->step_interp_inputs(drawn); \
    return drawn;                               \
  } while (0)
