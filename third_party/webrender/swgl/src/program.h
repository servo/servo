/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct VertexAttrib;

namespace glsl {

// Type holding group of scalars interpolated across rasterized rows and spans,
// shuttling values between vertex shaders and fragment shaders.
// GCC requires power-of-two vector sizes, so must use glsl type as workaround
// to operate in Float-sized chunks.
typedef vec3 Interpolants;

// Clip distances, if enabled, are always stored in the first SIMD chunk of the
// interpolants.
static ALWAYS_INLINE Float get_clip_distances(const Interpolants& interp) {
  return interp.x;
}

struct VertexShaderImpl;
struct FragmentShaderImpl;

struct ProgramImpl {
  virtual ~ProgramImpl() {}
  virtual int get_uniform(const char* name) const = 0;
  virtual void bind_attrib(const char* name, int index) = 0;
  virtual int get_attrib(const char* name) const = 0;
  virtual size_t interpolants_size() const = 0;
  virtual VertexShaderImpl* get_vertex_shader() = 0;
  virtual FragmentShaderImpl* get_fragment_shader() = 0;
  virtual const char* get_name() const = 0;
};

typedef ProgramImpl* (*ProgramLoader)();

// The maximum size of the gl_ClipDistance array.
constexpr int32_t gl_MaxClipDistances = 4;

struct VertexShaderImpl {
  typedef void (*SetUniform1iFunc)(VertexShaderImpl*, int index, int value);
  typedef void (*SetUniform4fvFunc)(VertexShaderImpl*, int index,
                                    const float* value);
  typedef void (*SetUniformMatrix4fvFunc)(VertexShaderImpl*, int index,
                                          const float* value);
  typedef void (*InitBatchFunc)(VertexShaderImpl*);
  typedef void (*LoadAttribsFunc)(VertexShaderImpl*, VertexAttrib* attribs,
                                  uint32_t start, int instance, int count);
  typedef void (*RunPrimitiveFunc)(VertexShaderImpl*, char* interps,
                                   size_t interp_stride);

  SetUniform1iFunc set_uniform_1i_func = nullptr;
  SetUniform4fvFunc set_uniform_4fv_func = nullptr;
  SetUniformMatrix4fvFunc set_uniform_matrix4fv_func = nullptr;
  InitBatchFunc init_batch_func = nullptr;
  LoadAttribsFunc load_attribs_func = nullptr;
  RunPrimitiveFunc run_primitive_func = nullptr;

  enum FLAGS {
    CLIP_DISTANCE = 1 << 0,
  };
  int flags = 0;
  void enable_clip_distance() { flags |= CLIP_DISTANCE; }
  ALWAYS_INLINE bool use_clip_distance() const {
    return (flags & CLIP_DISTANCE) != 0;
  }

  vec4 gl_Position;
  Float gl_ClipDistance[gl_MaxClipDistances];

  void set_uniform_1i(int index, int value) {
    (*set_uniform_1i_func)(this, index, value);
  }

  void set_uniform_4fv(int index, const float* value) {
    (*set_uniform_4fv_func)(this, index, value);
  }

  void set_uniform_matrix4fv(int index, const float* value) {
    (*set_uniform_matrix4fv_func)(this, index, value);
  }

  void init_batch() { (*init_batch_func)(this); }

  ALWAYS_INLINE void load_attribs(VertexAttrib* attribs, uint32_t start,
                                  int instance, int count) {
    (*load_attribs_func)(this, attribs, start, instance, count);
  }

  ALWAYS_INLINE void run_primitive(char* interps, size_t interp_stride) {
    (*run_primitive_func)(this, interps, interp_stride);
  }
};

// The number of pixels in a step.
constexpr int32_t swgl_StepSize = 4;

struct FragmentShaderImpl {
  typedef void (*InitSpanFunc)(FragmentShaderImpl*, const void* interps,
                               const void* step);
  typedef void (*RunFunc)(FragmentShaderImpl*);
  typedef void (*SkipFunc)(FragmentShaderImpl*, int steps);
  typedef void (*InitSpanWFunc)(FragmentShaderImpl*, const void* interps,
                                const void* step);
  typedef void (*RunWFunc)(FragmentShaderImpl*);
  typedef void (*SkipWFunc)(FragmentShaderImpl*, int steps);
  typedef int (*DrawSpanRGBA8Func)(FragmentShaderImpl*);
  typedef int (*DrawSpanR8Func)(FragmentShaderImpl*);

  InitSpanFunc init_span_func = nullptr;
  RunFunc run_func = nullptr;
  SkipFunc skip_func = nullptr;
  InitSpanWFunc init_span_w_func = nullptr;
  RunWFunc run_w_func = nullptr;
  SkipWFunc skip_w_func = nullptr;
  DrawSpanRGBA8Func draw_span_RGBA8_func = nullptr;
  DrawSpanR8Func draw_span_R8_func = nullptr;

  enum FLAGS {
    DISCARD = 1 << 0,
    PERSPECTIVE = 1 << 1,
  };
  int flags = 0;
  void enable_discard() { flags |= DISCARD; }
  void enable_perspective() { flags |= PERSPECTIVE; }
  ALWAYS_INLINE bool use_discard() const { return (flags & DISCARD) != 0; }
  ALWAYS_INLINE bool use_perspective() const {
    return (flags & PERSPECTIVE) != 0;
  }

  vec4 gl_FragCoord;
  vec4 gl_FragColor;
  vec4 gl_SecondaryFragColor;

  vec2_scalar swgl_StepZW;
  Bool swgl_IsPixelDiscarded = false;
  // The current buffer position for committing span output.
  uint32_t* swgl_OutRGBA8 = nullptr;
  uint8_t* swgl_OutR8 = nullptr;
  // The remaining number of pixels in the span.
  int32_t swgl_SpanLength = 0;

  ALWAYS_INLINE void step_fragcoord(int steps = 4) { gl_FragCoord.x += steps; }

  ALWAYS_INLINE void step_perspective(int steps = 4) {
    gl_FragCoord.z += swgl_StepZW.x * steps;
    gl_FragCoord.w += swgl_StepZW.y * steps;
  }

  template <bool W = false>
  ALWAYS_INLINE void init_span(const void* interps, const void* step) {
    (*(W ? init_span_w_func : init_span_func))(this, interps, step);
  }

  template <bool W = false>
  ALWAYS_INLINE void run() {
    (*(W ? run_w_func : run_func))(this);
  }

  template <bool W = false>
  ALWAYS_INLINE void skip(int steps = 4) {
    (*(W ? skip_w_func : skip_func))(this, steps);
  }

  ALWAYS_INLINE int draw_span(uint32_t* buf, int len) {
    swgl_OutRGBA8 = buf;
    swgl_SpanLength = len;
    return (*draw_span_RGBA8_func)(this);
  }

  ALWAYS_INLINE bool has_draw_span(uint32_t*) {
    return draw_span_RGBA8_func != nullptr;
  }

  ALWAYS_INLINE int draw_span(uint8_t* buf, int len) {
    swgl_OutR8 = buf;
    swgl_SpanLength = len;
    return (*draw_span_R8_func)(this);
  }

  ALWAYS_INLINE bool has_draw_span(uint8_t*) {
    return draw_span_R8_func != nullptr;
  }
};

}  // namespace glsl
