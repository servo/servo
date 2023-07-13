/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

template <bool COMPOSITE, typename P>
static inline void copy_row(P* dst, const P* src, int span) {
  // No scaling, so just do a fast copy.
  memcpy(dst, src, span * sizeof(P));
}

template <>
void copy_row<true, uint32_t>(uint32_t* dst, const uint32_t* src, int span) {
  // No scaling, so just do a fast composite.
  auto* end = dst + span;
  while (dst + 4 <= end) {
    WideRGBA8 srcpx = unpack(unaligned_load<PackedRGBA8>(src));
    WideRGBA8 dstpx = unpack(unaligned_load<PackedRGBA8>(dst));
    PackedRGBA8 r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    unaligned_store(dst, r);
    src += 4;
    dst += 4;
  }
  if (dst < end) {
    WideRGBA8 srcpx = unpack(partial_load_span<PackedRGBA8>(src, end - dst));
    WideRGBA8 dstpx = unpack(partial_load_span<PackedRGBA8>(dst, end - dst));
    auto r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    partial_store_span(dst, r, end - dst);
  }
}

template <bool COMPOSITE, typename P>
static inline void scale_row(P* dst, int dstWidth, const P* src, int srcWidth,
                             int span, int frac) {
  // Do scaling with different source and dest widths.
  for (P* end = dst + span; dst < end; dst++) {
    *dst = *src;
    // Step source according to width ratio.
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
  }
}

template <>
void scale_row<true, uint32_t>(uint32_t* dst, int dstWidth, const uint32_t* src,
                               int srcWidth, int span, int frac) {
  // Do scaling with different source and dest widths.
  // Gather source pixels four at a time for better packing.
  auto* end = dst + span;
  for (; dst + 4 <= end; dst += 4) {
    U32 srcn;
    srcn.x = *src;
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
    srcn.y = *src;
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
    srcn.z = *src;
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
    srcn.w = *src;
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
    WideRGBA8 srcpx = unpack(bit_cast<PackedRGBA8>(srcn));
    WideRGBA8 dstpx = unpack(unaligned_load<PackedRGBA8>(dst));
    PackedRGBA8 r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    unaligned_store(dst, r);
  }
  if (dst < end) {
    // Process any remaining pixels. Try to gather as many pixels as possible
    // into a single source chunk for compositing.
    U32 srcn = {*src, 0, 0, 0};
    if (end - dst > 1) {
      for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
        src++;
      }
      srcn.y = *src;
      if (end - dst > 2) {
        for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
          src++;
        }
        srcn.z = *src;
      }
    }
    WideRGBA8 srcpx = unpack(bit_cast<PackedRGBA8>(srcn));
    WideRGBA8 dstpx = unpack(partial_load_span<PackedRGBA8>(dst, end - dst));
    auto r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    partial_store_span(dst, r, end - dst);
  }
}

template <bool COMPOSITE = false>
static NO_INLINE void scale_blit(Texture& srctex, const IntRect& srcReq,
                                 Texture& dsttex, const IntRect& dstReq,
                                 bool invertY, const IntRect& clipRect) {
  assert(!COMPOSITE || (srctex.internal_format == GL_RGBA8 &&
                        dsttex.internal_format == GL_RGBA8));
  // Cache scaling ratios
  int srcWidth = srcReq.width();
  int srcHeight = srcReq.height();
  int dstWidth = dstReq.width();
  int dstHeight = dstReq.height();
  // Compute valid dest bounds
  IntRect dstBounds = dsttex.sample_bounds(dstReq).intersect(clipRect);
  // Compute valid source bounds
  IntRect srcBounds = srctex.sample_bounds(srcReq, invertY);
  // If srcReq is outside the source texture, we need to clip the sampling
  // bounds so that we never sample outside valid source bounds. Get texture
  // bounds relative to srcReq and scale to dest-space rounding inward, using
  // this rect to limit the dest bounds further.
  IntRect srcClip = srctex.bounds() - srcReq.origin();
  if (invertY) {
    srcClip.invert_y(srcReq.height());
  }
  srcClip.scale(srcWidth, srcHeight, dstWidth, dstHeight, true);
  dstBounds.intersect(srcClip);
  // Check if clipped sampling bounds are empty
  if (dstBounds.is_empty()) {
    return;
  }

  // Calculate source and dest pointers from clamped offsets
  int bpp = srctex.bpp();
  int srcStride = srctex.stride();
  int destStride = dsttex.stride();
  char* dest = dsttex.sample_ptr(dstReq, dstBounds);
  // Clip the source bounds by the destination offset.
  int fracX = srcWidth * dstBounds.x0;
  int fracY = srcHeight * dstBounds.y0;
  srcBounds.x0 = max(fracX / dstWidth, srcBounds.x0);
  srcBounds.y0 = max(fracY / dstHeight, srcBounds.y0);
  fracX %= dstWidth;
  fracY %= dstHeight;
  char* src = srctex.sample_ptr(srcReq, srcBounds, invertY);
  // Inverted Y must step downward along source rows
  if (invertY) {
    srcStride = -srcStride;
  }
  int span = dstBounds.width();
  for (int rows = dstBounds.height(); rows > 0; rows--) {
    switch (bpp) {
      case 1:
        if (srcWidth == dstWidth)
          copy_row<COMPOSITE>((uint8_t*)dest, (uint8_t*)src, span);
        else
          scale_row<COMPOSITE>((uint8_t*)dest, dstWidth, (uint8_t*)src,
                               srcWidth, span, fracX);
        break;
      case 2:
        if (srcWidth == dstWidth)
          copy_row<COMPOSITE>((uint16_t*)dest, (uint16_t*)src, span);
        else
          scale_row<COMPOSITE>((uint16_t*)dest, dstWidth, (uint16_t*)src,
                               srcWidth, span, fracX);
        break;
      case 4:
        if (srcWidth == dstWidth)
          copy_row<COMPOSITE>((uint32_t*)dest, (uint32_t*)src, span);
        else
          scale_row<COMPOSITE>((uint32_t*)dest, dstWidth, (uint32_t*)src,
                               srcWidth, span, fracX);
        break;
      default:
        assert(false);
        break;
    }
    dest += destStride;
    // Step source according to height ratio.
    for (fracY += srcHeight; fracY >= dstHeight; fracY -= dstHeight) {
      src += srcStride;
    }
  }
}

template <bool COMPOSITE>
static void linear_row_blit(uint32_t* dest, int span, const vec2_scalar& srcUV,
                            float srcDU, sampler2D sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    auto srcpx = textureLinearPackedRGBA8(sampler, ivec2(uv));
    unaligned_store(dest, srcpx);
    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    auto srcpx = textureLinearPackedRGBA8(sampler, ivec2(uv));
    partial_store_span(dest, srcpx, span);
  }
}

template <>
void linear_row_blit<true>(uint32_t* dest, int span, const vec2_scalar& srcUV,
                           float srcDU, sampler2D sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    WideRGBA8 srcpx = textureLinearUnpackedRGBA8(sampler, ivec2(uv));
    WideRGBA8 dstpx = unpack(unaligned_load<PackedRGBA8>(dest));
    PackedRGBA8 r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    unaligned_store(dest, r);

    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    WideRGBA8 srcpx = textureLinearUnpackedRGBA8(sampler, ivec2(uv));
    WideRGBA8 dstpx = unpack(partial_load_span<PackedRGBA8>(dest, span));
    PackedRGBA8 r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
    partial_store_span(dest, r, span);
  }
}

template <bool COMPOSITE>
static void linear_row_blit(uint8_t* dest, int span, const vec2_scalar& srcUV,
                            float srcDU, sampler2D sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    auto srcpx = textureLinearPackedR8(sampler, ivec2(uv));
    unaligned_store(dest, srcpx);
    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    auto srcpx = textureLinearPackedR8(sampler, ivec2(uv));
    partial_store_span(dest, srcpx, span);
  }
}

template <bool COMPOSITE>
static void linear_row_blit(uint16_t* dest, int span, const vec2_scalar& srcUV,
                            float srcDU, sampler2D sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    auto srcpx = textureLinearPackedRG8(sampler, ivec2(uv));
    unaligned_store(dest, srcpx);
    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    auto srcpx = textureLinearPackedRG8(sampler, ivec2(uv));
    partial_store_span(dest, srcpx, span);
  }
}

template <bool COMPOSITE = false>
static NO_INLINE void linear_blit(Texture& srctex, const IntRect& srcReq,
                                  Texture& dsttex, const IntRect& dstReq,
                                  bool invertY, const IntRect& clipRect) {
  assert(srctex.internal_format == GL_RGBA8 ||
         srctex.internal_format == GL_R8 || srctex.internal_format == GL_RG8);
  assert(!COMPOSITE || (srctex.internal_format == GL_RGBA8 &&
                        dsttex.internal_format == GL_RGBA8));
  // Compute valid dest bounds
  IntRect dstBounds = dsttex.sample_bounds(dstReq);
  dstBounds.intersect(clipRect);
  // Check if sampling bounds are empty
  if (dstBounds.is_empty()) {
    return;
  }
  // Initialize sampler for source texture
  sampler2D_impl sampler;
  init_sampler(&sampler, srctex);
  sampler.filter = TextureFilter::LINEAR;
  // Compute source UVs
  vec2_scalar srcUV(srcReq.x0, srcReq.y0);
  vec2_scalar srcDUV(float(srcReq.width()) / dstReq.width(),
                     float(srcReq.height()) / dstReq.height());
  // Inverted Y must step downward along source rows
  if (invertY) {
    srcUV.y += srcReq.height();
    srcDUV.y = -srcDUV.y;
  }
  // Skip to clamped source start
  srcUV += srcDUV * (vec2_scalar(dstBounds.x0, dstBounds.y0) + 0.5f);
  // Scale UVs by lerp precision
  srcUV = linearQuantize(srcUV, 128);
  srcDUV *= 128.0f;
  // Calculate dest pointer from clamped offsets
  int bpp = dsttex.bpp();
  int destStride = dsttex.stride();
  char* dest = dsttex.sample_ptr(dstReq, dstBounds);
  int span = dstBounds.width();
  for (int rows = dstBounds.height(); rows > 0; rows--) {
    switch (bpp) {
      case 1:
        linear_row_blit<COMPOSITE>((uint8_t*)dest, span, srcUV, srcDUV.x,
                                   &sampler);
        break;
      case 2:
        linear_row_blit<COMPOSITE>((uint16_t*)dest, span, srcUV, srcDUV.x,
                                   &sampler);
        break;
      case 4:
        linear_row_blit<COMPOSITE>((uint32_t*)dest, span, srcUV, srcDUV.x,
                                   &sampler);
        break;
      default:
        assert(false);
        break;
    }
    dest += destStride;
    srcUV.y += srcDUV.y;
  }
}

extern "C" {

void BlitFramebuffer(GLint srcX0, GLint srcY0, GLint srcX1, GLint srcY1,
                     GLint dstX0, GLint dstY0, GLint dstX1, GLint dstY1,
                     GLbitfield mask, GLenum filter) {
  assert(mask == GL_COLOR_BUFFER_BIT);
  Framebuffer* srcfb = get_framebuffer(GL_READ_FRAMEBUFFER);
  if (!srcfb) return;
  Framebuffer* dstfb = get_framebuffer(GL_DRAW_FRAMEBUFFER);
  if (!dstfb) return;
  Texture& srctex = ctx->textures[srcfb->color_attachment];
  if (!srctex.buf) return;
  Texture& dsttex = ctx->textures[dstfb->color_attachment];
  if (!dsttex.buf) return;
  assert(!dsttex.locked);
  if (srctex.internal_format != dsttex.internal_format) {
    assert(false);
    return;
  }
  // Force flipped Y onto dest coordinates
  if (srcY1 < srcY0) {
    swap(srcY0, srcY1);
    swap(dstY0, dstY1);
  }
  bool invertY = dstY1 < dstY0;
  if (invertY) {
    swap(dstY0, dstY1);
  }
  IntRect srcReq = IntRect{srcX0, srcY0, srcX1, srcY1} - srctex.offset;
  IntRect dstReq = IntRect{dstX0, dstY0, dstX1, dstY1} - dsttex.offset;
  if (srcReq.is_empty() || dstReq.is_empty()) {
    return;
  }
  IntRect clipRect = {0, 0, dstReq.width(), dstReq.height()};
  prepare_texture(srctex);
  prepare_texture(dsttex, &dstReq);
  if (!srcReq.same_size(dstReq) && srctex.width >= 2 && filter == GL_LINEAR &&
      (srctex.internal_format == GL_RGBA8 || srctex.internal_format == GL_R8 ||
       srctex.internal_format == GL_RG8)) {
    linear_blit(srctex, srcReq, dsttex, dstReq, invertY, dstReq);
  } else {
    scale_blit(srctex, srcReq, dsttex, dstReq, invertY, clipRect);
  }
}

typedef Texture LockedTexture;

// Lock the given texture to prevent modification.
LockedTexture* LockTexture(GLuint texId) {
  Texture& tex = ctx->textures[texId];
  if (!tex.buf) {
    assert(tex.buf != nullptr);
    return nullptr;
  }
  if (__sync_fetch_and_add(&tex.locked, 1) == 0) {
    // If this is the first time locking the texture, flush any delayed clears.
    prepare_texture(tex);
  }
  return (LockedTexture*)&tex;
}

// Lock the given framebuffer's color attachment to prevent modification.
LockedTexture* LockFramebuffer(GLuint fboId) {
  Framebuffer& fb = ctx->framebuffers[fboId];
  // Only allow locking a framebuffer if it has a valid color attachment.
  if (!fb.color_attachment) {
    assert(fb.color_attachment != 0);
    return nullptr;
  }
  return LockTexture(fb.color_attachment);
}

// Reference an already locked resource
void LockResource(LockedTexture* resource) {
  if (!resource) {
    return;
  }
  __sync_fetch_and_add(&resource->locked, 1);
}

// Remove a lock on a texture that has been previously locked
void UnlockResource(LockedTexture* resource) {
  if (!resource) {
    return;
  }
  if (__sync_fetch_and_add(&resource->locked, -1) <= 0) {
    // The lock should always be non-zero before unlocking.
    assert(0);
  }
}

// Get the underlying buffer for a locked resource
void* GetResourceBuffer(LockedTexture* resource, int32_t* width,
                        int32_t* height, int32_t* stride) {
  *width = resource->width;
  *height = resource->height;
  *stride = resource->stride();
  return resource->buf;
}

// Extension for optimized compositing of textures or framebuffers that may be
// safely used across threads. The source and destination must be locked to
// ensure that they can be safely accessed while the SWGL context might be used
// by another thread. Band extents along the Y axis may be used to clip the
// destination rectangle without effecting the integer scaling ratios.
void Composite(LockedTexture* lockedDst, LockedTexture* lockedSrc, GLint srcX,
               GLint srcY, GLsizei srcWidth, GLsizei srcHeight, GLint dstX,
               GLint dstY, GLsizei dstWidth, GLsizei dstHeight,
               GLboolean opaque, GLboolean flip, GLenum filter, GLint clipX,
               GLint clipY, GLsizei clipWidth, GLsizei clipHeight) {
  if (!lockedDst || !lockedSrc) {
    return;
  }
  Texture& srctex = *lockedSrc;
  Texture& dsttex = *lockedDst;
  assert(srctex.bpp() == 4);
  assert(dsttex.bpp() == 4);

  IntRect srcReq =
      IntRect{srcX, srcY, srcX + srcWidth, srcY + srcHeight} - srctex.offset;
  IntRect dstReq =
      IntRect{dstX, dstY, dstX + dstWidth, dstY + dstHeight} - dsttex.offset;
  // Compute clip rect as relative to the dstReq, as that's the same coords
  // as used for the sampling bounds.
  IntRect clipRect = {clipX - dstX, clipY - dstY, clipX - dstX + clipWidth,
                      clipY - dstY + clipHeight};

  if (opaque) {
    // Ensure we have rows of at least 2 pixels when using the linear filter
    // to avoid overreading the row.
    if (!srcReq.same_size(dstReq) && srctex.width >= 2 && filter == GL_LINEAR) {
      linear_blit<false>(srctex, srcReq, dsttex, dstReq, flip, clipRect);
    } else {
      scale_blit<false>(srctex, srcReq, dsttex, dstReq, flip, clipRect);
    }
  } else {
    if (!srcReq.same_size(dstReq) && srctex.width >= 2 && filter == GL_LINEAR) {
      linear_blit<true>(srctex, srcReq, dsttex, dstReq, flip, clipRect);
    } else {
      scale_blit<true>(srctex, srcReq, dsttex, dstReq, flip, clipRect);
    }
  }
}

}  // extern "C"

// Saturated add helper for YUV conversion. Supported platforms have intrinsics
// to do this natively, but support a slower generic fallback just in case.
static inline V8<int16_t> addsat(V8<int16_t> x, V8<int16_t> y) {
#if USE_SSE2
  return _mm_adds_epi16(x, y);
#elif USE_NEON
  return vqaddq_s16(x, y);
#else
  auto r = x + y;
  // An overflow occurred if the signs of both inputs x and y did not differ
  // but yet the sign of the result did differ.
  auto overflow = (~(x ^ y) & (r ^ x)) >> 15;
  // If there was an overflow, we need to choose the appropriate limit to clamp
  // to depending on whether or not the inputs are negative.
  auto limit = (x >> 15) ^ 0x7FFF;
  // If we didn't overflow, just use the result, and otherwise, use the limit.
  return (~overflow & r) | (overflow & limit);
#endif
}

// Interleave and packing helper for YUV conversion. During transform by the
// color matrix, the color components are de-interleaved as this format is
// usually what comes out of the planar YUV textures. The components thus need
// to be interleaved before finally getting packed to BGRA format. Alpha is
// forced to be opaque.
static inline PackedRGBA8 packYUV(V8<int16_t> gg, V8<int16_t> br) {
  return pack(bit_cast<WideRGBA8>(zip(br, gg))) |
         PackedRGBA8{0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255};
}

// clang-format off
// Supports YUV color matrixes of the form:
// [R]   [1.1643835616438356,  0.0,  rv ]   [Y -  16]
// [G] = [1.1643835616438358, -gu,  -gv ] x [U - 128]
// [B]   [1.1643835616438356,  bu,  0.0 ]   [V - 128]
// We must be able to multiply a YUV input by a matrix coefficient ranging as
// high as ~2.2 in the U/V cases, where U/V can be signed values between -128
// and 127. The largest fixed-point representation we can thus support without
// overflowing 16 bit integers leaves us 6 bits of fractional precision while
// also supporting a sign bit. The closest representation of the Y coefficient
// ~1.164 in this precision is 74.5/2^6 which is common to all color spaces
// we support. Conversions can still sometimes overflow the precision and
// require clamping back into range, so we use saturated additions to do this
// efficiently at no extra cost.
// clang-format on
struct YUVMatrix {
  // These constants are loaded off the "this" pointer via relative addressing
  // modes and should be about as quick to load as directly addressed SIMD
  // constant memory.
  V8<int16_t> rbCoeffs;
  V8<int16_t> gCoeffs;
  V8<uint16_t> yScale;
  V8<int16_t> yBias;
  V8<int16_t> uvBias;
  V8<int16_t> brMask;

  // Set the coefficients to cancel out and pass through YUV as GBR. All biases
  // are set to zero and the BR-mask is set to remove the contribution of Y to
  // the BR channels. Scales are set such that the shift by 6 in convert is
  // balanced.
  YUVMatrix()
      : rbCoeffs(1 << 6),
        gCoeffs(0),
        yScale(1 << (6 + 1)),
        yBias(0),
        uvBias(0),
        brMask(0) {}

  // Convert matrix coefficients to fixed-point representation.
  YUVMatrix(double rv, double gu, double gv, double bu)
      : rbCoeffs(
            zip(I16(int16_t(bu * 64.0 + 0.5)), I16(int16_t(rv * 64.0 + 0.5)))),
        gCoeffs(zip(I16(-int16_t(gu * -64.0 + 0.5)),
                    I16(-int16_t(gv * -64.0 + 0.5)))),
        yScale(2 * 74 + 1),
        yBias(int16_t(-16 * 74.5) + (1 << 5)),
        uvBias(-128),
        brMask(-1) {}

  ALWAYS_INLINE PackedRGBA8 convert(V8<int16_t> yy, V8<int16_t> uv) const {
    // Bias Y values by -16 and multiply by 74.5. Add 2^5 offset to round to
    // nearest 2^6. Note that we have to use an unsigned multiply with a 2x
    // scale to represent a fractional scale and to avoid shifting with the sign
    // bit.
    yy = bit_cast<V8<int16_t>>((bit_cast<V8<uint16_t>>(yy) * yScale) >> 1) +
         yBias;

    // Bias U/V values by -128.
    uv += uvBias;

    // Compute (R, B) = (74.5*Y + rv*V, 74.5*Y + bu*U)
    auto br = rbCoeffs * uv;
    br = addsat(yy & brMask, br);
    br >>= 6;

    // Compute G = 74.5*Y + -gu*U + -gv*V
    auto gg = gCoeffs * uv;
    gg = addsat(
        yy,
        addsat(gg, bit_cast<V8<int16_t>>(bit_cast<V4<uint32_t>>(gg) >> 16)));
    gg >>= 6;

    // Interleave B/R and G values. Force alpha to opaque.
    return packYUV(gg, br);
  }
};

enum YUVColorSpace { REC_601 = 0, REC_709, REC_2020, IDENTITY };

static const YUVMatrix yuvMatrix[IDENTITY + 1] = {
    // clang-format off
// From Rec601:
// [R]   [1.1643835616438356,  0.0,                 1.5960267857142858   ]   [Y -  16]
// [G] = [1.1643835616438358, -0.3917622900949137, -0.8129676472377708   ] x [U - 128]
// [B]   [1.1643835616438356,  2.017232142857143,   8.862867620416422e-17]   [V - 128]
  {1.5960267857142858, -0.3917622900949137, -0.8129676472377708, 2.017232142857143},

// From Rec709:
// [R]   [1.1643835616438356,  0.0,                  1.7927410714285714]   [Y -  16]
// [G] = [1.1643835616438358, -0.21324861427372963, -0.532909328559444 ] x [U - 128]
// [B]   [1.1643835616438356,  2.1124017857142854,   0.0               ]   [V - 128]
  {1.7927410714285714, -0.21324861427372963, -0.532909328559444, 2.1124017857142854},

// From Re2020:
// [R]   [1.16438356164384,  0.0,                1.678674107142860 ]   [Y -  16]
// [G] = [1.16438356164384, -0.187326104219343, -0.650424318505057 ] x [U - 128]
// [B]   [1.16438356164384,  2.14177232142857,   0.0               ]   [V - 128]
  {1.678674107142860, -0.187326104219343, -0.650424318505057, 2.14177232142857},

// Identity
// [R]   [V]
// [G] = [Y]
// [B]   [U]
  {},
    // clang-format on
};

// Helper function for textureLinearRowR8 that samples horizontal taps and
// combines them based on Y fraction with next row.
template <typename S>
static ALWAYS_INLINE V8<int16_t> linearRowTapsR8(S sampler, I32 ix,
                                                 int32_t offsety,
                                                 int32_t stridey,
                                                 int16_t fracy) {
  uint8_t* buf = (uint8_t*)sampler->buf + offsety;
  auto a0 = unaligned_load<V2<uint8_t>>(&buf[ix.x]);
  auto b0 = unaligned_load<V2<uint8_t>>(&buf[ix.y]);
  auto c0 = unaligned_load<V2<uint8_t>>(&buf[ix.z]);
  auto d0 = unaligned_load<V2<uint8_t>>(&buf[ix.w]);
  auto abcd0 = CONVERT(combine(a0, b0, c0, d0), V8<int16_t>);
  buf += stridey;
  auto a1 = unaligned_load<V2<uint8_t>>(&buf[ix.x]);
  auto b1 = unaligned_load<V2<uint8_t>>(&buf[ix.y]);
  auto c1 = unaligned_load<V2<uint8_t>>(&buf[ix.z]);
  auto d1 = unaligned_load<V2<uint8_t>>(&buf[ix.w]);
  auto abcd1 = CONVERT(combine(a1, b1, c1, d1), V8<int16_t>);
  abcd0 += ((abcd1 - abcd0) * fracy) >> 7;
  return abcd0;
}

// Optimized version of textureLinearPackedR8 for Y R8 texture. This assumes
// constant Y and returns a duplicate of the result interleaved with itself
// to aid in later YUV transformation.
template <typename S>
static inline V8<int16_t> textureLinearRowR8(S sampler, I32 ix, int32_t offsety,
                                             int32_t stridey, int16_t fracy) {
  assert(sampler->format == TextureFormat::R8);

  // Calculate X fraction and clamp X offset into range.
  I32 fracx = ix;
  ix >>= 7;
  fracx = ((fracx & (ix >= 0)) | (ix > int32_t(sampler->width) - 2)) & 0x7F;
  ix = clampCoord(ix, sampler->width - 1);

  // Load the sample taps and combine rows.
  auto abcd = linearRowTapsR8(sampler, ix, offsety, stridey, fracy);

  // Unzip the result and do final horizontal multiply-add base on X fraction.
  auto abcdl = SHUFFLE(abcd, abcd, 0, 0, 2, 2, 4, 4, 6, 6);
  auto abcdh = SHUFFLE(abcd, abcd, 1, 1, 3, 3, 5, 5, 7, 7);
  abcdl += ((abcdh - abcdl) * CONVERT(fracx, I16).xxyyzzww) >> 7;

  // The final result is the packed values interleaved with a duplicate of
  // themselves.
  return abcdl;
}

// Optimized version of textureLinearPackedR8 for paired U/V R8 textures.
// Since the two textures have the same dimensions and stride, the addressing
// math can be shared between both samplers. This also allows a coalesced
// multiply in the final stage by packing both U/V results into a single
// operation.
template <typename S>
static inline V8<int16_t> textureLinearRowPairedR8(S sampler, S sampler2,
                                                   I32 ix, int32_t offsety,
                                                   int32_t stridey,
                                                   int16_t fracy) {
  assert(sampler->format == TextureFormat::R8 &&
         sampler2->format == TextureFormat::R8);
  assert(sampler->width == sampler2->width &&
         sampler->height == sampler2->height);
  assert(sampler->stride == sampler2->stride);

  // Calculate X fraction and clamp X offset into range.
  I32 fracx = ix;
  ix >>= 7;
  fracx = ((fracx & (ix >= 0)) | (ix > int32_t(sampler->width) - 2)) & 0x7F;
  ix = clampCoord(ix, sampler->width - 1);

  // Load the sample taps for the first sampler and combine rows.
  auto abcd = linearRowTapsR8(sampler, ix, offsety, stridey, fracy);

  // Load the sample taps for the second sampler and combine rows.
  auto xyzw = linearRowTapsR8(sampler2, ix, offsety, stridey, fracy);

  // We are left with a result vector for each sampler with values for adjacent
  // pixels interleaved together in each. We need to unzip these values so that
  // we can do the final horizontal multiply-add based on the X fraction.
  auto abcdxyzwl = SHUFFLE(abcd, xyzw, 0, 8, 2, 10, 4, 12, 6, 14);
  auto abcdxyzwh = SHUFFLE(abcd, xyzw, 1, 9, 3, 11, 5, 13, 7, 15);
  abcdxyzwl += ((abcdxyzwh - abcdxyzwl) * CONVERT(fracx, I16).xxyyzzww) >> 7;

  // The final result is the packed values for the first sampler interleaved
  // with the packed values for the second sampler.
  return abcdxyzwl;
}

// Casting to int loses some precision while stepping that can offset the
// image, so shift the values by some extra bits of precision to minimize
// this. We support up to 16 bits of image size, 7 bits of quantization,
// and 1 bit for sign, which leaves 8 bits left for extra precision.
const int STEP_BITS = 8;

// Optimized version of textureLinearPackedR8 for Y R8 texture with
// half-resolution paired U/V R8 textures. This allows us to more efficiently
// pack YUV samples into vectors to substantially reduce math operations even
// further.
template <bool BLEND>
static inline void upscaleYUV42R8(uint32_t* dest, int span, uint8_t* yRow,
                                  I32 yU, int32_t yDU, int32_t yStrideV,
                                  int16_t yFracV, uint8_t* cRow1,
                                  uint8_t* cRow2, I32 cU, int32_t cDU,
                                  int32_t cStrideV, int16_t cFracV,
                                  const YUVMatrix& colorSpace) {
  // As much as possible try to utilize the fact that we're only using half
  // the UV samples to combine Y and UV samples into single vectors. Here we
  // need to initialize several useful vector quantities for stepping fractional
  // offsets. For the UV samples, we take the average of the first+second and
  // third+fourth samples in a chunk which conceptually correspond to offsets
  // 0.5 and 1.5 (in 0..2 range). This allows us to reconstruct intermediate
  // samples 0.25, 0.75, 1.25, and 1.75 later. X fraction is shifted over into
  // the top 7 bits of an unsigned short so that we can mask off the exact
  // fractional bits we need to blend merely by right shifting them into
  // position.
  cU = (cU.xzxz + cU.ywyw) >> 1;
  auto ycFracX = CONVERT(combine(yU, cU), V8<uint16_t>)
                 << (16 - (STEP_BITS + 7));
  auto ycFracDX = combine(I16(yDU), I16(cDU)) << (16 - (STEP_BITS + 7));
  auto ycFracV = combine(I16(yFracV), I16(cFracV));
  I32 yI = yU >> (STEP_BITS + 7);
  I32 cI = cU >> (STEP_BITS + 7);
  // Load initial combined YUV samples for each row and blend them.
  auto ycSrc0 =
      CONVERT(combine(unaligned_load<V4<uint8_t>>(&yRow[yI.x]),
                      combine(unaligned_load<V2<uint8_t>>(&cRow1[cI.x]),
                              unaligned_load<V2<uint8_t>>(&cRow2[cI.x]))),
              V8<int16_t>);
  auto ycSrc1 = CONVERT(
      combine(unaligned_load<V4<uint8_t>>(&yRow[yI.x + yStrideV]),
              combine(unaligned_load<V2<uint8_t>>(&cRow1[cI.x + cStrideV]),
                      unaligned_load<V2<uint8_t>>(&cRow2[cI.x + cStrideV]))),
      V8<int16_t>);
  auto ycSrc = ycSrc0 + (((ycSrc1 - ycSrc0) * ycFracV) >> 7);

  // Here we shift in results from the next sample while caching results from
  // the previous sample. This allows us to reduce the multiplications in the
  // inner loop down to only two since we just need to blend the new samples
  // horizontally and then vertically once each.
  for (uint32_t* end = dest + span; dest < end; dest += 4) {
    yU += yDU;
    I32 yIn = yU >> (STEP_BITS + 7);
    cU += cDU;
    I32 cIn = cU >> (STEP_BITS + 7);
    // Load combined YUV samples for the next chunk on each row and blend them.
    auto ycSrc0n =
        CONVERT(combine(unaligned_load<V4<uint8_t>>(&yRow[yIn.x]),
                        combine(unaligned_load<V2<uint8_t>>(&cRow1[cIn.x]),
                                unaligned_load<V2<uint8_t>>(&cRow2[cIn.x]))),
                V8<int16_t>);
    auto ycSrc1n = CONVERT(
        combine(unaligned_load<V4<uint8_t>>(&yRow[yIn.x + yStrideV]),
                combine(unaligned_load<V2<uint8_t>>(&cRow1[cIn.x + cStrideV]),
                        unaligned_load<V2<uint8_t>>(&cRow2[cIn.x + cStrideV]))),
        V8<int16_t>);
    auto ycSrcn = ycSrc0n + (((ycSrc1n - ycSrc0n) * ycFracV) >> 7);

    // The source samples for the chunk may not match the actual tap offsets.
    // Since we're upscaling, we know the tap offsets fall within all the
    // samples in a 4-wide chunk. Since we can't rely on PSHUFB or similar,
    // instead we do laborious shuffling here for the Y samples and then the UV
    // samples.
    auto yshuf = lowHalf(ycSrc);
    auto yshufn =
        SHUFFLE(yshuf, yIn.x == yI.w ? lowHalf(ycSrcn).yyyy : lowHalf(ycSrcn),
                1, 2, 3, 4);
    if (yI.y == yI.x) {
      yshuf = yshuf.xxyz;
      yshufn = yshufn.xxyz;
    }
    if (yI.z == yI.y) {
      yshuf = yshuf.xyyz;
      yshufn = yshufn.xyyz;
    }
    if (yI.w == yI.z) {
      yshuf = yshuf.xyzz;
      yshufn = yshufn.xyzz;
    }

    auto cshuf = highHalf(ycSrc);
    auto cshufn =
        SHUFFLE(cshuf, cIn.x == cI.y ? highHalf(ycSrcn).yyww : highHalf(ycSrcn),
                1, 4, 3, 6);
    if (cI.y == cI.x) {
      cshuf = cshuf.xxzz;
      cshufn = cshufn.xxzz;
    }

    // After shuffling, combine the Y and UV samples back into a single vector
    // for blending. Shift X fraction into position as unsigned to mask off top
    // bits and get rid of low bits to avoid multiplication overflow.
    auto yuvPx = combine(yshuf, cshuf);
    yuvPx += ((combine(yshufn, cshufn) - yuvPx) *
              bit_cast<V8<int16_t>>(ycFracX >> (16 - 7))) >>
             7;

    // Cache the new samples as the current samples on the next iteration.
    ycSrc = ycSrcn;
    ycFracX += ycFracDX;
    yI = yIn;
    cI = cIn;

    // De-interleave the Y and UV results. We need to average the UV results
    // to produce values for intermediate samples. Taps for UV were collected at
    // offsets 0.5 and 1.5, such that if we take a quarter of the difference
    // (1.5-0.5)/4, subtract it from even samples, and add it to odd samples,
    // we can estimate samples 0.25, 0.75, 1.25, and 1.75.
    auto yPx = SHUFFLE(yuvPx, yuvPx, 0, 0, 1, 1, 2, 2, 3, 3);
    auto uvPx = SHUFFLE(yuvPx, yuvPx, 4, 6, 4, 6, 5, 7, 5, 7) +
                ((SHUFFLE(yuvPx, yuvPx, 4, 6, 5, 7, 4, 6, 5, 7) -
                  SHUFFLE(yuvPx, yuvPx, 5, 7, 4, 6, 5, 7, 4, 6)) >>
                 2);

    commit_blend_span<BLEND>(dest, colorSpace.convert(yPx, uvPx));
  }
}

// This is the inner loop driver of CompositeYUV that processes an axis-aligned
// YUV span, dispatching based on appropriate format and scaling. This is also
// reused by blendYUV to accelerate some cases of texture sampling in the
// shader.
template <bool BLEND = false>
static void linear_row_yuv(uint32_t* dest, int span, sampler2DRect samplerY,
                           const vec2_scalar& srcUV, float srcDU,
                           sampler2DRect samplerU, sampler2DRect samplerV,
                           const vec2_scalar& chromaUV, float chromaDU,
                           int colorDepth, const YUVMatrix& colorSpace) {
  // Calculate varying and constant interp data for Y plane.
  I32 yU = cast(init_interp(srcUV.x, srcDU) * (1 << STEP_BITS));
  int32_t yV = int32_t(srcUV.y);

  // Calculate varying and constant interp data for chroma planes.
  I32 cU = cast(init_interp(chromaUV.x, chromaDU) * (1 << STEP_BITS));
  int32_t cV = int32_t(chromaUV.y);

  // We need to skip 4 pixels per chunk.
  int32_t yDU = int32_t((4 << STEP_BITS) * srcDU);
  int32_t cDU = int32_t((4 << STEP_BITS) * chromaDU);

  if (samplerY->width < 2 || samplerU->width < 2) {
    // If the source row has less than 2 pixels, it's not safe to use a linear
    // filter because it may overread the row. Just convert the single pixel
    // with nearest filtering and fill the row with it.
    I16 yuv = CONVERT(
        round_pixel((Float){texelFetch(samplerY, ivec2(srcUV)).x.x,
                            texelFetch(samplerU, ivec2(chromaUV)).x.x,
                            texelFetch(samplerV, ivec2(chromaUV)).x.x, 1.0f}),
        I16);
    commit_solid_span<BLEND>(
        dest,
        unpack(colorSpace.convert(V8<int16_t>(yuv.x),
                                  zip(I16(yuv.y), I16(yuv.z)))),
        span);
  } else if (samplerY->format == TextureFormat::R16) {
    // Sample each YUV plane, rescale it to fit in low 8 bits of word, and
    // then transform them by the appropriate color space.
    assert(colorDepth > 8);
    // Need to right shift the sample by the amount of bits over 8 it
    // occupies. On output from textureLinearUnpackedR16, we have lost 1 bit
    // of precision at the low end already, hence 1 is subtracted from the
    // color depth.
    int rescaleBits = (colorDepth - 1) - 8;
    for (; span >= 4; span -= 4) {
      auto yPx =
          textureLinearUnpackedR16(samplerY, ivec2(yU >> STEP_BITS, yV)) >>
          rescaleBits;
      auto uPx =
          textureLinearUnpackedR16(samplerU, ivec2(cU >> STEP_BITS, cV)) >>
          rescaleBits;
      auto vPx =
          textureLinearUnpackedR16(samplerV, ivec2(cU >> STEP_BITS, cV)) >>
          rescaleBits;
      commit_blend_span<BLEND>(
          dest, colorSpace.convert(zip(yPx, yPx), zip(uPx, vPx)));
      dest += 4;
      yU += yDU;
      cU += cDU;
    }
    if (span > 0) {
      // Handle any remaining pixels...
      auto yPx =
          textureLinearUnpackedR16(samplerY, ivec2(yU >> STEP_BITS, yV)) >>
          rescaleBits;
      auto uPx =
          textureLinearUnpackedR16(samplerU, ivec2(cU >> STEP_BITS, cV)) >>
          rescaleBits;
      auto vPx =
          textureLinearUnpackedR16(samplerV, ivec2(cU >> STEP_BITS, cV)) >>
          rescaleBits;
      commit_blend_span<BLEND>(
          dest, colorSpace.convert(zip(yPx, yPx), zip(uPx, vPx)), span);
    }
  } else {
    assert(samplerY->format == TextureFormat::R8);
    assert(colorDepth == 8);

    // Calculate varying and constant interp data for Y plane.
    int16_t yFracV = yV & 0x7F;
    yV >>= 7;
    int32_t yOffsetV = clampCoord(yV, samplerY->height) * samplerY->stride;
    int32_t yStrideV =
        yV >= 0 && yV < int32_t(samplerY->height) - 1 ? samplerY->stride : 0;

    // Calculate varying and constant interp data for chroma planes.
    int16_t cFracV = cV & 0x7F;
    cV >>= 7;
    int32_t cOffsetV = clampCoord(cV, samplerU->height) * samplerU->stride;
    int32_t cStrideV =
        cV >= 0 && cV < int32_t(samplerU->height) - 1 ? samplerU->stride : 0;

    // If we're sampling the UV planes at half the resolution of the Y plane,
    // then try to use half resolution fast-path.
    if (yDU >= cDU && cDU > 0 && yDU <= (4 << (STEP_BITS + 7)) &&
        cDU <= (2 << (STEP_BITS + 7))) {
      // Ensure that samples don't fall outside of the valid bounds of each
      // planar texture. Step until the initial X coordinates are positive.
      for (; (yU.x < 0 || cU.x < 0) && span >= 4; span -= 4) {
        auto yPx = textureLinearRowR8(samplerY, yU >> STEP_BITS, yOffsetV,
                                      yStrideV, yFracV);
        auto uvPx = textureLinearRowPairedR8(
            samplerU, samplerV, cU >> STEP_BITS, cOffsetV, cStrideV, cFracV);
        commit_blend_span<BLEND>(dest, colorSpace.convert(yPx, uvPx));
        dest += 4;
        yU += yDU;
        cU += cDU;
      }
      // Calculate the number of aligned chunks that we can step inside the
      // bounds of each planar texture without overreading.
      int inside = min(
          min((((int(samplerY->width) - 4) << (STEP_BITS + 7)) - yU.x) / yDU,
              (((int(samplerU->width) - 4) << (STEP_BITS + 7)) - cU.x) / cDU) *
              4,
          span & ~3);
      if (inside > 0) {
        uint8_t* yRow = (uint8_t*)samplerY->buf + yOffsetV;
        uint8_t* cRow1 = (uint8_t*)samplerU->buf + cOffsetV;
        uint8_t* cRow2 = (uint8_t*)samplerV->buf + cOffsetV;
        upscaleYUV42R8<BLEND>(dest, inside, yRow, yU, yDU, yStrideV, yFracV,
                              cRow1, cRow2, cU, cDU, cStrideV, cFracV,
                              colorSpace);
        span -= inside;
        dest += inside;
        yU += (inside / 4) * yDU;
        cU += (inside / 4) * cDU;
      }
      // If there are any remaining chunks that weren't inside, handle them
      // below.
    }
    for (; span >= 4; span -= 4) {
      // Sample each YUV plane and then transform them by the appropriate
      // color space.
      auto yPx = textureLinearRowR8(samplerY, yU >> STEP_BITS, yOffsetV,
                                    yStrideV, yFracV);
      auto uvPx = textureLinearRowPairedR8(samplerU, samplerV, cU >> STEP_BITS,
                                           cOffsetV, cStrideV, cFracV);
      commit_blend_span<BLEND>(dest, colorSpace.convert(yPx, uvPx));
      dest += 4;
      yU += yDU;
      cU += cDU;
    }
    if (span > 0) {
      // Handle any remaining pixels...
      auto yPx = textureLinearRowR8(samplerY, yU >> STEP_BITS, yOffsetV,
                                    yStrideV, yFracV);
      auto uvPx = textureLinearRowPairedR8(samplerU, samplerV, cU >> STEP_BITS,
                                           cOffsetV, cStrideV, cFracV);
      commit_blend_span<BLEND>(dest, colorSpace.convert(yPx, uvPx), span);
    }
  }
}

static void linear_convert_yuv(Texture& ytex, Texture& utex, Texture& vtex,
                               YUVColorSpace colorSpace, int colorDepth,
                               const IntRect& srcReq, Texture& dsttex,
                               const IntRect& dstReq, bool invertY,
                               const IntRect& clipRect) {
  // Compute valid dest bounds
  IntRect dstBounds = dsttex.sample_bounds(dstReq, invertY);
  dstBounds.intersect(clipRect);
  // Check if sampling bounds are empty
  if (dstBounds.is_empty()) {
    return;
  }
  // Initialize samplers for source textures
  sampler2DRect_impl sampler[3];
  init_sampler(&sampler[0], ytex);
  init_sampler(&sampler[1], utex);
  init_sampler(&sampler[2], vtex);

  // Compute source UVs
  vec2_scalar srcUV(srcReq.x0, srcReq.y0);
  vec2_scalar srcDUV(float(srcReq.width()) / dstReq.width(),
                     float(srcReq.height()) / dstReq.height());
  // Inverted Y must step downward along source rows
  if (invertY) {
    srcUV.y += srcReq.height();
    srcDUV.y = -srcDUV.y;
  }
  // Skip to clamped source start
  srcUV += srcDUV * (vec2_scalar(dstBounds.x0, dstBounds.y0) + 0.5f);
  // Calculate separate chroma UVs for chroma planes with different scale
  vec2_scalar chromaScale(float(utex.width) / ytex.width,
                          float(utex.height) / ytex.height);
  vec2_scalar chromaUV = srcUV * chromaScale;
  vec2_scalar chromaDUV = srcDUV * chromaScale;
  // Scale UVs by lerp precision. If the row has only 1 pixel, then don't
  // quantize so that we can use nearest filtering instead to avoid overreads.
  if (ytex.width >= 2 && utex.width >= 2) {
    srcUV = linearQuantize(srcUV, 128);
    srcDUV *= 128.0f;
    chromaUV = linearQuantize(chromaUV, 128);
    chromaDUV *= 128.0f;
  }
  // Calculate dest pointer from clamped offsets
  int destStride = dsttex.stride();
  char* dest = dsttex.sample_ptr(dstReq, dstBounds);
  int span = dstBounds.width();
  for (int rows = dstBounds.height(); rows > 0; rows--) {
    linear_row_yuv((uint32_t*)dest, span, &sampler[0], srcUV, srcDUV.x,
                   &sampler[1], &sampler[2], chromaUV, chromaDUV.x, colorDepth,
                   yuvMatrix[colorSpace]);
    dest += destStride;
    srcUV.y += srcDUV.y;
    chromaUV.y += chromaDUV.y;
  }
}

extern "C" {

// Extension for compositing a YUV surface represented by separate YUV planes
// to a BGRA destination. The supplied color space is used to determine the
// transform from YUV to BGRA after sampling.
void CompositeYUV(LockedTexture* lockedDst, LockedTexture* lockedY,
                  LockedTexture* lockedU, LockedTexture* lockedV,
                  YUVColorSpace colorSpace, GLuint colorDepth, GLint srcX,
                  GLint srcY, GLsizei srcWidth, GLsizei srcHeight, GLint dstX,
                  GLint dstY, GLsizei dstWidth, GLsizei dstHeight,
                  GLboolean flip, GLint clipX, GLint clipY, GLsizei clipWidth,
                  GLsizei clipHeight) {
  if (!lockedDst || !lockedY || !lockedU || !lockedV) {
    return;
  }
  if (colorSpace > IDENTITY) {
    assert(false);
    return;
  }
  Texture& ytex = *lockedY;
  Texture& utex = *lockedU;
  Texture& vtex = *lockedV;
  Texture& dsttex = *lockedDst;
  // All YUV planes must currently be represented by R8 or R16 textures.
  // The chroma (U/V) planes must have matching dimensions.
  assert(ytex.bpp() == utex.bpp() && ytex.bpp() == vtex.bpp());
  assert((ytex.bpp() == 1 && colorDepth == 8) ||
         (ytex.bpp() == 2 && colorDepth > 8));
  // assert(ytex.width == utex.width && ytex.height == utex.height);
  assert(utex.width == vtex.width && utex.height == vtex.height);
  assert(ytex.offset == utex.offset && ytex.offset == vtex.offset);
  assert(dsttex.bpp() == 4);

  IntRect srcReq =
      IntRect{srcX, srcY, srcX + srcWidth, srcY + srcHeight} - ytex.offset;
  IntRect dstReq =
      IntRect{dstX, dstY, dstX + dstWidth, dstY + dstHeight} - dsttex.offset;
  // Compute clip rect as relative to the dstReq, as that's the same coords
  // as used for the sampling bounds.
  IntRect clipRect = {clipX - dstX, clipY - dstY, clipX - dstX + clipWidth,
                      clipY - dstY + clipHeight};
  // For now, always use a linear filter path that would be required for
  // scaling. Further fast-paths for non-scaled video might be desirable in the
  // future.
  linear_convert_yuv(ytex, utex, vtex, colorSpace, colorDepth, srcReq, dsttex,
                     dstReq, flip, clipRect);
}

}  // extern "C"
