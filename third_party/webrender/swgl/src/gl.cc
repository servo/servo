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

#ifdef _WIN32
#  define ALWAYS_INLINE __forceinline
#else
#  define ALWAYS_INLINE __attribute__((always_inline)) inline
#endif

#define UNREACHABLE __builtin_unreachable()

#define UNUSED __attribute__((unused))

#ifdef MOZILLA_CLIENT
#  define IMPLICIT __attribute__((annotate("moz_implicit")))
#else
#  define IMPLICIT
#endif

#include "gl_defs.h"
#include "glsl.h"
#include "program.h"

using namespace glsl;

struct IntRect {
  int x0;
  int y0;
  int x1;
  int y1;

  int width() const { return x1 - x0; }
  int height() const { return y1 - y0; }
  bool is_empty() const { return width() <= 0 || height() <= 0; }

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

  IntRect& offset(int dx, int dy) {
    x0 += dx;
    y0 += dy;
    x1 += dx;
    y1 += dy;
    return *this;
  }
};

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
    case GL_DEPTH_COMPONENT:
    case GL_DEPTH_COMPONENT16:
      return 2;
    case GL_DEPTH_COMPONENT24:
    case GL_DEPTH_COMPONENT32:
      return 4;
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

  bool allocate(size_t new_size) {
    if (new_size != size) {
      char* new_buf = (char*)realloc(buf, new_size);
      assert(new_buf);
      if (new_buf) {
        buf = new_buf;
        size = new_size;
        return true;
      }
      cleanup();
    }
    return false;
  }

  void cleanup() {
    if (buf) {
      free(buf);
      buf = nullptr;
      size = 0;
    }
  }

  ~Buffer() { cleanup(); }
};

struct Framebuffer {
  GLuint color_attachment = 0;
  GLint layer = 0;
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
  int depth = 0;
  char* buf = nullptr;
  size_t buf_size = 0;
  GLenum min_filter = GL_NEAREST;
  GLenum mag_filter = GL_LINEAR;

  enum FLAGS {
    SHOULD_FREE = 1 << 1,
  };
  int flags = SHOULD_FREE;
  bool should_free() const { return bool(flags & SHOULD_FREE); }

  void set_flag(int flag, bool val) {
    if (val) {
      flags |= flag;
    } else {
      flags &= ~flag;
    }
  }
  void set_should_free(bool val) { set_flag(SHOULD_FREE, val); }

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

  int bpp() const { return bytes_for_internal_format(internal_format); }

  size_t stride(int b = 0, int min_width = 0) const {
    return aligned_stride((b ? b : bpp()) * max(width, min_width));
  }

  size_t layer_stride(int b = 0, int min_width = 0, int min_height = 0) const {
    return stride(b ? b : bpp(), min_width) * max(height, min_height);
  }

  bool allocate(bool force = false, int min_width = 0, int min_height = 0) {
    if ((!buf || force) && should_free()) {
      size_t size = layer_stride(bpp(), min_width, min_height) * max(depth, 1);
      if (!buf || size > buf_size) {
        // Allocate with a SIMD register-sized tail of padding at the end so we
        // can safely read or write past the end of the texture with SIMD ops.
        char* new_buf = (char*)realloc(buf, size + sizeof(Float));
        assert(new_buf);
        if (new_buf) {
          buf = new_buf;
          buf_size = size;
          return true;
        }
        cleanup();
      }
    }
    return false;
  }

  void cleanup() {
    if (buf && should_free()) {
      free(buf);
      buf = nullptr;
      buf_size = 0;
    }
    disable_delayed_clear();
  }

  ~Texture() { cleanup(); }

  IntRect bounds() const { return IntRect{0, 0, width, height}; }

  // Find the valid sampling bounds relative to the requested region
  IntRect sample_bounds(const IntRect& req, bool invertY = false) const {
    IntRect bb = bounds().intersect(req).offset(-req.x0, -req.y0);
    if (invertY) bb.invert_y(req.height());
    return bb;
  }

  // Get a pointer for sampling at the given offset
  char* sample_ptr(int x, int y, int z, int bpp, size_t stride) const {
    return buf + (height * z + y) * stride + x * bpp;
  }

  char* sample_ptr(int x, int y, int z, int bpp) const {
    return sample_ptr(x, y, z, bpp, stride(bpp));
  }

  char* sample_ptr(int x, int y, int z) const {
    return sample_ptr(x, y, z, bpp());
  }

  // Get a pointer for sampling the requested region and limit to the provided
  // sampling bounds
  char* sample_ptr(const IntRect& req, const IntRect& bounds, int z,
                   bool invertY = false) const {
    // Offset the sample pointer by the clamped bounds
    int x = req.x0 + bounds.x0;
    // Invert the Y offset if necessary
    int y = invertY ? req.y1 - 1 - bounds.y0 : req.y0 + bounds.y0;
    return sample_ptr(x, y, z);
  }
};

#define MAX_ATTRIBS 16
#define NULL_ATTRIB 15
struct VertexArray {
  VertexAttrib attribs[MAX_ATTRIBS];
  int max_attrib = -1;

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

  ~Program() {
    delete impl;
  }
};

// for GL defines to fully expand
#define CONCAT_KEY(prefix, x, y, z, w, ...) prefix##x##y##z##w
#define BLEND_KEY(...) CONCAT_KEY(BLEND_, __VA_ARGS__, 0, 0)
#define FOR_EACH_BLEND_KEY(macro)                                              \
  macro(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE)                  \
      macro(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, 0, 0)                              \
          macro(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, 0, 0)                         \
              macro(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, GL_ZERO, GL_ONE)          \
                  macro(GL_ZERO, GL_ONE_MINUS_SRC_ALPHA, 0, 0) macro(          \
                      GL_ZERO, GL_SRC_COLOR, 0, 0) macro(GL_ONE, GL_ONE, 0, 0) \
                      macro(GL_ONE, GL_ONE, GL_ONE, GL_ONE_MINUS_SRC_ALPHA)    \
                          macro(GL_ONE, GL_ZERO, 0, 0) macro(                  \
                              GL_ONE_MINUS_DST_ALPHA, GL_ONE, GL_ZERO, GL_ONE) \
                              macro(GL_CONSTANT_COLOR, GL_ONE_MINUS_SRC_COLOR, \
                                    0, 0)                                      \
                                  macro(GL_ONE, GL_ONE_MINUS_SRC1_COLOR, 0, 0)

#define DEFINE_BLEND_KEY(...) BLEND_KEY(__VA_ARGS__),
enum BlendKey : uint8_t {
  BLEND_KEY_NONE = 0,
  FOR_EACH_BLEND_KEY(DEFINE_BLEND_KEY)
};

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

  template <typename T> void on_erase(T*, ...) {}
  template <typename T> void on_erase(T* o, decltype(&T::on_erase)) {
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

  uint32_t clearcolor = 0;
  GLdouble cleardepth = 1;

  int unpack_row_length = 0;

  int shaded_rows = 0;
  int shaded_pixels = 0;

  struct TextureUnit {
    GLuint texture_2d_binding = 0;
    GLuint texture_3d_binding = 0;
    GLuint texture_2d_array_binding = 0;
    GLuint texture_rectangle_binding = 0;

    void unlink(GLuint n) {
      ::unlink(texture_2d_binding, n);
      ::unlink(texture_3d_binding, n);
      ::unlink(texture_2d_array_binding, n);
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
  GLuint element_array_buffer_binding = 0;
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
        return element_array_buffer_binding;
      case GL_TEXTURE_2D:
        return texture_units[active_texture_unit].texture_2d_binding;
      case GL_TEXTURE_2D_ARRAY:
        return texture_units[active_texture_unit].texture_2d_array_binding;
      case GL_TEXTURE_3D:
        return texture_units[active_texture_unit].texture_3d_binding;
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

  Texture& get_texture(sampler2DArray, int unit) {
    return textures[texture_units[unit].texture_2d_array_binding];
  }

  Texture& get_texture(sampler2DRect, int unit) {
    return textures[texture_units[unit].texture_rectangle_binding];
  }

  IntRect apply_scissor(IntRect bb) const {
    return scissortest ? bb.intersect(scissor) : bb;
  }
};
static Context* ctx = nullptr;
static VertexShaderImpl* vertex_shader = nullptr;
static FragmentShaderImpl* fragment_shader = nullptr;
static BlendKey blend_key = BLEND_KEY_NONE;

static void prepare_texture(Texture& t, const IntRect* skip = nullptr);

template <typename S>
static inline void init_depth(S* s, Texture& t) {
  s->depth = max(t.depth, 1);
  s->height_stride = s->stride * t.height;
}

template <typename S>
static inline void init_filter(S* s, Texture& t) {
  s->filter = gl_filter_to_texture_filter(t.mag_filter);
}

template <typename S>
static inline void init_sampler(S* s, Texture& t) {
  prepare_texture(t);
  s->width = t.width;
  s->height = t.height;
  int bpp = t.bpp();
  s->stride = t.stride(bpp);
  if (bpp >= 4) s->stride /= 4;
  // Use uint32_t* for easier sampling, but need to cast to uint8_t* for formats
  // with bpp < 4.
  s->buf = (uint32_t*)t.buf;
  s->format = gl_format_to_texture_format(t.internal_format);
}

template <typename S>
S* lookup_sampler(S* s, int texture) {
  Texture& t = ctx->get_texture(s, texture);
  if (!t.buf) {
    *s = S();
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
    *s = S();
  } else {
    init_sampler(s, t);
  }
  return s;
}

template <typename S>
S* lookup_sampler_array(S* s, int texture) {
  Texture& t = ctx->get_texture(s, texture);
  if (!t.buf) {
    *s = S();
  } else {
    init_sampler(s, t);
    init_depth(s, t);
    init_filter(s, t);
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
    // Triangles must be indexed at offsets 0, 1, 2.
    // Quads must be successive triangles indexed at offsets 0, 1, 2, 2, 1, 3.
    // Triangle vertexes fill vertex shader SIMD lanes as 0, 1, 2, 2.
    // Quad vertexes fill vertex shader SIMD lanes as 0, 1, 3, 2, so that the
    // points form a convex path that can be traversed by the rasterizer.
    if (!count) return;
    assert(count == 3 || count == 4);
    char* src = (char*)va.buf + va.stride * start + va.offset;
    attrib = (T){
        load_attrib_scalar<scalar_type>(va, src),
        load_attrib_scalar<scalar_type>(va, src + va.stride),
        load_attrib_scalar<scalar_type>(va, src + va.stride * 2 +
                                            (count > 3 ? va.stride : 0)),
        load_attrib_scalar<scalar_type>(va, src + va.stride * 2)
    };
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
      blend_key = ctx->blend_key;
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
      blend_key = BLEND_KEY_NONE;
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
    "GL_ARB_blend_func_extended", "GL_ARB_copy_image",
    "GL_ARB_draw_instanced",      "GL_ARB_explicit_attrib_location",
    "GL_ARB_instanced_arrays",    "GL_ARB_invalidate_subdata",
    "GL_ARB_texture_storage",     "GL_EXT_timer_query",
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
      params[0] = 1 << 15;
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

void BlendFunc(GLenum srgb, GLenum drgb, GLenum sa, GLenum da) {
  ctx->blendfunc_srgb = srgb;
  ctx->blendfunc_drgb = drgb;
  sa = remap_blendfunc(srgb, sa);
  da = remap_blendfunc(drgb, da);
  ctx->blendfunc_sa = sa;
  ctx->blendfunc_da = da;

#define HASH_BLEND_KEY(x, y, z, w) ((x << 4) | (y) | (z << 24) | (w << 20))
  int hash = HASH_BLEND_KEY(srgb, drgb, 0, 0);
  if (srgb != sa || drgb != da) hash |= HASH_BLEND_KEY(0, 0, sa, da);
  switch (hash) {
#define MAP_BLEND_KEY(...)                   \
  case HASH_BLEND_KEY(__VA_ARGS__):          \
    ctx->blend_key = BLEND_KEY(__VA_ARGS__); \
    break;
    FOR_EACH_BLEND_KEY(MAP_BLEND_KEY)
    default:
      debugf("blendfunc: %x, %x, separate: %x, %x\n", srgb, drgb, sa, da);
      assert(false);
      break;
  }

  if (ctx->blend) {
    blend_key = ctx->blend_key;
  }
}

void BlendColor(GLfloat r, GLfloat g, GLfloat b, GLfloat a) {
  I32 c = round_pixel((Float){b, g, r, a});
  ctx->blendcolor = CONVERT(c, U16).xyzwxyzw;
}

void BlendEquation(GLenum mode) {
  assert(mode == GL_FUNC_ADD);
  ctx->blend_equation = mode;
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
  I32 c = round_pixel((Float){b, g, r, a});
  ctx->clearcolor = bit_cast<uint32_t>(CONVERT(c, U8));
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
    unlink(ctx->element_array_buffer_binding, n);
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
  assert(p.impl->interpolants_size() <= sizeof(Interpolants));
  if (!p.vert_impl) p.vert_impl = p.impl->get_vertex_shader();
  if (!p.frag_impl) p.frag_impl = p.impl->get_fragment_shader();
}

void BindAttribLocation(GLuint program, GLuint index, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  p.impl->bind_attrib(name, index);
}

GLint GetAttribLocation(GLuint program, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  return p.impl->get_attrib(name);
}

GLint GetUniformLocation(GLuint program, char* name) {
  Program& p = ctx->programs[program];
  assert(p.impl);
  GLint loc = p.impl->get_uniform(name);
  // debugf("location: %d\n", loc);
  return loc;
}

static uint64_t get_time_value() {
#ifdef __MACH__
  return mach_absolute_time();
#elif defined(_WIN32)
  return uint64_t(clock()) * (1000000000ULL / CLOCKS_PER_SEC);
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
      return GL_DEPTH_COMPONENT16;
    case GL_RGBA:
      return GL_RGBA8;
    case GL_RED:
      return GL_R8;
    default:
      return format;
  }
}

void TexStorage3D(GLenum target, GLint levels, GLenum internal_format,
                  GLsizei width, GLsizei height, GLsizei depth) {
  assert(levels == 1);
  Texture& t = ctx->textures[ctx->get_binding(target)];
  internal_format = remap_internal_format(internal_format);
  bool changed = false;
  if (t.width != width || t.height != height || t.depth != depth ||
      t.internal_format != internal_format) {
    changed = true;
    t.internal_format = internal_format;
    t.width = width;
    t.height = height;
    t.depth = depth;
  }
  t.disable_delayed_clear();
  t.allocate(changed);
}

static void set_tex_storage(Texture& t, GLenum internal_format,
                            GLsizei width, GLsizei height,
                            bool should_free = true, void* buf = nullptr,
                            GLsizei min_width = 0, GLsizei min_height = 0) {
  internal_format = remap_internal_format(internal_format);
  bool changed = false;
  if (t.width != width || t.height != height || t.depth != 0 ||
      t.internal_format != internal_format) {
    changed = true;
    t.internal_format = internal_format;
    t.width = width;
    t.height = height;
    t.depth = 0;
  }
  if (t.should_free() != should_free || buf != nullptr) {
    if (t.should_free()) {
      t.cleanup();
    }
    t.set_should_free(should_free);
    t.buf = (char*)buf;
    t.buf_size = 0;
  }
  t.disable_delayed_clear();
  t.allocate(changed, min_width, min_height);
}

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
             ty == GL_UNSIGNED_BYTE) {
    return GL_RGBA8;
  } else if (format == GL_RGBA && ty == GL_FLOAT) {
    return GL_RGBA32F;
  } else if (format == GL_RGBA_INTEGER && ty == GL_INT) {
    return GL_RGBA32I;
  } else {
    debugf("unknown internal format for format %x, type %x\n", format, ty);
    assert(false);
    return 0;
  }
}

static inline void copy_bgra8_to_rgba8(uint32_t* dest, uint32_t* src,
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
  if (level != 0) { assert(false); return; }
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
  int bpp = t.bpp();
  if (!bpp || !t.buf) return;
  size_t dest_stride = t.stride(bpp);
  char* dest = t.sample_ptr(xoffset, yoffset, 0, bpp, dest_stride);
  char* src = (char*)data;
  for (int y = 0; y < height; y++) {
    if (t.internal_format == GL_RGBA8 && format != GL_BGRA) {
      copy_bgra8_to_rgba8((uint32_t*)dest, (uint32_t*)src, width);
    } else {
      memcpy(dest, src, width * bpp);
    }
    dest += dest_stride;
    src += row_length * bpp;
  }
}

void TexImage2D(GLenum target, GLint level, GLint internal_format,
                GLsizei width, GLsizei height, GLint border, GLenum format,
                GLenum ty, void* data) {
  if (level != 0) { assert(false); return; }
  assert(border == 0);
  TexStorage2D(target, 1, internal_format, width, height);
  TexSubImage2D(target, 0, 0, 0, width, height, format, ty, data);
}

void TexSubImage3D(GLenum target, GLint level, GLint xoffset, GLint yoffset,
                   GLint zoffset, GLsizei width, GLsizei height, GLsizei depth,
                   GLenum format, GLenum ty, void* data) {
  if (level != 0) { assert(false); return; }
  data = get_pixel_unpack_buffer_data(data);
  if (!data) return;
  Texture& t = ctx->textures[ctx->get_binding(target)];
  prepare_texture(t);
  assert(ctx->unpack_row_length == 0 || ctx->unpack_row_length >= width);
  GLsizei row_length =
      ctx->unpack_row_length != 0 ? ctx->unpack_row_length : width;
  if (format == GL_BGRA) {
    assert(ty == GL_UNSIGNED_BYTE);
    assert(t.internal_format == GL_RGBA8);
  } else {
    assert(t.internal_format == internal_format_for_data(format, ty));
  }
  int bpp = t.bpp();
  if (!bpp || !t.buf) return;
  char* src = (char*)data;
  assert(xoffset + width <= t.width);
  assert(yoffset + height <= t.height);
  assert(zoffset + depth <= t.depth);
  size_t dest_stride = t.stride(bpp);
  for (int z = 0; z < depth; z++) {
    char* dest = t.sample_ptr(xoffset, yoffset, zoffset + z, bpp, dest_stride);
    for (int y = 0; y < height; y++) {
      if (t.internal_format == GL_RGBA8 && format != GL_BGRA) {
        copy_bgra8_to_rgba8((uint32_t*)dest, (uint32_t*)src, width);
      } else {
        memcpy(dest, src, width * bpp);
      }
      dest += dest_stride;
      src += row_length * bpp;
    }
  }
}

void TexImage3D(GLenum target, GLint level, GLint internal_format,
                GLsizei width, GLsizei height, GLsizei depth, GLint border,
                GLenum format, GLenum ty, void* data) {
  if (level != 0) { assert(false); return; }
  assert(border == 0);
  TexStorage3D(target, 1, internal_format, width, height, depth);
  TexSubImage3D(target, 0, 0, 0, 0, width, height, depth, format, ty, data);
}

void GenerateMipmap(UNUSED GLenum target) {
  // TODO: support mipmaps
}

void TexParameteri(GLenum target, GLenum pname, GLint param) {
  Texture& t = ctx->textures[ctx->get_binding(target)];
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
      if (unlink(fb->color_attachment, texture)) {
        fb->layer = 0;
      }
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
    case GL_DEPTH_COMPONENT24:
    case GL_DEPTH_COMPONENT32:
      // Force depth format to 16 bits...
      internal_format = GL_DEPTH_COMPONENT16;
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

void BufferData(GLenum target, GLsizeiptr size, void* data, UNUSED GLenum usage) {
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
  vertex_shader->set_uniform_1i(location, V0);
}
void Uniform4fv(GLint location, GLsizei count, const GLfloat* v) {
  assert(count == 1);
  vertex_shader->set_uniform_4fv(location, v);
}
void UniformMatrix4fv(GLint location, GLsizei count, GLboolean transpose,
                      const GLfloat* value) {
  assert(count == 1);
  assert(!transpose);
  vertex_shader->set_uniform_matrix4fv(location, value);
}

void FramebufferTexture2D(GLenum target, GLenum attachment, GLenum textarget,
                          GLuint texture, GLint level) {
  assert(target == GL_READ_FRAMEBUFFER || target == GL_DRAW_FRAMEBUFFER);
  assert(textarget == GL_TEXTURE_2D || textarget == GL_TEXTURE_RECTANGLE);
  assert(level == 0);
  Framebuffer& fb = ctx->framebuffers[ctx->get_binding(target)];
  if (attachment == GL_COLOR_ATTACHMENT0) {
    fb.color_attachment = texture;
    fb.layer = 0;
  } else if (attachment == GL_DEPTH_ATTACHMENT) {
    fb.depth_attachment = texture;
  } else {
    assert(0);
  }
}

void FramebufferTextureLayer(GLenum target, GLenum attachment, GLuint texture,
                             GLint level, GLint layer) {
  assert(target == GL_READ_FRAMEBUFFER || target == GL_DRAW_FRAMEBUFFER);
  assert(level == 0);
  Framebuffer& fb = ctx->framebuffers[ctx->get_binding(target)];
  if (attachment == GL_COLOR_ATTACHMENT0) {
    fb.color_attachment = texture;
    fb.layer = layer;
  } else if (attachment == GL_DEPTH_ATTACHMENT) {
    assert(layer == 0);
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
    fb.layer = 0;
  } else if (attachment == GL_DEPTH_ATTACHMENT) {
    fb.depth_attachment = rb.texture;
  } else {
    assert(0);
  }
}

}  // extern "C"

static inline Framebuffer* get_framebuffer(GLenum target) {
  if (target == GL_FRAMEBUFFER) {
    target = GL_DRAW_FRAMEBUFFER;
  }
  return ctx->framebuffers.find(ctx->get_binding(target));
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

static inline uint32_t clear_chunk(uint32_t value) {
  return value;
}

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
static void clear_buffer(Texture& t, T value, int layer, IntRect bb,
                         int skip_start = 0, int skip_end = 0) {
  if (!t.buf) return;
  skip_start = max(skip_start, bb.x0);
  skip_end = max(skip_end, skip_start);
  assert(sizeof(T) == t.bpp());
  size_t stride = t.stride(sizeof(T));
  // When clearing multiple full-width rows, collapse them into a single
  // large "row" to avoid redundant setup from clearing each row individually.
  if (bb.width() == t.width && bb.height() > 1 && skip_start >= skip_end) {
    bb.x1 += (stride / sizeof(T)) * (bb.height() - 1);
    bb.y1 = bb.y0 + 1;
  }
  T* buf = (T*)t.sample_ptr(bb.x0, bb.y0, layer, sizeof(T), stride);
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
static inline void clear_buffer(Texture& t, T value, int layer = 0) {
  IntRect bb = ctx->apply_scissor(t.bounds());
  if (bb.width() > 0) {
    clear_buffer<T>(t, value, layer, bb);
  }
}

template <typename T>
static inline void force_clear_row(Texture& t, int y, int skip_start = 0,
                                   int skip_end = 0) {
  assert(t.buf != nullptr);
  assert(sizeof(T) == t.bpp());
  assert(skip_start <= skip_end);
  T* buf = (T*)t.sample_ptr(0, y, 0, sizeof(T));
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
          clear_buffer<T>(t, t.clear_val, 0,
                          IntRect{0, start, t.width, start + count},
                          skip_start, skip_end);
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
        clear_buffer<T>(t, t.clear_val, 0,
                        IntRect{0, start, t.width, start + count},
                        skip_start, skip_end);
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
      case GL_DEPTH_COMPONENT16:
        force_clear<uint16_t>(t, skip);
        break;
      default:
        assert(false);
        break;
    }
  }
}

extern "C" {

void InitDefaultFramebuffer(int width, int height) {
  Framebuffer& fb = ctx->framebuffers[0];
  if (!fb.color_attachment) {
    GenTextures(1, &fb.color_attachment);
    fb.layer = 0;
  }
  Texture& colortex = ctx->textures[fb.color_attachment];
  if (colortex.width != width || colortex.height != height) {
    colortex.cleanup();
    set_tex_storage(colortex, GL_RGBA8, width, height);
  }
  if (!fb.depth_attachment) {
    GenTextures(1, &fb.depth_attachment);
  }
  Texture& depthtex = ctx->textures[fb.depth_attachment];
  if (depthtex.width != width || depthtex.height != height) {
    depthtex.cleanup();
    set_tex_storage(depthtex, GL_DEPTH_COMPONENT16, width, height);
  }
}

void* GetColorBuffer(GLuint fbo, GLboolean flush, int32_t* width,
                     int32_t* height) {
  Framebuffer* fb = ctx->framebuffers.find(fbo);
  if (!fb || !fb->color_attachment) {
    return nullptr;
  }
  Texture& colortex = ctx->textures[fb->color_attachment];
  if (flush) {
    prepare_texture(colortex);
  }
  *width = colortex.width;
  *height = colortex.height;
  return colortex.buf ? colortex.sample_ptr(0, 0, fb->layer) : nullptr;
}

void SetTextureBuffer(GLuint texid, GLenum internal_format, GLsizei width,
                      GLsizei height, void* buf, GLsizei min_width,
                      GLsizei min_height) {
  Texture& t = ctx->textures[texid];
  set_tex_storage(t, internal_format, width, height, !buf, buf, min_width,
                  min_height);
}

GLenum CheckFramebufferStatus(GLenum target) {
  Framebuffer* fb = get_framebuffer(target);
  if (!fb || !fb->color_attachment) {
    return GL_FRAMEBUFFER_UNSUPPORTED;
  }
  return GL_FRAMEBUFFER_COMPLETE;
}

static inline bool clear_requires_scissor(Texture& t) {
  return ctx->scissortest && !ctx->scissor.contains(t.bounds());
}

void Clear(GLbitfield mask) {
  Framebuffer& fb = *get_framebuffer(GL_DRAW_FRAMEBUFFER);
  if ((mask & GL_COLOR_BUFFER_BIT) && fb.color_attachment) {
    Texture& t = ctx->textures[fb.color_attachment];
    if (t.internal_format == GL_RGBA8) {
      uint32_t color = ctx->clearcolor;
      // If the clear would require a scissor, force clear anything outside
      // the scissor, and then immediately clear anything inside the scissor.
      if (clear_requires_scissor(t)) {
        force_clear<uint32_t>(t, &ctx->scissor);
        clear_buffer<uint32_t>(t, color, fb.layer);
      } else if (t.depth > 1) {
        // Delayed clear is not supported on texture arrays.
        t.disable_delayed_clear();
        clear_buffer<uint32_t>(t, color, fb.layer);
      } else {
        // Do delayed clear for 2D texture without scissor.
        t.enable_delayed_clear(color);
      }
    } else if (t.internal_format == GL_R8) {
      uint8_t color = uint8_t((ctx->clearcolor >> 16) & 0xFF);
      if (clear_requires_scissor(t)) {
        force_clear<uint8_t>(t, &ctx->scissor);
        clear_buffer<uint8_t>(t, color, fb.layer);
      } else if (t.depth > 1) {
        t.disable_delayed_clear();
        clear_buffer<uint8_t>(t, color, fb.layer);
      } else {
        t.enable_delayed_clear(color);
      }
    } else {
      assert(false);
    }
  }
  if ((mask & GL_DEPTH_BUFFER_BIT) && fb.depth_attachment) {
    Texture& t = ctx->textures[fb.depth_attachment];
    assert(t.internal_format == GL_DEPTH_COMPONENT16);
    uint16_t depth = uint16_t(0xFFFF * ctx->cleardepth) - 0x8000;
    if (clear_requires_scissor(t)) {
      force_clear<uint16_t>(t, &ctx->scissor);
      clear_buffer<uint16_t>(t, depth);
    } else {
      t.enable_delayed_clear(depth);
    }
  }
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
        t.disable_delayed_clear();
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
         format == GL_BGRA);
  Texture& t = ctx->textures[fb->color_attachment];
  if (!t.buf) return;
  prepare_texture(t);
  // debugf("read pixels %d, %d, %d, %d from fb %d with format %x\n", x, y,
  // width, height, ctx->read_framebuffer_binding, t.internal_format);
  assert(x + width <= t.width);
  assert(y + height <= t.height);
  if (internal_format_for_data(format, type) != t.internal_format) {
    debugf("mismatched format for read pixels: %x vs %x\n", t.internal_format,
           internal_format_for_data(format, type));
    assert(false);
  }
  int bpp = t.bpp();
  char* dest = (char*)data;
  size_t src_stride = t.stride(bpp);
  char* src = t.sample_ptr(x, y, fb->layer, bpp, src_stride);
  for (; height > 0; height--) {
    if (t.internal_format == GL_RGBA8 && format != GL_BGRA) {
      copy_bgra8_to_rgba8((uint32_t*)dest, (uint32_t*)src, width);
    } else {
      memcpy(dest, src, width * bpp);
    }
    dest += width * bpp;
    src += src_stride;
  }
}

void CopyImageSubData(GLuint srcName, GLenum srcTarget, UNUSED GLint srcLevel,
                      GLint srcX, GLint srcY, GLint srcZ, GLuint dstName,
                      GLenum dstTarget, UNUSED GLint dstLevel, GLint dstX, GLint dstY,
                      GLint dstZ, GLsizei srcWidth, GLsizei srcHeight,
                      GLsizei srcDepth) {
  assert(srcLevel == 0 && dstLevel == 0);
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
  IntRect skip = {dstX, dstY, dstX + srcWidth, dstY + srcHeight};
  prepare_texture(dsttex, &skip);
  assert(srctex.internal_format == dsttex.internal_format);
  assert(srcWidth >= 0);
  assert(srcHeight >= 0);
  assert(srcDepth >= 0);
  assert(srcX + srcWidth <= srctex.width);
  assert(srcY + srcHeight <= srctex.height);
  assert(srcZ + srcDepth <= max(srctex.depth, 1));
  assert(dstX + srcWidth <= dsttex.width);
  assert(dstY + srcHeight <= dsttex.height);
  assert(dstZ + srcDepth <= max(dsttex.depth, 1));
  int bpp = srctex.bpp();
  int src_stride = srctex.stride(bpp);
  int dest_stride = dsttex.stride(bpp);
  for (int z = 0; z < srcDepth; z++) {
    char* dest = dsttex.sample_ptr(dstX, dstY, dstZ + z, bpp, dest_stride);
    char* src = srctex.sample_ptr(srcX, srcY, srcZ + z, bpp, src_stride);
    for (int y = 0; y < srcHeight; y++) {
      memcpy(dest, src, srcWidth * bpp);
      dest += dest_stride;
      src += src_stride;
    }
  }
}

void CopyTexSubImage3D(GLenum target, UNUSED GLint level, GLint xoffset, GLint yoffset,
                       GLint zoffset, GLint x, GLint y, GLsizei width,
                       GLsizei height) {
  assert(level == 0);
  Framebuffer* fb = get_framebuffer(GL_READ_FRAMEBUFFER);
  if (!fb) return;
  CopyImageSubData(fb->color_attachment, GL_TEXTURE_3D, 0, x, y, fb->layer,
                   ctx->get_binding(target), GL_TEXTURE_3D, 0, xoffset, yoffset,
                   zoffset, width, height, 1);
}

void CopyTexSubImage2D(GLenum target, UNUSED GLint level, GLint xoffset, GLint yoffset,
                       GLint x, GLint y, GLsizei width, GLsizei height) {
  assert(level == 0);
  Framebuffer* fb = get_framebuffer(GL_READ_FRAMEBUFFER);
  if (!fb) return;
  CopyImageSubData(fb->color_attachment, GL_TEXTURE_2D_ARRAY, 0, x, y,
                   fb->layer, ctx->get_binding(target), GL_TEXTURE_2D_ARRAY, 0,
                   xoffset, yoffset, 0, width, height, 1);
}

}  // extern "C"

using PackedRGBA8 = V16<uint8_t>;
using WideRGBA8 = V16<uint16_t>;
using HalfRGBA8 = V8<uint16_t>;

static inline WideRGBA8 unpack(PackedRGBA8 p) { return CONVERT(p, WideRGBA8); }

static inline PackedRGBA8 pack(WideRGBA8 p) {
#if USE_SSE2
  return _mm_packus_epi16(lowHalf(p), highHalf(p));
#elif USE_NEON
  return vcombine_u8(vqmovn_u16(lowHalf(p)), vqmovn_u16(highHalf(p)));
#else
  return CONVERT(p, PackedRGBA8);
#endif
}

static inline HalfRGBA8 packRGBA8(I32 a, I32 b) {
#if USE_SSE2
  return _mm_packs_epi32(a, b);
#elif USE_NEON
  return vcombine_u16(vqmovun_s32(a), vqmovun_s32(b));
#else
  return CONVERT(combine(a, b), HalfRGBA8);
#endif
}

using PackedR8 = V4<uint8_t>;
using WideR8 = V4<uint16_t>;

static inline WideR8 unpack(PackedR8 p) { return CONVERT(p, WideR8); }

static inline WideR8 packR8(I32 a) {
#if USE_SSE2
  return lowHalf(bit_cast<V8<uint16_t>>(_mm_packs_epi32(a, a)));
#elif USE_NEON
  return vqmovun_s32(a);
#else
  return CONVERT(a, WideR8);
#endif
}

static inline PackedR8 pack(WideR8 p) {
#if USE_SSE2
  auto m = expand(p);
  auto r = bit_cast<V16<uint8_t>>(_mm_packus_epi16(m, m));
  return SHUFFLE(r, r, 0, 1, 2, 3);
#elif USE_NEON
  return lowHalf(bit_cast<V8<uint8_t>>(vqmovn_u16(expand(p))));
#else
  return CONVERT(p, PackedR8);
#endif
}

using ZMask4 = V4<int16_t>;
using ZMask8 = V8<int16_t>;

static inline PackedRGBA8 unpack(ZMask4 mask, uint32_t*) {
  return bit_cast<PackedRGBA8>(mask.xxyyzzww);
}

static inline WideR8 unpack(ZMask4 mask, uint8_t*) {
  return bit_cast<WideR8>(mask);
}

#if USE_SSE2
#  define ZMASK_NONE_PASSED 0xFFFF
#  define ZMASK_ALL_PASSED 0
static inline uint32_t zmask_code(ZMask8 mask) {
  return _mm_movemask_epi8(mask);
}
static inline uint32_t zmask_code(ZMask4 mask) {
  return zmask_code(mask.xyzwxyzw);
}
#else
using ZMask4Code = V4<uint8_t>;
using ZMask8Code = V8<uint8_t>;
#  define ZMASK_NONE_PASSED 0xFFFFFFFFU
#  define ZMASK_ALL_PASSED 0
static inline uint32_t zmask_code(ZMask4 mask) {
  return bit_cast<uint32_t>(CONVERT(mask, ZMask4Code));
}
static inline uint32_t zmask_code(ZMask8 mask) {
  return zmask_code(
      ZMask4((U16(lowHalf(mask)) >> 12) | (U16(highHalf(mask)) << 4)));
}
#endif

template <int FUNC, bool MASK>
static ALWAYS_INLINE int check_depth8(uint16_t z, uint16_t* zbuf,
                                      ZMask8& outmask) {
  ZMask8 dest = unaligned_load<ZMask8>(zbuf);
  ZMask8 src = int16_t(z);
  // Invert the depth test to check which pixels failed and should be discarded.
  ZMask8 mask = FUNC == GL_LEQUAL ?
                                  // GL_LEQUAL: Not(LessEqual) = Greater
                    ZMask8(src > dest)
                                  :
                                  // GL_LESS: Not(Less) = GreaterEqual
                    ZMask8(src >= dest);
  switch (zmask_code(mask)) {
    case ZMASK_NONE_PASSED:
      return 0;
    case ZMASK_ALL_PASSED:
      if (MASK) {
        unaligned_store(zbuf, src);
      }
      return -1;
    default:
      if (MASK) {
        unaligned_store(zbuf, (mask & dest) | (~mask & src));
      }
      outmask = mask;
      return 1;
  }
}

template <bool FULL_SPANS, bool DISCARD>
static ALWAYS_INLINE bool check_depth4(ZMask4 src, uint16_t* zbuf,
                                       ZMask4& outmask, int span = 0) {
  ZMask4 dest = unaligned_load<ZMask4>(zbuf);
  // Invert the depth test to check which pixels failed and should be discarded.
  ZMask4 mask = ctx->depthfunc == GL_LEQUAL
                    ?
                    // GL_LEQUAL: Not(LessEqual) = Greater
                    ZMask4(src > dest)
                    :
                    // GL_LESS: Not(Less) = GreaterEqual
                    ZMask4(src >= dest);
  if (!FULL_SPANS) {
    mask |= ZMask4(span) < ZMask4{1, 2, 3, 4};
  }
  if (zmask_code(mask) == ZMASK_NONE_PASSED) {
    return false;
  }
  if (!DISCARD && ctx->depthmask) {
    unaligned_store(zbuf, (mask & dest) | (~mask & src));
  }
  outmask = mask;
  return true;
}

template <bool FULL_SPANS, bool DISCARD>
static ALWAYS_INLINE bool check_depth4(uint16_t z, uint16_t* zbuf,
                                       ZMask4& outmask, int span = 0) {
  return check_depth4<FULL_SPANS, DISCARD>(ZMask4(int16_t(z)), zbuf, outmask,
                                           span);
}

template <typename T>
static inline ZMask4 packZMask4(T a) {
#if USE_SSE2
  return lowHalf(bit_cast<ZMask8>(_mm_packs_epi32(a, a)));
#elif USE_NEON
  return vqmovn_s32(a);
#else
  return CONVERT(a, ZMask4);
#endif
}

static ALWAYS_INLINE ZMask4 packDepth() {
  return packZMask4(cast(fragment_shader->gl_FragCoord.z * 0xFFFF) - 0x8000);
}

static ALWAYS_INLINE void discard_depth(ZMask4 src, uint16_t* zbuf,
                                        ZMask4 mask) {
  if (ctx->depthmask) {
    ZMask4 dest = unaligned_load<ZMask4>(zbuf);
    mask |= packZMask4(fragment_shader->isPixelDiscarded);
    unaligned_store(zbuf, (mask & dest) | (~mask & src));
  }
}

static ALWAYS_INLINE void discard_depth(uint16_t z, uint16_t* zbuf,
                                        ZMask4 mask) {
  discard_depth(ZMask4(int16_t(z)), zbuf, mask);
}

static inline WideRGBA8 pack_pixels_RGBA8(const vec4& v) {
  ivec4 i = round_pixel(v);
  HalfRGBA8 xz = packRGBA8(i.z, i.x);
  HalfRGBA8 yw = packRGBA8(i.y, i.w);
  HalfRGBA8 xy = zipLow(xz, yw);
  HalfRGBA8 zw = zipHigh(xz, yw);
  HalfRGBA8 lo = zip2Low(xy, zw);
  HalfRGBA8 hi = zip2High(xy, zw);
  return combine(lo, hi);
}

static inline WideRGBA8 pack_pixels_RGBA8(const vec4_scalar& v) {
  I32 i = round_pixel((Float){v.z, v.y, v.x, v.w});
  HalfRGBA8 c = packRGBA8(i, i);
  return combine(c, c);
}

static inline WideRGBA8 pack_pixels_RGBA8() {
  return pack_pixels_RGBA8(fragment_shader->gl_FragColor);
}

template <typename V>
static inline PackedRGBA8 pack_span(uint32_t*, const V& v) {
  return pack(pack_pixels_RGBA8(v));
}

static inline PackedRGBA8 pack_span(uint32_t*) {
  return pack(pack_pixels_RGBA8());
}

// (x*y + x) >> 8, cheap approximation of (x*y) / 255
template <typename T>
static inline T muldiv255(T x, T y) {
  return (x * y + x) >> 8;
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
static inline T addlow(T x, T y) {
  typedef VectorType<uint8_t, sizeof(T)> bytes;
  return bit_cast<T>(bit_cast<bytes>(x) + bit_cast<bytes>(y));
}

static inline WideRGBA8 alphas(WideRGBA8 c) {
  return SHUFFLE(c, c, 3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15);
}

static inline WideRGBA8 blend_pixels_RGBA8(PackedRGBA8 pdst, WideRGBA8 src) {
  WideRGBA8 dst = unpack(pdst);
  const WideRGBA8 RGB_MASK = {0xFFFF, 0xFFFF, 0xFFFF, 0,      0xFFFF, 0xFFFF,
                              0xFFFF, 0,      0xFFFF, 0xFFFF, 0xFFFF, 0,
                              0xFFFF, 0xFFFF, 0xFFFF, 0};
  const WideRGBA8 ALPHA_MASK = {0, 0, 0, 0xFFFF, 0, 0, 0, 0xFFFF,
                                0, 0, 0, 0xFFFF, 0, 0, 0, 0xFFFF};
  const WideRGBA8 ALPHA_OPAQUE = {0, 0, 0, 255, 0, 0, 0, 255,
                                  0, 0, 0, 255, 0, 0, 0, 255};
  switch (blend_key) {
    case BLEND_KEY_NONE:
      return src;
    case BLEND_KEY(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA, GL_ONE, GL_ONE):
      // dst + src.a*(src.rgb1 - dst.rgb0)
      // use addlow for signed overflow
      return addlow(dst,
          muldiv255(alphas(src), (src | ALPHA_OPAQUE) - (dst & RGB_MASK)));
    case BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC_ALPHA):
      return src + dst - muldiv255(dst, alphas(src));
    case BLEND_KEY(GL_ZERO, GL_ONE_MINUS_SRC_COLOR):
      return dst - muldiv255(dst, src);
    case BLEND_KEY(GL_ZERO, GL_ONE_MINUS_SRC_COLOR, GL_ZERO, GL_ONE):
      return dst - (muldiv255(dst, src) & RGB_MASK);
    case BLEND_KEY(GL_ZERO, GL_ONE_MINUS_SRC_ALPHA):
      return dst - muldiv255(dst, alphas(src));
    case BLEND_KEY(GL_ZERO, GL_SRC_COLOR):
      return muldiv255(src, dst);
    case BLEND_KEY(GL_ONE, GL_ONE):
      return src + dst;
    case BLEND_KEY(GL_ONE, GL_ONE, GL_ONE, GL_ONE_MINUS_SRC_ALPHA):
      return src + dst - (muldiv255(dst, src) & ALPHA_MASK);
    case BLEND_KEY(GL_ONE, GL_ZERO):
      return src;
    case BLEND_KEY(GL_ONE_MINUS_DST_ALPHA, GL_ONE, GL_ZERO, GL_ONE):
      // src*(1-dst.a) + dst*1 = src - src*dst.a + dst
      return dst + ((src - muldiv255(src, alphas(dst))) & RGB_MASK);
    case BLEND_KEY(GL_CONSTANT_COLOR, GL_ONE_MINUS_SRC_COLOR):
      // src*k + (1-src)*dst = src*k + dst - src*dst = dst + src*(k - dst)
      // use addlow for signed overflow
      return addlow(dst,
          muldiv255(src, combine(ctx->blendcolor, ctx->blendcolor) - dst));
    case BLEND_KEY(GL_ONE, GL_ONE_MINUS_SRC1_COLOR): {
      WideRGBA8 secondary =
          pack_pixels_RGBA8(fragment_shader->gl_SecondaryFragColor);
      return src + dst - muldiv255(dst, secondary);
    }
    default:
      UNREACHABLE;
      // return src;
  }
}

template <bool DISCARD>
static inline void discard_output(uint32_t* buf, PackedRGBA8 mask) {
  PackedRGBA8 dst = unaligned_load<PackedRGBA8>(buf);
  WideRGBA8 r = pack_pixels_RGBA8();
  if (blend_key) r = blend_pixels_RGBA8(dst, r);
  if (DISCARD) mask |= bit_cast<PackedRGBA8>(fragment_shader->isPixelDiscarded);
  unaligned_store(buf, (mask & dst) | (~mask & pack(r)));
}

template <bool DISCARD>
static inline void discard_output(uint32_t* buf) {
  discard_output<DISCARD>(buf, 0);
}

template <>
inline void discard_output<false>(uint32_t* buf) {
  WideRGBA8 r = pack_pixels_RGBA8();
  if (blend_key) r = blend_pixels_RGBA8(unaligned_load<PackedRGBA8>(buf), r);
  unaligned_store(buf, pack(r));
}

static inline PackedRGBA8 span_mask_RGBA8(int span) {
  return bit_cast<PackedRGBA8>(I32(span) < I32{1, 2, 3, 4});
}

static inline PackedRGBA8 span_mask(uint32_t*, int span) {
  return span_mask_RGBA8(span);
}

static inline WideR8 pack_pixels_R8(Float c) {
  return packR8(round_pixel(c));
}

static inline WideR8 pack_pixels_R8() {
  return pack_pixels_R8(fragment_shader->gl_FragColor.x);
}

template <typename C>
static inline PackedR8 pack_span(uint8_t*, C c) {
  return pack(pack_pixels_R8(c));
}

static inline PackedR8 pack_span(uint8_t*) { return pack(pack_pixels_R8()); }

static inline WideR8 blend_pixels_R8(WideR8 dst, WideR8 src) {
  switch (blend_key) {
    case BLEND_KEY_NONE:
      return src;
    case BLEND_KEY(GL_ZERO, GL_SRC_COLOR):
      return muldiv255(src, dst);
    case BLEND_KEY(GL_ONE, GL_ONE):
      return src + dst;
    case BLEND_KEY(GL_ONE, GL_ZERO):
      return src;
    default:
      UNREACHABLE;
      // return src;
  }
}

template <bool DISCARD>
static inline void discard_output(uint8_t* buf, WideR8 mask) {
  WideR8 dst = unpack(unaligned_load<PackedR8>(buf));
  WideR8 r = pack_pixels_R8();
  if (blend_key) r = blend_pixels_R8(dst, r);
  if (DISCARD) mask |= packR8(fragment_shader->isPixelDiscarded);
  unaligned_store(buf, pack((mask & dst) | (~mask & r)));
}

template <bool DISCARD>
static inline void discard_output(uint8_t* buf) {
  discard_output<DISCARD>(buf, 0);
}

template <>
inline void discard_output<false>(uint8_t* buf) {
  WideR8 r = pack_pixels_R8();
  if (blend_key) r = blend_pixels_R8(unpack(unaligned_load<PackedR8>(buf)), r);
  unaligned_store(buf, pack(r));
}

static inline WideR8 span_mask_R8(int span) {
  return bit_cast<WideR8>(WideR8(span) < WideR8{1, 2, 3, 4});
}

static inline WideR8 span_mask(uint8_t*, int span) {
  return span_mask_R8(span);
}

template <bool DISCARD, bool W, typename P, typename M>
static inline void commit_output(P* buf, M mask) {
  fragment_shader->run<W>();
  discard_output<DISCARD>(buf, mask);
}

template <bool DISCARD, bool W, typename P>
static inline void commit_output(P* buf) {
  fragment_shader->run<W>();
  discard_output<DISCARD>(buf);
}

template <bool DISCARD, bool W, typename P>
static inline void commit_output(P* buf, int span) {
  commit_output<DISCARD, W>(buf, span_mask(buf, span));
}

template <bool DISCARD, bool W, typename P, typename Z>
static inline void commit_output(P* buf, Z z, uint16_t* zbuf) {
  ZMask4 zmask;
  if (check_depth4<true, DISCARD>(z, zbuf, zmask)) {
    commit_output<DISCARD, W>(buf, unpack(zmask, buf));
    if (DISCARD) {
      discard_depth(z, zbuf, zmask);
    }
  } else {
    fragment_shader->skip<W>();
  }
}

template <bool DISCARD, bool W, typename P, typename Z>
static inline void commit_output(P* buf, Z z, uint16_t* zbuf, int span) {
  ZMask4 zmask;
  if (check_depth4<false, DISCARD>(z, zbuf, zmask, span)) {
    commit_output<DISCARD, W>(buf, unpack(zmask, buf));
    if (DISCARD) {
      discard_depth(z, zbuf, zmask);
    }
  }
}

static inline void commit_span(uint32_t* buf, PackedRGBA8 r) {
  if (blend_key)
    r = pack(blend_pixels_RGBA8(unaligned_load<PackedRGBA8>(buf), unpack(r)));
  unaligned_store(buf, r);
}

UNUSED static inline void commit_solid_span(uint32_t* buf, PackedRGBA8 r,
                                            int len) {
  if (blend_key) {
    auto src = unpack(r);
    for (uint32_t* end = &buf[len]; buf < end; buf += 4) {
      unaligned_store(
          buf, pack(blend_pixels_RGBA8(unaligned_load<PackedRGBA8>(buf), src)));
    }
  } else {
    fill_n(buf, len, bit_cast<U32>(r).x);
  }
}

UNUSED static inline void commit_texture_span(uint32_t* buf, uint32_t* src,
                                              int len) {
  if (blend_key) {
    for (uint32_t* end = &buf[len]; buf < end; buf += 4, src += 4) {
      PackedRGBA8 r = unaligned_load<PackedRGBA8>(src);
      unaligned_store(buf, pack(blend_pixels_RGBA8(
                               unaligned_load<PackedRGBA8>(buf), unpack(r))));
    }
  } else {
    memcpy(buf, src, len * sizeof(uint32_t));
  }
}

static inline void commit_span(uint8_t* buf, PackedR8 r) {
  if (blend_key)
    r = pack(blend_pixels_R8(unpack(unaligned_load<PackedR8>(buf)), unpack(r)));
  unaligned_store(buf, r);
}

UNUSED static inline void commit_solid_span(uint8_t* buf, PackedR8 r, int len) {
  if (blend_key) {
    auto src = unpack(r);
    for (uint8_t* end = &buf[len]; buf < end; buf += 4) {
      unaligned_store(buf, pack(blend_pixels_R8(
                               unpack(unaligned_load<PackedR8>(buf)), src)));
    }
  } else {
    fill_n((uint32_t*)buf, len / 4, bit_cast<uint32_t>(r));
  }
}

#define DISPATCH_DRAW_SPAN(self, buf, len) do {           \
  int drawn = self->draw_span(buf, len);                  \
  if (drawn) self->step_interp_inputs(drawn >> 2);        \
  for (buf += drawn; drawn < len; drawn += 4, buf += 4) { \
    run(self);                                            \
    commit_span(buf, pack_span(buf));                     \
  }                                                       \
} while (0)

#include "texture.h"

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wuninitialized"
#pragma GCC diagnostic ignored "-Wunused-function"
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wimplicit-fallthrough"
#ifdef __clang__
#pragma GCC diagnostic ignored "-Wunused-private-field"
#else
#pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#include "load_shader.h"
#pragma GCC diagnostic pop

typedef vec2_scalar Point2D;
typedef vec4_scalar Point3D;

struct ClipRect {
  float x0;
  float y0;
  float x1;
  float y1;

  ClipRect(const IntRect& i) : x0(i.x0), y0(i.y0), x1(i.x1), y1(i.y1) {}
  ClipRect(Texture& t) : ClipRect(ctx->apply_scissor(t.bounds())) {}

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

// Helper function for drawing 8-pixel wide chunks of a span with depth buffer.
// Using 8-pixel chunks maximizes use of 16-bit depth values in 128-bit wide
// SIMD register. However, since fragment shaders process only 4 pixels per
// invocation, we need to run fragment shader twice for every 8 pixel batch
// of results we get from the depth test. Perspective is not supported.
template <int FUNC, bool MASK, typename P>
static inline void draw_depth_span(uint16_t z, P* buf, uint16_t* depth,
                                   int span) {
  int skip = 0;
  // Check if the fragment shader has an optimized draw specialization.
  if (fragment_shader->has_draw_span(buf)) {
    // The loop tries to accumulate runs of pixels that passed (len) and
    // runs of pixels that failed (skip). This allows it to pass the largest
    // possible span in between changes in depth pass or fail status to the
    // fragment shader's draw specialer.
    int len = 0;
    do {
      ZMask8 zmask;
      // Process depth in 8-pixel chunks.
      switch (check_depth8<FUNC, MASK>(z, depth, zmask)) {
        case 0: // All pixels failed the depth test.
          if (len) {
            // Flush out passed pixels.
            fragment_shader->draw_span(buf - len, len);
            len = 0;
          }
          // Accumulate 2 skipped chunks.
          skip += 2;
          break;
        case -1: // All pixels passed the depth test.
          if (skip) {
            // Flushed out any skipped chunks.
            fragment_shader->skip(skip);
            skip = 0;
          }
          // Accumulate 8 passed pixels.
          len += 8;
          break;
        default: // Mixture of pass and fail results.
          if (len) {
            // Flush out any passed pixels.
            fragment_shader->draw_span(buf - len, len);
            len = 0;
          } else if (skip) {
            // Flush out any skipped chunks.
            fragment_shader->skip(skip);
            skip = 0;
          }
          // Run fragment shader on first 4 depth results.
          commit_output<false, false>(buf, unpack(lowHalf(zmask), buf));
          // Run fragment shader on next 4 depth results.
          commit_output<false, false>(buf + 4, unpack(highHalf(zmask), buf));
          break;
      }
      // Advance to next 8 pixels...
      buf += 8;
      depth += 8;
      span -= 8;
    } while (span >= 8);
    // Flush out any remaining passed pixels.
    if (len) {
      fragment_shader->draw_span(buf - len, len);
    }
  } else {
    // No draw specialization, so we can use a simpler loop here that just
    // accumulates depth failures, but otherwise invokes fragment shader
    // immediately on depth pass.
    do {
      ZMask8 zmask;
      // Process depth in 8-pixel chunks.
      switch (check_depth8<FUNC, MASK>(z, depth, zmask)) {
        case 0: // All pixels failed the depth test.
          // Accumulate 2 skipped chunks.
          skip += 2;
          break;
        case -1: // All pixels passed the depth test.
          if (skip) {
            // Flush out any skipped chunks.
            fragment_shader->skip(skip);
            skip = 0;
          }
          // Run the fragment shader for two 4-pixel chunks.
          commit_output<false, false>(buf);
          commit_output<false, false>(buf + 4);
          break;
        default: // Mixture of pass and fail results.
          if (skip) {
            // Flush out any skipped chunks.
            fragment_shader->skip(skip);
            skip = 0;
          }
          // Run fragment shader on first 4 depth results.
          commit_output<false, false>(buf, unpack(lowHalf(zmask), buf));
          // Run fragment shader on next 4 depth results.
          commit_output<false, false>(buf + 4, unpack(highHalf(zmask), buf));
          break;
      }
      // Advance to next 8 pixels...
      buf += 8;
      depth += 8;
      span -= 8;
    } while (span >= 8);
  }
  // Flush out any remaining skipped chunks.
  if (skip) {
    fragment_shader->skip(skip);
  }
}

// Draw a simple span in 4-pixel wide chunks, optionally using depth.
template <bool DISCARD, bool W, typename P, typename Z>
static ALWAYS_INLINE void draw_span(P* buf, uint16_t* depth, int span, Z z) {
  if (depth) {
    // Depth testing is enabled. If perspective is used, Z values will vary
    // across the span, we use packDepth to generate 16-bit Z values suitable
    // for depth testing based on current values from gl_FragCoord.z.
    // Otherwise, for the no-perspective case, we just use the provided Z.
    // Process 4-pixel chunks first.
    for (; span >= 4; span -= 4, buf += 4, depth += 4) {
      commit_output<DISCARD, W>(buf, z(), depth);
    }
    // If there are any remaining pixels, do a partial chunk.
    if (span > 0) {
      commit_output<DISCARD, W>(buf, z(), depth, span);
    }
  } else {
    // Process 4-pixel chunks first.
    for (; span >= 4; span -= 4, buf += 4) {
      commit_output<DISCARD, W>(buf);
    }
    // If there are any remaining pixels, do a partial chunk.
    if (span > 0) {
      commit_output<DISCARD, W>(buf, span);
    }
  }
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
static inline void draw_quad_spans(int nump, Point2D p[4], uint16_t z,
                                   Interpolants interp_outs[4],
                                   Texture& colortex, int layer,
                                   Texture& depthtex,
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
    l0 = p[l0i]; // Start of left edge
    r0 = p[r0i]; // End of left edge
    l1 = p[l1i]; // Start of right edge
    r1 = p[r1i]; // End of right edge
    //    debugf("l0: %d(%f,%f), r0: %d(%f,%f) -> l1: %d(%f,%f), r1:
    //    %d(%f,%f)\n", l0i, l0.x, l0.y, r0i, r0.x, r0.y, l1i, l1.x, l1.y, r1i,
    //    r1.x, r1.y);
  }

  struct Edge
  {
    float yScale;
    float xSlope;
    float x;
    Interpolants interpSlope;
    Interpolants interp;

    Edge(float y, const Point2D& p0, const Point2D& p1,
         const Interpolants& i0, const Interpolants& i1) :
      // Inverse Y scale for slope calculations. Avoid divide on 0-length edge.
      // Later checks below ensure that Y <= p1.y, or otherwise we don't use
      // this edge. We just need to guard against Y == p1.y == p0.y. In that
      // case, Y - p0.y == 0 and will cancel out the slopes below, except if
      // yScale is Inf for some reason (or worse, NaN), which 1/(p1.y-p0.y)
      // might produce if we don't bound it.
      yScale(1.0f / max(p1.y - p0.y, 1.0f / 256)),
      // Calculate dX/dY slope
      xSlope((p1.x - p0.x) * yScale),
      // Initialize current X based on Y and slope
      x(p0.x + (y - p0.y) * xSlope),
      // Calculate change in interpolants per change in Y
      interpSlope((i1 - i0) * yScale),
      // Initialize current interpolants based on Y and slope
      interp(i0 + (y - p0.y) * interpSlope)
    {}

    void nextRow() {
      // step current X and interpolants to next row from slope
      x += xSlope;
      interp += interpSlope;
    }
  };

  // Vertex selection above should result in equal left and right start rows
  assert(l0.y == r0.y);
  // Find the start y, clip to within the clip rect, and round to row center.
  float y = floor(max(l0.y, clipRect.y0) + 0.5f) + 0.5f;
  // Initialize left and right edges from end points and start Y
  Edge left(y, l0, l1, interp_outs[l0i], interp_outs[l1i]);
  Edge right(y, r0, r1, interp_outs[r0i], interp_outs[r1i]);
  // Get pointer to color buffer and depth buffer at current Y
  P* fbuf = (P*)colortex.sample_ptr(0, int(y), layer, sizeof(P));
  uint16_t* fdepth =
    (uint16_t*)depthtex.sample_ptr(0, int(y), 0, sizeof(uint16_t));
  // Loop along advancing Ys, rasterizing spans at each row
  float checkY = min(min(l1.y, r1.y), clipRect.y1);
  for (;;) {
    // Check if we maybe passed edge ends or outside clip rect...
    if (y > checkY) {
      // If we're outside the clip rect, we're done.
      if (y > clipRect.y1) break;
      // Helper to find the next non-duplicate vertex that doesn't loop back.
#define STEP_EDGE(e0i, e0, e1i, e1, STEP_POINT, end)                   \
      for (;;) {                                                       \
        /* Set new start of edge to be end of old edge */              \
        e0i = e1i;                                                     \
        e0 = e1;                                                       \
        /* Set new end of edge to next point */                        \
        e1i = STEP_POINT(e1i);                                         \
        e1 = p[e1i];                                                   \
        /* If the edge is descending, use it. */                       \
        if (e1.y > e0.y) break;                                        \
        /* If the edge is ascending or crossed the end, we're done. */ \
        if (e1.y < e0.y || e0i == end) return;                         \
        /* Otherwise, it's a duplicate, so keep searching. */          \
      }
      // Check if Y advanced past the end of the left edge
      if (y > l1.y) {
        // Step to next left edge past Y and reset edge interpolants.
        do { STEP_EDGE(l0i, l0, l1i, l1, NEXT_POINT, r1i); } while (y > l1.y);
        left = Edge(y, l0, l1, interp_outs[l0i], interp_outs[l1i]);
      }
      // Check if Y advanced past the end of the right edge
      if (y > r1.y) {
        // Step to next right edge past Y and reset edge interpolants.
        do { STEP_EDGE(r0i, r0, r1i, r1, PREV_POINT, l1i); } while (y > r1.y);
        right = Edge(y, r0, r1, interp_outs[r0i], interp_outs[r1i]);
      }
      // Reset check condition for next time around.
      checkY = min(min(l1.y, r1.y), clipRect.y1);
    }
    // lx..rx form the bounds of the span. WR does not use backface culling,
    // so we need to use min/max to support the span in either orientation.
    // Clip the span to fall within the clip rect and then round to nearest
    // column.
    int startx = int(max(min(left.x, right.x), clipRect.x0) + 0.5f);
    int endx = int(min(max(left.x, right.x), clipRect.x1) + 0.5f);
    // Check if span is non-empty.
    int span = endx - startx;
    if (span > 0) {
      ctx->shaded_rows++;
      ctx->shaded_pixels += span;
      // Advance color/depth buffer pointers to the start of the span.
      P* buf = fbuf + startx;
      // Check if the we will need to use depth-buffer or discard on this span.
      uint16_t* depth = depthtex.buf != nullptr ? fdepth + startx : nullptr;
      bool use_discard = fragment_shader->use_discard();
      if (depthtex.delay_clear) {
        // Delayed clear is enabled for the depth buffer. Check if this row
        // needs to be cleared.
        int yi = int(y);
        uint32_t& mask = depthtex.cleared_rows[yi / 32];
        if ((mask & (1 << (yi & 31))) == 0) {
          // The depth buffer is unitialized on this row, but we know it will
          // thus be cleared entirely to the clear value. This lets us quickly
          // check the constant Z value of the quad against the clear Z to know
          // if the entire span passes or fails the depth test all at once.
          switch (ctx->depthfunc) {
            case GL_LESS:
              if (int16_t(z) < int16_t(depthtex.clear_val))
                break;
              else
                goto next_span;
            case GL_LEQUAL:
              if (int16_t(z) <= int16_t(depthtex.clear_val))
                break;
              else
                goto next_span;
          }
          // If we got here, we passed the depth test.
          if (ctx->depthmask) {
            // Depth writes are enabled, so we need to initialize depth.
            mask |= 1 << (yi & 31);
            depthtex.delay_clear--;
            if (use_discard) {
              // if discard is enabled, we don't know what pixels may be
              // written to, so we have to clear the entire row.
              force_clear_row<uint16_t>(depthtex, yi);
            } else {
              // Otherwise, we only need to clear the pixels that fall outside
              // the current span on this row.
              if (startx > 0 || endx < depthtex.width) {
                force_clear_row<uint16_t>(depthtex, yi, startx, endx);
              }
              // Fill in the span's Z values with constant Z.
              clear_buffer<uint16_t>(depthtex, z, 0,
                                     IntRect{startx, yi, endx, yi + 1});
              // We already passed the depth test, so no need to test depth
              // any more.
              depth = nullptr;
            }
          } else {
            // No depth writes, so don't clear anything, and no need to test.
            depth = nullptr;
          }
        }
      }
      if (colortex.delay_clear) {
        // Delayed clear is enabled for the color buffer. Check if needs clear.
        int yi = int(y);
        uint32_t& mask = colortex.cleared_rows[yi / 32];
        if ((mask & (1 << (yi & 31))) == 0) {
          mask |= 1 << (yi & 31);
          colortex.delay_clear--;
          if (depth || blend_key || use_discard) {
            // If depth test, blending, or discard is used, old color values
            // might be sampled, so we need to clear the entire row to fill it.
            force_clear_row<P>(colortex, yi);
          } else if (startx > 0 || endx < colortex.width) {
            // Otherwise, we only need to clear the row outside of the span.
            // The fragment shader will fill the row within the span itself.
            force_clear_row<P>(colortex, yi, startx, endx);
          }
        }
      }
      // Initialize fragment shader interpolants to current span position.
      fragment_shader->gl_FragCoord.x = init_interp(startx + 0.5f, 1);
      fragment_shader->gl_FragCoord.y = y;
      {
        // Change in interpolants is difference between current right and left
        // edges per the change in right and left X.
        Interpolants step =
            (right.interp - left.interp) * (1.0f / (right.x - left.x));
        // Advance current interpolants to X at start of span.
        Interpolants o = left.interp + step * (startx + 0.5f - left.x);
        fragment_shader->init_span(&o, &step, 4.0f);
      }
      if (!use_discard) {
        // Fast paths for the case where fragment discard is not used.
        if (depth) {
          // If depth is used, we want to process spans in 8-pixel chunks to
          // maximize sampling and testing 16-bit depth values within the 128-
          // bit width of a SIMD register.
          if (span >= 8) {
            // Specializations for supported depth functions depending on
            // whether depth writes are enabled.
            if (ctx->depthfunc == GL_LEQUAL) {
              if (ctx->depthmask)
                draw_depth_span<GL_LEQUAL, true>(z, buf, depth, span);
              else
                draw_depth_span<GL_LEQUAL, false>(z, buf, depth, span);
            } else {
              if (ctx->depthmask)
                draw_depth_span<GL_LESS, true>(z, buf, depth, span);
              else
                draw_depth_span<GL_LESS, false>(z, buf, depth, span);
            }
            // Advance buffers past processed chunks.
            buf += span & ~7;
            depth += span & ~7;
            span &= 7;
          }
        } else {
          // Check if the fragment shader has an optimized draw specialization.
          if (span >= 4 && fragment_shader->has_draw_span(buf)) {
            // Draw specialization expects 4-pixel chunks.
            int len = span & ~3;
            fragment_shader->draw_span(buf, len);
            buf += len;
            span &= 3;
          }
        }
        draw_span<false, false>(buf, depth, span, [=]{ return z; });
      } else {
        // If discard is used, then use slower fallbacks. This should be rare.
        // Just needs to work, doesn't need to be too fast yet...
        draw_span<true, false>(buf, depth, span, [=]{ return z; });
      }
    }
  next_span:
    // Advance Y and edge interpolants to next row.
    y++;
    left.nextRow();
    right.nextRow();
    // Advance buffers to next row.
    fbuf += colortex.stride(sizeof(P)) / sizeof(P);
    fdepth += depthtex.stride(sizeof(uint16_t)) / sizeof(uint16_t);
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
                                          Texture& colortex, int layer,
                                          Texture& depthtex,
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
    l0 = p[l0i]; // Start of left edge
    r0 = p[r0i]; // End of left edge
    l1 = p[l1i]; // Start of right edge
    r1 = p[r1i]; // End of right edge
  }

  struct Edge
  {
    float yScale;
    // Current coordinates for edge. Where in the 2D case of draw_quad_spans,
    // it is enough to just track the X coordinate as we advance along the rows,
    // for the perspective case we also need to keep track of Z and W. For
    // simplicity, we just use the full 3D point to track all these coordinates.
    Point3D pSlope;
    Point3D p;
    Interpolants interpSlope;
    Interpolants interp;

    Edge(float y, const Point3D& p0, const Point3D& p1,
         const Interpolants& i0, const Interpolants& i1) :
      // Inverse Y scale for slope calculations. Avoid divide on 0-length edge.
      yScale(1.0f / max(p1.y - p0.y, 1.0f / 256)),
      // Calculate dX/dY slope
      pSlope((p1 - p0) * yScale),
      // Initialize current coords based on Y and slope
      p(p0 + (y - p0.y) * pSlope),
      // Crucially, these interpolants must be scaled by the point's 1/w value,
      // which allows linear interpolation in a perspective-correct manner.
      // This will be canceled out inside the fragment shader later.
      // Calculate change in interpolants per change in Y
      interpSlope((i1 * p1.w - i0 * p0.w) * yScale),
      // Initialize current interpolants based on Y and slope
      interp(i0 * p0.w + (y - p0.y) * interpSlope)
    {}

    float x() const { return p.x; }
    vec2_scalar zw() const { return {p.z, p.w}; }

    void nextRow() {
      // step current coords and interpolants to next row from slope
      p += pSlope;
      interp += interpSlope;
    }
  };

  // Vertex selection above should result in equal left and right start rows
  assert(l0.y == r0.y);
  // Find the start y, clip to within the clip rect, and round to row center.
  float y = floor(max(l0.y, clipRect.y0) + 0.5f) + 0.5f;
  // Initialize left and right edges from end points and start Y
  Edge left(y, l0, l1, interp_outs[l0i], interp_outs[l1i]);
  Edge right(y, r0, r1, interp_outs[r0i], interp_outs[r1i]);
  // Get pointer to color buffer and depth buffer at current Y
  P* fbuf = (P*)colortex.sample_ptr(0, int(y), layer, sizeof(P));
  uint16_t* fdepth =
    (uint16_t*)depthtex.sample_ptr(0, int(y), 0, sizeof(uint16_t));
  // Loop along advancing Ys, rasterizing spans at each row
  float checkY = min(min(l1.y, r1.y), clipRect.y1);
  for (;;) {
    // Check if we maybe passed edge ends or outside clip rect...
    if (y > checkY) {
      // If we're outside the clip rect, we're done.
      if (y > clipRect.y1) break;
      // Check if Y advanced past the end of the left edge
      if (y > l1.y) {
        // Step to next left edge past Y and reset edge interpolants.
        do { STEP_EDGE(l0i, l0, l1i, l1, NEXT_POINT, r1i); } while (y > l1.y);
        left = Edge(y, l0, l1, interp_outs[l0i], interp_outs[l1i]);
      }
      // Check if Y advanced past the end of the right edge
      if (y > r1.y) {
        // Step to next right edge past Y and reset edge interpolants.
        do { STEP_EDGE(r0i, r0, r1i, r1, PREV_POINT, l1i); } while (y > r1.y);
        right = Edge(y, r0, r1, interp_outs[r0i], interp_outs[r1i]);
      }
      // Reset check condition for next time around.
      checkY = min(min(l1.y, r1.y), clipRect.y1);
    }
    // lx..rx form the bounds of the span. WR does not use backface culling,
    // so we need to use min/max to support the span in either orientation.
    // Clip the span to fall within the clip rect and then round to nearest
    // column.
    int startx = int(max(min(left.x(), right.x()), clipRect.x0) + 0.5f);
    int endx = int(min(max(left.x(), right.x()), clipRect.x1) + 0.5f);
    // Check if span is non-empty.
    int span = endx - startx;
    if (span > 0) {
      ctx->shaded_rows++;
      ctx->shaded_pixels += span;
      // Advance color/depth buffer pointers to the start of the span.
      P* buf = fbuf + startx;
      // Check if the we will need to use depth-buffer or discard on this span.
      uint16_t* depth = depthtex.buf != nullptr ? fdepth + startx : nullptr;
      bool use_discard = fragment_shader->use_discard();
      if (depthtex.delay_clear) {
        // Delayed clear is enabled for the depth buffer. Check if this row
        // needs to be cleared.
        int yi = int(y);
        uint32_t& mask = depthtex.cleared_rows[yi / 32];
        if ((mask & (1 << (yi & 31))) == 0) {
          mask |= 1 << (yi & 31);
          depthtex.delay_clear--;
          // Since Z varies across the span, it's easier to just clear the
          // row and rely on later depth testing. If necessary, this could be
          // optimized to test against the start and end Z values of the span
          // here.
          force_clear_row<uint16_t>(depthtex, yi);
        }
      }
      if (colortex.delay_clear) {
        // Delayed clear is enabled for the color buffer. Check if needs clear.
        int yi = int(y);
        uint32_t& mask = colortex.cleared_rows[yi / 32];
        if ((mask & (1 << (yi & 31))) == 0) {
          mask |= 1 << (yi & 31);
          colortex.delay_clear--;
          if (depth || blend_key || use_discard) {
            // If depth test, blending, or discard is used, old color values
            // might be sampled, so we need to clear the entire row to fill it.
            force_clear_row<P>(colortex, yi);
          } else if (startx > 0 || endx < colortex.width) {
            // Otherwise, we only need to clear the row outside of the span.
            // The fragment shader will fill the row within the span itself.
            force_clear_row<P>(colortex, yi, startx, endx);
          }
        }
      }
      // Initialize fragment shader interpolants to current span position.
      fragment_shader->gl_FragCoord.x = init_interp(startx + 0.5f, 1);
      fragment_shader->gl_FragCoord.y = y;
      {
        // Calculate the fragment Z and W change per change in fragment X step.
        vec2_scalar stepZW =
            (right.zw() - left.zw()) * (1.0f / (right.x() - left.x()));
        // Calculate initial Z and W values for span start.
        vec2_scalar zw = left.zw() + stepZW * (startx + 0.5f - left.x());
        // Set fragment shader's Z and W values so that it can use them to
        // cancel out the 1/w baked into the interpolants.
        fragment_shader->gl_FragCoord.z = init_interp(zw.x, stepZW.x);
        fragment_shader->gl_FragCoord.w = init_interp(zw.y, stepZW.y);
        fragment_shader->stepZW = stepZW * 4.0f;
        // Change in interpolants is difference between current right and left
        // edges per the change in right and left X. The left and right
        // interpolant values were previously multipled by 1/w, so the step and
        // initial span values take this into account.
        Interpolants step =
            (right.interp - left.interp) * (1.0f / (right.x() - left.x()));
        // Advance current interpolants to X at start of span.
        Interpolants o = left.interp + step * (startx + 0.5f - left.x());
        fragment_shader->init_span<true>(&o, &step, 4.0f);
      }
      if (!use_discard) {
        // No discard is used. Common case.
        draw_span<false, true>(buf, depth, span, packDepth);
      } else {
        // Discard is used. Rare.
        draw_span<true, true>(buf, depth, span, packDepth);
      }
    }
    // Advance Y and edge interpolants to next row.
    y++;
    left.nextRow();
    right.nextRow();
    // Advance buffers to next row.
    fbuf += colortex.stride(sizeof(P)) / sizeof(P);
    fdepth += depthtex.stride(sizeof(uint16_t)) / sizeof(uint16_t);
  }
}

// Clip a primitive against both sides of a view-frustum axis, producing
// intermediate vertexes with interpolated attributes that will no longer
// intersect the selected axis planes. This assumes the primitive is convex
// and should produce at most N+2 vertexes for each invocation (only in the
// worst case where one point falls outside on each of the opposite sides
// with the rest of the points inside).
template <XYZW AXIS>
static int clip_side(int nump, Point3D* p, Interpolants* interp, Point3D* outP,
                     Interpolants* outInterp) {
  int numClip = 0;
  Point3D prev = p[nump - 1];
  Interpolants prevInterp = interp[nump - 1];
  float prevCoord = prev.select(AXIS);
  // Coordinate must satisfy -W <= C <= W. Determine if it is outside, and
  // if so, remember which side it is outside of.
  int prevSide = prevCoord < -prev.w ? -1 : (prevCoord > prev.w ? 1 : 0);
  // Loop through points, finding edges that cross the planes by evaluating
  // the side at each point.
  for (int i = 0; i < nump; i++) {
    Point3D cur = p[i];
    Interpolants curInterp = interp[i];
    float curCoord = cur.select(AXIS);
    int curSide = curCoord < -cur.w ? -1 : (curCoord > cur.w ? 1 : 0);
    // Check if the previous and current end points are on different sides.
    if (curSide != prevSide) {
      // One of the edge's end points is outside the plane with the other
      // inside the plane. Find the offset where it crosses the plane and
      // adjust the point and interpolants to there.
      if (prevSide) {
        // Edge that was previously outside crosses inside.
        // Evaluate plane equation for previous and current end-point
        // based on previous side and calculate relative offset.
        assert(numClip < nump + 2);
        float prevDist = prevCoord - prevSide * prev.w;
        float curDist = curCoord - prevSide * cur.w;
        float k = prevDist / (prevDist - curDist);
        outP[numClip] = prev + (cur - prev) * k;
        outInterp[numClip] = prevInterp + (curInterp - prevInterp) * k;
        numClip++;
      }
      if (curSide) {
        // Edge that was previously inside crosses outside.
        // Evaluate plane equation for previous and current end-point
        // based on current side and calculate relative offset.
        assert(numClip < nump + 2);
        float prevDist = prevCoord - curSide * prev.w;
        float curDist = curCoord - curSide * cur.w;
        float k = prevDist / (prevDist - curDist);
        outP[numClip] = prev + (cur - prev) * k;
        outInterp[numClip] = prevInterp + (curInterp - prevInterp) * k;
        numClip++;
      }
    }
    if (!curSide) {
      // The current end point is inside the plane, so output point unmodified.
      assert(numClip < nump + 2);
      outP[numClip] = cur;
      outInterp[numClip] = curInterp;
      numClip++;
    }
    prev = cur;
    prevInterp = curInterp;
    prevCoord = curCoord;
    prevSide = curSide;
  }
  return numClip;
}

// Helper function to dispatch to perspective span drawing with points that
// have already been transformed and clipped.
static inline void draw_perspective_clipped(int nump, Point3D* p_clip,
                                            Interpolants* interp_clip,
                                            Texture& colortex, int layer,
                                            Texture& depthtex) {
  // If polygon is ouside clip rect, nothing to draw.
  ClipRect clipRect(colortex);
  if (!clipRect.overlaps(nump, p_clip)) {
    return;
  }

  // Finally draw perspective-correct spans for the polygon.
  if (colortex.internal_format == GL_RGBA8) {
    draw_perspective_spans<uint32_t>(nump, p_clip, interp_clip, colortex,
                                     layer, depthtex, clipRect);
  } else if (colortex.internal_format == GL_R8) {
    draw_perspective_spans<uint8_t>(nump, p_clip, interp_clip, colortex,
                                    layer, depthtex, clipRect);
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
static void draw_perspective(int nump,
                             Interpolants interp_outs[4],
                             Texture& colortex, int layer,
                             Texture& depthtex) {
  // Convert output of vertex shader to screen space.
  vec4 pos = vertex_shader->gl_Position;
  vec3_scalar scale =
    vec3_scalar(ctx->viewport.width(), ctx->viewport.height(), 1) * 0.5f;
  vec3_scalar offset =
    vec3_scalar(ctx->viewport.x0, ctx->viewport.y0, 0.0f) + scale;
  if (test_none(pos.z <= -pos.w || pos.z >= pos.w)) {
    // No points cross the near or far planes, so no clipping required.
    // Just divide coords by W and convert to viewport.
    Float w = 1.0f / pos.w;
    vec3 screen = pos.sel(X, Y, Z) * w * scale + offset;
    Point3D p[4] = {
        {screen.x.x, screen.y.x, screen.z.x, w.x},
        {screen.x.y, screen.y.y, screen.z.y, w.y},
        {screen.x.z, screen.y.z, screen.z.z, w.z},
        {screen.x.w, screen.y.w, screen.z.w, w.w}
    };
    draw_perspective_clipped(nump, p, interp_outs, colortex, layer, depthtex);
  } else {
    // Points cross the near or far planes, so we need to clip.
    // Start with the original 3 or 4 points...
    Point3D p[4] = {
        {pos.x.x, pos.y.x, pos.z.x, pos.w.x},
        {pos.x.y, pos.y.y, pos.z.y, pos.w.y},
        {pos.x.z, pos.y.z, pos.z.z, pos.w.z},
        {pos.x.w, pos.y.w, pos.z.w, pos.w.w}
    };
    // Clipping can expand the points by 1 for each of 6 view frustum planes.
    Point3D p_clip[4 + 6];
    Interpolants interp_clip[4 + 6];
    // Clip against near and far Z planes.
    nump = clip_side<Z>(nump, p, interp_outs, p_clip, interp_clip);
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
        nump = clip_side<X>(nump, p_clip, interp_clip, p_tmp, interp_tmp);
        if (nump < 3) return;
        nump = clip_side<Y>(nump, p_tmp, interp_tmp, p_clip, interp_clip);
        if (nump < 3) return;
        // After clipping against X and Y planes, there's still points left
        // to draw, so proceed to trying projection now...
        break;
      }
    }
    // Divide coords by W and convert to viewport.
    for (int i = 0; i < nump; i++) {
      float w = 1.0f / p_clip[i].w;
      p_clip[i] = Point3D(p_clip[i].sel(X, Y, Z) * w * scale + offset, w);
    }
    draw_perspective_clipped(nump, p_clip, interp_clip, colortex, layer,
                             depthtex);
  }
}

static void draw_quad(int nump, Texture& colortex, int layer,
                      Texture& depthtex) {
  // Run vertex shader once for the primitive's vertices.
  // Reserve space for 6 sets of interpolants, in case we need to clip against
  // near and far planes in the perspective case.
  Interpolants interp_outs[4];
  vertex_shader->run_primitive((char*)interp_outs, sizeof(Interpolants));
  vec4 pos = vertex_shader->gl_Position;
  // Check if any vertex W is different from another. If so, use perspective.
  if (test_any(pos.w != pos.w.x)) {
    draw_perspective(nump, interp_outs, colortex, layer, depthtex);
    return;
  }

  // Convert output of vertex shader to screen space.
  // Divide coords by W and convert to viewport.
  float w = 1.0f / pos.w.x;
  vec2 screen =
      (pos.sel(X, Y) * w + 1) * 0.5f *
          vec2_scalar(ctx->viewport.width(), ctx->viewport.height()) +
      vec2_scalar(ctx->viewport.x0, ctx->viewport.y0);
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
  // SSE2 does not support unsigned comparison, so bias Z to be negative.
  uint16_t z = uint16_t(0xFFFF * screenZ) - 0x8000;
  fragment_shader->gl_FragCoord.z = screenZ;
  fragment_shader->gl_FragCoord.w = w;

  // Finally draw 2D spans for the quad. Currently only supports drawing to
  // RGBA8 and R8 color buffers.
  if (colortex.internal_format == GL_RGBA8) {
    draw_quad_spans<uint32_t>(nump, p, z, interp_outs, colortex, layer,
                              depthtex, clipRect);
  } else if (colortex.internal_format == GL_R8) {
    draw_quad_spans<uint8_t>(nump, p, z, interp_outs, colortex, layer, depthtex,
                             clipRect);
  } else {
    assert(false);
  }
}

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

template <typename INDEX>
static inline void draw_elements(GLsizei count, GLsizei instancecount,
                                 Buffer& indices_buf, size_t offset,
                                 VertexArray& v, Texture& colortex, int layer,
                                 Texture& depthtex) {
  assert((offset & (sizeof(INDEX) - 1)) == 0);
  INDEX* indices = (INDEX*)(indices_buf.buf + offset);
  count = min(count,
              (GLsizei)((indices_buf.size - offset) / sizeof(INDEX)));
  // Triangles must be indexed at offsets 0, 1, 2.
  // Quads must be successive triangles indexed at offsets 0, 1, 2, 2, 1, 3.
  if (count == 6 && indices[1] == indices[0] + 1 &&
      indices[2] == indices[0] + 2 && indices[5] == indices[0] + 3) {
    assert(indices[3] == indices[0] + 2 && indices[4] == indices[0] + 1);
    // Fast path - since there is only a single quad, we only load per-vertex
    // attribs once for all instances, as they won't change across instances
    // or within an instance.
    vertex_shader->load_attribs(v.attribs, indices[0], 0, 4);
    draw_quad(4, colortex, layer, depthtex);
    for (GLsizei instance = 1; instance < instancecount; instance++) {
      vertex_shader->load_attribs(v.attribs, indices[0], instance, 0);
      draw_quad(4, colortex, layer, depthtex);
    }
  } else {
    for (GLsizei instance = 0; instance < instancecount; instance++) {
      for (GLsizei i = 0; i + 3 <= count; i += 3) {
        if (indices[i + 1] != indices[i] + 1 ||
            indices[i + 2] != indices[i] + 2) {
          continue;
        }
        int nump = 3;
        if (i + 6 <= count && indices[i + 5] == indices[i] + 3) {
          assert(indices[i + 3] == indices[i] + 2 &&
                 indices[i + 4] == indices[i] + 1);
          nump = 4;
          i += 3;
        }
        vertex_shader->load_attribs(v.attribs, indices[i], instance, nump);
        draw_quad(nump, colortex, layer, depthtex);
      }
    }
  }
}

extern "C" {

void DrawElementsInstanced(GLenum mode, GLsizei count, GLenum type,
                           void* indicesptr, GLsizei instancecount) {
  assert(mode == GL_TRIANGLES);
  assert(type == GL_UNSIGNED_SHORT || type == GL_UNSIGNED_INT);
  if (count <= 0 || instancecount <= 0) {
    return;
  }

  Framebuffer& fb = *get_framebuffer(GL_DRAW_FRAMEBUFFER);
  Texture& colortex = ctx->textures[fb.color_attachment];
  if (!colortex.buf) {
    return;
  }
  assert(colortex.internal_format == GL_RGBA8 ||
         colortex.internal_format == GL_R8);
  Texture& depthtex = ctx->textures[ctx->depthtest ? fb.depth_attachment : 0];
  if (depthtex.buf) {
    assert(depthtex.internal_format == GL_DEPTH_COMPONENT16);
    assert(colortex.width == depthtex.width &&
           colortex.height == depthtex.height);
  }

  Buffer& indices_buf = ctx->buffers[ctx->element_array_buffer_binding];
  size_t offset = (size_t)indicesptr;
  if (!indices_buf.buf || offset >= indices_buf.size) {
    return;
  }

  // debugf("current_vertex_array %d\n", ctx->current_vertex_array);
  // debugf("indices size: %d\n", indices_buf.size);
  VertexArray& v = ctx->vertex_arrays[ctx->current_vertex_array];
  if (ctx->validate_vertex_array) {
    ctx->validate_vertex_array = false;
    v.validate();
  }

#ifndef NDEBUG
  // uint64_t start = get_time_value();
#endif

  ctx->shaded_rows = 0;
  ctx->shaded_pixels = 0;

  vertex_shader->init_batch();

  if (type == GL_UNSIGNED_SHORT) {
    draw_elements<uint16_t>(count, instancecount, indices_buf, offset, v,
                            colortex, fb.layer, depthtex);
  } else if (type == GL_UNSIGNED_INT) {
    draw_elements<uint32_t>(count, instancecount, indices_buf, offset, v,
                            colortex, fb.layer, depthtex);
  } else {
    assert(false);
  }

  if (ctx->samples_passed_query) {
    Query& q = ctx->queries[ctx->samples_passed_query];
    q.value += ctx->shaded_pixels;
  }

#ifndef NDEBUG
  // uint64_t end = get_time_value();
  // debugf("draw(%d): %fms for %d pixels in %d rows (avg %f pixels/row, %f
  // ns/pixel)\n", instancecount, double(end - start)/(1000.*1000.),
  // ctx->shaded_pixels, ctx->shaded_rows,
  // double(ctx->shaded_pixels)/ctx->shaded_rows, double(end -
  // start)/max(ctx->shaded_pixels, 1));
#endif
}

} // extern "C"

template <typename P>
static inline void scale_row(P* dst, int dstWidth, const P* src, int srcWidth,
                             int span) {
  int frac = 0;
  for (P* end = dst + span; dst < end; dst++) {
    *dst = *src;
    // Step source according to width ratio.
    for (frac += srcWidth; frac >= dstWidth; frac -= dstWidth) {
      src++;
    }
  }
}

static void scale_blit(Texture& srctex, const IntRect& srcReq, int srcZ,
                       Texture& dsttex, const IntRect& dstReq, int dstZ,
                       bool invertY) {
  // Cache scaling ratios
  int srcWidth = srcReq.width();
  int srcHeight = srcReq.height();
  int dstWidth = dstReq.width();
  int dstHeight = dstReq.height();
  // Compute valid dest bounds
  IntRect dstBounds = dsttex.sample_bounds(dstReq, invertY);
  // Compute valid source bounds
  // Scale source to dest, rounding inward to avoid sampling outside source
  IntRect srcBounds = srctex.sample_bounds(srcReq)
    .scale(srcWidth, srcHeight, dstWidth, dstHeight, true);
  // Limit dest sampling bounds to overlap source bounds
  dstBounds.intersect(srcBounds);
  // Check if sampling bounds are empty
  if (dstBounds.is_empty()) {
    return;
  }
  // Compute final source bounds from clamped dest sampling bounds
  srcBounds = IntRect(dstBounds)
    .scale(dstWidth, dstHeight, srcWidth, srcHeight);
  // Calculate source and dest pointers from clamped offsets
  int bpp = srctex.bpp();
  int srcStride = srctex.stride(bpp);
  int destStride = dsttex.stride(bpp);
  char* dest = dsttex.sample_ptr(dstReq, dstBounds, dstZ, invertY);
  char* src = srctex.sample_ptr(srcReq, srcBounds, srcZ);
  // Inverted Y must step downward along dest rows
  if (invertY) {
    destStride = -destStride;
  }
  int span = dstBounds.width();
  int frac = 0;
  for (int rows = dstBounds.height(); rows > 0; rows--) {
    if (srcWidth == dstWidth) {
      // No scaling, so just do a fast copy.
      memcpy(dest, src, span * bpp);
    } else {
      // Do scaling with different source and dest widths.
      switch (bpp) {
        case 1:
          scale_row((uint8_t*)dest, dstWidth, (uint8_t*)src, srcWidth, span);
          break;
        case 2:
          scale_row((uint16_t*)dest, dstWidth, (uint16_t*)src, srcWidth, span);
          break;
        case 4:
          scale_row((uint32_t*)dest, dstWidth, (uint32_t*)src, srcWidth, span);
          break;
        default:
          assert(false);
          break;
      }
    }
    dest += destStride;
    // Step source according to height ratio.
    for (frac += srcHeight; frac >= dstHeight; frac -= dstHeight) {
      src += srcStride;
    }
  }
}

static void linear_row(uint32_t* dest, int span, const vec2_scalar& srcUV,
                       float srcDU, int srcZOffset, sampler2DArray sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    auto srcpx = textureLinearPackedRGBA8(sampler, ivec2(uv), srcZOffset);
    unaligned_store(dest, srcpx);
    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    auto srcpx = textureLinearPackedRGBA8(sampler, ivec2(uv), srcZOffset);
    auto mask = span_mask_RGBA8(span);
    auto dstpx = unaligned_load<PackedRGBA8>(dest);
    unaligned_store(dest, (mask & dstpx) | (~mask & srcpx));
  }
}

static void linear_row(uint8_t* dest, int span, const vec2_scalar& srcUV,
                       float srcDU, int srcZOffset, sampler2DArray sampler) {
  vec2 uv = init_interp(srcUV, vec2_scalar(srcDU, 0.0f));
  for (; span >= 4; span -= 4) {
    auto srcpx = textureLinearPackedR8(sampler, ivec2(uv), srcZOffset);
    unaligned_store(dest, pack(srcpx));
    dest += 4;
    uv.x += 4 * srcDU;
  }
  if (span > 0) {
    auto srcpx = textureLinearPackedR8(sampler, ivec2(uv), srcZOffset);
    auto mask = span_mask_R8(span);
    auto dstpx = unpack(unaligned_load<PackedR8>(dest));
    unaligned_store(dest, pack((mask & dstpx) | (~mask & srcpx)));
  }
}

static void linear_blit(Texture& srctex, const IntRect& srcReq, int srcZ,
                        Texture& dsttex, const IntRect& dstReq, int dstZ,
                        bool invertY) {
  assert(srctex.internal_format == GL_RGBA8 ||
         srctex.internal_format == GL_R8);
  // Compute valid dest bounds
  IntRect dstBounds = dsttex.sample_bounds(dstReq, invertY);
  // Check if sampling bounds are empty
  if (dstBounds.is_empty()) {
    return;
  }
  // Initialize sampler for source texture
  sampler2DArray_impl sampler;
  init_sampler(&sampler, srctex);
  init_depth(&sampler, srctex);
  sampler.filter = TextureFilter::LINEAR;
  // Compute source UVs
  int srcZOffset = srcZ * sampler.height_stride;
  vec2_scalar srcUV(srcReq.x0, srcReq.y0);
  vec2_scalar srcDUV(float(srcReq.width()) / dstReq.width(),
                     float(srcReq.height()) / dstReq.height());
  // Skip to clamped source start
  srcUV += srcDUV * vec2_scalar(dstBounds.x0, dstBounds.y0);
  // Offset source UVs to texel centers and scale by lerp precision
  srcUV = linearQuantize(srcUV + 0.5f, 128);
  srcDUV *= 128.0f;
  // Calculate dest pointer from clamped offsets
  int bpp = dsttex.bpp();
  int destStride = dsttex.stride(bpp);
  char* dest = dsttex.sample_ptr(dstReq, dstBounds, dstZ, invertY);
  // Inverted Y must step downward along dest rows
  if (invertY) {
    destStride = -destStride;
  }
  int span = dstBounds.width();
  for (int rows = dstBounds.height(); rows > 0; rows--) {
    switch (bpp) {
      case 1:
        linear_row((uint8_t*)dest, span, srcUV, srcDUV.x, srcZOffset,
                   &sampler);
        break;
      case 4:
        linear_row((uint32_t*)dest, span, srcUV, srcDUV.x, srcZOffset,
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
  if (!srcfb || srcfb->layer < 0) return;
  Framebuffer* dstfb = get_framebuffer(GL_DRAW_FRAMEBUFFER);
  if (!dstfb || dstfb->layer < 0) return;
  Texture& srctex = ctx->textures[srcfb->color_attachment];
  if (!srctex.buf || srcfb->layer >= max(srctex.depth, 1)) return;
  Texture& dsttex = ctx->textures[dstfb->color_attachment];
  if (!dsttex.buf || dstfb->layer >= max(dsttex.depth, 1)) return;
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
  IntRect srcReq = {srcX0, srcY0, srcX1, srcY1};
  IntRect dstReq = {dstX0, dstY0, dstX1, dstY1};
  if (srcReq.is_empty() || dstReq.is_empty()) {
    return;
  }
  prepare_texture(srctex);
  prepare_texture(dsttex, &dstReq);
  if (!srcReq.same_size(dstReq) && filter == GL_LINEAR &&
      (srctex.internal_format == GL_RGBA8 ||
       srctex.internal_format == GL_R8)) {
    linear_blit(srctex, srcReq, srcfb->layer, dsttex, dstReq, dstfb->layer,
                invertY);
  } else {
    scale_blit(srctex, srcReq, srcfb->layer, dsttex, dstReq, dstfb->layer,
               invertY);
  }
}

void Finish() {}

void MakeCurrent(void* ctx_ptr) {
  ctx = (Context*)ctx_ptr;
  if (ctx) {
    setup_program(ctx->current_program);
    blend_key = ctx->blend ? ctx->blend_key : BLEND_KEY_NONE;
  } else {
    setup_program(0);
    blend_key = BLEND_KEY_NONE;
  }
}

void* CreateContext() { return new Context; }

void DestroyContext(void* ctx_ptr) {
  if (!ctx_ptr) {
    return;
  }
  if (ctx == ctx_ptr) {
    MakeCurrent(nullptr);
  }
  delete (Context*)ctx_ptr;
}

void Composite(GLuint srcId, GLint srcX, GLint srcY, GLsizei srcWidth,
               GLsizei srcHeight, GLint dstX, GLint dstY, GLboolean opaque,
               GLboolean flip) {
  Framebuffer& fb = ctx->framebuffers[0];
  if (!fb.color_attachment) {
    return;
  }
  Texture& srctex = ctx->textures[srcId];
  if (!srctex.buf) return;
  prepare_texture(srctex);
  Texture& dsttex = ctx->textures[fb.color_attachment];
  if (!dsttex.buf) return;
  assert(srctex.bpp() == 4);
  const int bpp = 4;
  size_t src_stride = srctex.stride(bpp);
  size_t dest_stride = dsttex.stride(bpp);
  if (srcY < 0) {
    dstY -= srcY;
    srcHeight += srcY;
    srcY = 0;
  }
  if (dstY < 0) {
    srcY -= dstY;
    srcHeight += dstY;
    dstY = 0;
  }
  if (srcY + srcHeight > srctex.height) {
    srcHeight = srctex.height - srcY;
  }
  if (dstY + srcHeight > dsttex.height) {
    srcHeight = dsttex.height - dstY;
  }
  IntRect skip = {dstX, dstY, dstX + srcWidth, dstY + srcHeight};
  prepare_texture(dsttex, &skip);
  char* dest = dsttex.sample_ptr(dstX, flip ? dsttex.height - 1 - dstY : dstY,
                                 fb.layer, bpp, dest_stride);
  char* src = srctex.sample_ptr(srcX, srcY, 0, bpp, src_stride);
  if (flip) {
    dest_stride = -dest_stride;
  }
  if (opaque) {
    for (int y = 0; y < srcHeight; y++) {
      memcpy(dest, src, srcWidth * bpp);
      dest += dest_stride;
      src += src_stride;
    }
  } else {
    for (int y = 0; y < srcHeight; y++) {
      char* end = src + srcWidth * bpp;
      while (src + 4 * bpp <= end) {
        WideRGBA8 srcpx = unpack(unaligned_load<PackedRGBA8>(src));
        WideRGBA8 dstpx = unpack(unaligned_load<PackedRGBA8>(dest));
        PackedRGBA8 r = pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx)));
        unaligned_store(dest, r);
        src += 4 * bpp;
        dest += 4 * bpp;
      }
      if (src < end) {
        WideRGBA8 srcpx = unpack(unaligned_load<PackedRGBA8>(src));
        WideRGBA8 dstpx = unpack(unaligned_load<PackedRGBA8>(dest));
        U32 r = bit_cast<U32>(
            pack(srcpx + dstpx - muldiv255(dstpx, alphas(srcpx))));
        unaligned_store(dest, r.x);
        if (src + bpp < end) {
          unaligned_store(dest + bpp, r.y);
          if (src + 2 * bpp < end) {
            unaligned_store(dest + 2 * bpp, r.z);
          }
        }
        dest += end - src;
        src = end;
      }
      dest += dest_stride - srcWidth * bpp;
      src += src_stride - srcWidth * bpp;
    }
  }
}

}  // extern "C"
