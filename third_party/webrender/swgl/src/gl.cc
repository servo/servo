/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <stdio.h>
#include <math.h>

#ifdef __MACH__
#  include <mach/mach.h>
#  include <mach/mach_time.h>
#else
#  include <time.h>
#endif

#ifdef NDEBUG
#  define debugf(...)
#else
#  define debugf(...) printf(__VA_ARGS__)
#endif

// #define PRINT_TIMINGS

#ifdef _WIN32
#  define ALWAYS_INLINE __forceinline
#  define NO_INLINE __declspec(noinline)

// Including Windows.h brings a huge amount of namespace polution so just
// define a couple of things manually
typedef int BOOL;
#  define WINAPI __stdcall
#  define DECLSPEC_IMPORT __declspec(dllimport)
#  define WINBASEAPI DECLSPEC_IMPORT
typedef unsigned long DWORD;
typedef long LONG;
typedef __int64 LONGLONG;
#  define DUMMYSTRUCTNAME

typedef union _LARGE_INTEGER {
  struct {
    DWORD LowPart;
    LONG HighPart;
  } DUMMYSTRUCTNAME;
  struct {
    DWORD LowPart;
    LONG HighPart;
  } u;
  LONGLONG QuadPart;
} LARGE_INTEGER;
extern "C" {
WINBASEAPI BOOL WINAPI
QueryPerformanceCounter(LARGE_INTEGER* lpPerformanceCount);

WINBASEAPI BOOL WINAPI QueryPerformanceFrequency(LARGE_INTEGER* lpFrequency);
}

#else
// GCC is slower when dealing with always_inline, especially in debug builds.
// When using Clang, use always_inline more aggressively.
#  if defined(__clang__) || defined(NDEBUG)
#    define ALWAYS_INLINE __attribute__((always_inline)) inline
#  else
#    define ALWAYS_INLINE inline
#  endif
#  define NO_INLINE __attribute__((noinline))
#endif

// Some functions may cause excessive binary bloat if inlined in debug or with
// GCC builds, so use PREFER_INLINE on these instead of ALWAYS_INLINE.
#if defined(__clang__) && defined(NDEBUG)
#  define PREFER_INLINE ALWAYS_INLINE
#else
#  define PREFER_INLINE inline
#endif

#define UNREACHABLE __builtin_unreachable()

#define UNUSED [[maybe_unused]]

#define FALLTHROUGH [[fallthrough]]

#ifdef MOZILLA_CLIENT
#  define IMPLICIT __attribute__((annotate("moz_implicit")))
#else
#  define IMPLICIT
#endif

#include "gl_defs.h"
#include "glsl.h"
#include "program.h"
#include "texture.h"

using namespace glsl;

typedef ivec2_scalar IntPoint;

struct IntRect {
  int x0;
  int y0;
  int x1;
  int y1;

  IntRect() : x0(0), y0(0), x1(0), y1(0) {}
  IntRect(int x0, int y0, int x1, int y1) : x0(x0), y0(y0), x1(x1), y1(y1) {}
  IntRect(IntPoint origin, IntPoint size)
      : x0(origin.x),
        y0(origin.y),
        x1(origin.x + size.x),
        y1(origin.y + size.y) {}

  int width() const { return x1 - x0; }
  int height() const { return y1 - y0; }
  bool is_empty() const { return width() <= 0 || height() <= 0; }

  IntPoint origin() const { return IntPoint(x0, y0); }

  bool same_size(const IntRect& o) const {
    return width() == o.width() && height() == o.height();
  }

  bool contains(const IntRect& o) const {
    return o.x0 >= x0 && o.y0 >= y0 && o.x1 <= x1 && o.y1 <= y1;
  }

  IntRect& intersect(const IntRect& o) {
    x0 = max(x0, o.x0);
    y0 = max(y0, o.y0);
    x1 = min(x1, o.x1);
    y1 = min(y1, o.y1);
    return *this;
  }

  IntRect intersection(const IntRect& o) {
    IntRect result = *this;
    result.intersect(o);
    return result;
  }

  // Scale from source-space to dest-space, optionally rounding inward
  IntRect& scale(int srcWidth, int srcHeight, int dstWidth, int dstHeight,
                 bool roundIn = false) {
    x0 = (x0 * dstWidth + (roundIn ? srcWidth - 1 : 0)) / srcWidth;
    y0 = (y0 * dstHeight + (roundIn ? srcHeight - 1 : 0)) / srcHeight;
    x1 = (x1 * dstWidth) / srcWidth;
    y1 = (y1 * dstHeight) / srcHeight;
    return *this;
  }

  // Flip the rect's Y coords around inflection point at Y=offset
  void invert_y(int offset) {
    y0 = offset - y0;
    y1 = offset - y1;
    swap(y0, y1);
  }

  IntRect& offset(const IntPoint& o) {
    x0 += o.x;
    y0 += o.y;
    x1 += o.x;
    y1 += o.y;
    return *this;
  }

  IntRect operator+(const IntPoint& o) const {
    return IntRect(*this).offset(o);
  }
  IntRect operator-(const IntPoint& o) const {
    return IntRect(*this).offset(-o);
  }
};

typedef vec2_scalar Point2D;
typedef vec4_scalar Point3D;

struct IntRange {
  int start;
  int end;

  int len() const { return end - start; }

  IntRange intersect(IntRange r) const {
    return {max(start, r.start), min(end, r.end)};
  }
};

struct FloatRange {
  float start;
  float end;

  float clip(float x) const { return clamp(x, start, end); }

  FloatRange clip(FloatRange r) const { return {clip(r.start), clip(r.end)}; }

  FloatRange merge(FloatRange r) const {
    return {min(start, r.start), max(end, r.end)};
  }

  IntRange round() const {
    return {int(floor(start + 0.5f)), int(floor(end + 0.5f))};
  }

  IntRange round_out() const { return {int(floor(start)), int(ceil(end))}; }
};

template <typename P>
static inline FloatRange x_range(P p0, P p1) {
  return {min(p0.x, p1.x), max(p0.x, p1.x)};
}

struct VertexAttrib {
  size_t size = 0;  // in bytes
  GLenum type = 0;
  bool normalized = false;
  GLsizei stride = 0;
  GLuint offset = 0;
  bool enabled = false;
  GLuint divisor = 0;
  int vertex_array = 0;
  int vertex_buffer = 0;
  char* buf = nullptr;  // XXX: this can easily dangle
  size_t buf_size = 0;  // this will let us bounds check
};

static int bytes_for_internal_format(GLenum internal_format) {
  switch (internal_format) {
    case GL_RGBA32F:
      return 4 * 4;
    case GL_RGBA32I:
      return 4 * 4;
    case GL_RGBA8:
    case GL_BGRA8:
    case GL_RGBA:
      return 4;
    case GL_R8:
    case GL_RED:
      return 1;
    case GL_RG8:
    case GL_RG:
      return 2;
    case GL_DEPTH_COMPONENT:
    case GL_DEPTH_COMPONENT16:
    case GL_DEPTH_COMPONENT24:
    case GL_DEPTH_COMPONENT32:
      return 4;
    case GL_RGB_RAW_422_APPLE:
      return 2;
    case GL_R16:
      return 2;
    default:
      debugf("internal format: %x\n", internal_format);
      assert(0);
      return 0;
  }
}

static inline int aligned_stride(int row_bytes) { return (row_bytes + 3) & ~3; }

static TextureFormat gl_format_to_texture_format(int type) {
  switch (type) {
    case GL_RGBA32F:
      return TextureFormat::RGBA32F;
    case GL_RGBA32I:
      return TextureFormat::RGBA32I;
    case GL_RGBA8:
      return TextureFormat::RGBA8;
    case GL_R8:
      return TextureFormat::R8;
    case GL_RG8:
      return TextureFormat::RG8;
    case GL_R16:
      return TextureFormat::R16;
    case GL_RGB_RAW_422_APPLE:
      return TextureFormat::YUV422;
    default:
      assert(0);
      return TextureFormat::RGBA8;
  }
}

struct Query {
  uint64_t value = 0;
};

struct Buffer {
  char* buf = nullptr;
  size_t size = 0;
  size_t capacity = 0;

  bool allocate(size_t new_size) {
    // If the size remains unchanged, don't allocate anything.
    if (new_size == size) {
      return false;
    }
    // If the new size is within the existing capacity of the buffer, just
    // reuse the existing buffer.
    if (new_size <= capacity) {
      size = new_size;
      return true;
    }
    // Otherwise we need to reallocate the buffer to hold up to the requested
    // larger size.
    char* new_buf = (char*)realloc(buf, new_size);
    assert(new_buf);
    if (!new_buf) {
      // If we fail, null out the buffer rather than leave around the old
      // allocation state.
      cleanup();
      return false;
    }
    // The reallocation succeeded, so install the buffer.
    buf = new_buf;
    size = new_size;
    capacity = new_size;
    return true;
  }

  void cleanup() {
    if (buf) {
      free(buf);
      buf = nullptr;
      size = 0;
      capacity = 0;
    }
  }

  ~Buffer() { cleanup(); }
};

struct Framebuffer {
  GLuint color_attachment = 0;
  GLuint depth_attachment = 0;
};

struct Renderbuffer {
  GLuint texture = 0;

  void on_erase();
};

TextureFilter gl_filter_to_texture_filter(int type) {
  switch (type) {
    case GL_NEAREST:
      return TextureFilter::NEAREST;
    case GL_NEAREST_MIPMAP_LINEAR:
      return TextureFilter::NEAREST;
    case GL_NEAREST_MIPMAP_NEAREST:
      return TextureFilter::NEAREST;
    case GL_LINEAR:
      return TextureFilter::LINEAR;
    case GL_LINEAR_MIPMAP_LINEAR:
      return TextureFilter::LINEAR;
    case GL_LINEAR_MIPMAP_NEAREST:
      return TextureFilter::LINEAR;
    default:
      assert(0);
      return TextureFilter::NEAREST;
  }
}

struct Texture {
  GLenum internal_format = 0;
  int width = 0;
  int height = 0;
  char* buf = nullptr;
  size_t buf_size = 0;
  uint32_t buf_stride = 0;
  uint8_t buf_bpp = 0;
  GLenum min_filter = GL_NEAREST;
  GLenum mag_filter = GL_LINEAR;
  // The number of active locks on this texture. If this texture has any active
  // locks, we need to disallow modifying or destroying the texture as it may
  // be accessed by other threads where modifications could lead to races.
  int32_t locked = 0;
  // When used as an attachment of a framebuffer, rendering to the texture
  // behaves as if it is located at the given offset such that the offset is
  // subtracted from all transformed vertexes after the viewport is applied.
  IntPoint offset;

  enum FLAGS {
    // If the buffer is internally-allocated by SWGL
    SHOULD_FREE = 1 << 1,
    // If the buffer has been cleared to initialize it. Currently this is only
    // utilized by depth buffers which need to know when depth runs have reset
    // to a valid row state. When unset, the depth runs may contain garbage.
    CLEARED = 1 << 2,
  };
  int flags = SHOULD_FREE;
  bool should_free() const { return bool(flags & SHOULD_FREE); }
  bool cleared() const { return bool(flags & CLEARED); }

  void set_flag(int flag, bool val) {
    if (val) {
      flags |= flag;
    } else {
      flags &= ~flag;
    }
  }
  void set_should_free(bool val) {
    // buf must be null before SHOULD_FREE can be safely toggled. Otherwise, we
    // might accidentally mistakenly realloc an externally allocated buffer as
    // if it were an internally allocated one.
    assert(!buf);
    set_flag(SHOULD_FREE, val);
  }
  void set_cleared(bool val) { set_flag(CLEARED, val); }

  // Delayed-clearing state. When a clear of an FB is requested, we don't
  // immediately clear each row, as the rows may be subsequently overwritten
  // by draw calls, allowing us to skip the work of clearing the affected rows
  // either fully or partially. Instead, we keep a bit vector of rows that need
  // to be cleared later and save the value they need to be cleared with so
  // that we can clear these rows individually when they are touched by draws.
  // This currently only works for 2D textures, but not on texture arrays.
  int delay_clear = 0;
  uint32_t clear_val = 0;
  uint32_t* cleared_rows = nullptr;

  void init_depth_runs(uint32_t z);
  void fill_depth_runs(uint32_t z, const IntRect& scissor);

  void enable_delayed_clear(uint32_t val) {
    delay_clear = height;
    clear_val = val;
    if (!cleared_rows) {
      cleared_rows = new uint32_t[(height + 31) / 32];
    }
    memset(cleared_rows, 0, ((height + 31) / 32) * sizeof(uint32_t));
    if (height & 31) {
      cleared_rows[height / 32] = ~0U << (height & 31);
    }
  }

  void disable_delayed_clear() {
    if (cleared_rows) {
      delete[] cleared_rows;
      cleared_rows = nullptr;
      delay_clear = 0;
    }
  }

  int bpp() const { return buf_bpp; }
  void set_bpp() { buf_bpp = bytes_for_internal_format(internal_format); }

  size_t stride() const { return buf_stride; }
  void set_stride() { buf_stride = aligned_stride(buf_bpp * width); }

  // Set an external backing buffer of this texture.
  void set_buffer(void* new_buf, size_t new_stride) {
    assert(!should_free());
    // Ensure that the supplied stride is at least as big as the row data and
    // is aligned to the smaller of either the BPP or word-size. We need to at
    // least be able to sample data from within a row and sample whole pixels
    // of smaller formats without risking unaligned access.
    set_bpp();
    set_stride();
    assert(new_stride >= size_t(bpp() * width) &&
           new_stride % min(bpp(), sizeof(uint32_t)) == 0);

    buf = (char*)new_buf;
    buf_size = 0;
    buf_stride = new_stride;
  }

  bool allocate(bool force = false, int min_width = 0, int min_height = 0) {
    assert(!locked);  // Locked textures shouldn't be reallocated
    // If we get here, some GL API call that invalidates the texture was used.
    // Mark the buffer as not-cleared to signal this.
    set_cleared(false);
    // Check if there is either no buffer currently or if we forced validation
    // of the buffer size because some dimension might have changed.
    if ((!buf || force) && should_free()) {
      // Initialize the buffer's BPP and stride, since they may have changed.
      set_bpp();
      set_stride();
      // Compute new size based on the maximum potential stride, rather than
      // the current stride, to hopefully avoid reallocations when size would
      // otherwise change too much...
      size_t max_stride = max(buf_stride, aligned_stride(buf_bpp * min_width));
      size_t size = max_stride * max(height, min_height);
      if ((!buf && size > 0) || size > buf_size) {
        // Allocate with a SIMD register-sized tail of padding at the end so we
        // can safely read or write past the end of the texture with SIMD ops.
        // Currently only the flat Z-buffer texture needs this padding due to
        // full-register loads and stores in check_depth and discard_depth. In
        // case some code in the future accidentally uses a linear filter on a
        // texture with less than 2 pixels per row, we also add this padding
        // just to be safe. All other texture types and use-cases should be
        // safe to omit padding.
        size_t padding =
            internal_format == GL_DEPTH_COMPONENT24 || max(width, min_width) < 2
                ? sizeof(Float)
                : 0;
        char* new_buf = (char*)realloc(buf, size + padding);
        assert(new_buf);
        if (new_buf) {
          // Successfully reallocated the buffer, so go ahead and set it.
          buf = new_buf;
          buf_size = size;
          return true;
        }
        // Allocation failed, so ensure we don't leave stale buffer state.
        cleanup();
      }
    }
    // Nothing changed...
    return false;
  }

  void cleanup() {
    assert(!locked);  // Locked textures shouldn't be destroyed
    if (buf) {
      // If we need to toggle SHOULD_FREE state, ensure that buf is nulled out,
      // regardless of whether we internally allocated it. This will prevent us
      // from wrongly treating buf as having been internally allocated for when
      // we go to realloc if it actually was externally allocted.
      if (should_free()) {
        free(buf);
      }
      buf = nullptr;
      buf_size = 0;
      buf_bpp = 0;
      buf_stride = 0;
    }
    disable_delayed_clear();
  }

  ~Texture() { cleanup(); }

  IntRect bounds() const { return IntRect{0, 0, width, height}; }
  IntRect offset_bounds() const { return bounds() + offset; }

  // Find the valid sampling bounds relative to the requested region
  IntRect sample_bounds(const IntRect& req, bool invertY = false) const {
    IntRect bb = bounds().intersect(req) - req.origin();
    if (invertY) bb.invert_y(req.height());
    return bb;
  }

  // Get a pointer for sampling at the given offset
  char* sample_ptr(int x, int y) const {
    return buf + y * stride() + x * bpp();
  }

  // Get a pointer for sampling the requested region and limit to the provided
  // sampling bounds
  char* sample_ptr(const IntRect& req, const IntRect& bounds,
                   bool invertY = false) const {
    // Offset the sample pointer by the clamped bounds
    int x = req.x0 + bounds.x0;
    // Invert the Y offset if necessary
    int y = invertY ? req.y1 - 1 - bounds.y0 : req.y0 + bounds.y0;
    return sample_ptr(x, y);
  }
};

// The last vertex attribute is reserved as a null attribute in case a vertex
// attribute is used without being set.
#define MAX_ATTRIBS 17
#define NULL_ATTRIB 16
struct VertexArray {
  VertexAttrib attribs[MAX_ATTRIBS];
  int max_attrib = -1;
  // The GL spec defines element array buffer binding to be part of VAO state.
  GLuint element_array_buffer_binding = 0;

  void validate();
};

struct Shader {
  GLenum type = 0;
  ProgramLoader loader = nullptr;
};

struct Program {
  ProgramImpl* impl = nullptr;
  VertexShaderImpl* vert_impl = nullptr;
  FragmentShaderImpl* frag_impl = nullptr;
  bool deleted = false;

  ~Program() { delete impl; }
};

// clang-format off
// Fully-expand GL defines while ignoring more than 4 suffixes
#define CONCAT_KEY(prefix, x, y, z, w, ...) prefix##x##y##z##w
// Generate a blend key enum symbol
#define BLEND_KEY(...) CONCAT_KEY(BLEND_, __VA_ARGS__, 0, 0, 0)
#define MASK_BLEND_KEY(...) CONCAT_KEY(MASK_BLEND_, __VA_ARGS__, 0, 0, 0)
#define AA_BLEND_KEY(...) CONCAT_KEY(AA_BLEND_, __VA_ARGS__, 0, 0, 0)
#define AA_MASK_BLEND_KEY(...) CONCAT_KEY(AA_MASK_BLEND_, __VA_ARGS__, 0, 0, 0)

// Utility macro to easily generate similar code for all implemented blend modes
#define FOR_EACH_BLEND_KEY(macro)                                              \
  macro(GL_ONE, GL_ZERO, 0, 0)                                                 \
  macro(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE_MINUS_SRC_ALPHA)  \
  macro(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, 0, 0)                                  \
  macro(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, 0, 0)                                 \
  macro(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, GL_ZERO, GL_ONE)                      \
  macro(GL_ZERO, GL_ONE_MINUS_SRC_ALPHA, 0, 0)                                 \
  macro(GL_ZERO, GL_SRC_COLOR, 0, 0)                                           \
  macro(GL_ONE, GL_ONE, 0, 0)                                                  \
  macro(GL_ONE, GL_ONE, GL_ONE, GL_ONE_MINUS_SRC_ALPHA)                        \
  macro(GL_ONE_MINUS_DST_ALPHA, GL_ONE, GL_ZERO, GL_ONE)                       \
  macro(GL_CONSTANT_COLOR, GL_ONE_MINUS_SRC_COLOR, 0, 0)                       \
  macro(GL_ONE, GL_ONE_MINUS_SRC1_COLOR, 0, 0)                                 \
  macro(GL_MIN, 0, 0, 0)                                                       \
  macro(GL_MAX, 0, 0, 0)                                                       \
  macro(GL_MULTIPLY_KHR, 0, 0, 0)                                              \
  macro(GL_SCREEN_KHR, 0, 0, 0)                                                \
  macro(GL_OVERLAY_KHR, 0, 0, 0)                                               \
  macro(GL_DARKEN_KHR, 0, 0, 0)                                                \
  macro(GL_LIGHTEN_KHR, 0, 0, 0)                                               \
  macro(GL_COLORDODGE_KHR, 0, 0, 0)                                            \
  macro(GL_COLORBURN_KHR, 0, 0, 0)                                             \
  macro(GL_HARDLIGHT_KHR, 0, 0, 0)                                             \
  macro(GL_SOFTLIGHT_KHR, 0, 0, 0)                                             \
  macro(GL_DIFFERENCE_KHR, 0, 0, 0)                                            \
  macro(GL_EXCLUSION_KHR, 0, 0, 0)                                             \
  macro(GL_HSL_HUE_KHR, 0, 0, 0)                                               \
  macro(GL_HSL_SATURATION_KHR, 0, 0, 0)                                        \
  macro(GL_HSL_COLOR_KHR, 0, 0, 0)                                             \
  macro(GL_HSL_LUMINOSITY_KHR, 0, 0, 0)                                        \
  macro(SWGL_BLEND_DROP_SHADOW, 0, 0, 0)                                       \
  macro(SWGL_BLEND_SUBPIXEL_TEXT, 0, 0, 0)

#define DEFINE_BLEND_KEY(...) BLEND_KEY(__VA_ARGS__),
#define DEFINE_MASK_BLEND_KEY(...) MASK_BLEND_KEY(__VA_ARGS__),
#define DEFINE_AA_BLEND_KEY(...) AA_BLEND_KEY(__VA_ARGS__),
#define DEFINE_AA_MASK_BLEND_KEY(...) AA_MASK_BLEND_KEY(__VA_ARGS__),
enum BlendKey : uint8_t {
  FOR_EACH_BLEND_KEY(DEFINE_BLEND_KEY)
  FOR_EACH_BLEND_KEY(DEFINE_MASK_BLEND_KEY)
  FOR_EACH_BLEND_KEY(DEFINE_AA_BLEND_KEY)
  FOR_EACH_BLEND_KEY(DEFINE_AA_MASK_BLEND_KEY)
  BLEND_KEY_NONE = BLEND_KEY(GL_ONE, GL_ZERO),
  MASK_BLEND_KEY_NONE = MASK_BLEND_KEY(GL_ONE, GL_ZERO),
  AA_BLEND_KEY_NONE = AA_BLEND_KEY(GL_ONE, GL_ZERO),
  AA_MASK_BLEND_KEY_NONE = AA_MASK_BLEND_KEY(GL_ONE, GL_ZERO),
};
// clang-format on

const size_t MAX_TEXTURE_UNITS = 16;

template <typename T>
static inline bool unlink(T& binding, T n) {
  if (binding == n) {
    binding = 0;
    return true;
  }
  return false;
}

template <typename O>
struct ObjectStore {
  O** objects = nullptr;
  size_t size = 0;
  // reserve object 0 as null
  size_t first_free = 1;
  O invalid;

  ~ObjectStore() {
    if (objects) {
      for (size_t i = 0; i < size; i++) delete objects[i];
      free(objects);
    }
  }

  bool grow(size_t i) {
    size_t new_size = size ? size : 8;
    while (new_size <= i) new_size += new_size / 2;
    O** new_objects = (O**)realloc(objects, new_size * sizeof(O*));
    assert(new_objects);
    if (!new_objects) return false;
    while (size < new_size) new_objects[size++] = nullptr;
    objects = new_objects;
    return true;
  }

  void insert(size_t i, const O& o) {
    if (i >= size && !grow(i)) return;
    if (!objects[i]) objects[i] = new O(o);
  }

  size_t next_free() {
    size_t i = first_free;
    while (i < size && objects[i]) i++;
    first_free = i;
    return i;
  }

  size_t insert(const O& o = O()) {
    size_t i = next_free();
    insert(i, o);
    return i;
  }

  O& operator[](size_t i) {
    insert(i, O());
    return i < size ? *objects[i] : invalid;
  }

  O* find(size_t i) const { return i < size ? objects[i] : nullptr; }

  template <typename T>
  void on_erase(T*, ...) {}
  template <typename T>
  void on_erase(T* o, decltype(&T::on_erase)) {
    o->on_erase();
  }

  bool erase(size_t i) {
    if (i < size && objects[i]) {
      on_erase(objects[i], nullptr);
      delete objects[i];
      objects[i] = nullptr;
      if (i < first_free) first_free = i;
      return true;
    }
    return false;
  }

  O** begin() const { return objects; }
  O** end() const { return &objects[size]; }
};

struct Context {
  int32_t references = 1;

  ObjectStore<Query> queries;
  ObjectStore<Buffer> buffers;
  ObjectStore<Texture> textures;
  ObjectStore<VertexArray> vertex_arrays;
  ObjectStore<Framebuffer> framebuffers;
  ObjectStore<Renderbuffer> renderbuffers;
  ObjectStore<Shader> shaders;
  ObjectStore<Program> programs;

  IntRect viewport = {0, 0, 0, 0};

  bool blend = false;
  GLenum blendfunc_srgb = GL_ONE;
  GLenum blendfunc_drgb = GL_ZERO;
  GLenum blendfunc_sa = GL_ONE;
  GLenum blendfunc_da = GL_ZERO;
  GLenum blend_equation = GL_FUNC_ADD;
  V8<uint16_t> blendcolor = 0;
  BlendKey blend_key = BLEND_KEY_NONE;

  bool depthtest = false;
  bool depthmask = true;
  GLenum depthfunc = GL_LESS;

  bool scissortest = false;
  IntRect scissor = {0, 0, 0, 0};

  GLfloat clearcolor[4] = {0, 0, 0, 0};
  GLdouble cleardepth = 1;

  int unpack_row_length = 0;

  int shaded_rows = 0;
  int shaded_pixels = 0;

  struct TextureUnit {
    GLuint texture_2d_binding = 0;
    GLuint texture_rectangle_binding = 0;

    void unlink(GLuint n) {
      ::unlink(texture_2d_binding, n);
      ::unlink(texture_rectangle_binding, n);
    }
  };
  TextureUnit texture_units[MAX_TEXTURE_UNITS];
  int active_texture_unit = 0;

  GLuint current_program = 0;

  GLuint current_vertex_array = 0;
  bool validate_vertex_array = true;

  GLuint pixel_pack_buffer_binding = 0;
  GLuint pixel_unpack_buffer_binding = 0;
  GLuint array_buffer_binding = 0;
  GLuint time_elapsed_query = 0;
  GLuint samples_passed_query = 0;
  GLuint renderbuffer_binding = 0;
  GLuint draw_framebuffer_binding = 0;
  GLuint read_framebuffer_binding = 0;
  GLuint unknown_binding = 0;

  GLuint& get_binding(GLenum name) {
    switch (name) {
      case GL_PIXEL_PACK_BUFFER:
        return pixel_pack_buffer_binding;
      case GL_PIXEL_UNPACK_BUFFER:
        return pixel_unpack_buffer_binding;
      case GL_ARRAY_BUFFER:
        return array_buffer_binding;
      case GL_ELEMENT_ARRAY_BUFFER:
        return vertex_arrays[current_vertex_array].element_array_buffer_binding;
      case GL_TEXTURE_2D:
        return texture_units[active_texture_unit].texture_2d_binding;
      case GL_TEXTURE_RECTANGLE:
        return texture_units[active_texture_unit].texture_rectangle_binding;
      case GL_TIME_ELAPSED:
        return time_elapsed_query;
      case GL_SAMPLES_PASSED:
        return samples_passed_query;
      case GL_RENDERBUFFER:
        return renderbuffer_binding;
      case GL_DRAW_FRAMEBUFFER:
        return draw_framebuffer_binding;
      case GL_READ_FRAMEBUFFER:
        return read_framebuffer_binding;
      default:
        debugf("unknown binding %x\n", name);
        assert(false);
        return unknown_binding;
    }
  }

  Texture& get_texture(sampler2D, int unit) {
    return textures[texture_units[unit].texture_2d_binding];
  }

  Texture& get_texture(isampler2D, int unit) {
    return textures[texture_units[unit].texture_2d_binding];
  }

  Texture& get_texture(sampler2DRect, int unit) {
    return textures[texture_units[unit].texture_rectangle_binding];
  }

  IntRect apply_scissor(IntRect bb,
                        const IntPoint& origin = IntPoint(0, 0)) const {
    return scissortest ? bb.intersect(scissor - origin) : bb;
  }

  IntRect apply_scissor(const Texture& t) const {
    return apply_scissor(t.bounds(), t.offset);
  }
};
static Context* ctx = nullptr;
static VertexShaderImpl* vertex_shader = nullptr;
static FragmentShaderImpl* fragment_shader = nullptr;
static BlendKey blend_key = BLEND_KEY_NONE;

static void prepare_texture(Texture& t, const IntRect* skip = nullptr);

template <typename S>
static inline void init_filter(S* s, Texture& t) {
  // If the width is not at least 2 pixels, then we can't safely sample the end
  // of the row with a linear filter. In that case, just punt to using nearest
  // filtering instead.
  s->filter = t.width >= 2 ? gl_filter_to_texture_filter(t.mag_filter)
                           : TextureFilter::NEAREST;
}

template <typename S>
static inline void init_sampler(S* s, Texture& t) {
  prepare_texture(t);
  s->width = t.width;
  s->height = t.height;
  s->stride = t.stride();
  int bpp = t.bpp();
  if (bpp >= 4)
    s->stride /= 4;
  else if (bpp == 2)
    s->stride /= 2;
  else
    assert(bpp == 1);
  // Use uint32_t* for easier sampling, but need to cast to uint8_t* or
  // uint16_t* for formats with bpp < 4.
  s->buf = (uint32_t*)t.buf;
  s->format = gl_format_to_texture_format(t.internal_format);
}

template <typename S>
static inline void null_sampler(S* s) {
  // For null texture data, just make the sampler provide a 1x1 buffer that is
  // transparent black. Ensure buffer holds at least a SIMD vector of zero data
  // for SIMD padding of unaligned loads.
  static const uint32_t zeroBuf[sizeof(Float) / sizeof(uint32_t)] = {0};
  s->width = 1;
  s->height = 1;
  s->stride = s->width;
  s->buf = (uint32_t*)zeroBuf;
  s->format = TextureFormat::RGBA8;
}

template <typename S>
static inline void null_filter(S* s) {
  s->filter = TextureFilter::NEAREST;
}

template <typename S>
S* lookup_sampler(S* s, int texture) {
  Texture& t = ctx->get_texture(s, texture);
  if (!t.buf) {
    null_sampler(s);
    null_filter(s);
  } else {
    init_sampler(s, t);
    init_filter(s, t);
  }
  return s;
}

template <typename S>
S* lookup_isampler(S* s, int texture) {
  Texture& t = ctx->get_texture(s, texture);
  if (!t.buf) {
    null_sampler(s);
  } else {
    init_sampler(s, t);
  }
  return s;
}

int bytes_per_type(GLenum type) {
  switch (type) {
    case GL_INT:
      return 4;
    case GL_FLOAT:
      return 4;
    case GL_UNSIGNED_SHORT:
      return 2;
    case GL_UNSIGNED_BYTE:
      return 1;
    default:
      assert(0);
      return 0;
  }
}

template <typename S, typename C>
static inline S expand_attrib(const char* buf, size_t size, bool normalized) {
  typedef typename ElementType<S>::ty elem_type;
  S scalar = {0};
  const C* src = reinterpret_cast<const C*>(buf);
  if (normalized) {
    const float scale = 1.0f / ((1 << (8 * sizeof(C))) - 1);
    for (size_t i = 0; i < size / sizeof(C); i++) {
      put_nth_component(scalar, i, elem_type(src[i]) * scale);
    }
  } else {
    for (size_t i = 0; i < size / sizeof(C); i++) {
      put_nth_component(scalar, i, elem_type(src[i]));
    }
  }
  return scalar;
}

template <typename S>
static inline S load_attrib_scalar(VertexAttrib& va, const char* src) {
  if (sizeof(S) <= va.size) {
    return *reinterpret_cast<const S*>(src);
  }
  if (va.type == GL_UNSIGNED_SHORT) {
    return expand_attrib<S, uint16_t>(src, va.size, va.normalized);
  }
  if (va.type == GL_UNSIGNED_BYTE) {
    return expand_attrib<S, uint8_t>(src, va.size, va.normalized);
  }
  assert(sizeof(typename ElementType<S>::ty) == bytes_per_type(va.type));
  S scalar = {0};
  memcpy(&scalar, src, va.size);
  return scalar;
}

template <typename T>
void load_attrib(T& attrib, VertexAttrib& va, uint32_t start, int instance,
                 int count) {
  typedef decltype(force_scalar(attrib)) scalar_type;
  if (!va.enabled) {
    attrib = T(scalar_type{0});
  } else if (va.divisor != 0) {
    char* src = (char*)va.buf + va.stride * instance + va.offset;
    assert(src + va.size <= va.buf + va.buf_size);
    attrib = T(load_attrib_scalar<scalar_type>(va, src));
  } else {
    // Specialized for WR's primitive vertex order/winding.
    if (!count) return;
    assert(count >= 2 && count <= 4);
    char* src = (char*)va.buf + va.stride * start + va.offset;
    switch (count) {
      case 2: {
        // Lines must be indexed at offsets 0, 1.
        // Line vertexes fill vertex shader SIMD lanes as 0, 1, 1, 0.
        scalar_type lanes[2] = {
            load_attrib_scalar<scalar_type>(va, src),
            load_attrib_scalar<scalar_type>(va, src + va.stride)};
        attrib = (T){lanes[0], lanes[1], lanes[1], lanes[0]};
        break;
      }
      case 3: {
        // Triangles must be indexed at offsets 0, 1, 2.
        // Triangle vertexes fill vertex shader SIMD lanes as 0, 1, 2, 2.
        scalar_type lanes[3] = {
            load_attrib_scalar<scalar_type>(va, src),
            load_attrib_scalar<scalar_type>(va, src + va.stride),
            load_attrib_scalar<scalar_type>(va, src + va.stride * 2)};
        attrib = (T){lanes[0], lanes[1], lanes[2], lanes[2]};
        break;
      }
      default:
        // Quads must be successive triangles indexed at offsets 0, 1, 2, 2,
        // 1, 3. Quad vertexes fill vertex shader SIMD lanes as 0, 1, 3, 2, so
        // that the points form a convex path that can be traversed by the
        // rasterizer.
        attrib = (T){load_attrib_scalar<scalar_type>(va, src),
                     load_attrib_scalar<scalar_type>(va, src + va.stride),
                     load_attrib_scalar<scalar_type>(va, src + va.stride * 3),
                     load_attrib_scalar<scalar_type>(va, src + va.stride * 2)};
        break;
    }
  }
}

template <typename T>
void load_flat_attrib(T& attrib, VertexAttrib& va, uint32_t start, int instance,
                      int count) {
  typedef decltype(force_scalar(attrib)) scalar_type;
  if (!va.enabled) {
    attrib = T{0};
    return;
  }
  char* src = nullptr;
  if (va.divisor != 0) {
    src = (char*)va.buf + va.stride * instance + va.offset;
  } else {
    if (!count) return;
    src = (char*)va.buf + va.stride * start + va.offset;
  }
  assert(src + va.size <= va.buf + va.buf_size);
  attrib = T(load_attrib_scalar<scalar_type>(va, src));
}

void setup_program(GLuint program) {
  if (!program) {
    vertex_shader = nullptr;
    fragment_shader = nullptr;
    return;
  }
  Program& p = ctx->programs[program];
  assert(p.impl);
  assert(p.vert_impl);
  assert(p.frag_impl);
  vertex_shader = p.vert_impl;
  fragment_shader = p.frag_impl;
}

extern ProgramLoader load_shader(const char* name);

extern "C" {

void UseProgram(GLuint program) {
  if (ctx->current_program && program != ctx->current_program) {
    auto* p = ctx->programs.find(ctx->current_program);
    if (p && p->deleted) {
      ctx->programs.erase(ctx->current_program);
    }
  }
  ctx->current_program = program;
  setup_program(program);
}

void SetViewport(GLint x, GLint y, GLsizei width, GLsizei height) {
  ctx->viewport = IntRect{x, y, x + width, y + height};
}

void Enable(GLenum cap) {
  switch (cap) {
    case GL_BLEND:
      ctx->blend = true;
      break;
    case GL_DEPTH_TEST:
      ctx->depthtest = true;
      break;
    case GL_SCISSOR_TEST:
      ctx->scissortest = true;
      break;
  }
}

void Disable(GLenum cap) {
  switch (cap) {
    case GL_BLEND:
      ctx->blend = false;
      break;
    case GL_DEPTH_TEST:
      ctx->depthtest = false;
      break;
    case GL_SCISSOR_TEST:
      ctx->scissortest = false;
      break;
  }
}

GLenum GetError() { return GL_NO_ERROR; }

static const char* const extensions[] = {
    "GL_ARB_blend_func_extended",
    "GL_ARB_clear_texture",
    "GL_ARB_copy_image",
    "GL_ARB_draw_instanced",
    "GL_ARB_explicit_attrib_location",
    "GL_ARB_instanced_arrays",
    "GL_ARB_invalidate_subdata",
    "GL_ARB_texture_storage",
    "GL_EXT_timer_query",
    "GL_KHR_blend_equation_advanced",
    "GL_KHR_blend_equation_advanced_coherent",
    "GL_APPLE_rgb_422",
};

void GetIntegerv(GLenum pname, GLint* params) {
  assert(params);
  switch (pname) {
    case GL_MAX_TEXTURE_UNITS:
    case GL_MAX_TEXTURE_IMAGE_UNITS:
      params[0] = MAX_TEXTURE_UNITS;
      break;
    case GL_MAX_TEXTURE_SIZE:
      params[0] = 1 << 15;
      break;
    case GL_MAX_ARRAY_TEXTURE_LAYERS:
      params[0] = 0;
      break;
    case GL_READ_FRAMEBUFFER_BINDING:
      params[0] = ctx->read_framebuffer_binding;
      break;
    case GL_DRAW_FRAMEBUFFER_BINDING:
      params[0] = ctx->draw_framebuffer_binding;
      break;
    case GL_PIXEL_PACK_BUFFER_BINDING:
      params[0] = ctx->pixel_pack_buffer_binding;
      break;
    case GL_PIXEL_UNPACK_BUFFER_BINDING:
      params[0] = ctx->pixel_unpack_buffer_binding;
      break;
    case GL_NUM_EXTENSIONS:
      params[0] = sizeof(extensions) / sizeof(extensions[0]);
      break;
    case GL_MAJOR_VERSION:
      params[0] = 3;
      break;
    case GL_MINOR_VERSION:
      params[0] = 2;
      break;
    default:
      debugf("unhandled glGetIntegerv parameter %x\n", pname);
      assert(false);
  }
}

void GetBooleanv(GLenum pname, GLboolean* params) {
  assert(params);
  switch (pname) {
    case GL_DEPTH_WRITEMASK:
      params[0] = ctx->depthmask;
      break;
    default:
      debugf("unhandled glGetBooleanv parameter %x\n", pname);
      assert(false);
  }
}

const char* GetString(GLenum name) {
  switch (name) {
    case GL_VENDOR:
      return "Mozilla Gfx";
    case GL_RENDERER:
      return "Software WebRender";
    case GL_VERSION:
      return "3.2";
    case GL_SHADING_LANGUAGE_VERSION:
      return "1.50";
    default:
      debugf("unhandled glGetString parameter %x\n", name);
      assert(false);
      return nullptr;
  }
}

const char* GetStringi(GLenum name, GLuint index) {
  switch (name) {
    case GL_EXTENSIONS:
      if (index >= sizeof(extensions) / sizeof(extensions[0])) {
        return nullptr;
      }
      return extensions[index];
    default:
      debugf("unhandled glGetStringi parameter %x\n", name);
      assert(false);
      return nullptr;
  }
}

GLenum remap_blendfunc(GLenum rgb, GLenum a) {
  switch (a) {
    case GL_SRC_ALPHA:
      if (rgb == GL_SRC_COLOR) a = GL_SRC_COLOR;
      break;
    case GL_ONE_MINUS_SRC_ALPHA:
      if (rgb == GL_ONE_MINUS_SRC_COLOR) a = GL_ONE_MINUS_SRC_COLOR;
      break;
    case GL_DST_ALPHA:
      if (rgb == GL_DST_COLOR) a = GL_DST_COLOR;
      break;
    case GL_ONE_MINUS_DST_ALPHA:
      if (rgb == GL_ONE_MINUS_DST_COLOR) a = GL_ONE_MINUS_DST_COLOR;
      break;
    case GL_CONSTANT_ALPHA:
      if (rgb == GL_CONSTANT_COLOR) a = GL_CONSTANT_COLOR;
      break;
    case GL_ONE_MINUS_CONSTANT_ALPHA:
      if (rgb == GL_ONE_MINUS_CONSTANT_COLOR) a = GL_ONE_MINUS_CONSTANT_COLOR;
      break;
    case GL_SRC_COLOR:
      if (rgb == GL_SRC_ALPHA) a = GL_SRC_ALPHA;
      break;
    case GL_ONE_MINUS_SRC_COLOR:
      if (rgb == GL_ONE_MINUS_SRC_ALPHA) a = GL_ONE_MINUS_SRC_ALPHA;
      break;
    case GL_DST_COLOR:
      if (rgb == GL_DST_ALPHA) a = GL_DST_ALPHA;
      break;
    case GL_ONE_MINUS_DST_COLOR:
      if (rgb == GL_ONE_MINUS_DST_ALPHA) a = GL_ONE_MINUS_DST_ALPHA;
      break;
    case GL_CONSTANT_COLOR:
      if (rgb == GL_CONSTANT_ALPHA) a = GL_CONSTANT_ALPHA;
      break;
    case GL_ONE_MINUS_CONSTANT_COLOR:
      if (rgb == GL_ONE_MINUS_CONSTANT_ALPHA) a = GL_ONE_MINUS_CONSTANT_ALPHA;
      break;
    case GL_SRC1_ALPHA:
      if (rgb == GL_SRC1_COLOR) a = GL_SRC1_COLOR;
      break;
    case GL_ONE_MINUS_SRC1_ALPHA:
      if (rgb == GL_ONE_MINUS_SRC1_COLOR) a = GL_ONE_MINUS_SRC1_COLOR;
      break;
    case GL_SRC1_COLOR:
      if (rgb == GL_SRC1_ALPHA) a = GL_SRC1_ALPHA;
      break;
    case GL_ONE_MINUS_SRC1_COLOR:
      if (rgb == GL_ONE_MINUS_SRC1_ALPHA) a = GL_ONE_MINUS_SRC1_ALPHA;
      break;
  }
  return a;
}

// Generate a hashed blend key based on blend func and equation state. This
// allows all the blend state to be processed down to a blend key that can be
// dealt with inside a single switch statement.
static void hash_blend_key() {
  GLenum srgb = ctx->blendfunc_srgb;
  GLenum drgb = ctx->blendfunc_drgb;
  GLenum sa = ctx->blendfunc_sa;
  GLenum da = ctx->blendfunc_da;
  GLenum equation = ctx->blend_equation;
#define HASH_BLEND_KEY(x, y, z, w) ((x << 4) | (y) | (z << 24) | (w << 20))
  // Basic non-separate blend funcs used the two argument form
  int hash = HASH_BLEND_KEY(srgb, drgb, 0, 0);
  // Separate alpha blend funcs use the 4 argument hash
  if (srgb != sa || drgb != da) hash |= HASH_BLEND_KEY(0, 0, sa, da);
  // Any other blend equation than the default func_add ignores the func and
  // instead generates a one-argument hash based on the equation
  if (equation != GL_FUNC_ADD) hash = HASH_BLEND_KEY(equation, 0, 0, 0);
  switch (hash) {
#define MAP_BLEND_KEY(...)                   \
  case HASH_BLEND_KEY(__VA_ARGS__):          \
    ctx->blend_key = BLEND_KEY(__VA_ARGS__); \
    break;
    FOR_EACH_BLEND_KEY(MAP_BLEND_KEY)
    default:
      debugf("blendfunc: %x, %x, separate: %x, %x, equation: %x\n", srgb, drgb,
             sa, da, equation);
      assert(false);
      break;
  }
}

void BlendFunc(GLenum srgb, GLenum drgb, GLenum sa, GLenum da) {
  ctx->blendfunc_srgb = srgb;
  ctx->blendfunc_drgb = drgb;
  sa = remap_blendfunc(srgb, sa);
  da = remap_blendfunc(drgb, da);
  ctx->blendfunc_sa = sa;
  ctx->blendfunc_da = da;

  hash_blend_key();
}

void BlendColor(GLfloat r, GLfloat g, GLfloat b, GLfloat a) {
  I32 c = round_pixel((Float){b, g, r, a});
  ctx->blendcolor = CONVERT(c, U16).xyzwxyzw;
}

void BlendEquation(GLenum mode) {
  assert(mode == GL_FUNC_ADD || mode == GL_MIN || mode == GL_MAX ||
         (mode >= GL_MULTIPLY_KHR && mode <= GL_HSL_LUMINOSITY_KHR));
  if (mode != ctx->blend_equation) {
    ctx->blend_equation = mode;
    hash_blend_key();
  }
}

void DepthMask(GLboolean flag) { ctx->depthmask = flag; }

void DepthFunc(GLenum func) {
  switch (func) {
    case GL_LESS:
    case GL_LEQUAL:
      break;
    default:
      assert(false);
  }
  ctx->depthfunc = func;
}

void SetScissor(GLint x, GLint y, GLsizei width, GLsizei height) {
  ctx->scissor = IntRect{x, y, x + width, y + height};
}

void ClearColor(GLfloat r, GLfloat g, GLfloat b, GLfloat a) {
  ctx->clearcolor[0] = r;
  ctx->clearcolor[1] = g;
  ctx->clearcolor[2] = b;
  ctx->clearcolor[3] = a;
}

void ClearDepth(GLdouble depth) { ctx->cleardepth = depth; }

void ActiveTexture(GLenum texture) {
  assert(texture >= GL_TEXTURE0);
  assert(texture < GL_TEXTURE0 + MAX_TEXTURE_UNITS);
  ctx->active_texture_unit =
      clamp(int(texture - GL_TEXTURE0), 0, int(MAX_TEXTURE_UNITS - 1));
}

void GenQueries(GLsizei n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    Query q;
    result[i] = ctx->queries.insert(q);
  }
}

void DeleteQuery(GLuint n) {
  if (n && ctx->queries.erase(n)) {
    unlink(ctx->time_elapsed_query, n);
    unlink(ctx->samples_passed_query, n);
  }
}

void GenBuffers(int n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    Buffer b;
    result[i] = ctx->buffers.insert(b);
  }
}

void DeleteBuffer(GLuint n) {
  if (n && ctx->buffers.erase(n)) {
    unlink(ctx->pixel_pack_buffer_binding, n);
    unlink(ctx->pixel_unpack_buffer_binding, n);
    unlink(ctx->array_buffer_binding, n);
  }
}

void GenVertexArrays(int n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    VertexArray v;
    result[i] = ctx->vertex_arrays.insert(v);
  }
}

void DeleteVertexArray(GLuint n) {
  if (n && ctx->vertex_arrays.erase(n)) {
    unlink(ctx->current_vertex_array, n);
  }
}

GLuint CreateShader(GLenum type) {
  Shader s;
  s.type = type;
  return ctx->shaders.insert(s);
}

void ShaderSourceByName(GLuint shader, char* name) {
  Shader& s = ctx->shaders[shader];
  s.loader = load_shader(name);
  if (!s.loader) {
    debugf("unknown shader %s\n", name);
  }
}

void AttachShader(GLuint program, GLuint shader) {
  Program& p = ctx->programs[program];
  Shader& s = ctx->shaders[shader];
  if (s.type == GL_VERTEX_SHADER) {
    if (!p.impl && s.loader) p.impl = s.loader();
  } else if (s.type == GL_FRAGMENT_SHADER) {
    if (!p.impl && s.loader) p.impl = s.loader();
  } else {
    assert(0);
  }
}

void DeleteShader(GLuint n) {
  if (n) ctx->shaders.erase(n);
}

GLuint CreateProgram() {
  Program p;
  return ctx->programs.insert(p);
}

void DeleteProgram(GLuint n) {
  if (!n) return;
  if (ctx->current_program == n) {
    if (auto* p = ctx->programs.find(n)) {
      p->deleted = true;
    }
  } else {
    ctx->programs.erase(n);
  }
}

void LinkProgram(GLuint program) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  if (!p.impl) {
    return;
  }
  assert(p.impl->interpolants_size() <= sizeof(Interpolants));
  if (!p.vert_impl) p.vert_impl = p.impl->get_vertex_shader();
  if (!p.frag_impl) p.frag_impl = p.impl->get_fragment_shader();
}

GLint GetLinkStatus(GLuint program) {
  if (auto* p = ctx->programs.find(program)) {
    return p->impl ? 1 : 0;
  }
  return 0;
}

void BindAttribLocation(GLuint program, GLuint index, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  if (!p.impl) {
    return;
  }
  p.impl->bind_attrib(name, index);
}

GLint GetAttribLocation(GLuint program, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  if (!p.impl) {
    return -1;
  }
  return p.impl->get_attrib(name);
}

GLint GetUniformLocation(GLuint program, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  if (!p.impl) {
    return -1;
  }
  GLint loc = p.impl->get_uniform(name);
  // debugf("location: %d\n", loc);
  return loc;
}

static uint64_t get_time_value() {
#ifdef __MACH__
  return mach_absolute_time();
#elif defined(_WIN32)
  LARGE_INTEGER time;
  static bool have_frequency = false;
  static LARGE_INTEGER frequency;
  if (!have_frequency) {
    QueryPerformanceFrequency(&frequency);
    have_frequency = true;
  }
  QueryPerformanceCounter(&time);
  return time.QuadPart * 1000000000ULL / frequency.QuadPart;
#else
  return ({
    struct timespec tp;
    clock_gettime(CLOCK_MONOTONIC, &tp);
    tp.tv_sec * 1000000000ULL + tp.tv_nsec;
  });
#endif
}

void BeginQuery(GLenum target, GLuint id) {
  ctx->get_binding(target) = id;
  Query& q = ctx->queries[id];
  switch (target) {
    case GL_SAMPLES_PASSED:
      q.value = 0;
      break;
    case GL_TIME_ELAPSED:
      q.value = get_time_value();
      break;
    default:
      debugf("unknown query target %x for query %d\n", target, id);
      assert(false);
  }
}

void EndQuery(GLenum target) {
  Query& q = ctx->queries[ctx->get_binding(target)];
  switch (target) {
    case GL_SAMPLES_PASSED:
      break;
    case GL_TIME_ELAPSED:
      q.value = get_time_value() - q.value;
      break;
    default:
      debugf("unknown query target %x\n", target);
      assert(false);
  }
  ctx->get_binding(target) = 0;
}

void GetQueryObjectui64v(GLuint id, GLenum pname, GLuint64* params) {
  Query& q = ctx->queries[id];
  switch (pname) {
    case GL_QUERY_RESULT:
      assert(params);
      params[0] = q.value;
      break;
    default:
      assert(false);
  }
}

void BindVertexArray(GLuint vertex_array) {
  if (vertex_array != ctx->current_vertex_array) {
    ctx->validate_vertex_array = true;
  }
  ctx->current_vertex_array = vertex_array;
}

void BindTexture(GLenum target, GLuint texture) {
  ctx->get_binding(target) = texture;
}

void BindBuffer(GLenum target, GLuint buffer) {
  ctx->get_binding(target) = buffer;
}

void BindFramebuffer(GLenum target, GLuint fb) {
  if (target == GL_FRAMEBUFFER) {
    ctx->read_framebuffer_binding = fb;
    ctx->draw_framebuffer_binding = fb;
  } else {
    assert(target == GL_READ_FRAMEBUFFER || target == GL_DRAW_FRAMEBUFFER);
    ctx->get_binding(target) = fb;
  }
}

void BindRenderbuffer(GLenum target, GLuint rb) {
  ctx->get_binding(target) = rb;
}

void PixelStorei(GLenum name, GLint param) {
  if (name == GL_UNPACK_ALIGNMENT) {
    assert(param == 1);
  } else if (name == GL_UNPACK_ROW_LENGTH) {
    ctx->unpack_row_length = param;
  }
}

static GLenum remap_internal_format(GLenum format) {
  switch (format) {
    case GL_DEPTH_COMPONENT:
      return GL_DEPTH_COMPONENT24;
    case GL_RGBA:
      return GL_RGBA8;
    case GL_RED:
      return GL_R8;
    case GL_RG:
      return GL_RG8;
    case GL_RGB_422_APPLE:
      return GL_RGB_RAW_422_APPLE;
    default:
      return format;
  }
}

}  // extern "C"

static bool format_requires_conversion(GLenum external_format,
                                       GLenum internal_format) {
  switch (external_format) {
    case GL_RGBA:
      return internal_format == GL_RGBA8;
    default:
      return false;
  }
}

static inline void copy_bgra8_to_rgba8(uint32_t* dest, const uint32_t* src,
                                       int width) {
  for (; width >= 4; width -= 4, dest += 4, src += 4) {
    U32 p = unaligned_load<U32>(src);
    U32 rb = p & 0x00FF00FF;
    unaligned_store(dest, (p & 0xFF00FF00) | (rb << 16) | (rb >> 16));
  }
  for (; width > 0; width--, dest++, src++) {
    uint32_t p = *src;
    uint32_t rb = p & 0x00FF00FF;
    *dest = (p & 0xFF00FF00) | (rb << 16) | (rb >> 16);
  }
}

static void convert_copy(GLenum external_format, GLenum internal_format,
                         uint8_t* dst_buf, size_t dst_stride,
                         const uint8_t* src_buf, size_t src_stride,
                         size_t width, size_t height) {
  switch (external_format) {
    case GL_RGBA:
      if (internal_format == GL_RGBA8) {
        for (; height; height--) {
          copy_bgra8_to_rgba8((uint32_t*)dst_buf, (const uint32_t*)src_buf,
                              width);
          dst_buf += dst_stride;
          src_buf += src_stride;
        }
        return;
      }
      break;
    default:
      break;
  }
  size_t row_bytes = width * bytes_for_internal_format(internal_format);
  for (; height; height--) {
    memcpy(dst_buf, src_buf, row_bytes);
    dst_buf += dst_stride;
    src_buf += src_stride;
  }
}

static void set_tex_storage(Texture& t, GLenum external_format, GLsizei width,
                            GLsizei height, void* buf = nullptr,
                            GLsizei stride = 0, GLsizei min_width = 0,
                            GLsizei min_height = 0) {
  GLenum internal_format = remap_internal_format(external_format);
  bool changed = false;
  if (t.width != width || t.height != height ||
      t.internal_format != internal_format) {
    changed = true;
    t.internal_format = internal_format;
    t.width = width;
    t.height = height;
  }
  // If we are changed from an internally managed buffer to an externally
  // supplied one or vice versa, ensure that we clean up old buffer state.
  // However, if we have to convert the data from a non-native format, then
  // always treat it as internally managed since we will need to copy to an
  // internally managed native format buffer.
  bool should_free = buf == nullptr || format_requires_conversion(
                                           external_format, internal_format);
  if (t.should_free() != should_free) {
    changed = true;
    t.cleanup();
    t.set_should_free(should_free);
  }
  // If now an external buffer, explicitly set it...
  if (!should_free) {
    t.set_buffer(buf, stride);
  }
  t.disable_delayed_clear();
  t.allocate(changed, min_width, min_height);
  // If we have a buffer that needs format conversion, then do that now.
  if (buf && should_free) {
    convert_copy(external_format, internal_format, (uint8_t*)t.buf, t.stride(),
                 (const uint8_t*)buf, stride, width, height);
  }
}

extern "C" {

void TexStorage2D(GLenum target, GLint levels, GLenum internal_format,
                  GLsizei width, GLsizei height) {
  assert(levels == 1);
  Texture& t = ctx->textures[ctx->get_binding(target)];
  set_tex_storage(t, internal_format, width, height);
}

GLenum internal_format_for_data(GLenum format, GLenum ty) {
  if (format == GL_RED && ty == GL_UNSIGNED_BYTE) {
    return GL_R8;
  } else if ((format == GL_RGBA || format == GL_BGRA) &&
             (ty == GL_UNSIGNED_BYTE || ty == GL_UNSIGNED_INT_8_8_8_8_REV)) {
    return GL_RGBA8;
  } else if (format == GL_RGBA && ty == GL_FLOAT) {
    return GL_RGBA32F;
  } else if (format == GL_RGBA_INTEGER && ty == GL_INT) {
    return GL_RGBA32I;
  } else if (format == GL_RG && ty == GL_UNSIGNED_BYTE) {
    return GL_RG8;
  } else if (format == GL_RGB_422_APPLE &&
             ty == GL_UNSIGNED_SHORT_8_8_REV_APPLE) {
    return GL_RGB_RAW_422_APPLE;
  } else if (format == GL_RED && ty == GL_UNSIGNED_SHORT) {
    return GL_R16;
  } else {
    debugf("unknown internal format for format %x, type %x\n", format, ty);
    assert(false);
    return 0;
  }
}

static Buffer* get_pixel_pack_buffer() {
  return ctx->pixel_pack_buffer_binding
             ? &ctx->buffers[ctx->pixel_pack_buffer_binding]
             : nullptr;
}

static void* get_pixel_pack_buffer_data(void* data) {
  if (Buffer* b = get_pixel_pack_buffer()) {
    return b->buf ? b->buf + (size_t)data : nullptr;
  }
  return data;
}

static Buffer* get_pixel_unpack_buffer() {
  return ctx->pixel_unpack_buffer_binding
             ? &ctx->buffers[ctx->pixel_unpack_buffer_binding]
             : nullptr;
}

static void* get_pixel_unpack_buffer_data(void* data) {
  if (Buffer* b = get_pixel_unpack_buffer()) {
    return b->buf ? b->buf + (size_t)data : nullptr;
  }
  return data;
}

void TexSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                   GLsizei width, GLsizei height, GLenum format, GLenum ty,
                   void* data) {
  if (level != 0) {
    assert(false);
    return;
  }
  data = get_pixel_unpack_buffer_data(data);
  if (!data) return;
  Texture& t = ctx->textures[ctx->get_binding(target)];
  IntRect skip = {xoffset, yoffset, xoffset + width, yoffset + height};
  prepare_texture(t, &skip);
  assert(xoffset + width <= t.width);
  assert(yoffset + height <= t.height);
  assert(ctx->unpack_row_length == 0 || ctx->unpack_row_length >= width);
  GLsizei row_length =
      ctx->unpack_row_length != 0 ? ctx->unpack_row_length : width;
  assert(t.internal_format == internal_format_for_data(format, ty));
  int src_bpp = format_requires_conversion(format, t.internal_format)
                    ? bytes_for_internal_format(format)
                    : t.bpp();
  if (!src_bpp || !t.buf) return;
  convert_copy(format, t.internal_format,
               (uint8_t*)t.sample_ptr(xoffset, yoffset), t.stride(),
               (const uint8_t*)data, row_length * src_bpp, width, height);
}

void TexImage2D(GLenum target, GLint level, GLint internal_format,
                GLsizei width, GLsizei height, GLint border, GLenum format,
                GLenum ty, void* data) {
  if (level != 0) {
    assert(false);
    return;
  }
  assert(border == 0);
  TexStorage2D(target, 1, internal_format, width, height);
  TexSubImage2D(target, 0, 0, 0, width, height, format, ty, data);
}

void GenerateMipmap(UNUSED GLenum target) {
  // TODO: support mipmaps
}

void SetTextureParameter(GLuint texid, GLenum pname, GLint param) {
  Texture& t = ctx->textures[texid];
  switch (pname) {
    case GL_TEXTURE_WRAP_S:
      assert(param == GL_CLAMP_TO_EDGE);
      break;
    case GL_TEXTURE_WRAP_T:
      assert(param == GL_CLAMP_TO_EDGE);
      break;
    case GL_TEXTURE_MIN_FILTER:
      t.min_filter = param;
      break;
    case GL_TEXTURE_MAG_FILTER:
      t.mag_filter = param;
      break;
    default:
      break;
  }
}

void TexParameteri(GLenum target, GLenum pname, GLint param) {
  SetTextureParameter(ctx->get_binding(target), pname, param);
}

void GenTextures(int n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    Texture t;
    result[i] = ctx->textures.insert(t);
  }
}

void DeleteTexture(GLuint n) {
  if (n && ctx->textures.erase(n)) {
    for (size_t i = 0; i < MAX_TEXTURE_UNITS; i++) {
      ctx->texture_units[i].unlink(n);
    }
  }
}

void GenRenderbuffers(int n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    Renderbuffer r;
    result[i] = ctx->renderbuffers.insert(r);
  }
}

void Renderbuffer::on_erase() {
  for (auto* fb : ctx->framebuffers) {
    if (fb) {
      unlink(fb->color_attachment, texture);
      unlink(fb->depth_attachment, texture);
    }
  }
  DeleteTexture(texture);
}

void DeleteRenderbuffer(GLuint n) {
  if (n && ctx->renderbuffers.erase(n)) {
    unlink(ctx->renderbuffer_binding, n);
  }
}

void GenFramebuffers(int n, GLuint* result) {
  for (int i = 0; i < n; i++) {
    Framebuffer f;
    result[i] = ctx->framebuffers.insert(f);
  }
}

void DeleteFramebuffer(GLuint n) {
  if (n && ctx->framebuffers.erase(n)) {
    unlink(ctx->read_framebuffer_binding, n);
    unlink(ctx->draw_framebuffer_binding, n);
  }
}

void RenderbufferStorage(GLenum target, GLenum internal_format, GLsizei width,
                         GLsizei height) {
  // Just refer a renderbuffer to a texture to simplify things for now...
  Renderbuffer& r = ctx->renderbuffers[ctx->get_binding(target)];
  if (!r.texture) {
    GenTextures(1, &r.texture);
  }
  switch (internal_format) {
    case GL_DEPTH_COMPONENT:
    case GL_DEPTH_COMPONENT16:
    case GL_DEPTH_COMPONENT24:
    case GL_DEPTH_COMPONENT32:
      // Force depth format to 24 bits...
      internal_format = GL_DEPTH_COMPONENT24;
      break;
  }
  set_tex_storage(ctx->textures[r.texture], internal_format, width, height);
}

void VertexAttribPointer(GLuint index, GLint size, GLenum type, bool normalized,
                         GLsizei stride, GLuint offset) {
  // debugf("cva: %d\n", ctx->current_vertex_array);
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (index >= NULL_ATTRIB) {
    assert(0);
    return;
  }
  VertexAttrib& va = v.attribs[index];
  va.size = size * bytes_per_type(type);
  va.type = type;
  va.normalized = normalized;
  va.stride = stride;
  va.offset = offset;
  // Buffer &vertex_buf = ctx->buffers[ctx->array_buffer_binding];
  va.vertex_buffer = ctx->array_buffer_binding;
  va.vertex_array = ctx->current_vertex_array;
  ctx->validate_vertex_array = true;
}

void VertexAttribIPointer(GLuint index, GLint size, GLenum type, GLsizei stride,
                          GLuint offset) {
  // debugf("cva: %d\n", ctx->current_vertex_array);
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (index >= NULL_ATTRIB) {
    assert(0);
    return;
  }
  VertexAttrib& va = v.attribs[index];
  va.size = size * bytes_per_type(type);
  va.type = type;
  va.normalized = false;
  va.stride = stride;
  va.offset = offset;
  // Buffer &vertex_buf = ctx->buffers[ctx->array_buffer_binding];
  va.vertex_buffer = ctx->array_buffer_binding;
  va.vertex_array = ctx->current_vertex_array;
  ctx->validate_vertex_array = true;
}

void EnableVertexAttribArray(GLuint index) {
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (index >= NULL_ATTRIB) {
    assert(0);
    return;
  }
  VertexAttrib& va = v.attribs[index];
  if (!va.enabled) {
    ctx->validate_vertex_array = true;
  }
  va.enabled = true;
  v.max_attrib = max(v.max_attrib, (int)index);
}

void DisableVertexAttribArray(GLuint index) {
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (index >= NULL_ATTRIB) {
    assert(0);
    return;
  }
  VertexAttrib& va = v.attribs[index];
  if (va.enabled) {
    ctx->validate_vertex_array = true;
  }
  va.enabled = false;
}

void VertexAttribDivisor(GLuint index, GLuint divisor) {
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  // Only support divisor being 0 (per-vertex) or 1 (per-instance).
  if (index >= NULL_ATTRIB || divisor > 1) {
    assert(0);
    return;
  }
  VertexAttrib& va = v.attribs[index];
  va.divisor = divisor;
}

void BufferData(GLenum target, GLsizeiptr size, void* data,
                UNUSED GLenum usage) {
  Buffer& b = ctx->buffers[ctx->get_binding(target)];
  if (b.allocate(size)) {
    ctx->validate_vertex_array = true;
  }
  if (data && b.buf && size <= b.size) {
    memcpy(b.buf, data, size);
  }
}

void BufferSubData(GLenum target, GLintptr offset, GLsizeiptr size,
                   void* data) {
  Buffer& b = ctx->buffers[ctx->get_binding(target)];
  assert(offset + size <= b.size);
  if (data && b.buf && offset + size <= b.size) {
    memcpy(&b.buf[offset], data, size);
  }
}

void* MapBuffer(GLenum target, UNUSED GLbitfield access) {
  Buffer& b = ctx->buffers[ctx->get_binding(target)];
  return b.buf;
}

void* MapBufferRange(GLenum target, GLintptr offset, GLsizeiptr length,
                     UNUSED GLbitfield access) {
  Buffer& b = ctx->buffers[ctx->get_binding(target)];
  if (b.buf && offset >= 0 && length > 0 && offset + length <= b.size) {
    return b.buf + offset;
  }
  return nullptr;
}

GLboolean UnmapBuffer(GLenum target) {
  Buffer& b = ctx->buffers[ctx->get_binding(target)];
  return b.buf != nullptr;
}

void Uniform1i(GLint location, GLint V0) {
  // debugf("tex: %d\n", (int)ctx->textures.size);
  if (vertex_shader) {
    vertex_shader->set_uniform_1i(location, V0);
  }
}
void Uniform4fv(GLint location, GLsizei count, const GLfloat* v) {
  assert(count == 1);
  if (vertex_shader) {
    vertex_shader->set_uniform_4fv(location, v);
  }
}
void UniformMatrix4fv(GLint location, GLsizei count, GLboolean transpose,
                      const GLfloat* value) {
  assert(count == 1);
  assert(!transpose);
  if (vertex_shader) {
    vertex_shader->set_uniform_matrix4fv(location, value);
  }
}

void FramebufferTexture2D(GLenum target, GLenum attachment, GLenum textarget,
                          GLuint texture, GLint level) {
  assert(target == GL_READ_FRAMEBUFFER || target == GL_DRAW_FRAMEBUFFER);
  assert(textarget == GL_TEXTURE_2D || textarget == GL_TEXTURE_RECTANGLE);
  assert(level == 0);
  Framebuffer& fb = ctx->framebuffers[ctx->get_binding(target)];
  if (attachment == GL_COLOR_ATTACHMENT0) {
    fb.color_attachment = texture;
  } else if (attachment == GL_DEPTH_ATTACHMENT) {
    fb.depth_attachment = texture;
  } else {
    assert(0);
  }
}

void FramebufferRenderbuffer(GLenum target, GLenum attachment,
                             GLenum renderbuffertarget, GLuint renderbuffer) {
  assert(target == GL_READ_FRAMEBUFFER || target == GL_DRAW_FRAMEBUFFER);
  assert(renderbuffertarget == GL_RENDERBUFFER);
  Framebuffer& fb = ctx->framebuffers[ctx->get_binding(target)];
  Renderbuffer& rb = ctx->renderbuffers[renderbuffer];
  if (attachment == GL_COLOR_ATTACHMENT0) {
    fb.color_attachment = rb.texture;
  } else if (attachment == GL_DEPTH_ATTACHMENT) {
    fb.depth_attachment = rb.texture;
  } else {
    assert(0);
  }
}

}  // extern "C"

static inline Framebuffer* get_framebuffer(GLenum target,
                                           bool fallback = false) {
  if (target == GL_FRAMEBUFFER) {
    target = GL_DRAW_FRAMEBUFFER;
  }
  Framebuffer* fb = ctx->framebuffers.find(ctx->get_binding(target));
  if (fallback && !fb) {
    // If the specified framebuffer isn't found and a fallback is requested,
    // use the default framebuffer.
    fb = &ctx->framebuffers[0];
  }
  return fb;
}

template <typename T>
static inline void fill_n(T* dst, size_t n, T val) {
  for (T* end = &dst[n]; dst < end; dst++) *dst = val;
}

#if USE_SSE2
template <>
inline void fill_n<uint32_t>(uint32_t* dst, size_t n, uint32_t val) {
  __asm__ __volatile__("rep stosl\n"
                       : "+D"(dst), "+c"(n)
                       : "a"(val)
                       : "memory", "cc");
}
#endif

static inline uint32_t clear_chunk(uint8_t value) {
  return uint32_t(value) * 0x01010101U;
}

static inline uint32_t clear_chunk(uint16_t value) {
  return uint32_t(value) | (uint32_t(value) << 16);
}

static inline uint32_t clear_chunk(uint32_t value) { return value; }

template <typename T>
static inline void clear_row(T* buf, size_t len, T value, uint32_t chunk) {
  const size_t N = sizeof(uint32_t) / sizeof(T);
  // fill any leading unaligned values
  if (N > 1) {
    size_t align = (-(intptr_t)buf & (sizeof(uint32_t) - 1)) / sizeof(T);
    if (align <= len) {
      fill_n(buf, align, value);
      len -= align;
      buf += align;
    }
  }
  // fill as many aligned chunks as possible
  fill_n((uint32_t*)buf, len / N, chunk);
  // fill any remaining values
  if (N > 1) {
    fill_n(buf + (len & ~(N - 1)), len & (N - 1), value);
  }
}

template <typename T>
static void clear_buffer(Texture& t, T value, IntRect bb, int skip_start = 0,
                         int skip_end = 0) {
  if (!t.buf) return;
  skip_start = max(skip_start, bb.x0);
  skip_end = max(skip_end, skip_start);
  assert(sizeof(T) == t.bpp());
  size_t stride = t.stride();
  // When clearing multiple full-width rows, collapse them into a single large
  // "row" to avoid redundant setup from clearing each row individually. Note
  // that we can only safely do this if the stride is tightly packed.
  if (bb.width() == t.width && bb.height() > 1 && skip_start >= skip_end &&
      (t.should_free() || stride == t.width * sizeof(T))) {
    bb.x1 += (stride / sizeof(T)) * (bb.height() - 1);
    bb.y1 = bb.y0 + 1;
  }
  T* buf = (T*)t.sample_ptr(bb.x0, bb.y0);
  uint32_t chunk = clear_chunk(value);
  for (int rows = bb.height(); rows > 0; rows--) {
    if (bb.x0 < skip_start) {
      clear_row(buf, skip_start - bb.x0, value, chunk);
    }
    if (skip_end < bb.x1) {
      clear_row(buf + (skip_end - bb.x0), bb.x1 - skip_end, value, chunk);
    }
    buf += stride / sizeof(T);
  }
}

template <typename T>
static inline void force_clear_row(Texture& t, int y, int skip_start = 0,
                                   int skip_end = 0) {
  assert(t.buf != nullptr);
  assert(sizeof(T) == t.bpp());
  assert(skip_start <= skip_end);
  T* buf = (T*)t.sample_ptr(0, y);
  uint32_t chunk = clear_chunk((T)t.clear_val);
  if (skip_start > 0) {
    clear_row<T>(buf, skip_start, t.clear_val, chunk);
  }
  if (skip_end < t.width) {
    clear_row<T>(buf + skip_end, t.width - skip_end, t.clear_val, chunk);
  }
}

template <typename T>
static void force_clear(Texture& t, const IntRect* skip = nullptr) {
  if (!t.delay_clear || !t.cleared_rows) {
    return;
  }
  int y0 = 0;
  int y1 = t.height;
  int skip_start = 0;
  int skip_end = 0;
  if (skip) {
    y0 = clamp(skip->y0, 0, t.height);
    y1 = clamp(skip->y1, y0, t.height);
    skip_start = clamp(skip->x0, 0, t.width);
    skip_end = clamp(skip->x1, skip_start, t.width);
    if (skip_start <= 0 && skip_end >= t.width && y0 <= 0 && y1 >= t.height) {
      t.disable_delayed_clear();
      return;
    }
  }
  int num_masks = (y1 + 31) / 32;
  uint32_t* rows = t.cleared_rows;
  for (int i = y0 / 32; i < num_masks; i++) {
    uint32_t mask = rows[i];
    if (mask != ~0U) {
      rows[i] = ~0U;
      int start = i * 32;
      while (mask) {
        int count = __builtin_ctz(mask);
        if (count > 0) {
          clear_buffer<T>(t, t.clear_val,
                          IntRect{0, start, t.width, start + count}, skip_start,
                          skip_end);
          t.delay_clear -= count;
          start += count;
          mask >>= count;
        }
        count = __builtin_ctz(mask + 1);
        start += count;
        mask >>= count;
      }
      int count = (i + 1) * 32 - start;
      if (count > 0) {
        clear_buffer<T>(t, t.clear_val,
                        IntRect{0, start, t.width, start + count}, skip_start,
                        skip_end);
        t.delay_clear -= count;
      }
    }
  }
  if (t.delay_clear <= 0) t.disable_delayed_clear();
}

static void prepare_texture(Texture& t, const IntRect* skip) {
  if (t.delay_clear) {
    switch (t.internal_format) {
      case GL_RGBA8:
        force_clear<uint32_t>(t, skip);
        break;
      case GL_R8:
        force_clear<uint8_t>(t, skip);
        break;
      case GL_RG8:
        force_clear<uint16_t>(t, skip);
        break;
      default:
        assert(false);
        break;
    }
  }
}

// Setup a clear on a texture. This may either force an immediate clear or
// potentially punt to a delayed clear, if applicable.
template <typename T>
static void request_clear(Texture& t, T value, const IntRect& scissor) {
  // If the clear would require a scissor, force clear anything outside
  // the scissor, and then immediately clear anything inside the scissor.
  if (!scissor.contains(t.offset_bounds())) {
    IntRect skip = scissor - t.offset;
    force_clear<T>(t, &skip);
    clear_buffer<T>(t, value, skip.intersection(t.bounds()));
  } else {
    // Do delayed clear for 2D texture without scissor.
    t.enable_delayed_clear(value);
  }
}

template <typename T>
static inline void request_clear(Texture& t, T value) {
  // If scissoring is enabled, use the scissor rect. Otherwise, just scissor to
  // the entire texture bounds.
  request_clear(t, value, ctx->scissortest ? ctx->scissor : t.offset_bounds());
}

extern "C" {

void InitDefaultFramebuffer(int x, int y, int width, int height, int stride,
                            void* buf) {
  Framebuffer& fb = ctx->framebuffers[0];
  if (!fb.color_attachment) {
    GenTextures(1, &fb.color_attachment);
  }
  // If the dimensions or buffer properties changed, we need to reallocate
  // the underlying storage for the color buffer texture.
  Texture& colortex = ctx->textures[fb.color_attachment];
  set_tex_storage(colortex, GL_RGBA8, width, height, buf, stride);
  colortex.offset = IntPoint(x, y);
  if (!fb.depth_attachment) {
    GenTextures(1, &fb.depth_attachment);
  }
  // Ensure dimensions of the depth buffer match the color buffer.
  Texture& depthtex = ctx->textures[fb.depth_attachment];
  set_tex_storage(depthtex, GL_DEPTH_COMPONENT24, width, height);
  depthtex.offset = IntPoint(x, y);
}

void* GetColorBuffer(GLuint fbo, GLboolean flush, int32_t* width,
                     int32_t* height, int32_t* stride) {
  Framebuffer* fb = ctx->framebuffers.find(fbo);
  if (!fb || !fb->color_attachment) {
    return nullptr;
  }
  Texture& colortex = ctx->textures[fb->color_attachment];
  if (flush) {
    prepare_texture(colortex);
  }
  assert(colortex.offset == IntPoint(0, 0));
  if (width) {
    *width = colortex.width;
  }
  if (height) {
    *height = colortex.height;
  }
  if (stride) {
    *stride = colortex.stride();
  }
  return colortex.buf ? colortex.sample_ptr(0, 0) : nullptr;
}

void ResolveFramebuffer(GLuint fbo) {
  Framebuffer* fb = ctx->framebuffers.find(fbo);
  if (!fb || !fb->color_attachment) {
    return;
  }
  Texture& colortex = ctx->textures[fb->color_attachment];
  prepare_texture(colortex);
}

void SetTextureBuffer(GLuint texid, GLenum internal_format, GLsizei width,
                      GLsizei height, GLsizei stride, void* buf,
                      GLsizei min_width, GLsizei min_height) {
  Texture& t = ctx->textures[texid];
  set_tex_storage(t, internal_format, width, height, buf, stride, min_width,
                  min_height);
}

GLenum CheckFramebufferStatus(GLenum target) {
  Framebuffer* fb = get_framebuffer(target);
  if (!fb || !fb->color_attachment) {
    return GL_FRAMEBUFFER_UNSUPPORTED;
  }
  return GL_FRAMEBUFFER_COMPLETE;
}

void ClearTexSubImage(GLuint texture, GLint level, GLint xoffset, GLint yoffset,
                      GLint zoffset, GLsizei width, GLsizei height,
                      GLsizei depth, GLenum format, GLenum type,
                      const void* data) {
  if (level != 0) {
    assert(false);
    return;
  }
  Texture& t = ctx->textures[texture];
  assert(!t.locked);
  if (width <= 0 || height <= 0 || depth <= 0) {
    return;
  }
  assert(zoffset == 0 && depth == 1);
  IntRect scissor = {xoffset, yoffset, xoffset + width, yoffset + height};
  if (t.internal_format == GL_DEPTH_COMPONENT24) {
    uint32_t value = 0xFFFFFF;
    switch (format) {
      case GL_DEPTH_COMPONENT:
        switch (type) {
          case GL_DOUBLE:
            value = uint32_t(*(const GLdouble*)data * 0xFFFFFF);
            break;
          case GL_FLOAT:
            value = uint32_t(*(const GLfloat*)data * 0xFFFFFF);
            break;
          default:
            assert(false);
            break;
        }
        break;
      default:
        assert(false);
        break;
    }
    if (t.cleared() && !scissor.contains(t.offset_bounds())) {
      // If we need to scissor the clear and the depth buffer was already
      // initialized, then just fill runs for that scissor area.
      t.fill_depth_runs(value, scissor);
    } else {
      // Otherwise, the buffer is either uninitialized or the clear would
      // encompass the entire buffer. If uninitialized, we can safely fill
      // the entire buffer with any value and thus ignore any scissoring.
      t.init_depth_runs(value);
    }
    return;
  }

  uint32_t color = 0xFF000000;
  switch (type) {
    case GL_FLOAT: {
      const GLfloat* f = (const GLfloat*)data;
      Float v = {0.0f, 0.0f, 0.0f, 1.0f};
      switch (format) {
        case GL_RGBA:
          v.w = f[3];  // alpha
          FALLTHROUGH;
        case GL_RGB:
          v.z = f[2];  // blue
          FALLTHROUGH;
        case GL_RG:
          v.y = f[1];  // green
          FALLTHROUGH;
        case GL_RED:
          v.x = f[0];  // red
          break;
        default:
          assert(false);
          break;
      }
      color = bit_cast<uint32_t>(CONVERT(round_pixel(v), U8));
      break;
    }
    case GL_UNSIGNED_BYTE: {
      const GLubyte* b = (const GLubyte*)data;
      switch (format) {
        case GL_RGBA:
          color = (color & ~0xFF000000) | (uint32_t(b[3]) << 24);  // alpha
          FALLTHROUGH;
        case GL_RGB:
          color = (color & ~0x00FF0000) | (uint32_t(b[2]) << 16);  // blue
          FALLTHROUGH;
        case GL_RG:
          color = (color & ~0x0000FF00) | (uint32_t(b[1]) << 8);  // green
          FALLTHROUGH;
        case GL_RED:
          color = (color & ~0x000000FF) | uint32_t(b[0]);  // red
          break;
        default:
          assert(false);
          break;
      }
      break;
    }
    default:
      assert(false);
      break;
  }

  switch (t.internal_format) {
    case GL_RGBA8:
      // Clear color needs to swizzle to BGRA.
      request_clear<uint32_t>(t,
                              (color & 0xFF00FF00) |
                                  ((color << 16) & 0xFF0000) |
                                  ((color >> 16) & 0xFF),
                              scissor);
      break;
    case GL_R8:
      request_clear<uint8_t>(t, uint8_t(color & 0xFF), scissor);
      break;
    case GL_RG8:
      request_clear<uint16_t>(t, uint16_t(color & 0xFFFF), scissor);
      break;
    default:
      assert(false);
      break;
  }
}

void ClearTexImage(GLuint texture, GLint level, GLenum format, GLenum type,
                   const void* data) {
  Texture& t = ctx->textures[texture];
  IntRect scissor = t.offset_bounds();
  ClearTexSubImage(texture, level, scissor.x0, scissor.y0, 0, scissor.width(),
                   scissor.height(), 1, format, type, data);
}

void Clear(GLbitfield mask) {
  Framebuffer& fb = *get_framebuffer(GL_DRAW_FRAMEBUFFER, true);
  if ((mask & GL_COLOR_BUFFER_BIT) && fb.color_attachment) {
    Texture& t = ctx->textures[fb.color_attachment];
    IntRect scissor = ctx->scissortest
                          ? ctx->scissor.intersection(t.offset_bounds())
                          : t.offset_bounds();
    ClearTexSubImage(fb.color_attachment, 0, scissor.x0, scissor.y0, 0,
                     scissor.width(), scissor.height(), 1, GL_RGBA, GL_FLOAT,
                     ctx->clearcolor);
  }
  if ((mask & GL_DEPTH_BUFFER_BIT) && fb.depth_attachment) {
    Texture& t = ctx->textures[fb.depth_attachment];
    IntRect scissor = ctx->scissortest
                          ? ctx->scissor.intersection(t.offset_bounds())
                          : t.offset_bounds();
    ClearTexSubImage(fb.depth_attachment, 0, scissor.x0, scissor.y0, 0,
                     scissor.width(), scissor.height(), 1, GL_DEPTH_COMPONENT,
                     GL_DOUBLE, &ctx->cleardepth);
  }
}

void ClearColorRect(GLuint fbo, GLint xoffset, GLint yoffset, GLsizei width,
                    GLsizei height, GLfloat r, GLfloat g, GLfloat b,
                    GLfloat a) {
  GLfloat color[] = {r, g, b, a};
  Framebuffer& fb = ctx->framebuffers[fbo];
  Texture& t = ctx->textures[fb.color_attachment];
  IntRect scissor =
      IntRect{xoffset, yoffset, xoffset + width, yoffset + height}.intersection(
          t.offset_bounds());
  ClearTexSubImage(fb.color_attachment, 0, scissor.x0, scissor.y0, 0,
                   scissor.width(), scissor.height(), 1, GL_RGBA, GL_FLOAT,
                   color);
}

void InvalidateFramebuffer(GLenum target, GLsizei num_attachments,
                           const GLenum* attachments) {
  Framebuffer* fb = get_framebuffer(target);
  if (!fb || num_attachments <= 0 || !attachments) {
    return;
  }
  for (GLsizei i = 0; i < num_attachments; i++) {
    switch (attachments[i]) {
      case GL_DEPTH_ATTACHMENT: {
        Texture& t = ctx->textures[fb->depth_attachment];
        t.set_cleared(false);
        break;
      }
      case GL_COLOR_ATTACHMENT0: {
        Texture& t = ctx->textures[fb->color_attachment];
        t.disable_delayed_clear();
        break;
      }
    }
  }
}

void ReadPixels(GLint x, GLint y, GLsizei width, GLsizei height, GLenum format,
                GLenum type, void* data) {
  data = get_pixel_pack_buffer_data(data);
  if (!data) return;
  Framebuffer* fb = get_framebuffer(GL_READ_FRAMEBUFFER);
  if (!fb) return;
  assert(format == GL_RED || format == GL_RGBA || format == GL_RGBA_INTEGER ||
         format == GL_BGRA || format == GL_RG);
  Texture& t = ctx->textures[fb->color_attachment];
  if (!t.buf) return;
  prepare_texture(t);
  // debugf("read pixels %d, %d, %d, %d from fb %d with format %x\n", x, y,
  // width, height, ctx->read_framebuffer_binding, t.internal_format);
  x -= t.offset.x;
  y -= t.offset.y;
  assert(x >= 0 && y >= 0);
  assert(x + width <= t.width);
  assert(y + height <= t.height);
  if (internal_format_for_data(format, type) != t.internal_format) {
    debugf("mismatched format for read pixels: %x vs %x\n", t.internal_format,
           internal_format_for_data(format, type));
    assert(false);
    return;
  }
  // Only support readback conversions that are reversible
  assert(!format_requires_conversion(format, t.internal_format) ||
         bytes_for_internal_format(format) == t.bpp());
  uint8_t* dest = (uint8_t*)data;
  size_t destStride = width * t.bpp();
  if (y < 0) {
    dest += -y * destStride;
    height += y;
    y = 0;
  }
  if (y + height > t.height) {
    height = t.height - y;
  }
  if (x < 0) {
    dest += -x * t.bpp();
    width += x;
    x = 0;
  }
  if (x + width > t.width) {
    width = t.width - x;
  }
  if (width <= 0 || height <= 0) {
    return;
  }
  convert_copy(format, t.internal_format, dest, destStride,
               (const uint8_t*)t.sample_ptr(x, y), t.stride(), width, height);
}

void CopyImageSubData(GLuint srcName, GLenum srcTarget, UNUSED GLint srcLevel,
                      GLint srcX, GLint srcY, GLint srcZ, GLuint dstName,
                      GLenum dstTarget, UNUSED GLint dstLevel, GLint dstX,
                      GLint dstY, GLint dstZ, GLsizei srcWidth,
                      GLsizei srcHeight, GLsizei srcDepth) {
  assert(srcLevel == 0 && dstLevel == 0);
  assert(srcZ == 0 && srcDepth == 1 && dstZ == 0);
  if (srcTarget == GL_RENDERBUFFER) {
    Renderbuffer& rb = ctx->renderbuffers[srcName];
    srcName = rb.texture;
  }
  if (dstTarget == GL_RENDERBUFFER) {
    Renderbuffer& rb = ctx->renderbuffers[dstName];
    dstName = rb.texture;
  }
  Texture& srctex = ctx->textures[srcName];
  if (!srctex.buf) return;
  prepare_texture(srctex);
  Texture& dsttex = ctx->textures[dstName];
  if (!dsttex.buf) return;
  assert(!dsttex.locked);
  IntRect skip = {dstX, dstY, dstX + srcWidth, dstY + srcHeight};
  prepare_texture(dsttex, &skip);
  assert(srctex.internal_format == dsttex.internal_format);
  assert(srcWidth >= 0);
  assert(srcHeight >= 0);
  assert(srcX + srcWidth <= srctex.width);
  assert(srcY + srcHeight <= srctex.height);
  assert(dstX + srcWidth <= dsttex.width);
  assert(dstY + srcHeight <= dsttex.height);
  int bpp = srctex.bpp();
  int src_stride = srctex.stride();
  int dest_stride = dsttex.stride();
  char* dest = dsttex.sample_ptr(dstX, dstY);
  char* src = srctex.sample_ptr(srcX, srcY);
  for (int y = 0; y < srcHeight; y++) {
    memcpy(dest, src, srcWidth * bpp);
    dest += dest_stride;
    src += src_stride;
  }
}

void CopyTexSubImage2D(GLenum target, UNUSED GLint level, GLint xoffset,
                       GLint yoffset, GLint x, GLint y, GLsizei width,
                       GLsizei height) {
  assert(level == 0);
  Framebuffer* fb = get_framebuffer(GL_READ_FRAMEBUFFER);
  if (!fb) return;
  CopyImageSubData(fb->color_attachment, GL_TEXTURE_2D, 0, x, y, 0,
                   ctx->get_binding(target), GL_TEXTURE_2D, 0, xoffset, yoffset,
                   0, width, height, 1);
}

}  // extern "C"

#include "blend.h"
#include "composite.h"
#include "swgl_ext.h"

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wuninitialized"
#pragma GCC diagnostic ignored "-Wunused-function"
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wimplicit-fallthrough"
#ifdef __clang__
#  pragma GCC diagnostic ignored "-Wunused-private-field"
#else
#  pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#include "load_shader.h"
#pragma GCC diagnostic pop

#include "rasterize.h"

void VertexArray::validate() {
  int last_enabled = -1;
  for (int i = 0; i <= max_attrib; i++) {
    VertexAttrib& attr = attribs[i];
    if (attr.enabled) {
      // VertexArray &v = ctx->vertex_arrays[attr.vertex_array];
      Buffer& vertex_buf = ctx->buffers[attr.vertex_buffer];
      attr.buf = vertex_buf.buf;
      attr.buf_size = vertex_buf.size;
      // debugf("%d %x %d %d %d %d\n", i, attr.type, attr.size, attr.stride,
      // attr.offset, attr.divisor);
      last_enabled = i;
    }
  }
  max_attrib = last_enabled;
}

extern "C" {

void DrawElementsInstanced(GLenum mode, GLsizei count, GLenum type,
                           GLintptr offset, GLsizei instancecount) {
  if (offset < 0 || count <= 0 || instancecount <= 0 || !vertex_shader ||
      !fragment_shader) {
    return;
  }

  Framebuffer& fb = *get_framebuffer(GL_DRAW_FRAMEBUFFER, true);
  if (!fb.color_attachment) {
    return;
  }
  Texture& colortex = ctx->textures[fb.color_attachment];
  if (!colortex.buf) {
    return;
  }
  assert(!colortex.locked);
  assert(colortex.internal_format == GL_RGBA8 ||
         colortex.internal_format == GL_R8);
  Texture& depthtex = ctx->textures[ctx->depthtest ? fb.depth_attachment : 0];
  if (depthtex.buf) {
    assert(depthtex.internal_format == GL_DEPTH_COMPONENT24);
    assert(colortex.width == depthtex.width &&
           colortex.height == depthtex.height);
    assert(colortex.offset == depthtex.offset);
  }

  // debugf("current_vertex_array %d\n", ctx->current_vertex_array);
  // debugf("indices size: %d\n", indices_buf.size);
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (ctx->validate_vertex_array) {
    ctx->validate_vertex_array = false;
    v.validate();
  }

#ifdef PRINT_TIMINGS
  uint64_t start = get_time_value();
#endif

  ctx->shaded_rows = 0;
  ctx->shaded_pixels = 0;

  vertex_shader->init_batch();

  switch (type) {
    case GL_UNSIGNED_SHORT:
      assert(mode == GL_TRIANGLES);
      draw_elements<uint16_t>(count, instancecount, offset, v, colortex,
                              depthtex);
      break;
    case GL_UNSIGNED_INT:
      assert(mode == GL_TRIANGLES);
      draw_elements<uint32_t>(count, instancecount, offset, v, colortex,
                              depthtex);
      break;
    case GL_NONE:
      // Non-standard GL extension - if element type is GL_NONE, then we don't
      // use any element buffer and behave as if DrawArrays was called instead.
      for (GLsizei instance = 0; instance < instancecount; instance++) {
        switch (mode) {
          case GL_LINES:
            for (GLsizei i = 0; i + 2 <= count; i += 2) {
              vertex_shader->load_attribs(v.attribs, offset + i, instance, 2);
              draw_quad(2, colortex, depthtex);
            }
            break;
          case GL_TRIANGLES:
            for (GLsizei i = 0; i + 3 <= count; i += 3) {
              vertex_shader->load_attribs(v.attribs, offset + i, instance, 3);
              draw_quad(3, colortex, depthtex);
            }
            break;
          default:
            assert(false);
            break;
        }
      }
      break;
    default:
      assert(false);
      break;
  }

  if (ctx->samples_passed_query) {
    Query& q = ctx->queries[ctx->samples_passed_query];
    q.value += ctx->shaded_pixels;
  }

#ifdef PRINT_TIMINGS
  uint64_t end = get_time_value();
  printf(
      "%7.3fms draw(%s, %d): %d pixels in %d rows (avg %f pixels/row, "
      "%fns/pixel)\n",
      double(end - start) / (1000. * 1000.),
      ctx->programs[ctx->current_program].impl->get_name(), instancecount,
      ctx->shaded_pixels, ctx->shaded_rows,
      double(ctx->shaded_pixels) / ctx->shaded_rows,
      double(end - start) / max(ctx->shaded_pixels, 1));
#endif
}

void Finish() {
#ifdef PRINT_TIMINGS
  printf("Finish\n");
#endif
}

void MakeCurrent(Context* c) {
  if (ctx == c) {
    return;
  }
  ctx = c;
  setup_program(ctx ? ctx->current_program : 0);
}

Context* CreateContext() { return new Context; }

void ReferenceContext(Context* c) {
  if (!c) {
    return;
  }
  ++c->references;
}

void DestroyContext(Context* c) {
  if (!c) {
    return;
  }
  assert(c->references > 0);
  --c->references;
  if (c->references > 0) {
    return;
  }
  if (ctx == c) {
    MakeCurrent(nullptr);
  }
  delete c;
}

size_t ReportMemory(size_t (*size_of_op)(void*)) {
  size_t size = 0;
  if (ctx) {
    for (auto& t : ctx->textures) {
      if (t && t->should_free()) {
        size += size_of_op(t->buf);
      }
    }
  }
  return size;
}
}  // extern "C"
