/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define SI ALWAYS_INLINE static

#include "vector_type.h"

namespace glsl {

enum TextureFormat { RGBA32F, RGBA32I, RGBA8, R8, RG8, R16, YUV422 };

enum TextureFilter { NEAREST, LINEAR };

struct samplerCommon {
  uint32_t* buf = nullptr;
  uint32_t stride = 0;  // in units of BPP if < 4, or dwords if BPP >= 4
  uint32_t height = 0;
  uint32_t width = 0;
  TextureFormat format = TextureFormat::RGBA8;
};

struct samplerFilter {
  TextureFilter filter = TextureFilter::NEAREST;
};

struct sampler2D_impl : samplerCommon, samplerFilter {};
typedef sampler2D_impl* sampler2D;

typedef struct sampler2DR8_impl : sampler2D_impl{} * sampler2DR8;
typedef struct sampler2DRG8_impl : sampler2D_impl{} * sampler2DRG8;
typedef struct sampler2DRGBA8_impl : sampler2D_impl{} * sampler2DRGBA8;
typedef struct sampler2DRGBA32F_impl : sampler2D_impl{} * sampler2DRGBA32F;

struct isampler2D_impl : samplerCommon {};
typedef isampler2D_impl* isampler2D;

struct isampler2DRGBA32I_impl : isampler2D_impl {};
typedef isampler2DRGBA32I_impl* isampler2DRGBA32I;

struct sampler2DRect_impl : samplerCommon, samplerFilter {};
typedef sampler2DRect_impl* sampler2DRect;

#if USE_SSE2
SI bool test_all(Bool cond) { return _mm_movemask_ps(cond) == 0xF; }
SI bool test_any(Bool cond) { return _mm_movemask_ps(cond) != 0; }
SI bool test_none(Bool cond) { return _mm_movemask_ps(cond) == 0; }
#else
SI bool test_all(Bool cond) {
  return bit_cast<uint32_t>(CONVERT(cond, U8)) == 0xFFFFFFFFU;
}
SI bool test_any(Bool cond) {
  return bit_cast<uint32_t>(CONVERT(cond, U8)) != 0;
}
SI bool test_none(Bool cond) {
  return bit_cast<uint32_t>(CONVERT(cond, U8)) == 0;
}
#endif
SI bool test_equal(Bool cond) { return test_none(cond != cond.x); }

float make_float(float n) { return n; }

float make_float(int32_t n) { return float(n); }

float make_float(uint32_t n) { return float(n); }

float make_float(bool n) { return float(n); }

template <typename T>
Float make_float(T v) {
  return CONVERT(v, Float);
}

int32_t make_int(uint32_t n) { return n; }

int32_t make_int(int32_t n) { return n; }

int32_t make_int(float n) { return int32_t(n); }

int32_t make_int(bool n) { return int32_t(n); }

template <typename T>
I32 make_int(T v) {
  return CONVERT(v, I32);
}

uint32_t make_uint(uint32_t n) { return n; }

uint32_t make_uint(int32_t n) { return n; }

uint32_t make_uint(float n) { return uint32_t(n); }

uint32_t make_uint(bool n) { return uint32_t(n); }

template <typename T>
U32 make_uint(T v) {
  return CONVERT(v, U32);
}

template <typename T>
T force_scalar(T n) {
  return n;
}

float force_scalar(Float f) { return f[0]; }

int32_t force_scalar(I32 i) { return i[0]; }

struct vec4;
struct ivec2;

SI int32_t if_then_else(int32_t c, int32_t t, int32_t e) { return c ? t : e; }
SI int32_t if_then_else(bool c, int32_t t, int32_t e) { return c ? t : e; }

SI float if_then_else(int32_t c, float t, float e) { return c ? t : e; }

SI Float if_then_else(I32 c, float t, float e) {
  return bit_cast<Float>((c & bit_cast<I32>(Float(t))) |
                         (~c & bit_cast<I32>(Float(e))));
}

SI I32 if_then_else(I32 c, int32_t t, int32_t e) {
  return (c & I32(t)) | (~c & I32(e));
}

SI U32 if_then_else(I32 c, U32 t, U32 e) {
  return bit_cast<U32>((c & bit_cast<I32>(t)) | (~c & bit_cast<I32>(e)));
}

SI Float if_then_else(I32 c, Float t, Float e) {
  return bit_cast<Float>((c & bit_cast<I32>(t)) | (~c & bit_cast<I32>(e)));
}

SI Float if_then_else(int32_t c, Float t, Float e) { return c ? t : e; }

SI Bool if_then_else(I32 c, Bool t, Bool e) { return (c & t) | (~c & e); }

SI Bool if_then_else(int32_t c, Bool t, Bool e) { return c ? t : e; }

SI I16 if_then_else(I16 c, I16 t, I16 e) { return (c & t) | (~c & e); }

template <typename T>
SI void swap(T& a, T& b) {
  T t(a);
  a = b;
  b = t;
}

SI int32_t min(int32_t a, int32_t b) { return a < b ? a : b; }
SI int32_t max(int32_t a, int32_t b) { return a > b ? a : b; }

SI int32_t clamp(int32_t a, int32_t minVal, int32_t maxVal) {
  return min(max(a, minVal), maxVal);
}

SI float min(float a, float b) { return a < b ? a : b; }
SI float max(float a, float b) { return a > b ? a : b; }

SI float clamp(float a, float minVal, float maxVal) {
  return min(max(a, minVal), maxVal);
}

SI Float min(Float a, Float b) {
#if USE_SSE2
  return _mm_min_ps(a, b);
#elif USE_NEON
  return vminq_f32(a, b);
#else
  return if_then_else(a < b, a, b);
#endif
}

SI Float max(Float a, Float b) {
#if USE_SSE2
  return _mm_max_ps(a, b);
#elif USE_NEON
  return vmaxq_f32(a, b);
#else
  return if_then_else(a > b, a, b);
#endif
}

SI Float clamp(Float a, Float minVal, Float maxVal) {
  return min(max(a, minVal), maxVal);
}

#define sqrt __glsl_sqrt

SI float sqrt(float x) { return sqrtf(x); }

SI Float sqrt(Float v) {
#if USE_SSE2
  return _mm_sqrt_ps(v);
#elif USE_NEON
  Float e = vrsqrteq_f32(v);
  e *= vrsqrtsq_f32(v, e * e);
  e *= vrsqrtsq_f32(v, e * e);
  return v * e;
#else
  return (Float){sqrtf(v.x), sqrtf(v.y), sqrtf(v.z), sqrtf(v.w)};
#endif
}

SI float recip(float x) {
#if USE_SSE2
  return _mm_cvtss_f32(_mm_rcp_ss(_mm_set_ss(x)));
#else
  return 1.0f / x;
#endif
}

// Use a fast vector reciprocal approximation when available. This should only
// be used in cases where it is okay that the approximation is imprecise -
// essentially visually correct but numerically wrong. Otherwise just rely on
// however the compiler would implement slower division if the platform doesn't
// provide a convenient intrinsic.
SI Float recip(Float v) {
#if USE_SSE2
  return _mm_rcp_ps(v);
#elif USE_NEON
  Float e = vrecpeq_f32(v);
  return vrecpsq_f32(v, e) * e;
#else
  return 1.0f / v;
#endif
}

SI float inversesqrt(float x) {
#if USE_SSE2
  return _mm_cvtss_f32(_mm_rsqrt_ss(_mm_set_ss(x)));
#else
  return 1.0f / sqrtf(x);
#endif
}

SI Float inversesqrt(Float v) {
#if USE_SSE2
  return _mm_rsqrt_ps(v);
#elif USE_NEON
  Float e = vrsqrteq_f32(v);
  return vrsqrtsq_f32(v, e * e) * e;
#else
  return 1.0f / sqrt(v);
#endif
}

SI float step(float edge, float x) { return float(x >= edge); }

SI Float step(Float edge, Float x) {
  return if_then_else(x < edge, Float(0), Float(1));
}

/*
enum RGBA {
        R,
        G,
        B,
        A
};*/

enum XYZW {
  X = 0,
  Y = 1,
  Z = 2,
  W = 3,
  R = 0,
  G = 1,
  B = 2,
  A = 3,
};

struct bvec4_scalar;

struct bvec2_scalar {
  bool x;
  bool y;

  bvec2_scalar() : bvec2_scalar(false) {}
  IMPLICIT constexpr bvec2_scalar(bool a) : x(a), y(a) {}
  constexpr bvec2_scalar(bool x, bool y) : x(x), y(y) {}

  bool& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  bool sel(XYZW c1) { return select(c1); }

  bvec2_scalar sel(XYZW c1, XYZW c2) {
    return bvec2_scalar(select(c1), select(c2));
  }
  bvec4_scalar sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4);
};

struct bvec2_scalar1 {
  bool x;

  IMPLICIT constexpr bvec2_scalar1(bool a) : x(a) {}

  operator bvec2_scalar() const { return bvec2_scalar(x); }
};

struct bvec2 {
  bvec2() : bvec2(0) {}
  IMPLICIT bvec2(Bool a) : x(a), y(a) {}
  bvec2(Bool x, Bool y) : x(x), y(y) {}
  Bool& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  Bool sel(XYZW c1) { return select(c1); }

  bvec2 sel(XYZW c1, XYZW c2) { return bvec2(select(c1), select(c2)); }

  bvec2 operator~() { return bvec2(~x, ~y); }

  Bool x;
  Bool y;
};

bvec2_scalar1 make_bvec2(bool n) { return bvec2_scalar1(n); }

bvec2_scalar make_bvec2(bool x, bool y) { return bvec2_scalar{x, y}; }

template <typename N>
bvec2 make_bvec2(const N& n) {
  return bvec2(n);
}

template <typename X, typename Y>
bvec2 make_bvec2(const X& x, const Y& y) {
  return bvec2(x, y);
}

struct vec4_scalar;

struct vec2_scalar {
  typedef struct vec2 vector_type;
  typedef float element_type;

  float x;
  float y;

  constexpr vec2_scalar() : vec2_scalar(0.0f) {}
  IMPLICIT constexpr vec2_scalar(float a) : x(a), y(a) {}
  IMPLICIT constexpr vec2_scalar(int a) : x(a), y(a) {}
  constexpr vec2_scalar(float x, float y) : x(x), y(y) {}

  float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  float& sel(XYZW c1) { return select(c1); }
  vec2_scalar sel(XYZW c1, XYZW c2) {
    return vec2_scalar(select(c1), select(c2));
  }
  vec4_scalar sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4);

  friend bool operator==(const vec2_scalar& l, const vec2_scalar& r) {
    return l.x == r.x && l.y == r.y;
  }

  friend bool operator!=(const vec2_scalar& l, const vec2_scalar& r) {
    return l.x != r.x || l.y != r.y;
  }

  friend vec2_scalar operator*(float a, vec2_scalar b) {
    return vec2_scalar(a * b.x, a * b.y);
  }
  friend vec2_scalar operator*(vec2_scalar a, float b) {
    return vec2_scalar(a.x * b, a.y * b);
  }
  friend vec2_scalar operator*(vec2_scalar a, vec2_scalar b) {
    return vec2_scalar(a.x * b.x, a.y * b.y);
  }
  friend vec2_scalar operator/(vec2_scalar a, float b) {
    return vec2_scalar(a.x / b, a.y / b);
  }
  friend vec2_scalar operator/(vec2_scalar a, vec2_scalar b) {
    return vec2_scalar(a.x / b.x, a.y / b.y);
  }

  friend vec2_scalar operator-(vec2_scalar a, vec2_scalar b) {
    return vec2_scalar(a.x - b.x, a.y - b.y);
  }
  friend vec2_scalar operator+(vec2_scalar a, vec2_scalar b) {
    return vec2_scalar(a.x + b.x, a.y + b.y);
  }
  friend vec2_scalar operator+(vec2_scalar a, float b) {
    return vec2_scalar(a.x + b, a.y + b);
  }

  vec2_scalar operator-() { return vec2_scalar(-x, -y); }

  vec2_scalar operator*=(vec2_scalar a) {
    x *= a.x;
    y *= a.y;
    return *this;
  }

  vec2_scalar operator/=(vec2_scalar a) {
    x /= a.x;
    y /= a.y;
    return *this;
  }

  vec2_scalar operator+=(vec2_scalar a) {
    x += a.x;
    y += a.y;
    return *this;
  }

  vec2_scalar operator-=(vec2_scalar a) {
    x -= a.x;
    y -= a.y;
    return *this;
  }
};

struct vec2_scalar_ref {
  vec2_scalar_ref(float& x, float& y) : x(x), y(y) {}
  float& x;
  float& y;

  float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  float& sel(XYZW c1) { return select(c1); }

  vec2_scalar_ref& operator=(const vec2_scalar& a) {
    x = a.x;
    y = a.y;
    return *this;
  }
  vec2_scalar_ref& operator*=(vec2_scalar a) {
    x *= a.x;
    y *= a.y;
    return *this;
  }
  operator vec2_scalar() const { return vec2_scalar{x, y}; }
};

struct vec2 {
  typedef struct vec2 vector_type;
  typedef float element_type;

  constexpr vec2() : vec2(Float(0.0f)) {}
  IMPLICIT constexpr vec2(Float a) : x(a), y(a) {}
  vec2(Float x, Float y) : x(x), y(y) {}
  IMPLICIT constexpr vec2(vec2_scalar s) : x(s.x), y(s.y) {}
  constexpr vec2(vec2_scalar s0, vec2_scalar s1, vec2_scalar s2, vec2_scalar s3)
      : x(Float{s0.x, s1.x, s2.x, s3.x}), y(Float{s0.y, s1.y, s2.y, s3.y}) {}
  explicit vec2(ivec2 a);
  Float x;
  Float y;

  Float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  Float& sel(XYZW c1) { return select(c1); }
  vec2 sel(XYZW c1, XYZW c2) { return vec2(select(c1), select(c2)); }

  vec4 sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4);

  vec2 operator*=(Float a) {
    x *= a;
    y *= a;
    return *this;
  }
  vec2 operator*=(vec2 a) {
    x *= a.x;
    y *= a.y;
    return *this;
  }

  vec2 operator/=(Float a) {
    x /= a;
    y /= a;
    return *this;
  }
  vec2 operator/=(vec2 a) {
    x /= a.x;
    y /= a.y;
    return *this;
  }

  vec2 operator+=(vec2 a) {
    x += a.x;
    y += a.y;
    return *this;
  }
  vec2 operator-=(vec2 a) {
    x -= a.x;
    y -= a.y;
    return *this;
  }
  vec2 operator-=(Float a) {
    x -= a;
    y -= a;
    return *this;
  }

  vec2 operator-() { return vec2(-x, -y); }

  friend I32 operator==(const vec2& l, const vec2& r) {
    return l.x == r.x && l.y == r.y;
  }

  friend I32 operator!=(const vec2& l, const vec2& r) {
    return l.x != r.x || l.y != r.y;
  }

  friend vec2 operator*(vec2 a, Float b) { return vec2(a.x * b, a.y * b); }
  friend vec2 operator*(vec2 a, vec2 b) { return vec2(a.x * b.x, a.y * b.y); }
  friend vec2 operator*(Float a, vec2 b) { return vec2(a * b.x, a * b.y); }

  friend vec2 operator/(vec2 a, vec2 b) { return vec2(a.x / b.x, a.y / b.y); }
  friend vec2 operator/(vec2 a, Float b) { return vec2(a.x / b, a.y / b); }

  friend vec2 operator-(vec2 a, vec2 b) { return vec2(a.x - b.x, a.y - b.y); }
  friend vec2 operator-(vec2 a, Float b) { return vec2(a.x - b, a.y - b); }
  friend vec2 operator-(Float a, vec2 b) { return vec2(a - b.x, a - b.y); }
  friend vec2 operator+(vec2 a, vec2 b) { return vec2(a.x + b.x, a.y + b.y); }
  friend vec2 operator+(vec2 a, Float b) { return vec2(a.x + b, a.y + b); }
  friend vec2 operator+(Float a, vec2 b) { return vec2(a + b.x, a + b.y); }
};

vec2_scalar force_scalar(const vec2& v) {
  return vec2_scalar{force_scalar(v.x), force_scalar(v.y)};
}

vec2_scalar make_vec2(float n) { return vec2_scalar{n, n}; }

vec2_scalar make_vec2(float x, float y) { return vec2_scalar{x, y}; }

vec2_scalar make_vec2(int32_t x, int32_t y) {
  return vec2_scalar{float(x), float(y)};
}

template <typename N>
vec2 make_vec2(const N& n) {
  return vec2(n);
}

template <typename X, typename Y>
vec2 make_vec2(const X& x, const Y& y) {
  return vec2(x, y);
}

vec2 operator*(vec2_scalar a, Float b) { return vec2(a.x * b, a.y * b); }

vec2 operator*(Float a, vec2_scalar b) { return vec2(a * b.x, a * b.y); }

SI vec2 min(vec2 a, vec2 b) { return vec2(min(a.x, b.x), min(a.y, b.y)); }
SI vec2 min(vec2 a, Float b) { return vec2(min(a.x, b), min(a.y, b)); }

SI vec2_scalar min(vec2_scalar a, vec2_scalar b) {
  return vec2_scalar{min(a.x, b.x), min(a.y, b.y)};
}

SI vec2 if_then_else(I32 c, vec2 t, vec2 e) {
  return vec2(if_then_else(c, t.x, e.x), if_then_else(c, t.y, e.y));
}

SI vec2 if_then_else(int32_t c, vec2 t, vec2 e) { return c ? t : e; }

vec2 step(vec2 edge, vec2 x) {
  return vec2(step(edge.x, x.x), step(edge.y, x.y));
}

vec2_scalar step(vec2_scalar edge, vec2_scalar x) {
  return vec2_scalar(step(edge.x, x.x), step(edge.y, x.y));
}

SI vec2 max(vec2 a, vec2 b) { return vec2(max(a.x, b.x), max(a.y, b.y)); }
SI vec2 max(vec2 a, Float b) { return vec2(max(a.x, b), max(a.y, b)); }

SI vec2_scalar max(vec2_scalar a, vec2_scalar b) {
  return vec2_scalar{max(a.x, b.x), max(a.y, b.y)};
}
SI vec2_scalar max(vec2_scalar a, float b) {
  return vec2_scalar{max(a.x, b), max(a.y, b)};
}

Float length(vec2 a) { return sqrt(a.x * a.x + a.y * a.y); }

float length(vec2_scalar a) { return hypotf(a.x, a.y); }

template <typename A, typename B>
SI auto distance(A a, B b) {
  return length(a - b);
}

template <typename T>
SI T normalize(T a) {
  return a / length(a);
}

SI vec2 sqrt(vec2 a) { return vec2(sqrt(a.x), sqrt(a.y)); }

SI vec2_scalar sqrt(vec2_scalar a) { return vec2_scalar(sqrt(a.x), sqrt(a.y)); }

SI vec2 recip(vec2 a) { return vec2(recip(a.x), recip(a.y)); }

SI vec2_scalar recip(vec2_scalar a) {
  return vec2_scalar(recip(a.x), recip(a.y));
}

SI vec2 inversesqrt(vec2 a) { return vec2(inversesqrt(a.x), inversesqrt(a.y)); }

SI vec2_scalar inversesqrt(vec2_scalar a) {
  return vec2_scalar(inversesqrt(a.x), inversesqrt(a.y));
}

#define abs __glsl_abs

int32_t abs(int32_t a) { return a < 0 ? -a : a; }

float abs(float a) { return fabsf(a); }

Float abs(Float v) {
#if USE_NEON
  return vabsq_f32(v);
#else
  return bit_cast<Float>(bit_cast<I32>(v) & bit_cast<I32>(0.0f - v));
#endif
}

float sign(float a) { return copysignf(1.0f, a); }

Float sign(Float v) {
  return bit_cast<Float>((bit_cast<I32>(v) & 0x80000000) |
                         bit_cast<I32>(Float(1.0f)));
}

Float cast(U32 v) { return CONVERT((I32)v, Float); }
Float cast(I32 v) { return CONVERT((I32)v, Float); }
I32 cast(Float v) { return CONVERT(v, I32); }

#define floor __glsl_floor

float floor(float a) { return floorf(a); }

Float floor(Float v) {
  Float roundtrip = cast(cast(v));
  return roundtrip - if_then_else(roundtrip > v, Float(1), Float(0));
}

vec2 floor(vec2 v) { return vec2(floor(v.x), floor(v.y)); }

vec2_scalar floor(vec2_scalar v) {
  return vec2_scalar{floorf(v.x), floorf(v.y)};
}

#define ceil __glsl_ceil

float ceil(float a) { return ceilf(a); }

Float ceil(Float v) {
  Float roundtrip = cast(cast(v));
  return roundtrip + if_then_else(roundtrip < v, Float(1), Float(0));
}

// Round to nearest even
SI int32_t roundeven(float v, float scale) {
#if USE_SSE2
  return _mm_cvtss_si32(_mm_set_ss(v * scale));
#else
  return bit_cast<int32_t>(v * scale + float(0xC00000)) - 0x4B400000;
#endif
}

SI I32 roundeven(Float v, Float scale) {
#if USE_SSE2
  return _mm_cvtps_epi32(v * scale);
#else
  // Magic number implementation of round-to-nearest-even
  // see http://stereopsis.com/sree/fpu2006.html
  return bit_cast<I32>(v * scale + Float(0xC00000)) - 0x4B400000;
#endif
}

// Round towards zero
SI int32_t roundzero(float v, float scale) { return int32_t(v * scale); }

SI I32 roundzero(Float v, Float scale) { return cast(v * scale); }

// Round whichever direction is fastest for positive numbers
SI I32 roundfast(Float v, Float scale) {
#if USE_SSE2
  return _mm_cvtps_epi32(v * scale);
#else
  return cast(v * scale + 0.5f);
#endif
}

template <typename T>
SI auto round_pixel(T v, float scale = 255.0f) {
  return roundfast(v, scale);
}

#define round __glsl_round

float round(float a) { return roundf(a); }

Float round(Float v) { return floor(v + 0.5f); }

float fract(float a) { return a - floor(a); }

Float fract(Float v) { return v - floor(v); }

vec2 fract(vec2 v) { return vec2(fract(v.x), fract(v.y)); }

// X derivatives can be approximated by dFdx(x) = x[1] - x[0].
// Y derivatives are not easily available since we operate in terms of X spans
// only. To work around, assume dFdy(p.x) = dFdx(p.y), which only holds for
// uniform scaling, and thus abs(dFdx(p.x)) + abs(dFdy(p.x)) = abs(dFdx(p.x)) +
// abs(dFdx(p.y)) which mirrors abs(dFdx(p.y)) + abs(dFdy(p.y)) = abs(dFdx(p.y))
// + abs(dFdx(p.x)).
vec2_scalar fwidth(vec2 p) {
  Float d = abs(SHUFFLE(p.x, p.y, 1, 1, 5, 5) - SHUFFLE(p.x, p.y, 0, 0, 4, 4));
  return vec2_scalar(d.x + d.z);
}

float dFdx(Float x) { return x.y - x.x; }

vec2_scalar dFdx(vec2 p) { return vec2_scalar(dFdx(p.x), dFdx(p.y)); }

// See
// http://www.machinedlearnings.com/2011/06/fast-approximate-logarithm-exponential.html.
Float approx_log2(Float x) {
  // e - 127 is a fair approximation of log2(x) in its own right...
  Float e = cast(bit_cast<U32>(x)) * (1.0f / (1 << 23));

  // ... but using the mantissa to refine its error is _much_ better.
  Float m = bit_cast<Float>((bit_cast<U32>(x) & 0x007fffff) | 0x3f000000);
  return e - 124.225514990f - 1.498030302f * m -
         1.725879990f / (0.3520887068f + m);
}

Float approx_pow2(Float x) {
  Float f = fract(x);
  return bit_cast<Float>(
      roundfast(1.0f * (1 << 23), x + 121.274057500f - 1.490129070f * f +
                                      27.728023300f / (4.84252568f - f)));
}

#define pow __glsl_pow

SI float pow(float x, float y) { return powf(x, y); }

Float pow(Float x, Float y) {
  return if_then_else((x == 0) | (x == 1), x, approx_pow2(approx_log2(x) * y));
}

#define exp __glsl_exp

SI float exp(float x) { return expf(x); }

Float exp(Float y) {
  float l2e = 1.4426950408889634074f;
  return approx_pow2(l2e * y);
}

#define exp2 __glsl_exp2

SI float exp2(float x) { return exp2f(x); }

Float exp2(Float x) { return approx_pow2(x); }

#define log __glsl_log

SI float log(float x) { return logf(x); }

Float log(Float x) { return approx_log2(x) * 0.69314718f; }

#define log2 __glsl_log2

SI float log2(float x) { return log2f(x); }

Float log2(Float x) { return approx_log2(x); }

struct ivec4;

struct ivec2_scalar {
  typedef int32_t element_type;

  int32_t x;
  int32_t y;

  ivec2_scalar() : ivec2_scalar(0) {}
  IMPLICIT constexpr ivec2_scalar(int32_t a) : x(a), y(a) {}
  constexpr ivec2_scalar(int32_t x, int32_t y) : x(x), y(y) {}

  int32_t& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  int32_t& sel(XYZW c1) { return select(c1); }
  ivec2_scalar sel(XYZW c1, XYZW c2) {
    return ivec2_scalar{select(c1), select(c2)};
  }

  ivec2_scalar operator-() const { return ivec2_scalar{-x, -y}; }

  ivec2_scalar& operator+=(ivec2_scalar a) {
    x += a.x;
    y += a.y;
    return *this;
  }
  ivec2_scalar& operator+=(int n) {
    x += n;
    y += n;
    return *this;
  }

  ivec2_scalar& operator>>=(int shift) {
    x >>= shift;
    y >>= shift;
    return *this;
  }

  friend ivec2_scalar operator&(ivec2_scalar a, int b) {
    return ivec2_scalar{a.x & b, a.y & b};
  }

  friend ivec2_scalar operator+(ivec2_scalar a, ivec2_scalar b) {
    return ivec2_scalar{a.x + b.x, a.y + b.y};
  }

  friend ivec2_scalar operator-(ivec2_scalar a, ivec2_scalar b) {
    return ivec2_scalar{a.x - b.x, a.y - b.y};
  }

  friend bool operator==(const ivec2_scalar& l, const ivec2_scalar& r) {
    return l.x == r.x && l.y == r.y;
  }
};

struct ivec2 {
  typedef int32_t element_type;

  ivec2() : ivec2(I32(0)) {}
  IMPLICIT ivec2(I32 a) : x(a), y(a) {}
  ivec2(I32 x, I32 y) : x(x), y(y) {}
  IMPLICIT ivec2(vec2 a) : x(cast(a.x)), y(cast(a.y)) {}
  ivec2(U32 x, U32 y) : x(CONVERT(x, I32)), y(CONVERT(y, I32)) {}
  IMPLICIT constexpr ivec2(ivec2_scalar s) : x(s.x), y(s.y) {}
  constexpr ivec2(ivec2_scalar s0, ivec2_scalar s1, ivec2_scalar s2,
                  ivec2_scalar s3)
      : x(I32{s0.x, s1.x, s2.x, s3.x}), y(I32{s0.y, s1.y, s2.y, s3.y}) {}
  I32 x;
  I32 y;

  I32& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  I32& sel(XYZW c1) { return select(c1); }

  ivec2 sel(XYZW c1, XYZW c2) { return ivec2(select(c1), select(c2)); }

  ivec4 sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4);

  ivec2& operator*=(I32 a) {
    x *= a;
    y *= a;
    return *this;
  }
  ivec2& operator+=(ivec2 a) {
    x += a.x;
    y += a.y;
    return *this;
  }
  ivec2& operator>>=(int shift) {
    x >>= shift;
    y >>= shift;
    return *this;
  }

  friend ivec2 operator*(ivec2 a, I32 b) { return ivec2(a.x * b, a.y * b); }
  friend ivec2 operator&(ivec2 a, ivec2 b) {
    return ivec2(a.x & b.x, a.y & b.y);
  }
  friend ivec2 operator&(ivec2 a, I32 b) { return ivec2(a.x & b, a.y & b); }
  friend ivec2 operator+(ivec2 a, ivec2 b) {
    return ivec2(a.x + b.x, a.y + b.y);
  }
};

vec2::vec2(ivec2 a) : x(cast(a.x)), y(cast(a.y)) {}

ivec2_scalar make_ivec2(int32_t n) { return ivec2_scalar{n, n}; }

ivec2_scalar make_ivec2(uint32_t n) {
  return ivec2_scalar{int32_t(n), int32_t(n)};
}

ivec2_scalar make_ivec2(int32_t x, int32_t y) { return ivec2_scalar{x, y}; }

ivec2_scalar make_ivec2(uint32_t x, uint32_t y) {
  return ivec2_scalar{int32_t(x), int32_t(y)};
}

vec2_scalar make_vec2(const ivec2_scalar& v) {
  return vec2_scalar{float(v.x), float(v.y)};
}

ivec2_scalar make_ivec2(const vec2_scalar& v) {
  return ivec2_scalar{int32_t(v.x), int32_t(v.y)};
}

template <typename N>
ivec2 make_ivec2(const N& n) {
  return ivec2(n);
}

template <typename X, typename Y>
ivec2 make_ivec2(const X& x, const Y& y) {
  return ivec2(x, y);
}

ivec2_scalar force_scalar(const ivec2& v) {
  return ivec2_scalar{force_scalar(v.x), force_scalar(v.y)};
}

struct ivec3_scalar {
  int32_t x;
  int32_t y;
  int32_t z;

  ivec3_scalar() : ivec3_scalar(0) {}
  IMPLICIT constexpr ivec3_scalar(int32_t a) : x(a), y(a), z(a) {}
  constexpr ivec3_scalar(int32_t x, int32_t y, int32_t z) : x(x), y(y), z(z) {}

  int32_t& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      default:
        UNREACHABLE;
    }
  }
  int32_t& sel(XYZW c1) { return select(c1); }
  ivec2_scalar sel(XYZW c1, XYZW c2) {
    return ivec2_scalar{select(c1), select(c2)};
  }
};

struct ivec3 {
  ivec3() : ivec3(0) {}
  IMPLICIT ivec3(I32 a) : x(a), y(a), z(a) {}
  ivec3(I32 x, I32 y, I32 z) : x(x), y(y), z(z) {}
  ivec3(ivec2 a, I32 b) : x(a.x), y(a.y), z(b) {}
  ivec3(vec2 a, Float b) : x(cast(a.x)), y(cast(a.y)), z(cast(b)) {}
  I32 x;
  I32 y;
  I32 z;

  friend ivec3 operator+(ivec3 a, ivec3 b) {
    return ivec3(a.x + b.x, a.y + b.y, a.z + b.z);
  }
};

vec2_scalar make_vec2(ivec3_scalar s) {
  return vec2_scalar{float(s.x), float(s.y)};
}

ivec3_scalar make_ivec3(int32_t n) { return ivec3_scalar{n, n, n}; }

ivec3_scalar make_ivec3(const ivec2_scalar& v, int32_t z) {
  return ivec3_scalar{v.x, v.y, z};
}

ivec3_scalar make_ivec3(int32_t x, int32_t y, int32_t z) {
  return ivec3_scalar{x, y, z};
}

template <typename N>
ivec3 make_ivec3(const N& n) {
  return ivec3(n);
}

template <typename X, typename Y>
ivec3 make_ivec3(const X& x, const Y& y) {
  return ivec3(x, y);
}

template <typename X, typename Y, typename Z>
ivec3 make_ivec3(const X& x, const Y& y, const Z& z) {
  return ivec3(x, y, z);
}

struct ivec4_scalar {
  typedef int32_t element_type;

  int32_t x;
  int32_t y;
  int32_t z;
  int32_t w;

  ivec4_scalar() : ivec4_scalar(0) {}
  IMPLICIT constexpr ivec4_scalar(int32_t a) : x(a), y(a), z(a), w(a) {}
  constexpr ivec4_scalar(int32_t x, int32_t y, int32_t z, int32_t w)
      : x(x), y(y), z(z), w(w) {}

  int32_t& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  int32_t& sel(XYZW c1) { return select(c1); }
  ivec2_scalar sel(XYZW c1, XYZW c2) {
    return ivec2_scalar{select(c1), select(c2)};
  }

  friend ivec4_scalar operator&(int32_t a, ivec4_scalar b) {
    return ivec4_scalar{a & b.x, a & b.y, a & b.z, a & b.w};
  }

  int32_t& operator[](int index) {
    switch (index) {
      case 0:
        return x;
      case 1:
        return y;
      case 2:
        return z;
      case 3:
        return w;
      default:
        UNREACHABLE;
    }
  }
};

struct ivec4 {
  typedef int32_t element_type;

  ivec4() : ivec4(I32(0)) {}
  IMPLICIT ivec4(I32 a) : x(a), y(a), z(a), w(a) {}
  ivec4(I32 x, I32 y, I32 z, I32 w) : x(x), y(y), z(z), w(w) {}
  ivec4(ivec2 a, I32 b, I32 c) : x(a.x), y(a.y), z(b), w(c) {}
  IMPLICIT constexpr ivec4(ivec4_scalar s) : x(s.x), y(s.y), z(s.z), w(s.w) {}
  constexpr ivec4(ivec4_scalar s0, ivec4_scalar s1, ivec4_scalar s2,
                  ivec4_scalar s3)
      : x(I32{s0.x, s1.x, s2.x, s3.x}),
        y(I32{s0.y, s1.y, s2.y, s3.y}),
        z(I32{s0.z, s1.z, s2.z, s3.z}),
        w(I32{s0.w, s1.w, s2.w, s3.w}) {}

  I32& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  I32 sel(XYZW c1) { return select(c1); }

  ivec2 sel(XYZW c1, XYZW c2) { return ivec2(select(c1), select(c2)); }

  ivec3 sel(XYZW c1, XYZW c2, XYZW c3) {
    return ivec3(select(c1), select(c2), select(c3));
  }

  friend ivec4 operator&(I32 a, ivec4 b) {
    return ivec4(a & b.x, a & b.y, a & b.z, a & b.w);
  }

  I32 x;
  I32 y;
  I32 z;
  I32 w;
};

ivec4_scalar force_scalar(const ivec4& v) {
  return ivec4_scalar{force_scalar(v.x), force_scalar(v.y), force_scalar(v.z),
                      force_scalar(v.w)};
}

ivec4_scalar make_ivec4(int32_t n) { return ivec4_scalar{n, n, n, n}; }

ivec4_scalar make_ivec4(const ivec2_scalar& xy, int32_t z, int32_t w) {
  return ivec4_scalar{xy.x, xy.y, z, w};
}

ivec4_scalar make_ivec4(int32_t x, int32_t y, int32_t z, int32_t w) {
  return ivec4_scalar{x, y, z, w};
}

template <typename N>
ivec4 make_ivec4(const N& n) {
  return ivec4(n);
}

template <typename X, typename Y, typename Z>
ivec4 make_ivec4(const X& x, const Y& y, const Z& z) {
  return ivec4(x, y, z);
}

template <typename X, typename Y, typename Z, typename W>
ivec4 make_ivec4(const X& x, const Y& y, const Z& z, const W& w) {
  return ivec4(x, y, z, w);
}

SI ivec2 if_then_else(I32 c, ivec2 t, ivec2 e) {
  return ivec2(if_then_else(c, t.x, e.x), if_then_else(c, t.y, e.y));
}

SI ivec2 if_then_else(int32_t c, ivec2 t, ivec2 e) { return c ? t : e; }

SI ivec4 if_then_else(I32 c, ivec4 t, ivec4 e) {
  return ivec4(if_then_else(c, t.x, e.x), if_then_else(c, t.y, e.y),
               if_then_else(c, t.z, e.z), if_then_else(c, t.w, e.w));
}

SI ivec4 if_then_else(int32_t c, ivec4 t, ivec4 e) { return c ? t : e; }

ivec4 operator&(I32 a, ivec4_scalar b) {
  return ivec4(a & b.x, a & b.y, a & b.z, a & b.w);
}

struct bvec3_scalar {
  bool x;
  bool y;
  bool z;

  bvec3_scalar() : bvec3_scalar(false) {}
  IMPLICIT constexpr bvec3_scalar(bool a) : x(a), y(a), z(a) {}
  constexpr bvec3_scalar(bool x, bool y, bool z) : x(x), y(y), z(z) {}
};

struct bvec3_scalar1 {
  bool x;

  IMPLICIT constexpr bvec3_scalar1(bool a) : x(a) {}

  operator bvec3_scalar() const { return bvec3_scalar(x); }
};

struct bvec3 {
  bvec3() : bvec3(0) {}
  IMPLICIT bvec3(Bool a) : x(a), y(a), z(a) {}
  bvec3(Bool x, Bool y, Bool z) : x(x), y(y), z(z) {}
  Bool& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      default:
        UNREACHABLE;
    }
  }
  Bool sel(XYZW c1) { return select(c1); }

  Bool x;
  Bool y;
  Bool z;
};

bvec3_scalar1 make_bvec3(bool n) { return bvec3_scalar1(n); }

struct bvec4_scalar {
  bool x;
  bool y;
  bool z;
  bool w;

  bvec4_scalar() : bvec4_scalar(false) {}
  IMPLICIT constexpr bvec4_scalar(bool a) : x(a), y(a), z(a), w(a) {}
  constexpr bvec4_scalar(bool x, bool y, bool z, bool w)
      : x(x), y(y), z(z), w(w) {}

  bool& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  bool sel(XYZW c1) { return select(c1); }
  bvec2_scalar sel(XYZW c1, XYZW c2) {
    return bvec2_scalar(select(c1), select(c2));
  }
};

bvec4_scalar bvec2_scalar::sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
  return bvec4_scalar{select(c1), select(c2), select(c3), select(c4)};
}

struct bvec4_scalar1 {
  bool x;

  IMPLICIT constexpr bvec4_scalar1(bool a) : x(a) {}

  operator bvec4_scalar() const { return bvec4_scalar(x); }
};

struct bvec4 {
  bvec4() : bvec4(0) {}
  IMPLICIT bvec4(Bool a) : x(a), y(a), z(a), w(a) {}
  bvec4(Bool x, Bool y, Bool z, Bool w) : x(x), y(y), z(z), w(w) {}
  bvec4(bvec2 x, bvec2 y) : x(x.x), y(x.y), z(y.x), w(y.y) {}
  Bool& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  Bool sel(XYZW c1) { return select(c1); }

  Bool x;
  Bool y;
  Bool z;
  Bool w;
};

bvec4_scalar1 make_bvec4(bool n) { return bvec4_scalar1(n); }

bvec4_scalar make_bvec4(bool x, bool y, bool z, bool w) {
  return bvec4_scalar{x, y, z, w};
}

bvec4_scalar make_bvec4(bvec2_scalar a, bvec2_scalar b) {
  return bvec4_scalar{a.x, a.y, b.x, b.y};
}

template <typename N>
bvec4 make_bvec4(const N& n) {
  return bvec4(n);
}

template <typename X, typename Y>
bvec4 make_bvec4(const X& x, const Y& y) {
  return bvec4(x, y);
}

template <typename X, typename Y, typename Z, typename W>
bvec4 make_bvec4(const X& x, const Y& y, const Z& z, const W& w) {
  return bvec4(x, y, z, w);
}

struct vec2_ref {
  vec2_ref(Float& x, Float& y) : x(x), y(y) {}
  Float& x;
  Float& y;

  Float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      default:
        UNREACHABLE;
    }
  }
  Float& sel(XYZW c1) { return select(c1); }

  vec2_ref& operator=(const vec2& a) {
    x = a.x;
    y = a.y;
    return *this;
  }

  vec2_ref& operator/=(Float a) {
    x /= a;
    y /= a;
    return *this;
  }

  vec2_ref& operator/=(vec2 a) {
    x /= a.x;
    y /= a.y;
    return *this;
  }

  vec2_ref& operator+=(vec2 a) {
    x += a.x;
    y += a.y;
    return *this;
  }
  vec2_ref& operator-=(vec2 a) {
    x -= a.x;
    y -= a.y;
    return *this;
  }
  vec2_ref& operator*=(vec2 a) {
    x *= a.x;
    y *= a.y;
    return *this;
  }
};

struct vec3_scalar {
  typedef struct vec3 vector_type;
  typedef float element_type;

  float x;
  float y;
  float z;

  constexpr vec3_scalar() : vec3_scalar(0.0f) {}
  IMPLICIT constexpr vec3_scalar(float a) : x(a), y(a), z(a) {}
  constexpr vec3_scalar(float x, float y, float z) : x(x), y(y), z(z) {}

  float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      default:
        UNREACHABLE;
    }
  }
  float& sel(XYZW c1) { return select(c1); }
  vec2_scalar sel(XYZW c1, XYZW c2) {
    return vec2_scalar(select(c1), select(c2));
  }
  vec3_scalar sel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3_scalar(select(c1), select(c2), select(c3));
  }
  vec2_scalar_ref lsel(XYZW c1, XYZW c2) {
    return vec2_scalar_ref(select(c1), select(c2));
  }

  friend vec3_scalar operator*(vec3_scalar a, vec3_scalar b) {
    return vec3_scalar{a.x * b.x, a.y * b.y, a.z * b.z};
  }
  friend vec3_scalar operator*(vec3_scalar a, float b) {
    return vec3_scalar{a.x * b, a.y * b, a.z * b};
  }

  friend vec3_scalar operator-(vec3_scalar a, vec3_scalar b) {
    return vec3_scalar{a.x - b.x, a.y - b.y, a.z - b.z};
  }
  friend vec3_scalar operator+(vec3_scalar a, vec3_scalar b) {
    return vec3_scalar{a.x + b.x, a.y + b.y, a.z + b.z};
  }

  friend vec3_scalar operator/(vec3_scalar a, float b) {
    return vec3_scalar{a.x / b, a.y / b, a.z / b};
  }

  vec3_scalar operator+=(vec3_scalar a) {
    x += a.x;
    y += a.y;
    z += a.z;
    return *this;
  }

  friend bool operator==(const vec3_scalar& l, const vec3_scalar& r) {
    return l.x == r.x && l.y == r.y && l.z == r.z;
  }
};

struct vec3_scalar_ref {
  vec3_scalar_ref(float& x, float& y, float& z) : x(x), y(y), z(z) {}
  float& x;
  float& y;
  float& z;

  float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      default:
        UNREACHABLE;
    }
  }
  float& sel(XYZW c1) { return select(c1); }

  vec3_scalar_ref& operator=(const vec3_scalar& a) {
    x = a.x;
    y = a.y;
    z = a.z;
    return *this;
  }

  operator vec3_scalar() const { return vec3_scalar{x, y, z}; }
};

struct vec3 {
  typedef struct vec3 vector_type;
  typedef float element_type;

  constexpr vec3() : vec3(Float(0.0f)) {}
  IMPLICIT constexpr vec3(Float a) : x(a), y(a), z(a) {}
  constexpr vec3(Float x, Float y, Float z) : x(x), y(y), z(z) {}
  vec3(vec2 a, Float z) : x(a.x), y(a.y), z(z) {}
  explicit vec3(vec4);
  IMPLICIT constexpr vec3(vec3_scalar s) : x(s.x), y(s.y), z(s.z) {}
  constexpr vec3(vec3_scalar s0, vec3_scalar s1, vec3_scalar s2, vec3_scalar s3)
      : x(Float{s0.x, s1.x, s2.x, s3.x}),
        y(Float{s0.y, s1.y, s2.y, s3.y}),
        z(Float{s0.z, s1.z, s2.z, s3.z}) {}
  Float x;
  Float y;
  Float z;

  Float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      default:
        UNREACHABLE;
    }
  }
  Float& sel(XYZW c1) { return select(c1); }

  vec2 sel(XYZW c1, XYZW c2) { return vec2(select(c1), select(c2)); }

  vec3 sel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3(select(c1), select(c2), select(c3));
  }

  vec4 sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4);

  vec2_ref lsel(XYZW c1, XYZW c2) { return vec2_ref(select(c1), select(c2)); }

  friend vec3 operator*(vec3 a, Float b) {
    return vec3(a.x * b, a.y * b, a.z * b);
  }
  friend vec3 operator*(vec3 a, vec3 b) {
    return vec3(a.x * b.x, a.y * b.y, a.z * b.z);
  }
  friend vec3 operator*(Float a, vec3 b) {
    return vec3(a * b.x, a * b.y, a * b.z);
  }

  friend vec3 operator/(vec3 a, Float b) {
    return vec3(a.x / b, a.y / b, a.z / b);
  }

  friend I32 operator==(const vec3& l, const vec3& r) {
    return l.x == r.x && l.y == r.y && l.z == r.z;
  }

  friend vec3 operator-(vec3 a, Float b) {
    return vec3(a.x - b, a.y - b, a.z - b);
  }
  friend vec3 operator-(vec3 a, vec3 b) {
    return vec3(a.x - b.x, a.y - b.y, a.z - b.z);
  }
  friend vec3 operator+(vec3 a, Float b) {
    return vec3(a.x + b, a.y + b, a.z + b);
  }
  friend vec3 operator+(vec3 a, vec3 b) {
    return vec3(a.x + b.x, a.y + b.y, a.z + b.z);
  }

  vec3 operator+=(vec3_scalar a) {
    x += a.x;
    y += a.y;
    z += a.z;
    return *this;
  }
  vec3& operator+=(vec3 a) {
    x += a.x;
    y += a.y;
    z += a.z;
    return *this;
  }
};

vec3_scalar force_scalar(const vec3& v) {
  return vec3_scalar{force_scalar(v.x), force_scalar(v.y), force_scalar(v.z)};
}

vec3_scalar make_vec3(float n) { return vec3_scalar{n, n, n}; }

vec3_scalar make_vec3(const vec2_scalar& v, float z) {
  return vec3_scalar{v.x, v.y, z};
}

vec3_scalar make_vec3(float x, float y, float z) {
  return vec3_scalar{x, y, z};
}

vec3_scalar make_vec3(int32_t x, int32_t y, float z) {
  return vec3_scalar{float(x), float(y), z};
}

template <typename N>
vec3 make_vec3(const N& n) {
  return vec3(n);
}

template <typename X, typename Y>
vec3 make_vec3(const X& x, const Y& y) {
  return vec3(x, y);
}

template <typename X, typename Y, typename Z>
vec3 make_vec3(const X& x, const Y& y, const Z& z) {
  return vec3(x, y, z);
}

SI vec3 if_then_else(I32 c, vec3 t, vec3 e) {
  return vec3(if_then_else(c, t.x, e.x), if_then_else(c, t.y, e.y),
              if_then_else(c, t.z, e.z));
}

SI vec3 if_then_else(int32_t c, vec3 t, vec3 e) { return c ? t : e; }

SI vec3 if_then_else(ivec3 c, vec3 t, vec3 e) {
  return vec3(if_then_else(c.x, t.x, e.x), if_then_else(c.y, t.y, e.y),
              if_then_else(c.z, t.z, e.z));
}

vec3 step(vec3 edge, vec3 x) {
  return vec3(step(edge.x, x.x), step(edge.y, x.y), step(edge.z, x.z));
}

vec3_scalar step(vec3_scalar edge, vec3_scalar x) {
  return vec3_scalar(step(edge.x, x.x), step(edge.y, x.y), step(edge.z, x.z));
}

SI vec3 min(vec3 a, vec3 b) {
  return vec3(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z));
}
SI vec3 min(vec3 a, Float b) {
  return vec3(min(a.x, b), min(a.y, b), min(a.z, b));
}
SI vec3_scalar min(vec3_scalar a, vec3_scalar b) {
  return vec3_scalar{min(a.x, b.x), min(a.y, b.y), min(a.z, b.z)};
}

SI vec3 max(vec3 a, vec3 b) {
  return vec3(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z));
}
SI vec3 max(vec3 a, Float b) {
  return vec3(max(a.x, b), max(a.y, b), max(a.z, b));
}
SI vec3_scalar max(vec3_scalar a, vec3_scalar b) {
  return vec3_scalar{max(a.x, b.x), max(a.y, b.y), max(a.z, b.z)};
}

vec3 pow(vec3 x, vec3 y) {
  return vec3(pow(x.x, y.x), pow(x.y, y.y), pow(x.z, y.z));
}

struct vec3_ref {
  vec3_ref(Float& x, Float& y, Float& z) : x(x), y(y), z(z) {}
  Float& x;
  Float& y;
  Float& z;
  vec3_ref& operator=(const vec3& a) {
    x = a.x;
    y = a.y;
    z = a.z;
    return *this;
  }

  vec3_ref& operator/=(Float a) {
    x /= a;
    y /= a;
    z /= a;
    return *this;
  }

  vec3_ref& operator*=(Float a) {
    x *= a;
    y *= a;
    z *= a;
    return *this;
  }
};

struct vec4_scalar {
  typedef struct vec4 vector_type;
  typedef float element_type;

  float x;
  float y;
  float z;
  float w;

  constexpr vec4_scalar() : vec4_scalar(0.0f) {}
  IMPLICIT constexpr vec4_scalar(float a) : x(a), y(a), z(a), w(a) {}
  constexpr vec4_scalar(float x, float y, float z, float w)
      : x(x), y(y), z(z), w(w) {}
  vec4_scalar(vec3_scalar xyz, float w) : x(xyz.x), y(xyz.y), z(xyz.z), w(w) {}

  static vec4_scalar load_from_ptr(const float* f) {
    return vec4_scalar(f[0], f[1], f[2], f[3]);
  }

  ALWAYS_INLINE float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  float& sel(XYZW c1) { return select(c1); }
  vec2_scalar sel(XYZW c1, XYZW c2) {
    return vec2_scalar{select(c1), select(c2)};
  }
  vec3_scalar sel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3_scalar{select(c1), select(c2), select(c3)};
  }
  vec4_scalar sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
    return vec4_scalar{select(c1), select(c2), select(c3), select(c4)};
  }
  vec2_scalar_ref lsel(XYZW c1, XYZW c2) {
    return vec2_scalar_ref(select(c1), select(c2));
  }
  vec3_scalar_ref lsel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3_scalar_ref(select(c1), select(c2), select(c3));
  }

  friend vec4_scalar operator*(vec4_scalar a, vec4_scalar b) {
    return vec4_scalar{a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w};
  }
  friend vec4_scalar operator*(vec4_scalar a, float b) {
    return vec4_scalar{a.x * b, a.y * b, a.z * b, a.w * b};
  }
  vec4_scalar& operator*=(float a) {
    x *= a;
    y *= a;
    z *= a;
    w *= a;
    return *this;
  }

  friend vec4_scalar operator-(vec4_scalar a, vec4_scalar b) {
    return vec4_scalar{a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w};
  }
  friend vec4_scalar operator+(vec4_scalar a, vec4_scalar b) {
    return vec4_scalar{a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w};
  }

  friend vec4_scalar operator/(vec4_scalar a, vec4_scalar b) {
    return vec4_scalar{a.x / b.x, a.y / b.y, a.z / b.z, a.w / b.w};
  }

  vec4_scalar& operator+=(vec4_scalar a) {
    x += a.x;
    y += a.y;
    z += a.z;
    w += a.w;
    return *this;
  }

  vec4_scalar& operator/=(vec4_scalar a) {
    x /= a.x;
    y /= a.y;
    z /= a.z;
    w /= a.w;
    return *this;
  }

  vec4_scalar& operator*=(vec4_scalar a) {
    x *= a.x;
    y *= a.y;
    z *= a.z;
    w *= a.w;
    return *this;
  }

  friend bool operator==(const vec4_scalar& l, const vec4_scalar& r) {
    return l.x == r.x && l.y == r.y && l.z == r.z && l.w == r.w;
  }

  friend bool operator!=(const vec4_scalar& l, const vec4_scalar& r) {
    return l.x != r.x || l.y != r.y || l.z != r.z || l.w != r.w;
  }
};

vec4_scalar vec2_scalar::sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
  return vec4_scalar{select(c1), select(c2), select(c3), select(c4)};
}

struct vec4_ref {
  vec4_ref(Float& x, Float& y, Float& z, Float& w) : x(x), y(y), z(z), w(w) {}
  Float& x;
  Float& y;
  Float& z;
  Float& w;

  vec4_ref& operator=(const vec4& a);
};

struct vec4 {
  typedef struct vec4 vector_type;
  typedef float element_type;

  constexpr vec4() : vec4(Float(0.0f)) {}
  IMPLICIT constexpr vec4(Float a) : x(a), y(a), z(a), w(a) {}
  vec4(Float x, Float y, Float z, Float w) : x(x), y(y), z(z), w(w) {}
  vec4(vec3 xyz, Float w) : x(xyz.x), y(xyz.y), z(xyz.z), w(w) {}
  vec4(vec2 xy, vec2 zw) : x(xy.x), y(xy.y), z(zw.x), w(zw.y) {}
  vec4(vec2 xy, Float z, Float w) : x(xy.x), y(xy.y), z(z), w(w) {}
  vec4(Float x, Float y, vec2 zw) : x(x), y(y), z(zw.x), w(zw.y) {}
  IMPLICIT constexpr vec4(vec4_scalar s) : x(s.x), y(s.y), z(s.z), w(s.w) {}
  constexpr vec4(vec4_scalar s0, vec4_scalar s1, vec4_scalar s2, vec4_scalar s3)
      : x(Float{s0.x, s1.x, s2.x, s3.x}),
        y(Float{s0.y, s1.y, s2.y, s3.y}),
        z(Float{s0.z, s1.z, s2.z, s3.z}),
        w(Float{s0.w, s1.w, s2.w, s3.w}) {}
  ALWAYS_INLINE Float& select(XYZW c) {
    switch (c) {
      case X:
        return x;
      case Y:
        return y;
      case Z:
        return z;
      case W:
        return w;
      default:
        UNREACHABLE;
    }
  }
  ALWAYS_INLINE Float& sel(XYZW c1) { return select(c1); }

  ALWAYS_INLINE vec2 sel(XYZW c1, XYZW c2) {
    return vec2(select(c1), select(c2));
  }

  ALWAYS_INLINE vec3 sel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3(select(c1), select(c2), select(c3));
  }
  ALWAYS_INLINE vec3_ref lsel(XYZW c1, XYZW c2, XYZW c3) {
    return vec3_ref(select(c1), select(c2), select(c3));
  }

  ALWAYS_INLINE vec2_ref lsel(XYZW c1, XYZW c2) {
    return vec2_ref(select(c1), select(c2));
  }

  ALWAYS_INLINE vec4 sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
    return vec4(select(c1), select(c2), select(c3), select(c4));
  }
  ALWAYS_INLINE vec4_ref lsel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
    return vec4_ref(select(c1), select(c2), select(c3), select(c4));
  }

  Float& operator[](int index) {
    switch (index) {
      case 0:
        return x;
      case 1:
        return y;
      case 2:
        return z;
      case 3:
        return w;
      default:
        UNREACHABLE;
    }
  }

  // glsl supports non-const indexing of vecs.
  // hlsl doesn't. The code it generates is probably not wonderful.
  Float operator[](I32 index) {
    float sel_x = 0;
    switch (index.x) {
      case 0:
        sel_x = x.x;
        break;
      case 1:
        sel_x = y.x;
        break;
      case 2:
        sel_x = z.x;
        break;
      case 3:
        sel_x = w.x;
        break;
    }
    float sel_y = 0;
    switch (index.y) {
      case 0:
        sel_y = x.y;
        break;
      case 1:
        sel_y = y.y;
        break;
      case 2:
        sel_y = z.y;
        break;
      case 3:
        sel_y = w.y;
        break;
    }
    float sel_z = 0;
    switch (index.z) {
      case 0:
        sel_z = x.z;
        break;
      case 1:
        sel_z = y.z;
        break;
      case 2:
        sel_z = z.z;
        break;
      case 3:
        sel_z = w.z;
        break;
    }
    float sel_w = 0;
    switch (index.w) {
      case 0:
        sel_w = x.w;
        break;
      case 1:
        sel_w = y.w;
        break;
      case 2:
        sel_w = z.w;
        break;
      case 3:
        sel_w = w.w;
        break;
    }
    Float ret = {sel_x, sel_y, sel_z, sel_w};
    return ret;
  }

  friend vec4 operator/(vec4 a, Float b) {
    return vec4(a.x / b, a.y / b, a.z / b, a.w / b);
  }
  friend vec4 operator/(vec4 a, vec4 b) {
    return vec4(a.x / b.x, a.y / b.y, a.z / b.z, a.w / b.w);
  }

  friend vec4 operator*(vec4 a, Float b) {
    return vec4(a.x * b, a.y * b, a.z * b, a.w * b);
  }

  friend vec4 operator*(Float b, vec4 a) {
    return vec4(a.x * b, a.y * b, a.z * b, a.w * b);
  }
  friend vec4 operator*(vec4 a, vec4 b) {
    return vec4(a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w);
  }

  friend vec4 operator-(vec4 a, vec4 b) {
    return vec4(a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w);
  }
  friend vec4 operator+(vec4 a, vec4 b) {
    return vec4(a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w);
  }
  vec4& operator+=(vec4 a) {
    x += a.x;
    y += a.y;
    z += a.z;
    w += a.w;
    return *this;
  }
  vec4& operator/=(vec4 a) {
    x /= a.x;
    y /= a.y;
    z /= a.z;
    w /= a.w;
    return *this;
  }
  vec4& operator*=(vec4 a) {
    x *= a.x;
    y *= a.y;
    z *= a.z;
    w *= a.w;
    return *this;
  }
  vec4& operator*=(Float a) {
    x *= a;
    y *= a;
    z *= a;
    w *= a;
    return *this;
  }

  Float x;
  Float y;
  Float z;
  Float w;
};

inline vec4_ref& vec4_ref::operator=(const vec4& a) {
  x = a.x;
  y = a.y;
  z = a.z;
  w = a.w;
  return *this;
}

inline vec4 vec3::sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
  return vec4(select(c1), select(c2), select(c3), select(c4));
}

vec4_scalar force_scalar(const vec4& v) {
  return vec4_scalar{force_scalar(v.x), force_scalar(v.y), force_scalar(v.z),
                     force_scalar(v.w)};
}

vec4_scalar make_vec4(float n) { return vec4_scalar{n, n, n, n}; }

vec4_scalar make_vec4(const vec2_scalar& v, float z, float w) {
  return vec4_scalar{v.x, v.y, z, w};
}

vec4_scalar make_vec4(const vec2_scalar& a, const vec2_scalar& b) {
  return vec4_scalar{a.x, a.y, b.x, b.y};
}

vec4_scalar make_vec4(const vec3_scalar& v, float w) {
  return vec4_scalar{v.x, v.y, v.z, w};
}

vec4_scalar make_vec4(float x, float y, float z, float w) {
  return vec4_scalar{x, y, z, w};
}

vec4_scalar make_vec4(float x, float y, const vec2_scalar& v) {
  return vec4_scalar{x, y, v.x, v.y};
}

ivec4_scalar make_ivec4(const vec4_scalar& v) {
  return ivec4_scalar{int32_t(v.x), int32_t(v.y), int32_t(v.z), int32_t(v.w)};
}

template <typename N>
vec4 make_vec4(const N& n) {
  return vec4(n);
}

template <typename X, typename Y>
vec4 make_vec4(const X& x, const Y& y) {
  return vec4(x, y);
}

template <typename X, typename Y, typename Z>
vec4 make_vec4(const X& x, const Y& y, const Z& z) {
  return vec4(x, y, z);
}

template <typename X, typename Y, typename Z, typename W>
vec4 make_vec4(const X& x, const Y& y, const Z& z, const W& w) {
  return vec4(x, y, z, w);
}

ALWAYS_INLINE vec3::vec3(vec4 v) : x(v.x), y(v.y), z(v.z) {}

SI ivec4 roundfast(vec4 v, Float scale) {
  return ivec4(roundfast(v.x, scale), roundfast(v.y, scale),
               roundfast(v.z, scale), roundfast(v.w, scale));
}

vec4 operator*(vec4_scalar a, Float b) {
  return vec4(a.x * b, a.y * b, a.z * b, a.w * b);
}

SI vec4 if_then_else(I32 c, vec4 t, vec4 e) {
  return vec4(if_then_else(c, t.x, e.x), if_then_else(c, t.y, e.y),
              if_then_else(c, t.z, e.z), if_then_else(c, t.w, e.w));
}

SI vec4 if_then_else(int32_t c, vec4 t, vec4 e) { return c ? t : e; }

SI vec4_scalar if_then_else(int32_t c, vec4_scalar t, vec4_scalar e) {
  return c ? t : e;
}

SI vec2 clamp(vec2 a, Float minVal, Float maxVal) {
  return vec2(clamp(a.x, minVal, maxVal), clamp(a.y, minVal, maxVal));
}

SI vec2 clamp(vec2 a, vec2 minVal, vec2 maxVal) {
  return vec2(clamp(a.x, minVal.x, maxVal.x), clamp(a.y, minVal.y, maxVal.y));
}

SI vec2_scalar clamp(vec2_scalar a, vec2_scalar minVal, vec2_scalar maxVal) {
  return vec2_scalar{clamp(a.x, minVal.x, maxVal.x),
                     clamp(a.y, minVal.y, maxVal.y)};
}

SI vec2_scalar clamp(vec2_scalar a, float minVal, float maxVal) {
  return vec2_scalar{clamp(a.x, minVal, maxVal), clamp(a.y, minVal, maxVal)};
}

SI I32 clamp(I32 a, I32 minVal, I32 maxVal) {
  a = if_then_else(a < minVal, minVal, a);
  return if_then_else(a > maxVal, maxVal, a);
}

SI vec3 clamp(vec3 a, Float minVal, Float maxVal) {
  return vec3(clamp(a.x, minVal, maxVal), clamp(a.y, minVal, maxVal),
              clamp(a.z, minVal, maxVal));
}

SI vec3 clamp(vec3 a, vec3 minVal, vec3 maxVal) {
  return vec3(clamp(a.x, minVal.x, maxVal.x), clamp(a.y, minVal.y, maxVal.y),
              clamp(a.z, minVal.z, maxVal.z));
}

SI vec4 clamp(vec4 a, Float minVal, Float maxVal) {
  return vec4(clamp(a.x, minVal, maxVal), clamp(a.y, minVal, maxVal),
              clamp(a.z, minVal, maxVal), clamp(a.w, minVal, maxVal));
}

SI vec4 clamp(vec4 a, vec4 minVal, vec4 maxVal) {
  return vec4(clamp(a.x, minVal.x, maxVal.x), clamp(a.y, minVal.y, maxVal.y),
              clamp(a.z, minVal.z, maxVal.z), clamp(a.w, minVal.w, maxVal.w));
}

SI vec4_scalar clamp(vec4_scalar a, vec4_scalar minVal, vec4_scalar maxVal) {
  return vec4_scalar{
      clamp(a.x, minVal.x, maxVal.x), clamp(a.y, minVal.y, maxVal.y),
      clamp(a.z, minVal.z, maxVal.z), clamp(a.w, minVal.w, maxVal.w)};
}

SI vec4_scalar clamp(vec4_scalar a, float minVal, float maxVal) {
  return vec4_scalar{clamp(a.x, minVal, maxVal), clamp(a.y, minVal, maxVal),
                     clamp(a.z, minVal, maxVal), clamp(a.w, minVal, maxVal)};
}

vec4 step(vec4 edge, vec4 x) {
  return vec4(step(edge.x, x.x), step(edge.y, x.y), step(edge.z, x.z),
              step(edge.w, x.w));
}

vec4_scalar step(vec4_scalar edge, vec4_scalar x) {
  return vec4_scalar(step(edge.x, x.x), step(edge.y, x.y), step(edge.z, x.z),
                     step(edge.w, x.w));
}

template <typename T>
auto lessThanEqual(T x, T y) -> decltype(x <= y) {
  return x <= y;
}

template <typename T>
auto lessThan(T x, T y) -> decltype(x < y) {
  return x < y;
}

SI bvec3 lessThanEqual(vec3 x, vec3 y) {
  return bvec3(lessThanEqual(x.x, y.x), lessThanEqual(x.y, y.y),
               lessThanEqual(x.z, y.z));
}

SI bvec2 lessThanEqual(vec2 x, vec2 y) {
  return bvec2(lessThanEqual(x.x, y.x), lessThanEqual(x.y, y.y));
}

SI bvec2_scalar lessThanEqual(vec2_scalar x, vec2_scalar y) {
  return bvec2_scalar{lessThanEqual(x.x, y.x), lessThanEqual(x.y, y.y)};
}

SI bvec4 lessThanEqual(vec4 x, vec4 y) {
  return bvec4(lessThanEqual(x.x, y.x), lessThanEqual(x.y, y.y),
               lessThanEqual(x.z, y.z), lessThanEqual(x.w, y.w));
}

SI bvec4_scalar lessThanEqual(vec4_scalar x, vec4_scalar y) {
  return bvec4_scalar{lessThanEqual(x.x, y.x), lessThanEqual(x.y, y.y),
                      lessThanEqual(x.z, y.z), lessThanEqual(x.w, y.w)};
}

SI bvec2 lessThan(vec2 x, vec2 y) {
  return bvec2(lessThan(x.x, y.x), lessThan(x.y, y.y));
}

SI bvec2_scalar lessThan(vec2_scalar x, vec2_scalar y) {
  return bvec2_scalar(lessThan(x.x, y.x), lessThan(x.y, y.y));
}

SI bvec4 lessThan(vec4 x, vec4 y) {
  return bvec4(lessThan(x.x, y.x), lessThan(x.y, y.y), lessThan(x.z, y.z),
               lessThan(x.w, y.w));
}

SI bvec4_scalar lessThan(vec4_scalar x, vec4_scalar y) {
  return bvec4_scalar{lessThan(x.x, y.x), lessThan(x.y, y.y),
                      lessThan(x.z, y.z), lessThan(x.w, y.w)};
}

template <typename T>
auto greaterThan(T x, T y) -> decltype(x > y) {
  return x > y;
}

bvec2 greaterThan(vec2 x, vec2 y) {
  return bvec2(greaterThan(x.x, y.x), greaterThan(x.y, y.y));
}

bvec2_scalar greaterThan(vec2_scalar x, vec2_scalar y) {
  return bvec2_scalar(greaterThan(x.x, y.x), greaterThan(x.y, y.y));
}

SI bvec4 greaterThan(vec4 x, vec4 y) {
  return bvec4(greaterThan(x.x, y.x), greaterThan(x.y, y.y),
               greaterThan(x.z, y.z), greaterThan(x.w, y.w));
}

SI bvec4_scalar greaterThan(vec4_scalar x, vec4_scalar y) {
  return bvec4_scalar{greaterThan(x.x, y.x), greaterThan(x.y, y.y),
                      greaterThan(x.z, y.z), greaterThan(x.w, y.w)};
}

template <typename T>
auto greaterThanEqual(T x, T y) -> decltype(x >= y) {
  return x >= y;
}

bvec4 greaterThanEqual(vec4 x, vec4 y) {
  return bvec4(greaterThanEqual(x.x, y.x), greaterThanEqual(x.y, y.y),
               greaterThanEqual(x.z, y.z), greaterThanEqual(x.w, y.w));
}

template <typename T>
auto equal(T x, T y) -> decltype(x > y) {
  return x == y;
}

bvec2 equal(vec2 x, vec2 y) { return bvec2(equal(x.x, y.x), equal(x.y, y.y)); }

bvec2_scalar equal(vec2_scalar x, vec2_scalar y) {
  return bvec2_scalar(equal(x.x, y.x), equal(x.y, y.y));
}

template <typename T>
auto notEqual(T x, T y) -> decltype(x > y) {
  return x != y;
}

bvec2 notEqual(vec2 x, vec2 y) {
  return bvec2(notEqual(x.x, y.x), notEqual(x.y, y.y));
}

bvec2_scalar notEqual(vec2_scalar x, vec2_scalar y) {
  return bvec2_scalar(notEqual(x.x, y.x), notEqual(x.y, y.y));
}

struct mat4_scalar;

struct mat2_scalar {
  vec2_scalar data[2];

  mat2_scalar() = default;
  IMPLICIT constexpr mat2_scalar(float a) {
    data[0] = vec2_scalar(a);
    data[1] = vec2_scalar(a);
  }
  constexpr mat2_scalar(vec2_scalar a, vec2_scalar b) {
    data[0] = a;
    data[1] = b;
  }
  IMPLICIT mat2_scalar(const mat4_scalar& mat);

  vec2_scalar& operator[](int index) { return data[index]; }
  const vec2_scalar& operator[](int index) const { return data[index]; }

  friend vec2_scalar operator*(mat2_scalar m, vec2_scalar v) {
    vec2_scalar u;
    u.x = m[0].x * v.x + m[1].x * v.y;
    u.y = m[0].y * v.x + m[1].y * v.y;
    return u;
  }

  friend vec2 operator*(mat2_scalar m, vec2 v) {
    vec2 u;
    u.x = m[0].x * v.x + m[1].x * v.y;
    u.y = m[0].y * v.x + m[1].y * v.y;
    return u;
  }

  friend mat2_scalar operator*(mat2_scalar m, float f) {
    mat2_scalar u = m;
    u[0].x *= f;
    u[0].y *= f;
    u[1].x *= f;
    u[1].y *= f;
    return u;
  }
};

struct mat4;

struct mat2 {
  vec2 data[2];

  vec2& operator[](int index) { return data[index]; }
  const vec2& operator[](int index) const { return data[index]; }
  mat2() = default;

  IMPLICIT mat2(Float a) {
    data[0] = vec2(a);
    data[1] = vec2(a);
  }

  mat2(vec2 a, vec2 b) {
    data[0] = a;
    data[1] = b;
  }
  IMPLICIT mat2(const mat4& mat);
  IMPLICIT constexpr mat2(mat2_scalar s) {
    data[0] = vec2(s.data[0]);
    data[1] = vec2(s.data[1]);
  }

  friend vec2 operator*(mat2 m, vec2 v) {
    vec2 u;
    u.x = m[0].x * v.x + m[1].x * v.y;
    u.y = m[0].y * v.x + m[1].y * v.y;
    return u;
  }
  friend mat2 operator*(mat2 m, Float f) {
    mat2 u = m;
    u[0].x *= f;
    u[0].y *= f;
    u[1].x *= f;
    u[1].y *= f;
    return u;
  }
};

mat2_scalar make_mat2(float n) { return mat2_scalar{{n, n}, {n, n}}; }

mat2_scalar make_mat2(const mat2_scalar& m) { return m; }

mat2_scalar make_mat2(const vec2_scalar& x, const vec2_scalar& y) {
  return mat2_scalar{x, y};
}

template <typename N>
mat2 make_mat2(const N& n) {
  return mat2(n);
}

template <typename X, typename Y>
mat2 make_mat2(const X& x, const Y& y) {
  return mat2(x, y);
}

SI mat2 if_then_else(I32 c, mat2 t, mat2 e) {
  return mat2(if_then_else(c, t[0], e[0]), if_then_else(c, t[0], e[1]));
}

SI mat2 if_then_else(int32_t c, mat2 t, mat2 e) { return c ? t : e; }

struct mat3_scalar {
  vec3_scalar data[3];

  mat3_scalar() = default;
  constexpr mat3_scalar(vec3_scalar a, vec3_scalar b, vec3_scalar c) {
    data[0] = a;
    data[1] = b;
    data[2] = c;
  }
  IMPLICIT mat3_scalar(const mat4_scalar& mat);

  vec3_scalar& operator[](int index) { return data[index]; }
  const vec3_scalar& operator[](int index) const { return data[index]; }

  friend vec3_scalar operator*(mat3_scalar m, vec3_scalar v) {
    vec3_scalar u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z;
    return u;
  }

  friend vec3 operator*(mat3_scalar m, vec3 v) {
    vec3 u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z;
    return u;
  }
};

struct mat3 {
  vec3 data[3];

  vec3& operator[](int index) { return data[index]; }
  const vec3& operator[](int index) const { return data[index]; }
  mat3() = default;
  mat3(vec3 a, vec3 b, vec3 c) {
    data[0] = a;
    data[1] = b;
    data[2] = c;
  }

  IMPLICIT constexpr mat3(mat3_scalar s) {
    data[0] = vec3(s.data[0]);
    data[1] = vec3(s.data[1]);
    data[2] = vec3(s.data[2]);
  }
  constexpr mat3(mat3_scalar s0, mat3_scalar s1, mat3_scalar s2,
                 mat3_scalar s3) {
    data[0] = vec3(s0.data[0], s1.data[0], s2.data[0], s3.data[0]);
    data[1] = vec3(s0.data[1], s1.data[1], s2.data[1], s3.data[1]);
    data[2] = vec3(s0.data[2], s1.data[2], s2.data[2], s3.data[2]);
  }

  constexpr mat3(Float d1, Float d2, Float d3, Float d4, Float d5, Float d6,
                 Float d7, Float d8, Float d9) {
    data[0] = vec3(d1, d2, d3);
    data[1] = vec3(d4, d5, d6);
    data[2] = vec3(d7, d8, d9);
  }

  IMPLICIT mat3(const mat4& mat);

  friend vec3 operator*(mat3 m, vec3 v) {
    vec3 u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z;
    return u;
  }
};

mat3_scalar force_scalar(const mat3& v) {
  return mat3_scalar{force_scalar(v[0]), force_scalar(v[1]),
                     force_scalar(v[2])};
}

mat3_scalar make_mat3(const mat3_scalar& m) { return m; }

mat3_scalar make_mat3(const vec3_scalar& x, const vec3_scalar& y,
                      const vec3_scalar& z) {
  return mat3_scalar{x, y, z};
}

constexpr mat3_scalar make_mat3(float m0, float m1, float m2, float m3,
                                float m4, float m5, float m6, float m7,
                                float m8) {
  return mat3_scalar{{m0, m1, m2}, {m3, m4, m5}, {m6, m7, m8}};
}

template <typename N>
mat3 make_mat3(const N& n) {
  return mat3(n);
}

template <typename X, typename Y, typename Z>
mat3 make_mat3(const X& x, const Y& y, const Z& z) {
  return mat3(x, y, z);
}

struct mat4_scalar {
  vec4_scalar data[4];

  mat4_scalar() = default;
  constexpr mat4_scalar(vec4_scalar a, vec4_scalar b, vec4_scalar c,
                        vec4_scalar d) {
    data[0] = a;
    data[1] = b;
    data[2] = c;
    data[3] = d;
  }

  vec4_scalar& operator[](int index) { return data[index]; }
  const vec4_scalar& operator[](int index) const { return data[index]; }

  static mat4_scalar load_from_ptr(const float* f) {
    mat4_scalar m;
    // XXX: hopefully this is in the right order
    m.data[0] = vec4_scalar{f[0], f[1], f[2], f[3]};
    m.data[1] = vec4_scalar{f[4], f[5], f[6], f[7]};
    m.data[2] = vec4_scalar{f[8], f[9], f[10], f[11]};
    m.data[3] = vec4_scalar{f[12], f[13], f[14], f[15]};
    return m;
  }

  friend vec4_scalar operator*(mat4_scalar m, vec4_scalar v) {
    vec4_scalar u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z + m[3].x * v.w;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z + m[3].y * v.w;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z + m[3].z * v.w;
    u.w = m[0].w * v.x + m[1].w * v.y + m[2].w * v.z + m[3].w * v.w;
    return u;
  }

  friend vec4 operator*(mat4_scalar m, vec4 v) {
    vec4 u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z + m[3].x * v.w;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z + m[3].y * v.w;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z + m[3].z * v.w;
    u.w = m[0].w * v.x + m[1].w * v.y + m[2].w * v.z + m[3].w * v.w;
    return u;
  }
};

struct mat4 {
  vec4 data[4];

  mat4() = default;
  IMPLICIT constexpr mat4(mat4_scalar s) {
    data[0] = vec4(s.data[0]);
    data[1] = vec4(s.data[1]);
    data[2] = vec4(s.data[2]);
    data[3] = vec4(s.data[3]);
  }

  mat4(vec4 a, vec4 b, vec4 c, vec4 d) {
    data[0] = a;
    data[1] = b;
    data[2] = c;
    data[3] = d;
  }

  vec4& operator[](int index) { return data[index]; }
  const vec4& operator[](int index) const { return data[index]; }

  friend vec4 operator*(mat4 m, vec4 v) {
    vec4 u;
    u.x = m[0].x * v.x + m[1].x * v.y + m[2].x * v.z + m[3].x * v.w;
    u.y = m[0].y * v.x + m[1].y * v.y + m[2].y * v.z + m[3].y * v.w;
    u.z = m[0].z * v.x + m[1].z * v.y + m[2].z * v.z + m[3].z * v.w;
    u.w = m[0].w * v.x + m[1].w * v.y + m[2].w * v.z + m[3].w * v.w;
    return u;
  }
};

mat3::mat3(const mat4& mat)
    : mat3(vec3(mat[0].x, mat[0].y, mat[0].z),
           vec3(mat[1].x, mat[1].y, mat[1].z),
           vec3(mat[2].x, mat[2].y, mat[2].z)) {}

IMPLICIT mat3_scalar::mat3_scalar(const mat4_scalar& mat)
    : mat3_scalar(vec3_scalar(mat[0].x, mat[0].y, mat[0].z),
                  vec3_scalar(mat[1].x, mat[1].y, mat[1].z),
                  vec3_scalar(mat[2].x, mat[2].y, mat[2].z)) {}

IMPLICIT mat2::mat2(const mat4& mat)
    : mat2(vec2(mat[0].x, mat[0].y), vec2(mat[1].x, mat[1].y)) {}

IMPLICIT mat2_scalar::mat2_scalar(const mat4_scalar& mat)
    : mat2_scalar(vec2_scalar(mat[0].x, mat[0].y),
                  vec2_scalar(mat[1].x, mat[1].y)) {}

mat2_scalar make_mat2(const mat4_scalar& m) { return mat2_scalar(m); }

mat3_scalar make_mat3(const mat4_scalar& m) { return mat3_scalar(m); }

mat4_scalar force_scalar(const mat4& v) {
  return mat4_scalar(force_scalar(v[0]), force_scalar(v[1]), force_scalar(v[2]),
                     force_scalar(v[3]));
}

mat4_scalar make_mat4(const mat4_scalar& m) { return m; }

mat4_scalar make_mat4(const vec4_scalar& x, const vec4_scalar& y,
                      const vec4_scalar& z, const vec4_scalar& w) {
  return mat4_scalar{x, y, z, w};
}

constexpr mat4_scalar make_mat4(float m0, float m1, float m2, float m3,
                                float m4, float m5, float m6, float m7,
                                float m8, float m9, float m10, float m11,
                                float m12, float m13, float m14, float m15) {
  return mat4_scalar{{m0, m1, m2, m3},
                     {m4, m5, m6, m7},
                     {m8, m9, m10, m11},
                     {m12, m13, m14, m15}};
}

template <typename N>
mat4 make_mat4(const N& n) {
  return mat4(n);
}

template <typename X, typename Y, typename Z, typename W>
mat4 make_mat4(const X& x, const Y& y, const Z& z, const W& w) {
  return mat4(x, y, z, w);
}

SI mat3 if_then_else(I32 c, mat3 t, mat3 e) {
  return mat3{if_then_else(c, t[0], e[0]), if_then_else(c, t[1], e[1]),
              if_then_else(c, t[2], e[2])};
}

SI mat3 if_then_else(int32_t c, mat3 t, mat3 e) { return c ? t : e; }

SI mat4 if_then_else(I32 c, mat4 t, mat4 e) {
  return mat4{if_then_else(c, t[0], e[0]), if_then_else(c, t[1], e[1]),
              if_then_else(c, t[2], e[2]), if_then_else(c, t[3], e[3])};
}

SI mat4 if_then_else(int32_t c, mat4 t, mat4 e) { return c ? t : e; }

template <typename T, typename U, typename A,
          typename R = typename T::vector_type>
SI R mix(T x, U y, A a) {
  return (y - x) * a + x;
}

SI Float mix(Float x, Float y, Float a) { return (y - x) * a + x; }

template <typename T>
SI T mix(T x, T y, float a) {
  return (y - x) * a + x;
}

template <typename T>
SI T mix(T x, T y, vec2_scalar a) {
  return T{mix(x.x, y.x, a.x), mix(x.y, y.y, a.y)};
}

template <typename T>
SI T mix(T x, T y, vec3_scalar a) {
  return T{mix(x.x, y.x, a.x), mix(x.y, y.y, a.y), mix(x.z, y.z, a.z)};
}

template <typename T>
SI T mix(T x, T y, vec4_scalar a) {
  return T{mix(x.x, y.x, a.x), mix(x.y, y.y, a.y), mix(x.z, y.z, a.z),
           mix(x.w, y.w, a.w)};
}

ivec4 ivec2::sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
  return ivec4(select(c1), select(c2), select(c3), select(c4));
}

vec4 vec2::sel(XYZW c1, XYZW c2, XYZW c3, XYZW c4) {
  return vec4(select(c1), select(c2), select(c3), select(c4));
}

bool any(bool x) { return x; }

Bool any(bvec4 x) { return x.x | x.y | x.z | x.w; }

bool any(bvec4_scalar x) { return x.x | x.y | x.z | x.w; }

Bool any(bvec2 x) { return x.x | x.y; }

bool any(bvec2_scalar x) { return x.x | x.y; }

bool all(bool x) { return x; }

Bool all(bvec2 x) { return x.x & x.y; }

bool all(bvec2_scalar x) { return x.x & x.y; }

Bool all(bvec4 x) { return x.x & x.y & x.z & x.w; }

bool all(bvec4_scalar x) { return x.x & x.y & x.z & x.w; }

SI vec4 if_then_else(bvec4 c, vec4 t, vec4 e) {
  return vec4(if_then_else(c.x, t.x, e.x), if_then_else(c.y, t.y, e.y),
              if_then_else(c.z, t.z, e.z), if_then_else(c.w, t.w, e.w));
}
SI vec3 if_then_else(bvec3 c, vec3 t, vec3 e) {
  return vec3(if_then_else(c.x, t.x, e.x), if_then_else(c.y, t.y, e.y),
              if_then_else(c.z, t.z, e.z));
}

SI vec2 if_then_else(bvec2 c, vec2 t, vec2 e) {
  return vec2(if_then_else(c.x, t.x, e.x), if_then_else(c.y, t.y, e.y));
}

template <typename T, typename R = typename T::vector_type>
SI R mix(T x, T y, bvec4 a) {
  return if_then_else(a, y, x);
}

template <typename T, typename R = typename T::vector_type>
SI R mix(T x, T y, bvec3 a) {
  return if_then_else(a, y, x);
}

template <typename T, typename R = typename T::vector_type>
SI R mix(T x, T y, bvec2 a) {
  return if_then_else(a, y, x);
}

template <typename T>
SI T mix(T x, T y, bvec4_scalar a) {
  return T{a.x ? y.x : x.x, a.y ? y.y : x.y, a.z ? y.z : x.z, a.w ? y.w : x.w};
}

template <typename T>
SI T mix(T x, T y, bvec4_scalar1 a) {
  return a.x ? y : x;
}

template <typename T>
SI T mix(T x, T y, bvec3_scalar a) {
  return T{a.x ? y.x : x.x, a.y ? y.y : x.y, a.z ? y.z : x.z};
}

template <typename T>
SI T mix(T x, T y, bvec3_scalar1 a) {
  return a.x ? y : x;
}

template <typename T>
SI T mix(T x, T y, bvec2_scalar a) {
  return T{a.x ? y.x : x.x, a.y ? y.y : x.y};
}

template <typename T>
SI T mix(T x, T y, bvec2_scalar1 a) {
  return a.x ? y : x;
}

float dot(vec3_scalar a, vec3_scalar b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

Float dot(vec3 a, vec3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }

float dot(vec2_scalar a, vec2_scalar b) { return a.x * b.x + a.y * b.y; }

Float dot(vec2 a, vec2 b) { return a.x * b.x + a.y * b.y; }

#define sin __glsl_sin

float sin(float x) { return sinf(x); }

Float sin(Float v) { return {sinf(v.x), sinf(v.y), sinf(v.z), sinf(v.w)}; }

#define cos __glsl_cos

float cos(float x) { return cosf(x); }

Float cos(Float v) { return {cosf(v.x), cosf(v.y), cosf(v.z), cosf(v.w)}; }

#define tan __glsl_tan

float tan(float x) { return tanf(x); }

Float tan(Float v) { return {tanf(v.x), tanf(v.y), tanf(v.z), tanf(v.w)}; }

#define atan __glsl_atan

float atan(float x) { return atanf(x); }

Float atan(Float v) { return {atanf(v.x), atanf(v.y), atanf(v.z), atanf(v.w)}; }

float atan(float a, float b) { return atan2f(a, b); }

Float atan(Float a, Float b) {
  return {atan2f(a.x, b.x), atan2f(a.y, b.y), atan2f(a.z, b.z),
          atan2f(a.w, b.w)};
}

bvec4 equal(vec4 x, vec4 y) {
  return bvec4(equal(x.x, y.x), equal(x.y, y.y), equal(x.z, y.z),
               equal(x.w, y.w));
}

bvec4_scalar equal(vec4_scalar x, vec4_scalar y) {
  return bvec4_scalar(equal(x.x, y.x), equal(x.y, y.y), equal(x.z, y.z),
                      equal(x.w, y.w));
}

bvec4 notEqual(vec4 x, vec4 y) {
  return bvec4(notEqual(x.x, y.x), notEqual(x.y, y.y), notEqual(x.z, y.z),
               notEqual(x.w, y.w));
}

bvec4_scalar notEqual(vec4_scalar x, vec4_scalar y) {
  return bvec4_scalar(notEqual(x.x, y.x), notEqual(x.y, y.y),
                      notEqual(x.z, y.z), notEqual(x.w, y.w));
}

bvec4 notEqual(ivec4 a, ivec4 b) {
  return bvec4(a.x != b.x, a.y != b.y, a.z != b.z, a.w != b.w);
}

bvec4_scalar notEqual(ivec4_scalar a, ivec4_scalar b) {
  return bvec4_scalar{a.x != b.x, a.y != b.y, a.z != b.z, a.w != b.w};
}

mat3 transpose(mat3 m) {
  return mat3(vec3(m[0].x, m[1].x, m[2].x), vec3(m[0].y, m[1].y, m[2].y),
              vec3(m[0].z, m[1].z, m[2].z));
}

mat3_scalar transpose(mat3_scalar m) {
  return mat3_scalar{vec3_scalar(m[0].x, m[1].x, m[2].x),
                     vec3_scalar(m[0].y, m[1].y, m[2].y),
                     vec3_scalar(m[0].z, m[1].z, m[2].z)};
}

vec2 abs(vec2 v) { return vec2(abs(v.x), abs(v.y)); }

vec2_scalar abs(vec2_scalar v) { return vec2_scalar{fabsf(v.x), fabsf(v.y)}; }

vec2 sign(vec2 v) { return vec2(sign(v.x), sign(v.y)); }

vec2_scalar sign(vec2_scalar v) { return vec2_scalar{sign(v.x), sign(v.y)}; }

Float mod(Float a, Float b) { return a - b * floor(a / b); }

vec2 mod(vec2 a, vec2 b) { return vec2(mod(a.x, b.x), mod(a.y, b.y)); }

vec3 abs(vec3 v) { return vec3(abs(v.x), abs(v.y), abs(v.z)); }

vec3 sign(vec3 v) { return vec3(sign(v.x), sign(v.y), sign(v.z)); }

mat2 inverse(mat2 v) {
  Float det = v[0].x * v[1].y - v[0].y * v[1].x;
  return mat2(vec2(v[1].y, -v[0].y), vec2(-v[1].x, v[0].x)) * (1. / det);
}

mat2_scalar inverse(mat2_scalar v) {
  float det = v[0].x * v[1].y - v[0].y * v[1].x;
  return mat2_scalar{{v[1].y, -v[0].y}, {-v[1].x, v[0].x}} * (1. / det);
}

int32_t get_nth(I32 a, int n) { return a[n]; }

float get_nth(Float a, int n) { return a[n]; }

float get_nth(float a, int) { return a; }

ivec2_scalar get_nth(ivec2 a, int n) { return ivec2_scalar{a.x[n], a.y[n]}; }

vec2_scalar get_nth(vec2 a, int n) { return vec2_scalar{a.x[n], a.y[n]}; }

vec3_scalar get_nth(vec3 a, int n) {
  return vec3_scalar{a.x[n], a.y[n], a.z[n]};
}

vec4_scalar get_nth(vec4 a, int n) {
  return vec4_scalar{a.x[n], a.y[n], a.z[n], a.w[n]};
}

ivec4_scalar get_nth(ivec4 a, int n) {
  return ivec4_scalar{a.x[n], a.y[n], a.z[n], a.w[n]};
}

mat3_scalar get_nth(mat3 a, int n) {
  return make_mat3(get_nth(a[0], n), get_nth(a[1], n), get_nth(a[2], n));
}

void put_nth(Float& dst, int n, float src) { dst[n] = src; }

void put_nth(I32& dst, int n, int32_t src) { dst[n] = src; }

void put_nth(ivec2& dst, int n, ivec2_scalar src) {
  dst.x[n] = src.x;
  dst.y[n] = src.y;
}

void put_nth(vec2& dst, int n, vec2_scalar src) {
  dst.x[n] = src.x;
  dst.y[n] = src.y;
}

void put_nth(vec3& dst, int n, vec3_scalar src) {
  dst.x[n] = src.x;
  dst.y[n] = src.y;
  dst.z[n] = src.z;
}

void put_nth(ivec4& dst, int n, ivec4_scalar src) {
  dst.x[n] = src.x;
  dst.y[n] = src.y;
  dst.z[n] = src.z;
  dst.w[n] = src.w;
}

void put_nth(vec4& dst, int n, vec4_scalar src) {
  dst.x[n] = src.x;
  dst.y[n] = src.y;
  dst.z[n] = src.z;
  dst.w[n] = src.w;
}

// Use an ElementType type constructor
// so that we can implement element_type for
// Int and Float
template <typename V>
struct ElementType {
  typedef typename V::element_type ty;
};

template <>
struct ElementType<float> {
  typedef float ty;
};

template <>
struct ElementType<int> {
  typedef float ty;
};

template <>
struct ElementType<Float> {
  typedef float ty;
};

template <>
struct ElementType<I32> {
  typedef int32_t ty;
};

void put_nth_component(ivec2_scalar& dst, int n, int32_t src) {
  switch (n) {
    case 0:
      dst.x = src;
      break;
    case 1:
      dst.y = src;
      break;
  }
}

void put_nth_component(ivec4_scalar& dst, int n, int32_t src) {
  switch (n) {
    case 0:
      dst.x = src;
      break;
    case 1:
      dst.y = src;
      break;
    case 2:
      dst.z = src;
      break;
    case 3:
      dst.w = src;
      break;
  }
}

void put_nth_component(int& dst, int n, int src) {
  switch (n) {
    case 0:
      dst = src;
      break;
  }
}

void put_nth_component(float& dst, int n, float src) {
  switch (n) {
    case 0:
      dst = src;
      break;
  }
}

void put_nth_component(vec2_scalar& dst, int n, float src) {
  switch (n) {
    case 0:
      dst.x = src;
      break;
    case 1:
      dst.y = src;
      break;
  }
}

void put_nth_component(vec3_scalar& dst, int n, float src) {
  switch (n) {
    case 0:
      dst.x = src;
      break;
    case 1:
      dst.y = src;
      break;
    case 2:
      dst.z = src;
      break;
  }
}

void put_nth_component(vec4_scalar& dst, int n, float src) {
  switch (n) {
    case 0:
      dst.x = src;
      break;
    case 1:
      dst.y = src;
      break;
    case 2:
      dst.z = src;
      break;
    case 3:
      dst.w = src;
      break;
  }
}

Float init_interp(float init0, float step) {
  float init1 = init0 + step;
  float init2 = init1 + step;
  float init3 = init2 + step;
  return {init0, init1, init2, init3};
}

vec2 init_interp(vec2_scalar init, vec2_scalar step) {
  return vec2(init_interp(init.x, step.x), init_interp(init.y, step.y));
}

vec3 init_interp(vec3_scalar init, vec3_scalar step) {
  return vec3(init_interp(init.x, step.x), init_interp(init.y, step.y),
              init_interp(init.z, step.z));
}

vec4 init_interp(vec4_scalar init, vec4_scalar step) {
  return vec4(init_interp(init.x, step.x), init_interp(init.y, step.y),
              init_interp(init.z, step.z), init_interp(init.w, step.w));
}

template <typename T, size_t N>
struct Array {
  T elements[N];
  T& operator[](size_t i) { return elements[i]; }
  const T& operator[](size_t i) const { return elements[i]; }
  template <typename S>
  void convert(const Array<S, N>& s) {
    for (size_t i = 0; i < N; ++i) elements[i] = T(s[i]);
  }
};

template <size_t SIZE>
Array<vec2, SIZE> if_then_else(I32 c, Array<vec2, SIZE> t,
                               Array<vec2, SIZE> e) {
  Array<vec2, SIZE> r;
  for (size_t i = 0; i < SIZE; i++) {
    r[i] = if_then_else(c, t[i], e[i]);
  }
  return r;
}

}  // namespace glsl
