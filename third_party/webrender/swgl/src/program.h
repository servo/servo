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
};

typedef ProgramImpl* (*ProgramLoader)();

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

  vec4 gl_Position;

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

struct FragmentShaderImpl {
  typedef void (*InitSpanFunc)(FragmentShaderImpl*, const void* interps,
                               const void* step, float step_width);
  typedef void (*RunFunc)(FragmentShaderImpl*);
  typedef void (*SkipFunc)(FragmentShaderImpl*, int chunks);
  typedef void (*InitSpanWFunc)(FragmentShaderImpl*, const void* interps,
                                const void* step, float step_width);
  typedef void (*RunWFunc)(FragmentShaderImpl*);
  typedef void (*SkipWFunc)(FragmentShaderImpl*, int chunks);
  typedef void (*DrawSpanRGBA8Func)(FragmentShaderImpl*, uint32_t* buf,
                                    int len);
  typedef void (*DrawSpanR8Func)(FragmentShaderImpl*, uint8_t* buf, int len);

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
  vec2_scalar stepZW;
  Bool isPixelDiscarded = false;
  vec4 gl_FragColor;
  vec4 gl_SecondaryFragColor;

  ALWAYS_INLINE void step_fragcoord() { gl_FragCoord.x += 4; }

  ALWAYS_INLINE void step_fragcoord(int chunks) {
    gl_FragCoord.x += 4 * chunks;
  }

  ALWAYS_INLINE void step_perspective() {
    gl_FragCoord.z += stepZW.x;
    gl_FragCoord.w += stepZW.y;
  }

  ALWAYS_INLINE void step_perspective(int chunks) {
    gl_FragCoord.z += stepZW.x * chunks;
    gl_FragCoord.w += stepZW.y * chunks;
  }

  template <bool W = false>
  ALWAYS_INLINE void init_span(const void* interps, const void* step,
                               float step_width) {
    (*(W ? init_span_w_func : init_span_func))(this, interps, step, step_width);
  }

  template <bool W = false>
  ALWAYS_INLINE void run() {
    (*(W ? run_w_func : run_func))(this);
  }

  template <bool W = false>
  ALWAYS_INLINE void skip(int chunks = 1) {
    (*(W ? skip_w_func : skip_func))(this, chunks);
  }

  ALWAYS_INLINE void draw_span(uint32_t* buf, int len) {
    (*draw_span_RGBA8_func)(this, buf, len);
  }

  ALWAYS_INLINE bool has_draw_span(uint32_t*) {
    return draw_span_RGBA8_func != nullptr;
  }

  ALWAYS_INLINE void draw_span(uint8_t* buf, int len) {
    (*draw_span_R8_func)(this, buf, len);
  }

  ALWAYS_INLINE bool has_draw_span(uint8_t*) {
    return draw_span_R8_func != nullptr;
  }
};

}  // namespace glsl
