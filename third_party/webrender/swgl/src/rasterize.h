/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The SWGL depth buffer is roughly organized as a span buffer where each row
// of the depth buffer is a list of spans, and each span has a constant depth
// and a run length (represented by DepthRun). The span from start..start+count
// is placed directly at that start index in the row's array of runs, so that
// there is no need to explicitly record the start index at all. This also
// avoids the need to move items around in the run array to manage insertions
// since space is implicitly always available for a run between any two
// pre-existing runs. Linkage from one run to the next is implicitly defined by
// the count, so if a run exists from start..start+count, the next run will
// implicitly pick up right at index start+count where that preceding run left
// off. All of the DepthRun items that are after the head of the run can remain
// uninitialized until the run needs to be split and a new run needs to start
// somewhere in between.
// For uses like perspective-correct rasterization or with a discard mask, a
// run is not an efficient representation, and it is more beneficial to have
// a flattened array of individual depth samples that can be masked off easily.
// To support this case, the first run in a given row's run array may have a
// zero count, signaling that this entire row is flattened. Critically, the
// depth and count fields in DepthRun are ordered (endian-dependently) so that
// the DepthRun struct can be interpreted as a sign-extended int32_t depth. It
// is then possible to just treat the entire row as an array of int32_t depth
// samples that can be processed with SIMD comparisons, since the count field
// behaves as just the sign-extension of the depth field. The count field is
// limited to 8 bits so that we can support depth values up to 24 bits.
// When a depth buffer is cleared, each row is initialized to a maximal runs
// spanning the entire row. In the normal case, the depth buffer will continue
// to manage itself as a list of runs. If perspective or discard is used for
// a given row, the row will be converted to the flattened representation to
// support it, after which it will only ever revert back to runs if the depth
// buffer is cleared.

// The largest 24-bit depth value supported.
constexpr uint32_t MAX_DEPTH_VALUE = 0xFFFFFF;
// The longest 8-bit depth run that is supported, aligned to SIMD chunk size.
constexpr uint32_t MAX_DEPTH_RUN = 255 & ~3;

struct DepthRun {
  // Ensure that depth always occupies the LSB and count the MSB so that we
  // can sign-extend depth just by setting count to zero, marking it flat.
  // When count is non-zero, then this is interpreted as an actual run and
  // depth is read in isolation.
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
  uint32_t depth : 24;
  uint32_t count : 8;
#else
  uint32_t count : 8;
  uint32_t depth : 24;
#endif

  DepthRun() = default;
  DepthRun(uint32_t depth, uint8_t count) : depth(depth), count(count) {}

  // If count is zero, this is actually a flat depth sample rather than a run.
  bool is_flat() const { return !count; }

  // Compare a source depth from rasterization with a stored depth value.
  template <int FUNC>
  ALWAYS_INLINE bool compare(uint32_t src) const {
    switch (FUNC) {
      case GL_LEQUAL:
        return src <= depth;
      case GL_LESS:
        return src < depth;
      case GL_ALWAYS:
        return true;
      default:
        assert(false);
        return false;
    }
  }
};

// Fills runs at the given position with the given depth up to the span width.
static ALWAYS_INLINE void set_depth_runs(DepthRun* runs, uint32_t depth,
                                         uint32_t width) {
  // If the width exceeds the maximum run size, then we need to output clamped
  // runs first.
  for (; width >= MAX_DEPTH_RUN;
       runs += MAX_DEPTH_RUN, width -= MAX_DEPTH_RUN) {
    *runs = DepthRun(depth, MAX_DEPTH_RUN);
  }
  // If there are still any left over samples to fill under the maximum run
  // size, then output one last run for them.
  if (width > 0) {
    *runs = DepthRun(depth, width);
  }
}

// A cursor for reading and modifying a row's depth run array. It locates
// and iterates through a desired span within all the runs, testing if
// the depth of this span passes or fails the depth test against existing
// runs. If desired, new runs may be inserted to represent depth occlusion
// from this span in the run array.
struct DepthCursor {
  // Current position of run the cursor has advanced to.
  DepthRun* cur = nullptr;
  // The start of the remaining potential samples in the desired span.
  DepthRun* start = nullptr;
  // The end of the potential samples in the desired span.
  DepthRun* end = nullptr;

  DepthCursor() = default;

  // Construct a cursor with runs for a given row's run array and the bounds
  // of the span we wish to iterate within it.
  DepthCursor(DepthRun* runs, int num_runs, int span_offset, int span_count)
      : cur(runs), start(&runs[span_offset]), end(start + span_count) {
    // This cursor should never iterate over flat runs
    assert(!runs->is_flat());
    DepthRun* end_runs = &runs[num_runs];
    // Clamp end of span to end of row
    if (end > end_runs) {
      end = end_runs;
    }
    // If the span starts past the end of the row, just advance immediately
    // to it to signal that we're done.
    if (start >= end_runs) {
      cur = end_runs;
      start = end_runs;
      return;
    }
    // Otherwise, find the first depth run that contains the start of the span.
    // If the span starts after the given run, then we need to keep searching
    // through the row to find an appropriate run. The check above already
    // guaranteed that the span starts within the row's runs, and the search
    // won't fall off the end.
    for (;;) {
      assert(cur < end);
      DepthRun* next = cur + cur->count;
      if (start < next) {
        break;
      }
      cur = next;
    }
  }

  // The cursor is valid if the current position is at the end or if the run
  // contains the start position.
  bool valid() const {
    return cur >= end || (cur <= start && start < cur + cur->count);
  }

  // Skip past any initial runs that fail the depth test. If we find a run that
  // would pass, then return the accumulated length between where we started
  // and that position. Otherwise, if we fall off the end, return -1 to signal
  // that there are no more passed runs at the end of this failed region and
  // so it is safe for the caller to stop processing any more regions in this
  // row.
  template <int FUNC>
  int skip_failed(uint32_t val) {
    assert(valid());
    DepthRun* prev = start;
    while (cur < end) {
      if (cur->compare<FUNC>(val)) {
        return start - prev;
      }
      cur += cur->count;
      start = cur;
    }
    return -1;
  }

  // Helper to convert function parameters into template parameters to hoist
  // some checks out of inner loops.
  ALWAYS_INLINE int skip_failed(uint32_t val, GLenum func) {
    switch (func) {
      case GL_LEQUAL:
        return skip_failed<GL_LEQUAL>(val);
      case GL_LESS:
        return skip_failed<GL_LESS>(val);
      default:
        assert(false);
        return -1;
    }
  }

  // Find a region of runs that passes the depth test. It is assumed the caller
  // has called skip_failed first to skip past any runs that failed the depth
  // test. This stops when it finds a run that fails the depth test or we fall
  // off the end of the row. If the write mask is enabled, this will insert runs
  // to represent this new region that passed the depth test. The length of the
  // region is returned.
  template <int FUNC, bool MASK>
  int check_passed(uint32_t val) {
    assert(valid());
    DepthRun* prev = cur;
    while (cur < end) {
      if (!cur->compare<FUNC>(val)) {
        break;
      }
      DepthRun* next = cur + cur->count;
      if (next > end) {
        if (MASK) {
          // Chop the current run where the end of the span falls, making a new
          // run from the end of the span till the next run. The beginning of
          // the current run will be folded into the run from the start of the
          // passed region before returning below.
          *end = DepthRun(cur->depth, next - end);
        }
        // If the next run starts past the end, then just advance the current
        // run to the end to signal that we're now at the end of the row.
        next = end;
      }
      cur = next;
    }
    // If we haven't advanced past the start of the span region, then we found
    // nothing that passed.
    if (cur <= start) {
      return 0;
    }
    // If 'end' fell within the middle of a passing run, then 'cur' will end up
    // pointing at the new partial run created at 'end' where the passing run
    // was split to accommodate starting in the middle. The preceding runs will
    // be fixed below to properly join with this new split.
    int passed = cur - start;
    if (MASK) {
      // If the search started from a run before the start of the span, then
      // edit that run to meet up with the start.
      if (prev < start) {
        prev->count = start - prev;
      }
      // Create a new run for the entirety of the passed samples.
      set_depth_runs(start, val, passed);
    }
    start = cur;
    return passed;
  }

  // Helper to convert function parameters into template parameters to hoist
  // some checks out of inner loops.
  template <bool MASK>
  ALWAYS_INLINE int check_passed(uint32_t val, GLenum func) {
    switch (func) {
      case GL_LEQUAL:
        return check_passed<GL_LEQUAL, MASK>(val);
      case GL_LESS:
        return check_passed<GL_LESS, MASK>(val);
      default:
        assert(false);
        return 0;
    }
  }

  ALWAYS_INLINE int check_passed(uint32_t val, GLenum func, bool mask) {
    return mask ? check_passed<true>(val, func)
                : check_passed<false>(val, func);
  }

  // Fill a region of runs with a given depth value, bypassing any depth test.
  ALWAYS_INLINE void fill(uint32_t depth) {
    check_passed<GL_ALWAYS, true>(depth);
  }
};

// Initialize a depth texture by setting the first run in each row to encompass
// the entire row.
void Texture::init_depth_runs(uint32_t depth) {
  if (!buf) return;
  DepthRun* runs = (DepthRun*)buf;
  for (int y = 0; y < height; y++) {
    set_depth_runs(runs, depth, width);
    runs += stride() / sizeof(DepthRun);
  }
  set_cleared(true);
}

// Fill a portion of the run array with flattened depth samples.
static ALWAYS_INLINE void fill_flat_depth(DepthRun* dst, size_t n,
                                          uint32_t depth) {
  fill_n((uint32_t*)dst, n, depth);
}

// Fills a scissored region of a depth texture with a given depth.
void Texture::fill_depth_runs(uint32_t depth, const IntRect& scissor) {
  if (!buf) return;
  assert(cleared());
  IntRect bb = bounds().intersection(scissor - offset);
  DepthRun* runs = (DepthRun*)sample_ptr(0, bb.y0);
  for (int rows = bb.height(); rows > 0; rows--) {
    if (bb.width() >= width) {
      // If the scissor region encompasses the entire row, reset the row to a
      // single run encompassing the entire row.
      set_depth_runs(runs, depth, width);
    } else if (runs->is_flat()) {
      // If the row is flattened, just directly fill the portion of the row.
      fill_flat_depth(&runs[bb.x0], bb.width(), depth);
    } else {
      // Otherwise, if we are still using runs, then set up a cursor to fill
      // it with depth runs.
      DepthCursor(runs, width, bb.x0, bb.width()).fill(depth);
    }
    runs += stride() / sizeof(DepthRun);
  }
}

using ZMask = I32;

#if USE_SSE2
#  define ZMASK_NONE_PASSED 0xFFFF
#  define ZMASK_ALL_PASSED 0
static inline uint32_t zmask_code(ZMask mask) {
  return _mm_movemask_epi8(mask);
}
#else
#  define ZMASK_NONE_PASSED 0xFFFFFFFFU
#  define ZMASK_ALL_PASSED 0
static inline uint32_t zmask_code(ZMask mask) {
  return bit_cast<uint32_t>(CONVERT(mask, U8));
}
#endif

// Interprets items in the depth buffer as sign-extended 32-bit depth values
// instead of as runs. Returns a mask that signals which samples in the given
// chunk passed or failed the depth test with given Z value.
template <bool DISCARD>
static ALWAYS_INLINE bool check_depth(I32 src, DepthRun* zbuf, ZMask& outmask,
                                      int span = 4) {
  // SSE2 does not support unsigned comparison. So ensure Z value is
  // sign-extended to int32_t.
  I32 dest = unaligned_load<I32>(zbuf);
  // Invert the depth test to check which pixels failed and should be discarded.
  ZMask mask = ctx->depthfunc == GL_LEQUAL
                   ?
                   // GL_LEQUAL: Not(LessEqual) = Greater
                   ZMask(src > dest)
                   :
                   // GL_LESS: Not(Less) = GreaterEqual
                   ZMask(src >= dest);
  // Mask off any unused lanes in the span.
  mask |= ZMask(span) < ZMask{1, 2, 3, 4};
  if (zmask_code(mask) == ZMASK_NONE_PASSED) {
    return false;
  }
  if (!DISCARD && ctx->depthmask) {
    unaligned_store(zbuf, (mask & dest) | (~mask & src));
  }
  outmask = mask;
  return true;
}

static ALWAYS_INLINE I32 packDepth() {
  return cast(fragment_shader->gl_FragCoord.z * MAX_DEPTH_VALUE);
}

static ALWAYS_INLINE void discard_depth(I32 src, DepthRun* zbuf, I32 mask) {
  if (ctx->depthmask) {
    I32 dest = unaligned_load<I32>(zbuf);
    mask |= fragment_shader->swgl_IsPixelDiscarded;
    unaligned_store(zbuf, (mask & dest) | (~mask & src));
  }
}

static ALWAYS_INLINE void mask_output(uint32_t* buf, ZMask zmask,
                                      int span = 4) {
  WideRGBA8 r = pack_pixels_RGBA8();
  PackedRGBA8 dst = load_span<PackedRGBA8>(buf, span);
  if (blend_key) r = blend_pixels(buf, dst, r, span);
  PackedRGBA8 mask = bit_cast<PackedRGBA8>(zmask);
  store_span(buf, (mask & dst) | (~mask & pack(r)), span);
}

template <bool DISCARD>
static ALWAYS_INLINE void discard_output(uint32_t* buf, int span = 4) {
  mask_output(buf, fragment_shader->swgl_IsPixelDiscarded, span);
}

template <>
ALWAYS_INLINE void discard_output<false>(uint32_t* buf, int span) {
  WideRGBA8 r = pack_pixels_RGBA8();
  if (blend_key)
    r = blend_pixels(buf, load_span<PackedRGBA8>(buf, span), r, span);
  store_span(buf, pack(r), span);
}

static ALWAYS_INLINE void mask_output(uint8_t* buf, ZMask zmask, int span = 4) {
  WideR8 r = pack_pixels_R8();
  WideR8 dst = unpack(load_span<PackedR8>(buf, span));
  if (blend_key) r = blend_pixels(buf, dst, r, span);
  WideR8 mask = packR8(zmask);
  store_span(buf, pack((mask & dst) | (~mask & r)), span);
}

template <bool DISCARD>
static ALWAYS_INLINE void discard_output(uint8_t* buf, int span = 4) {
  mask_output(buf, fragment_shader->swgl_IsPixelDiscarded, span);
}

template <>
ALWAYS_INLINE void discard_output<false>(uint8_t* buf, int span) {
  WideR8 r = pack_pixels_R8();
  if (blend_key)
    r = blend_pixels(buf, unpack(load_span<PackedR8>(buf, span)), r, span);
  store_span(buf, pack(r), span);
}

struct ClipRect {
  float x0;
  float y0;
  float x1;
  float y1;

  explicit ClipRect(const IntRect& i)
      : x0(i.x0), y0(i.y0), x1(i.x1), y1(i.y1) {}
  explicit ClipRect(const Texture& t) : ClipRect(ctx->apply_scissor(t)) {
    // If blending is enabled, set blend_key to reflect the resolved blend
    // state for the currently drawn primitive.
    if (ctx->blend) {
      blend_key = ctx->blend_key;
      if (swgl_ClipFlags) {
        // If there is a blend override set, replace the blend key with it.
        if (swgl_ClipFlags & SWGL_CLIP_FLAG_BLEND_OVERRIDE) {
          blend_key = swgl_BlendOverride;
        }
        // If a clip mask is available, set up blending state to use the clip
        // mask.
        if (swgl_ClipFlags & SWGL_CLIP_FLAG_MASK) {
          assert(swgl_ClipMask->format == TextureFormat::R8);
          // Constrain the clip mask bounds to always fall within the clip mask.
          swgl_ClipMaskBounds.intersect(IntRect{0, 0, int(swgl_ClipMask->width),
                                                int(swgl_ClipMask->height)});
          // The clip mask offset is relative to the viewport.
          swgl_ClipMaskOffset += ctx->viewport.origin() - t.offset;
          // The clip mask bounds are relative to the clip mask offset.
          swgl_ClipMaskBounds.offset(swgl_ClipMaskOffset);
          // Finally, constrain the clip rectangle by the clip mask bounds.
          intersect(swgl_ClipMaskBounds);
          // Modify the blend key so that it will use the clip mask while
          // blending.
          restore_clip_mask();
        }
        if (swgl_ClipFlags & SWGL_CLIP_FLAG_AA) {
          // Modify the blend key so that it will use AA while blending.
          restore_aa();
        }
      }
    } else {
      blend_key = BLEND_KEY_NONE;
      swgl_ClipFlags = 0;
    }
  }

  FloatRange x_range() const { return {x0, x1}; }

  void intersect(const IntRect& c) {
    x0 = max(x0, float(c.x0));
    y0 = max(y0, float(c.y0));
    x1 = min(x1, float(c.x1));
    y1 = min(y1, float(c.y1));
  }

  template <typename P>
  void set_clip_mask(int x, int y, P* buf) const {
    if (swgl_ClipFlags & SWGL_CLIP_FLAG_MASK) {
      swgl_SpanBuf = buf;
      swgl_ClipMaskBuf = (uint8_t*)swgl_ClipMask->buf +
                         (y - swgl_ClipMaskOffset.y) * swgl_ClipMask->stride +
                         (x - swgl_ClipMaskOffset.x);
    }
  }

  template <typename P>
  bool overlaps(int nump, const P* p) const {
    // Generate a mask of which side of the clip rect all of a polygon's points
    // fall inside of. This is a cheap conservative estimate of whether the
    // bounding box of the polygon might overlap the clip rect, rather than an
    // exact test that would require multiple slower line intersections.
    int sides = 0;
    for (int i = 0; i < nump; i++) {
      sides |= p[i].x < x1 ? (p[i].x > x0 ? 1 | 2 : 1) : 2;
      sides |= p[i].y < y1 ? (p[i].y > y0 ? 4 | 8 : 4) : 8;
    }
    return sides == 0xF;
  }
};

// Given a current X position at the center Y position of a row, return the X
// position of the left and right intercepts of the row top and bottom.
template <typename E>
static ALWAYS_INLINE FloatRange x_intercepts(const E& e) {
  float rad = 0.5f * abs(e.x_slope());
  return {e.cur_x() - rad, e.cur_x() + rad};
}

// Return the AA sub-span corresponding to a given edge. If AA is requested,
// then this finds the X intercepts with the row clipped into range of the
// edge and finally conservatively rounds them out. If there is no AA, then
// it just returns the current rounded X position clipped within bounds.
template <typename E>
static ALWAYS_INLINE IntRange aa_edge(const E& e, const FloatRange& bounds) {
  return e.edgeMask ? bounds.clip(x_intercepts(e)).round_out()
                    : bounds.clip({e.cur_x(), e.cur_x()}).round();
}

// Calculate the initial AA coverage as an approximation of the distance from
// the center of the pixel in the direction of the edge slope. Given an edge
// (x,y)..(x+dx,y+dy), then the normalized tangent vector along the edge is
// (dx,dy)/sqrt(dx^2+dy^2). We know that for dy=1 then dx=e.x_slope. We rotate
// the tangent vector either -90 or +90 degrees to get the edge normal vector,
// where 'dx=-dy and 'dy=dx. Once normalized by 1/sqrt(dx^2+dy^2), scale into
// the range of 0..256 so that we can cheaply convert to a fixed-point scale
// factor. It is assumed that at exactly the pixel center the opacity is half
// (128) and linearly decreases along the normal vector at 1:1 scale with the
// slope. While not entirely accurate, this gives a reasonably agreeable looking
// approximation of AA. For edges on which there is no AA, just force the
// opacity to maximum (256) with no slope, relying on the span clipping to trim
// pixels outside the span.
template <typename E>
static ALWAYS_INLINE FloatRange aa_dist(const E& e, float dir) {
  if (e.edgeMask) {
    float dx = (dir * 256.0f) * inversesqrt(1.0f + e.x_slope() * e.x_slope());
    return {128.0f + dx * (e.cur_x() - 0.5f), -dx};
  } else {
    return {256.0f, 0.0f};
  }
}

template <typename P, typename E>
static ALWAYS_INLINE IntRange aa_span(P* buf, const E& left, const E& right,
                                      const FloatRange& bounds) {
  // If there is no AA, just return the span from the rounded left edge X
  // position to the rounded right edge X position. Clip the span to be within
  // the valid bounds.
  if (!(swgl_ClipFlags & SWGL_CLIP_FLAG_AA)) {
    return bounds.clip({left.cur_x(), right.cur_x()}).round();
  }

  // Calculate the left and right AA spans along with the coverage distances
  // and slopes necessary to do blending.
  IntRange leftAA = aa_edge(left, bounds);
  FloatRange leftDist = aa_dist(left, -1.0f);
  IntRange rightAA = aa_edge(right, bounds);
  FloatRange rightDist = aa_dist(right, 1.0f);

  // Use the pointer into the destination buffer as a status indicator of the
  // coverage offset. The pointer is calculated so that subtracting it with
  // the current destination pointer will yield a negative value if the span
  // is outside the opaque area and otherwise will yield a positive value
  // above the opaque size. This pointer is stored as a uint8 pointer so that
  // there are no hidden multiplication instructions and will just return a
  // 1:1 linear memory address. Thus the size of the opaque region must also
  // be scaled by the pixel size in bytes.
  swgl_OpaqueStart = (const uint8_t*)(buf + leftAA.end);
  swgl_OpaqueSize = max(rightAA.start - leftAA.end - 3, 0) * sizeof(P);

  // Offset the coverage distances by the end of the left AA span, which
  // corresponds to the opaque start pointer, so that pixels become opaque
  // immediately after. The distances are also offset for each lane in the
  // chunk.
  Float offset = cast(leftAA.end + (I32){0, 1, 2, 3});
  swgl_LeftAADist = leftDist.start + offset * leftDist.end;
  swgl_RightAADist = rightDist.start + offset * rightDist.end;
  swgl_AASlope =
      (Float){leftDist.end, rightDist.end, 0.0f, 0.0f} / float(sizeof(P));

  // Return the full span width from the start of the left span to the end of
  // the right span.
  return {leftAA.start, rightAA.end};
}

// Calculate the span the user clip distances occupy from the left and right
// edges at the current row.
template <typename E>
static ALWAYS_INLINE IntRange clip_distance_range(const E& left,
                                                  const E& right) {
  Float leftClip = get_clip_distances(left.interp);
  Float rightClip = get_clip_distances(right.interp);
  // Get the change in clip dist per X step.
  Float clipStep = (rightClip - leftClip) / (right.cur_x() - left.cur_x());
  // Find the zero intercepts starting from the left edge.
  Float clipDist = left.cur_x() - leftClip * recip(clipStep);
  // Find the distance to the start of the span for any clip distances that
  // are increasing in value. If the clip distance is constant or decreasing
  // in value, then check if it starts outside the clip volume.
  Float start = if_then_else(clipStep > 0.0f, clipDist,
                             if_then_else(leftClip < 0.0f, 1.0e6f, 0.0f));
  // Find the distance to the end of the span for any clip distances that are
  // decreasing in value. If the clip distance is constant or increasing in
  // value, then check if it ends inside the clip volume.
  Float end = if_then_else(clipStep < 0.0f, clipDist,
                           if_then_else(rightClip >= 0.0f, 1.0e6f, 0.0f));
  // Find the furthest start offset.
  start = max(start, start.zwxy);
  // Find the closest end offset.
  end = min(end, end.zwxy);
  // Finally, round the offsets to an integer span that can be used to bound
  // the current span.
  return FloatRange{max(start.x, start.y), min(end.x, end.y)}.round();
}

// Converts a run array into a flattened array of depth samples. This just
// walks through every run and fills the samples with the depth value from
// the run.
static void flatten_depth_runs(DepthRun* runs, size_t width) {
  if (runs->is_flat()) {
    return;
  }
  while (width > 0) {
    size_t n = runs->count;
    fill_flat_depth(runs, n, runs->depth);
    runs += n;
    width -= n;
  }
}

// Helper function for drawing passed depth runs within the depth buffer.
// Flattened depth (perspective or discard) is not supported.
template <typename P>
static ALWAYS_INLINE void draw_depth_span(uint32_t z, P* buf,
                                          DepthCursor& cursor) {
  for (;;) {
    // Get the span that passes the depth test. Assume on entry that
    // any failed runs have already been skipped.
    int span = cursor.check_passed(z, ctx->depthfunc, ctx->depthmask);
    // If nothing passed, since we already skipped passed failed runs
    // previously, we must have hit the end of the row. Bail out.
    if (span <= 0) {
      break;
    }
    if (span >= 4) {
      // If we have a draw specialization, try to process as many 4-pixel
      // chunks as possible using it.
      if (fragment_shader->has_draw_span(buf)) {
        int drawn = fragment_shader->draw_span(buf, span & ~3);
        buf += drawn;
        span -= drawn;
      }
      // Otherwise, just process each chunk individually.
      while (span >= 4) {
        fragment_shader->run();
        discard_output<false>(buf);
        buf += 4;
        span -= 4;
      }
    }
    // If we have a partial chunk left over, we still have to process it as if
    // it were a full chunk. Mask off only the part of the chunk we want to
    // use.
    if (span > 0) {
      fragment_shader->run();
      discard_output<false>(buf, span);
      buf += span;
    }
    // Skip past any runs that fail the depth test.
    int skip = cursor.skip_failed(z, ctx->depthfunc);
    // If there aren't any, that means we won't encounter any more passing runs
    // and so it's safe to bail out.
    if (skip <= 0) {
      break;
    }
    // Advance interpolants for the fragment shader past the skipped region.
    // If we processed a partial chunk above, we actually advanced the
    // interpolants a full chunk in the fragment shader's run function. Thus,
    // we need to first subtract off that 4-pixel chunk and only partially
    // advance them to that partial chunk before we can add on the rest of the
    // skips. This is combined with the skip here for efficiency's sake.
    fragment_shader->skip(skip - (span > 0 ? 4 - span : 0));
    buf += skip;
  }
}

// Draw a simple span in 4-pixel wide chunks, optionally using depth.
template <bool DISCARD, bool W, typename P, typename Z>
static ALWAYS_INLINE void draw_span(P* buf, DepthRun* depth, int span, Z z) {
  if (depth) {
    // Depth testing is enabled. If perspective is used, Z values will vary
    // across the span, we use packDepth to generate packed Z values suitable
    // for depth testing based on current values from gl_FragCoord.z.
    // Otherwise, for the no-perspective case, we just use the provided Z.
    // Process 4-pixel chunks first.
    for (; span >= 4; span -= 4, buf += 4, depth += 4) {
      I32 zsrc = z();
      ZMask zmask;
      if (check_depth<DISCARD>(zsrc, depth, zmask)) {
        fragment_shader->run<W>();
        mask_output(buf, zmask);
        if (DISCARD) discard_depth(zsrc, depth, zmask);
      } else {
        fragment_shader->skip<W>();
      }
    }
    // If there are any remaining pixels, do a partial chunk.
    if (span > 0) {
      I32 zsrc = z();
      ZMask zmask;
      if (check_depth<DISCARD>(zsrc, depth, zmask, span)) {
        fragment_shader->run<W>();
        mask_output(buf, zmask, span);
        if (DISCARD) discard_depth(zsrc, depth, zmask);
      }
    }
  } else {
    // Process 4-pixel chunks first.
    for (; span >= 4; span -= 4, buf += 4) {
      fragment_shader->run<W>();
      discard_output<DISCARD>(buf);
    }
    // If there are any remaining pixels, do a partial chunk.
    if (span > 0) {
      fragment_shader->run<W>();
      discard_output<DISCARD>(buf, span);
    }
  }
}

// Called during rasterization to forcefully clear a row on which delayed clear
// has been enabled. If we know that we are going to completely overwrite a part
// of the row, then we only need to clear the row outside of that part. However,
// if blending or discard is enabled, the values of that underlying part of the
// row may be used regardless to produce the final rasterization result, so we
// have to then clear the entire underlying row to prepare it.
template <typename P>
static inline void prepare_row(Texture& colortex, int y, int startx, int endx,
                               bool use_discard, DepthRun* depth,
                               uint32_t z = 0, DepthCursor* cursor = nullptr) {
  assert(colortex.delay_clear > 0);
  // Delayed clear is enabled for the color buffer. Check if needs clear.
  uint32_t& mask = colortex.cleared_rows[y / 32];
  if ((mask & (1 << (y & 31))) == 0) {
    mask |= 1 << (y & 31);
    colortex.delay_clear--;
    if (blend_key || use_discard) {
      // If depth test, blending, or discard is used, old color values
      // might be sampled, so we need to clear the entire row to fill it.
      force_clear_row<P>(colortex, y);
    } else if (depth) {
      if (depth->is_flat() || !cursor) {
        // If flat depth is used, we can't cheaply predict if which samples will
        // pass.
        force_clear_row<P>(colortex, y);
      } else {
        // Otherwise if depth runs are used, see how many samples initially pass
        // the depth test and only fill the row outside those. The fragment
        // shader will fill the row within the passed samples.
        int passed =
            DepthCursor(*cursor).check_passed<false>(z, ctx->depthfunc);
        if (startx > 0 || startx + passed < colortex.width) {
          force_clear_row<P>(colortex, y, startx, startx + passed);
        }
      }
    } else if (startx > 0 || endx < colortex.width) {
      // Otherwise, we only need to clear the row outside of the span.
      // The fragment shader will fill the row within the span itself.
      force_clear_row<P>(colortex, y, startx, endx);
    }
  }
}

// Perpendicular dot-product is the dot-product of a vector with the
// perpendicular vector of the other, i.e. dot(a, {-b.y, b.x})
template <typename T>
static ALWAYS_INLINE auto perpDot(T a, T b) {
  return a.x * b.y - a.y * b.x;
}

// Check if the winding of the initial edges is flipped, requiring us to swap
// the edges to avoid spans having negative lengths. Assume that l0.y == r0.y
// due to the initial edge scan in draw_quad/perspective_spans.
template <typename T>
static ALWAYS_INLINE bool checkIfEdgesFlipped(T l0, T l1, T r0, T r1) {
  // If the starting point of the left edge is to the right of the starting
  // point of the right edge, then just assume the edges are flipped. If the
  // left and right starting points are the same, then check the sign of the
  // cross-product of the edges to see if the edges are flipped. Otherwise,
  // if the left starting point is actually just to the left of the right
  // starting point, then assume no edge flip.
  return l0.x > r0.x || (l0.x == r0.x && perpDot(l1 - l0, r1 - r0) > 0.0f);
}

// Draw spans for each row of a given quad (or triangle) with a constant Z
// value. The quad is assumed convex. It is clipped to fall within the given
// clip rect. In short, this function rasterizes a quad by first finding a
// top most starting point and then from there tracing down the left and right
// sides of this quad until it hits the bottom, outputting a span between the
// current left and right positions at each row along the way. Points are
// assumed to be ordered in either CW or CCW to support this, but currently
// both orders (CW and CCW) are supported and equivalent.
template <typename P>
static inline void draw_quad_spans(int nump, Point2D p[4], uint32_t z,
                                   Interpolants interp_outs[4],
                                   Texture& colortex, Texture& depthtex,
                                   const ClipRect& clipRect) {
  // Only triangles and convex quads supported.
  assert(nump == 3 || nump == 4);

  Point2D l0, r0, l1, r1;
  int l0i, r0i, l1i, r1i;
  {
    // Find the index of the top-most (smallest Y) point from which
    // rasterization can start.
    int top = nump > 3 && p[3].y < p[2].y
                  ? (p[0].y < p[1].y ? (p[0].y < p[3].y ? 0 : 3)
                                     : (p[1].y < p[3].y ? 1 : 3))
                  : (p[0].y < p[1].y ? (p[0].y < p[2].y ? 0 : 2)
                                     : (p[1].y < p[2].y ? 1 : 2));
    // Helper to find next index in the points array, walking forward.
#define NEXT_POINT(idx)   \
  ({                      \
    int cur = (idx) + 1;  \
    cur < nump ? cur : 0; \
  })
    // Helper to find the previous index in the points array, walking backward.
#define PREV_POINT(idx)        \
  ({                           \
    int cur = (idx)-1;         \
    cur >= 0 ? cur : nump - 1; \
  })
    // Start looking for "left"-side and "right"-side descending edges starting
    // from the determined top point.
    int next = NEXT_POINT(top);
    int prev = PREV_POINT(top);
    if (p[top].y == p[next].y) {
      // If the next point is on the same row as the top, then advance one more
      // time to the next point and use that as the "left" descending edge.
      l0i = next;
      l1i = NEXT_POINT(next);
      // Assume top and prev form a descending "right" edge, as otherwise this
      // will be a collapsed polygon and harmlessly bail out down below.
      r0i = top;
      r1i = prev;
    } else if (p[top].y == p[prev].y) {
      // If the prev point is on the same row as the top, then advance to the
      // prev again and use that as the "right" descending edge.
      // Assume top and next form a non-empty descending "left" edge.
      l0i = top;
      l1i = next;
      r0i = prev;
      r1i = PREV_POINT(prev);
    } else {
      // Both next and prev are on distinct rows from top, so both "left" and
      // "right" edges are non-empty/descending.
      l0i = r0i = top;
      l1i = next;
      r1i = prev;
    }
    // Load the points from the indices.
    l0 = p[l0i];  // Start of left edge
    r0 = p[r0i];  // End of left edge
    l1 = p[l1i];  // Start of right edge
    r1 = p[r1i];  // End of right edge
    //    debugf("l0: %d(%f,%f), r0: %d(%f,%f) -> l1: %d(%f,%f), r1:
    //    %d(%f,%f)\n", l0i, l0.x, l0.y, r0i, r0.x, r0.y, l1i, l1.x, l1.y, r1i,
    //    r1.x, r1.y);
  }

  struct Edge {
    float yScale;
    float xSlope;
    float x;
    Interpolants interpSlope;
    Interpolants interp;
    bool edgeMask;

    Edge(float y, const Point2D& p0, const Point2D& p1, const Interpolants& i0,
         const Interpolants& i1, int edgeIndex)
        :  // Inverse Y scale for slope calculations. Avoid divide on 0-length
           // edge. Later checks below ensure that Y <= p1.y, or otherwise we
           // don't use this edge. We just need to guard against Y == p1.y ==
           // p0.y. In that case, Y - p0.y == 0 and will cancel out the slopes
           // below, except if yScale is Inf for some reason (or worse, NaN),
           // which 1/(p1.y-p0.y) might produce if we don't bound it.
          yScale(1.0f / max(p1.y - p0.y, 1.0f / 256)),
          // Calculate dX/dY slope
          xSlope((p1.x - p0.x) * yScale),
          // Initialize current X based on Y and slope
          x(p0.x + (y - p0.y) * xSlope),
          // Calculate change in interpolants per change in Y
          interpSlope((i1 - i0) * yScale),
          // Initialize current interpolants based on Y and slope
          interp(i0 + (y - p0.y) * interpSlope),
          // Extract the edge mask status for this edge
          edgeMask((swgl_AAEdgeMask >> edgeIndex) & 1) {}

    void nextRow() {
      // step current X and interpolants to next row from slope
      x += xSlope;
      interp += interpSlope;
    }

    float cur_x() const { return x; }
    float x_slope() const { return xSlope; }
  };

  // Vertex selection above should result in equal left and right start rows
  assert(l0.y == r0.y);
  // Find the start y, clip to within the clip rect, and round to row center.
  // If AA is enabled, round out conservatively rather than round to nearest.
  float aaRound = swgl_ClipFlags & SWGL_CLIP_FLAG_AA ? 0.0f : 0.5f;
  float y = floor(max(l0.y, clipRect.y0) + aaRound) + 0.5f;
  // Initialize left and right edges from end points and start Y
  Edge left(y, l0, l1, interp_outs[l0i], interp_outs[l1i], l1i);
  Edge right(y, r0, r1, interp_outs[r0i], interp_outs[r1i], r0i);
  // WR does not use backface culling, so check if edges are flipped.
  bool flipped = checkIfEdgesFlipped(l0, l1, r0, r1);
  if (flipped) swap(left, right);
  // Get pointer to color buffer and depth buffer at current Y
  P* fbuf = (P*)colortex.sample_ptr(0, int(y));
  DepthRun* fdepth = (DepthRun*)depthtex.sample_ptr(0, int(y));
  // Loop along advancing Ys, rasterizing spans at each row
  float checkY = min(min(l1.y, r1.y), clipRect.y1);
  // Ensure we don't rasterize out edge bounds
  FloatRange clipSpan =
      clipRect.x_range().clip(x_range(l0, l1).merge(x_range(r0, r1)));
  for (;;) {
    // Check if we maybe passed edge ends or outside clip rect...
    if (y > checkY) {
      // If we're outside the clip rect, we're done.
      if (y > clipRect.y1) break;
        // Helper to find the next non-duplicate vertex that doesn't loop back.
#define STEP_EDGE(y, e0i, e0, e1i, e1, STEP_POINT, end)     \
  do {                                                      \
    /* Set new start of edge to be end of old edge */       \
    e0i = e1i;                                              \
    e0 = e1;                                                \
    /* Set new end of edge to next point */                 \
    e1i = STEP_POINT(e1i);                                  \
    e1 = p[e1i];                                            \
    /* If the edge crossed the end, we're done. */          \
    if (e0i == end) return;                                 \
    /* Otherwise, it doesn't advance, so keep searching. */ \
  } while (y > e1.y)
      // Check if Y advanced past the end of the left edge
      if (y > l1.y) {
        // Step to next left edge past Y and reset edge interpolants.
        STEP_EDGE(y, l0i, l0, l1i, l1, NEXT_POINT, r1i);
        (flipped ? right : left) =
            Edge(y, l0, l1, interp_outs[l0i], interp_outs[l1i], l1i);
      }
      // Check if Y advanced past the end of the right edge
      if (y > r1.y) {
        // Step to next right edge past Y and reset edge interpolants.
        STEP_EDGE(y, r0i, r0, r1i, r1, PREV_POINT, l1i);
        (flipped ? left : right) =
            Edge(y, r0, r1, interp_outs[r0i], interp_outs[r1i], r0i);
      }
      // Reset the clip bounds for the new edges
      clipSpan =
          clipRect.x_range().clip(x_range(l0, l1).merge(x_range(r0, r1)));
      // Reset check condition for next time around.
      checkY = min(ceil(min(l1.y, r1.y) - aaRound), clipRect.y1);
    }

    // Calculate a potentially AA'd span and check if it is non-empty.
    IntRange span = aa_span(fbuf, left, right, clipSpan);
    if (span.len() > 0) {
      // If user clip planes are enabled, use them to bound the current span.
      if (vertex_shader->use_clip_distance()) {
        span = span.intersect(clip_distance_range(left, right));
        if (span.len() <= 0) goto next_span;
      }
      ctx->shaded_rows++;
      ctx->shaded_pixels += span.len();
      // Advance color/depth buffer pointers to the start of the span.
      P* buf = fbuf + span.start;
      // Check if we will need to use depth-buffer or discard on this span.
      DepthRun* depth =
          depthtex.buf != nullptr && depthtex.cleared() ? fdepth : nullptr;
      DepthCursor cursor;
      bool use_discard = fragment_shader->use_discard();
      if (use_discard) {
        if (depth) {
          // If we're using discard, we may have to unpredictably drop out some
          // samples. Flatten the depth run array here to allow this.
          if (!depth->is_flat()) {
            flatten_depth_runs(depth, depthtex.width);
          }
          // Advance to the depth sample at the start of the span.
          depth += span.start;
        }
      } else if (depth) {
        if (!depth->is_flat()) {
          // We're not using discard and the depth row is still organized into
          // runs. Skip past any runs that would fail the depth test so we
          // don't have to do any extra work to process them with the rest of
          // the span.
          cursor = DepthCursor(depth, depthtex.width, span.start, span.len());
          int skipped = cursor.skip_failed(z, ctx->depthfunc);
          // If we fell off the row, that means we couldn't find any passing
          // runs. We can just skip the entire span.
          if (skipped < 0) {
            goto next_span;
          }
          buf += skipped;
          span.start += skipped;
        } else {
          // The row is already flattened, so just advance to the span start.
          depth += span.start;
        }
      }

      if (colortex.delay_clear) {
        // Delayed clear is enabled for the color buffer. Check if needs clear.
        prepare_row<P>(colortex, int(y), span.start, span.end, use_discard,
                       depth, z, &cursor);
      }

      // Initialize fragment shader interpolants to current span position.
      fragment_shader->gl_FragCoord.x = init_interp(span.start + 0.5f, 1);
      fragment_shader->gl_FragCoord.y = y;
      {
        // Change in interpolants is difference between current right and left
        // edges per the change in right and left X.
        Interpolants step =
            (right.interp - left.interp) * (1.0f / (right.x - left.x));
        // Advance current interpolants to X at start of span.
        Interpolants o = left.interp + step * (span.start + 0.5f - left.x);
        fragment_shader->init_span(&o, &step);
      }
      clipRect.set_clip_mask(span.start, y, buf);
      if (!use_discard) {
        // Fast paths for the case where fragment discard is not used.
        if (depth) {
          // If depth is used, we want to process entire depth runs if depth is
          // not flattened.
          if (!depth->is_flat()) {
            draw_depth_span(z, buf, cursor);
            goto next_span;
          }
          // Otherwise, flattened depth must fall back to the slightly slower
          // per-chunk depth test path in draw_span below.
        } else {
          // Check if the fragment shader has an optimized draw specialization.
          if (span.len() >= 4 && fragment_shader->has_draw_span(buf)) {
            // Draw specialization expects 4-pixel chunks.
            int drawn = fragment_shader->draw_span(buf, span.len() & ~3);
            buf += drawn;
            span.start += drawn;
          }
        }
        draw_span<false, false>(buf, depth, span.len(), [=] { return z; });
      } else {
        // If discard is used, then use slower fallbacks. This should be rare.
        // Just needs to work, doesn't need to be too fast yet...
        draw_span<true, false>(buf, depth, span.len(), [=] { return z; });
      }
    }
  next_span:
    // Advance Y and edge interpolants to next row.
    y++;
    left.nextRow();
    right.nextRow();
    // Advance buffers to next row.
    fbuf += colortex.stride() / sizeof(P);
    fdepth += depthtex.stride() / sizeof(DepthRun);
  }
}

// Draw perspective-correct spans for a convex quad that has been clipped to
// the near and far Z planes, possibly producing a clipped convex polygon with
// more than 4 sides. This assumes the Z value will vary across the spans and
// requires interpolants to factor in W values. This tends to be slower than
// the simpler 2D draw_quad_spans above, especially since we can't optimize the
// depth test easily when Z values, and should be used only rarely if possible.
template <typename P>
static inline void draw_perspective_spans(int nump, Point3D* p,
                                          Interpolants* interp_outs,
                                          Texture& colortex, Texture& depthtex,
                                          const ClipRect& clipRect) {
  Point3D l0, r0, l1, r1;
  int l0i, r0i, l1i, r1i;
  {
    // Find the index of the top-most point (smallest Y) from which
    // rasterization can start.
    int top = 0;
    for (int i = 1; i < nump; i++) {
      if (p[i].y < p[top].y) {
        top = i;
      }
    }
    // Find left-most top point, the start of the left descending edge.
    // Advance forward in the points array, searching at most nump points
    // in case the polygon is flat.
    l0i = top;
    for (int i = top + 1; i < nump && p[i].y == p[top].y; i++) {
      l0i = i;
    }
    if (l0i == nump - 1) {
      for (int i = 0; i <= top && p[i].y == p[top].y; i++) {
        l0i = i;
      }
    }
    // Find right-most top point, the start of the right descending edge.
    // Advance backward in the points array, searching at most nump points.
    r0i = top;
    for (int i = top - 1; i >= 0 && p[i].y == p[top].y; i--) {
      r0i = i;
    }
    if (r0i == 0) {
      for (int i = nump - 1; i >= top && p[i].y == p[top].y; i--) {
        r0i = i;
      }
    }
    // End of left edge is next point after left edge start.
    l1i = NEXT_POINT(l0i);
    // End of right edge is prev point after right edge start.
    r1i = PREV_POINT(r0i);
    l0 = p[l0i];  // Start of left edge
    r0 = p[r0i];  // End of left edge
    l1 = p[l1i];  // Start of right edge
    r1 = p[r1i];  // End of right edge
  }

  struct Edge {
    float yScale;
    // Current coordinates for edge. Where in the 2D case of draw_quad_spans,
    // it is enough to just track the X coordinate as we advance along the rows,
    // for the perspective case we also need to keep track of Z and W. For
    // simplicity, we just use the full 3D point to track all these coordinates.
    Point3D pSlope;
    Point3D p;
    Interpolants interpSlope;
    Interpolants interp;
    bool edgeMask;

    Edge(float y, const Point3D& p0, const Point3D& p1, const Interpolants& i0,
         const Interpolants& i1, int edgeIndex)
        :  // Inverse Y scale for slope calculations. Avoid divide on 0-length
           // edge.
          yScale(1.0f / max(p1.y - p0.y, 1.0f / 256)),
          // Calculate dX/dY slope
          pSlope((p1 - p0) * yScale),
          // Initialize current coords based on Y and slope
          p(p0 + (y - p0.y) * pSlope),
          // Crucially, these interpolants must be scaled by the point's 1/w
          // value, which allows linear interpolation in a perspective-correct
          // manner. This will be canceled out inside the fragment shader later.
          // Calculate change in interpolants per change in Y
          interpSlope((i1 * p1.w - i0 * p0.w) * yScale),
          // Initialize current interpolants based on Y and slope
          interp(i0 * p0.w + (y - p0.y) * interpSlope),
          // Extract the edge mask status for this edge
          edgeMask((swgl_AAEdgeMask >> edgeIndex) & 1) {}

    float x() const { return p.x; }
    vec2_scalar zw() const { return {p.z, p.w}; }

    void nextRow() {
      // step current coords and interpolants to next row from slope
      p += pSlope;
      interp += interpSlope;
    }

    float cur_x() const { return p.x; }
    float x_slope() const { return pSlope.x; }
  };

  // Vertex selection above should result in equal left and right start rows
  assert(l0.y == r0.y);
  // Find the start y, clip to within the clip rect, and round to row center.
  // If AA is enabled, round out conservatively rather than round to nearest.
  float aaRound = swgl_ClipFlags & SWGL_CLIP_FLAG_AA ? 0.0f : 0.5f;
  float y = floor(max(l0.y, clipRect.y0) + aaRound) + 0.5f;
  // Initialize left and right edges from end points and start Y
  Edge left(y, l0, l1, interp_outs[l0i], interp_outs[l1i], l1i);
  Edge right(y, r0, r1, interp_outs[r0i], interp_outs[r1i], r0i);
  // WR does not use backface culling, so check if edges are flipped.
  bool flipped = checkIfEdgesFlipped(l0, l1, r0, r1);
  if (flipped) swap(left, right);
  // Get pointer to color buffer and depth buffer at current Y
  P* fbuf = (P*)colortex.sample_ptr(0, int(y));
  DepthRun* fdepth = (DepthRun*)depthtex.sample_ptr(0, int(y));
  // Loop along advancing Ys, rasterizing spans at each row
  float checkY = min(min(l1.y, r1.y), clipRect.y1);
  // Ensure we don't rasterize out edge bounds
  FloatRange clipSpan =
      clipRect.x_range().clip(x_range(l0, l1).merge(x_range(r0, r1)));
  for (;;) {
    // Check if we maybe passed edge ends or outside clip rect...
    if (y > checkY) {
      // If we're outside the clip rect, we're done.
      if (y > clipRect.y1) break;
      // Check if Y advanced past the end of the left edge
      if (y > l1.y) {
        // Step to next left edge past Y and reset edge interpolants.
        STEP_EDGE(y, l0i, l0, l1i, l1, NEXT_POINT, r1i);
        (flipped ? right : left) =
            Edge(y, l0, l1, interp_outs[l0i], interp_outs[l1i], l1i);
      }
      // Check if Y advanced past the end of the right edge
      if (y > r1.y) {
        // Step to next right edge past Y and reset edge interpolants.
        STEP_EDGE(y, r0i, r0, r1i, r1, PREV_POINT, l1i);
        (flipped ? left : right) =
            Edge(y, r0, r1, interp_outs[r0i], interp_outs[r1i], r0i);
      }
      // Reset the clip bounds for the new edges
      clipSpan =
          clipRect.x_range().clip(x_range(l0, l1).merge(x_range(r0, r1)));
      // Reset check condition for next time around.
      checkY = min(ceil(min(l1.y, r1.y) - aaRound), clipRect.y1);
    }

    // Calculate a potentially AA'd span and check if it is non-empty.
    IntRange span = aa_span(fbuf, left, right, clipSpan);
    if (span.len() > 0) {
      // If user clip planes are enabled, use them to bound the current span.
      if (vertex_shader->use_clip_distance()) {
        span = span.intersect(clip_distance_range(left, right));
        if (span.len() <= 0) goto next_span;
      }
      ctx->shaded_rows++;
      ctx->shaded_pixels += span.len();
      // Advance color/depth buffer pointers to the start of the span.
      P* buf = fbuf + span.start;
      // Check if the we will need to use depth-buffer or discard on this span.
      DepthRun* depth =
          depthtex.buf != nullptr && depthtex.cleared() ? fdepth : nullptr;
      bool use_discard = fragment_shader->use_discard();
      if (depth) {
        // Perspective may cause the depth value to vary on a per sample basis.
        // Ensure the depth row is flattened to allow testing of individual
        // samples
        if (!depth->is_flat()) {
          flatten_depth_runs(depth, depthtex.width);
        }
        // Advance to the depth sample at the start of the span.
        depth += span.start;
      }
      if (colortex.delay_clear) {
        // Delayed clear is enabled for the color buffer. Check if needs clear.
        prepare_row<P>(colortex, int(y), span.start, span.end, use_discard,
                       depth);
      }
      // Initialize fragment shader interpolants to current span position.
      fragment_shader->gl_FragCoord.x = init_interp(span.start + 0.5f, 1);
      fragment_shader->gl_FragCoord.y = y;
      {
        // Calculate the fragment Z and W change per change in fragment X step.
        vec2_scalar stepZW =
            (right.zw() - left.zw()) * (1.0f / (right.x() - left.x()));
        // Calculate initial Z and W values for span start.
        vec2_scalar zw = left.zw() + stepZW * (span.start + 0.5f - left.x());
        // Set fragment shader's Z and W values so that it can use them to
        // cancel out the 1/w baked into the interpolants.
        fragment_shader->gl_FragCoord.z = init_interp(zw.x, stepZW.x);
        fragment_shader->gl_FragCoord.w = init_interp(zw.y, stepZW.y);
        fragment_shader->swgl_StepZW = stepZW;
        // Change in interpolants is difference between current right and left
        // edges per the change in right and left X. The left and right
        // interpolant values were previously multipled by 1/w, so the step and
        // initial span values take this into account.
        Interpolants step =
            (right.interp - left.interp) * (1.0f / (right.x() - left.x()));
        // Advance current interpolants to X at start of span.
        Interpolants o = left.interp + step * (span.start + 0.5f - left.x());
        fragment_shader->init_span<true>(&o, &step);
      }
      clipRect.set_clip_mask(span.start, y, buf);
      if (!use_discard) {
        // No discard is used. Common case.
        draw_span<false, true>(buf, depth, span.len(), packDepth);
      } else {
        // Discard is used. Rare.
        draw_span<true, true>(buf, depth, span.len(), packDepth);
      }
    }
  next_span:
    // Advance Y and edge interpolants to next row.
    y++;
    left.nextRow();
    right.nextRow();
    // Advance buffers to next row.
    fbuf += colortex.stride() / sizeof(P);
    fdepth += depthtex.stride() / sizeof(DepthRun);
  }
}

// Clip a primitive against both sides of a view-frustum axis, producing
// intermediate vertexes with interpolated attributes that will no longer
// intersect the selected axis planes. This assumes the primitive is convex
// and should produce at most N+2 vertexes for each invocation (only in the
// worst case where one point falls outside on each of the opposite sides
// with the rest of the points inside). The supplied AA edge mask will be
// modified such that it corresponds to the clipped polygon edges.
template <XYZW AXIS>
static int clip_side(int nump, Point3D* p, Interpolants* interp, Point3D* outP,
                     Interpolants* outInterp, int& outEdgeMask) {
  // Potential mask bits of which side of a plane a coordinate falls on.
  enum SIDE { POSITIVE = 1, NEGATIVE = 2 };
  int numClip = 0;
  int edgeMask = outEdgeMask;
  Point3D prev = p[nump - 1];
  Interpolants prevInterp = interp[nump - 1];
  float prevCoord = prev.select(AXIS);
  // Coordinate must satisfy -W <= C <= W. Determine if it is outside, and
  // if so, remember which side it is outside of. In the special case that W is
  // negative and |C| < |W|, both -W <= C and C <= W will be false, such that
  // we must consider the coordinate as falling outside of both plane sides
  // simultaneously. We test each condition separately and combine them to form
  // a mask of which plane sides we exceeded. If we neglect to consider both
  // sides simultaneously, points can erroneously oscillate from one plane side
  // to the other and exceed the supported maximum number of clip outputs.
  int prevMask = (prevCoord < -prev.w ? NEGATIVE : 0) |
                 (prevCoord > prev.w ? POSITIVE : 0);
  // Loop through points, finding edges that cross the planes by evaluating
  // the side at each point.
  outEdgeMask = 0;
  for (int i = 0; i < nump; i++, edgeMask >>= 1) {
    Point3D cur = p[i];
    Interpolants curInterp = interp[i];
    float curCoord = cur.select(AXIS);
    int curMask =
        (curCoord < -cur.w ? NEGATIVE : 0) | (curCoord > cur.w ? POSITIVE : 0);
    // Check if the previous and current end points are on different sides. If
    // the masks of sides intersect, then we consider them to be on the same
    // side. So in the case the masks do not intersect, we then consider them
    // to fall on different sides.
    if (!(curMask & prevMask)) {
      // One of the edge's end points is outside the plane with the other
      // inside the plane. Find the offset where it crosses the plane and
      // adjust the point and interpolants to there.
      if (prevMask) {
        // Edge that was previously outside crosses inside.
        // Evaluate plane equation for previous and current end-point
        // based on previous side and calculate relative offset.
        if (numClip >= nump + 2) {
          // If for some reason we produced more vertexes than we support, just
          // bail out.
          assert(false);
          return 0;
        }
        // The positive plane is assigned the sign 1, and the negative plane is
        // assigned -1. If the point falls outside both planes, that means W is
        // negative. To compensate for this, we must interpolate the coordinate
        // till W=0, at which point we can choose a single plane side for the
        // coordinate to fall on since W will no longer be negative. To compute
        // the coordinate where W=0, we compute K = prev.w / (prev.w-cur.w) and
        // interpolate C = prev.C + K*(cur.C - prev.C). The sign of C will be
        // the side of the plane we need to consider. Substituting K into the
        // comparison C < 0, we can then avoid the division in K with a
        // cross-multiplication.
        float prevSide =
            (prevMask & NEGATIVE) && (!(prevMask & POSITIVE) ||
                                      prevCoord * (cur.w - prev.w) <
                                          prev.w * (curCoord - prevCoord))
                ? -1
                : 1;
        float prevDist = prevCoord - prevSide * prev.w;
        float curDist = curCoord - prevSide * cur.w;
        // It may happen that after we interpolate by the weight k that due to
        // floating point rounding we've underestimated the value necessary to
        // push it over the clipping boundary. Just in case, nudge the mantissa
        // by a single increment so that we essentially round it up and move it
        // further inside the clipping boundary. We use nextafter to do this in
        // a portable fashion.
        float k = prevDist / (prevDist - curDist);
        Point3D clipped = prev + (cur - prev) * k;
        if (prevSide * clipped.select(AXIS) > clipped.w) {
          k = nextafterf(k, 1.0f);
          clipped = prev + (cur - prev) * k;
        }
        outP[numClip] = clipped;
        outInterp[numClip] = prevInterp + (curInterp - prevInterp) * k;
        // Don't output the current edge mask since start point was outside.
        numClip++;
      }
      if (curMask) {
        // Edge that was previously inside crosses outside.
        // Evaluate plane equation for previous and current end-point
        // based on current side and calculate relative offset.
        if (numClip >= nump + 2) {
          assert(false);
          return 0;
        }
        // In the case the coordinate falls on both plane sides, the computation
        // here is much the same as for prevSide, but since we are going from a
        // previous W that is positive to current W that is negative, then the
        // sign of cur.w - prev.w will flip in the equation. The resulting sign
        // is negated to compensate for this.
        float curSide =
            (curMask & POSITIVE) && (!(curMask & NEGATIVE) ||
                                     prevCoord * (cur.w - prev.w) <
                                         prev.w * (curCoord - prevCoord))
                ? 1
                : -1;
        float prevDist = prevCoord - curSide * prev.w;
        float curDist = curCoord - curSide * cur.w;
        // Calculate interpolation weight k and the nudge it inside clipping
        // boundary with nextafter. Note that since we were previously inside
        // and now crossing outside, we have to flip the nudge direction for
        // the weight towards 0 instead of 1.
        float k = prevDist / (prevDist - curDist);
        Point3D clipped = prev + (cur - prev) * k;
        if (curSide * clipped.select(AXIS) > clipped.w) {
          k = nextafterf(k, 0.0f);
          clipped = prev + (cur - prev) * k;
        }
        outP[numClip] = clipped;
        outInterp[numClip] = prevInterp + (curInterp - prevInterp) * k;
        // Output the current edge mask since the end point is inside.
        outEdgeMask |= (edgeMask & 1) << numClip;
        numClip++;
      }
    }
    if (!curMask) {
      // The current end point is inside the plane, so output point unmodified.
      if (numClip >= nump + 2) {
        assert(false);
        return 0;
      }
      outP[numClip] = cur;
      outInterp[numClip] = curInterp;
      // Output the current edge mask since the end point is inside.
      outEdgeMask |= (edgeMask & 1) << numClip;
      numClip++;
    }
    prev = cur;
    prevInterp = curInterp;
    prevCoord = curCoord;
    prevMask = curMask;
  }
  return numClip;
}

// Helper function to dispatch to perspective span drawing with points that
// have already been transformed and clipped.
static inline void draw_perspective_clipped(int nump, Point3D* p_clip,
                                            Interpolants* interp_clip,
                                            Texture& colortex,
                                            Texture& depthtex) {
  // If polygon is ouside clip rect, nothing to draw.
  ClipRect clipRect(colortex);
  if (!clipRect.overlaps(nump, p_clip)) {
    return;
  }

  // Finally draw perspective-correct spans for the polygon.
  if (colortex.internal_format == GL_RGBA8) {
    draw_perspective_spans<uint32_t>(nump, p_clip, interp_clip, colortex,
                                     depthtex, clipRect);
  } else if (colortex.internal_format == GL_R8) {
    draw_perspective_spans<uint8_t>(nump, p_clip, interp_clip, colortex,
                                    depthtex, clipRect);
  } else {
    assert(false);
  }
}

// Draws a perspective-correct 3D primitive with varying Z value, as opposed
// to a simple 2D planar primitive with a constant Z value that could be
// trivially Z rejected. This requires clipping the primitive against the near
// and far planes to ensure it stays within the valid Z-buffer range. The Z
// and W of each fragment of the primitives are interpolated across the
// generated spans and then depth-tested as appropriate.
// Additionally, vertex attributes must be interpolated with perspective-
// correction by dividing by W before interpolation, and then later multiplied
// by W again to produce the final correct attribute value for each fragment.
// This process is expensive and should be avoided if possible for primitive
// batches that are known ahead of time to not need perspective-correction.
static void draw_perspective(int nump, Interpolants interp_outs[4],
                             Texture& colortex, Texture& depthtex) {
  // Lines are not supported with perspective.
  assert(nump >= 3);
  // Convert output of vertex shader to screen space.
  vec4 pos = vertex_shader->gl_Position;
  vec3_scalar scale =
      vec3_scalar(ctx->viewport.width(), ctx->viewport.height(), 1) * 0.5f;
  vec3_scalar offset =
      make_vec3(make_vec2(ctx->viewport.origin() - colortex.offset), 0.0f) +
      scale;
  // Verify if point is between near and far planes, rejecting NaN.
  if (test_all(pos.z > -pos.w && pos.z < pos.w)) {
    // No points cross the near or far planes, so no clipping required.
    // Just divide coords by W and convert to viewport. We assume the W
    // coordinate is non-zero and the reciprocal is finite since it would
    // otherwise fail the test_none condition.
    Float w = 1.0f / pos.w;
    vec3 screen = pos.sel(X, Y, Z) * w * scale + offset;
    Point3D p[4] = {{screen.x.x, screen.y.x, screen.z.x, w.x},
                    {screen.x.y, screen.y.y, screen.z.y, w.y},
                    {screen.x.z, screen.y.z, screen.z.z, w.z},
                    {screen.x.w, screen.y.w, screen.z.w, w.w}};
    draw_perspective_clipped(nump, p, interp_outs, colortex, depthtex);
  } else {
    // Points cross the near or far planes, so we need to clip.
    // Start with the original 3 or 4 points...
    Point3D p[4] = {{pos.x.x, pos.y.x, pos.z.x, pos.w.x},
                    {pos.x.y, pos.y.y, pos.z.y, pos.w.y},
                    {pos.x.z, pos.y.z, pos.z.z, pos.w.z},
                    {pos.x.w, pos.y.w, pos.z.w, pos.w.w}};
    // Clipping can expand the points by 1 for each of 6 view frustum planes.
    Point3D p_clip[4 + 6];
    Interpolants interp_clip[4 + 6];
    // Clip against near and far Z planes.
    nump = clip_side<Z>(nump, p, interp_outs, p_clip, interp_clip,
                        swgl_AAEdgeMask);
    // If no points are left inside the view frustum, there's nothing to draw.
    if (nump < 3) {
      return;
    }
    // After clipping against only the near and far planes, we might still
    // produce points where W = 0, exactly at the camera plane. OpenGL specifies
    // that for clip coordinates, points must satisfy:
    //   -W <= X <= W
    //   -W <= Y <= W
    //   -W <= Z <= W
    // When Z = W = 0, this is trivially satisfied, but when we transform and
    // divide by W below it will produce a divide by 0. Usually we want to only
    // clip Z to avoid the extra work of clipping X and Y. We can still project
    // points that fall outside the view frustum X and Y so long as Z is valid.
    // The span drawing code will then ensure X and Y are clamped to viewport
    // boundaries. However, in the Z = W = 0 case, sometimes clipping X and Y,
    // will push W further inside the view frustum so that it is no longer 0,
    // allowing us to finally proceed to projecting the points to the screen.
    for (int i = 0; i < nump; i++) {
      // Found an invalid W, so need to clip against X and Y...
      if (p_clip[i].w <= 0.0f) {
        // Ping-pong p_clip -> p_tmp -> p_clip.
        Point3D p_tmp[4 + 6];
        Interpolants interp_tmp[4 + 6];
        nump = clip_side<X>(nump, p_clip, interp_clip, p_tmp, interp_tmp,
                            swgl_AAEdgeMask);
        if (nump < 3) return;
        nump = clip_side<Y>(nump, p_tmp, interp_tmp, p_clip, interp_clip,
                            swgl_AAEdgeMask);
        if (nump < 3) return;
        // After clipping against X and Y planes, there's still points left
        // to draw, so proceed to trying projection now...
        break;
      }
    }
    // Divide coords by W and convert to viewport.
    for (int i = 0; i < nump; i++) {
      float w = 1.0f / p_clip[i].w;
      // If the W coord is essentially zero, small enough that division would
      // result in Inf/NaN, then just set the reciprocal itself to zero so that
      // the coordinates becomes zeroed out, as the only valid point that
      // satisfies -W <= X/Y/Z <= W is all zeroes.
      if (!isfinite(w)) w = 0.0f;
      p_clip[i] = Point3D(p_clip[i].sel(X, Y, Z) * w * scale + offset, w);
    }
    draw_perspective_clipped(nump, p_clip, interp_clip, colortex, depthtex);
  }
}

static void draw_quad(int nump, Texture& colortex, Texture& depthtex) {
  // Run vertex shader once for the primitive's vertices.
  // Reserve space for 6 sets of interpolants, in case we need to clip against
  // near and far planes in the perspective case.
  Interpolants interp_outs[4];
  swgl_ClipFlags = 0;
  vertex_shader->run_primitive((char*)interp_outs, sizeof(Interpolants));
  vec4 pos = vertex_shader->gl_Position;
  // Check if any vertex W is different from another. If so, use perspective.
  if (test_any(pos.w != pos.w.x)) {
    draw_perspective(nump, interp_outs, colortex, depthtex);
    return;
  }

  // Convert output of vertex shader to screen space.
  // Divide coords by W and convert to viewport.
  float w = 1.0f / pos.w.x;
  // If the W coord is essentially zero, small enough that division would
  // result in Inf/NaN, then just set the reciprocal itself to zero so that
  // the coordinates becomes zeroed out, as the only valid point that
  // satisfies -W <= X/Y/Z <= W is all zeroes.
  if (!isfinite(w)) w = 0.0f;
  vec2 screen = (pos.sel(X, Y) * w + 1) * 0.5f *
                    vec2_scalar(ctx->viewport.width(), ctx->viewport.height()) +
                make_vec2(ctx->viewport.origin() - colortex.offset);
  Point2D p[4] = {{screen.x.x, screen.y.x},
                  {screen.x.y, screen.y.y},
                  {screen.x.z, screen.y.z},
                  {screen.x.w, screen.y.w}};

  // If quad is ouside clip rect, nothing to draw.
  ClipRect clipRect(colortex);
  if (!clipRect.overlaps(nump, p)) {
    return;
  }

  // Since the quad is assumed 2D, Z is constant across the quad.
  float screenZ = (pos.z.x * w + 1) * 0.5f;
  if (screenZ < 0 || screenZ > 1) {
    // Z values would cross the near or far plane, so just bail.
    return;
  }
  // Since Z doesn't need to be interpolated, just set the fragment shader's
  // Z and W values here, once and for all fragment shader invocations.
  uint32_t z = uint32_t(MAX_DEPTH_VALUE * screenZ);
  fragment_shader->gl_FragCoord.z = screenZ;
  fragment_shader->gl_FragCoord.w = w;

  // If supplied a line, adjust it so that it is a quad at least 1 pixel thick.
  // Assume that for a line that all 4 SIMD lanes were actually filled with
  // vertexes 0, 1, 1, 0.
  if (nump == 2) {
    // Nudge Y height to span at least 1 pixel by advancing to next pixel
    // boundary so that we step at least 1 row when drawing spans.
    if (int(p[0].y + 0.5f) == int(p[1].y + 0.5f)) {
      p[2].y = 1 + int(p[1].y + 0.5f);
      p[3].y = p[2].y;
      // Nudge X width to span at least 1 pixel so that rounded coords fall on
      // separate pixels.
      if (int(p[0].x + 0.5f) == int(p[1].x + 0.5f)) {
        p[1].x += 1.0f;
        p[2].x += 1.0f;
      }
    } else {
      // If the line already spans at least 1 row, then assume line is vertical
      // or diagonal and just needs to be dilated horizontally.
      p[2].x += 1.0f;
      p[3].x += 1.0f;
    }
    // Pretend that it's a quad now...
    nump = 4;
  }

  // Finally draw 2D spans for the quad. Currently only supports drawing to
  // RGBA8 and R8 color buffers.
  if (colortex.internal_format == GL_RGBA8) {
    draw_quad_spans<uint32_t>(nump, p, z, interp_outs, colortex, depthtex,
                              clipRect);
  } else if (colortex.internal_format == GL_R8) {
    draw_quad_spans<uint8_t>(nump, p, z, interp_outs, colortex, depthtex,
                             clipRect);
  } else {
    assert(false);
  }
}

template <typename INDEX>
static inline void draw_elements(GLsizei count, GLsizei instancecount,
                                 size_t offset, VertexArray& v,
                                 Texture& colortex, Texture& depthtex) {
  Buffer& indices_buf = ctx->buffers[v.element_array_buffer_binding];
  if (!indices_buf.buf || offset >= indices_buf.size) {
    return;
  }
  assert((offset & (sizeof(INDEX) - 1)) == 0);
  INDEX* indices = (INDEX*)(indices_buf.buf + offset);
  count = min(count, (GLsizei)((indices_buf.size - offset) / sizeof(INDEX)));
  // Triangles must be indexed at offsets 0, 1, 2.
  // Quads must be successive triangles indexed at offsets 0, 1, 2, 2, 1, 3.
  if (count == 6 && indices[1] == indices[0] + 1 &&
      indices[2] == indices[0] + 2 && indices[5] == indices[0] + 3) {
    assert(indices[3] == indices[0] + 2 && indices[4] == indices[0] + 1);
    // Fast path - since there is only a single quad, we only load per-vertex
    // attribs once for all instances, as they won't change across instances
    // or within an instance.
    vertex_shader->load_attribs(v.attribs, indices[0], 0, 4);
    draw_quad(4, colortex, depthtex);
    for (GLsizei instance = 1; instance < instancecount; instance++) {
      vertex_shader->load_attribs(v.attribs, indices[0], instance, 0);
      draw_quad(4, colortex, depthtex);
    }
  } else {
    for (GLsizei instance = 0; instance < instancecount; instance++) {
      for (GLsizei i = 0; i + 3 <= count; i += 3) {
        if (indices[i + 1] != indices[i] + 1 ||
            indices[i + 2] != indices[i] + 2) {
          continue;
        }
        if (i + 6 <= count && indices[i + 5] == indices[i] + 3) {
          assert(indices[i + 3] == indices[i] + 2 &&
                 indices[i + 4] == indices[i] + 1);
          vertex_shader->load_attribs(v.attribs, indices[i], instance, 4);
          draw_quad(4, colortex, depthtex);
          i += 3;
        } else {
          vertex_shader->load_attribs(v.attribs, indices[i], instance, 3);
          draw_quad(3, colortex, depthtex);
        }
      }
    }
  }
}
