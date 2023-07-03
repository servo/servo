/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifdef __clang__
#  ifdef __SSE2__
#    include <xmmintrin.h>
#    define USE_SSE2 1
#  endif
#  ifdef __ARM_NEON
#    include <arm_neon.h>
#    define USE_NEON 1
#  endif
#endif

namespace glsl {

#ifdef __clang__
template <typename T, int N>
using VectorType = T __attribute__((ext_vector_type(N)));

#  define CONVERT(vector, type) __builtin_convertvector(vector, type)
#  define SHUFFLE(a, b, ...) __builtin_shufflevector(a, b, __VA_ARGS__)

template <typename T>
SI VectorType<T, 4> combine(VectorType<T, 2> a, VectorType<T, 2> b) {
  return __builtin_shufflevector(a, b, 0, 1, 2, 3);
}

template <typename T>
SI VectorType<T, 8> combine(VectorType<T, 4> a, VectorType<T, 4> b) {
  return __builtin_shufflevector(a, b, 0, 1, 2, 3, 4, 5, 6, 7);
}

template <typename T>
SI VectorType<T, 16> combine(VectorType<T, 8> a, VectorType<T, 8> b) {
  return __builtin_shufflevector(a, b, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
                                 13, 14, 15);
}

template <typename T>
SI VectorType<T, 4> lowHalf(VectorType<T, 8> a) {
  return __builtin_shufflevector(a, a, 0, 1, 2, 3);
}

template <typename T>
SI VectorType<T, 4> highHalf(VectorType<T, 8> a) {
  return __builtin_shufflevector(a, a, 4, 5, 6, 7);
}

template <typename T>
SI VectorType<T, 8> lowHalf(VectorType<T, 16> a) {
  return __builtin_shufflevector(a, a, 0, 1, 2, 3, 4, 5, 6, 7);
}

template <typename T>
SI VectorType<T, 8> highHalf(VectorType<T, 16> a) {
  return __builtin_shufflevector(a, a, 8, 9, 10, 11, 12, 13, 14, 15);
}

template <typename T>
SI VectorType<T, 8> expand(VectorType<T, 4> a) {
  return __builtin_shufflevector(a, a, 0, 1, 2, 3, -1, -1, -1, -1);
}
#else
template <typename T>
struct VectorMask {
  typedef T type;
};
template <>
struct VectorMask<uint32_t> {
  typedef int32_t type;
};
template <>
struct VectorMask<uint16_t> {
  typedef int16_t type;
};
template <>
struct VectorMask<uint8_t> {
  typedef int8_t type;
};
template <>
struct VectorMask<float> {
  typedef int type;
};

template <typename T, int N>
struct VectorType {
  enum { SIZE = N };

  typedef T data_type __attribute__((vector_size(sizeof(T) * N)));
  typedef typename VectorMask<T>::type mask_index;
  typedef mask_index mask_type
      __attribute__((vector_size(sizeof(mask_index) * N)));
  typedef T half_type __attribute__((vector_size(sizeof(T) * (N / 2))));
  union {
    data_type data;
    struct {
      T x, y, z, w;
    };
    T elements[N];
    struct {
      half_type low_half, high_half;
    };
  };

  VectorType() : data{0} { }

  constexpr VectorType(const VectorType& rhs) : data(rhs.data) {}
  // GCC vector extensions only support broadcasting scalars on arithmetic ops,
  // but not on initializers, hence the following...
  constexpr VectorType(T n) : data((data_type){0} + n) {}
  constexpr VectorType(T a, T b, T c, T d) : data{a, b, c, d} {}
  constexpr VectorType(T a, T b, T c, T d, T e, T f, T g, T h)
      : data{a, b, c, d, e, f, g, h} {}
  constexpr VectorType(T a, T b, T c, T d, T e, T f, T g, T h, T i, T j, T k,
                       T l, T m, T n, T o, T p)
      : data{a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p} {}

  SI VectorType wrap(const data_type& data) {
    VectorType v;
    v.data = data;
    return v;
  }

  T& operator[](size_t i) { return elements[i]; }
  T operator[](size_t i) const { return elements[i]; }

  template <typename U>
  operator VectorType<U, 2>() const {
    return VectorType<U, 2>::wrap(
        (typename VectorType<U, N>::data_type){U(x), U(y)});
  }
  template <typename U>
  operator VectorType<U, 4>() const {
    return VectorType<U, 4>::wrap(
        (typename VectorType<U, N>::data_type){U(x), U(y), U(z), U(w)});
  }
  template <typename U>
  operator VectorType<U, 8>() const {
    return VectorType<U, 8>::wrap((typename VectorType<U, N>::data_type){
        U(elements[0]), U(elements[1]), U(elements[2]), U(elements[3]),
        U(elements[4]), U(elements[5]), U(elements[6]), U(elements[7])});
  }
  template <typename U>
  operator VectorType<U, 16>() const {
    return VectorType<U, 16>::wrap((typename VectorType<U, N>::data_type){
        U(elements[0]),
        U(elements[1]),
        U(elements[2]),
        U(elements[3]),
        U(elements[4]),
        U(elements[5]),
        U(elements[6]),
        U(elements[7]),
        U(elements[8]),
        U(elements[9]),
        U(elements[10]),
        U(elements[11]),
        U(elements[12]),
        U(elements[13]),
        U(elements[14]),
        U(elements[15]),
    });
  }

  VectorType operator-() const { return wrap(-data); }
  VectorType operator~() const { return wrap(~data); }

  VectorType operator&(VectorType x) const { return wrap(data & x.data); }
  VectorType operator&(T x) const { return wrap(data & x); }
  VectorType operator|(VectorType x) const { return wrap(data | x.data); }
  VectorType operator|(T x) const { return wrap(data | x); }
  VectorType operator^(VectorType x) const { return wrap(data ^ x.data); }
  VectorType operator^(T x) const { return wrap(data ^ x); }
  VectorType operator<<(int x) const { return wrap(data << x); }
  VectorType operator>>(int x) const { return wrap(data >> x); }
  VectorType operator+(VectorType x) const { return wrap(data + x.data); }
  VectorType operator+(T x) const { return wrap(data + x); }
  friend VectorType operator+(T x, VectorType y) { return wrap(x + y.data); }
  VectorType operator-(VectorType x) const { return wrap(data - x.data); }
  VectorType operator-(T x) const { return wrap(data - x); }
  friend VectorType operator-(T x, VectorType y) { return wrap(x - y.data); }
  VectorType operator*(VectorType x) const { return wrap(data * x.data); }
  VectorType operator*(T x) const { return wrap(data * x); }
  friend VectorType operator*(T x, VectorType y) { return wrap(x * y.data); }
  VectorType operator/(VectorType x) const { return wrap(data / x.data); }
  VectorType operator/(T x) const { return wrap(data / x); }
  friend VectorType operator/(T x, VectorType y) { return wrap(x / y.data); }
  VectorType operator%(int x) const { return wrap(data % x); }

  VectorType& operator&=(VectorType x) {
    data &= x.data;
    return *this;
  }
  VectorType& operator|=(VectorType x) {
    data |= x.data;
    return *this;
  }
  VectorType& operator^=(VectorType x) {
    data ^= x.data;
    return *this;
  }
  VectorType& operator<<=(int x) {
    data <<= x;
    return *this;
  }
  VectorType& operator>>=(int x) {
    data >>= x;
    return *this;
  }
  VectorType& operator+=(VectorType x) {
    data += x.data;
    return *this;
  }
  VectorType& operator-=(VectorType x) {
    data -= x.data;
    return *this;
  }
  VectorType& operator*=(VectorType x) {
    data *= x.data;
    return *this;
  }
  VectorType& operator/=(VectorType x) {
    data /= x.data;
    return *this;
  }
  VectorType& operator%=(int x) {
    data %= x;
    return *this;
  }

  VectorType<mask_type, N> operator==(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data == x.data);
  }
  VectorType<mask_type, N> operator!=(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data != x.data);
  }
  VectorType<mask_type, N> operator<(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data < x.data);
  }
  VectorType<mask_type, N> operator>(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data > x.data);
  }
  VectorType<mask_type, N> operator<=(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data <= x.data);
  }
  VectorType<mask_type, N> operator>=(VectorType x) const {
    return VectorType<mask_type, N>::wrap(data >= x.data);
  }

  VectorType operator!() const { return wrap(!data); }
  VectorType operator&&(VectorType x) const { return wrap(data & x.data); }
  VectorType operator||(VectorType x) const { return wrap(data | x.data); }

  VectorType& operator=(VectorType x) {
    data = x.data;
    return *this;
  }

  VectorType<T, 4> shuffle(VectorType b, mask_index x, mask_index y,
                           mask_index z, mask_index w) const {
    return VectorType<T, 4>::wrap(__builtin_shuffle(
        data, b.data, (typename VectorType<T, 4>::mask_type){x, y, z, w}));
  }
  VectorType<T, 8> shuffle(VectorType b, mask_index x, mask_index y,
                           mask_index z, mask_index w, mask_index s,
                           mask_index t, mask_index u, mask_index v) const {
    return VectorType<T, 8>::wrap(__builtin_shuffle(
        data, b.data,
        (typename VectorType<T, 8>::mask_type){x, y, z, w, s, t, u, v}));
  }
  VectorType<T, 16> shuffle(VectorType b, mask_index x, mask_index y,
                            mask_index z, mask_index w, mask_index s,
                            mask_index t, mask_index u, mask_index v,
                            mask_index i, mask_index j, mask_index k,
                            mask_index l, mask_index m, mask_index n,
                            mask_index o, mask_index p) const {
    return VectorType<T, 16>::wrap(
        __builtin_shuffle(data, b.data,
                          (typename VectorType<T, 16>::mask_type){
                              x, y, z, w, s, t, u, v, i, j, k, l, m, n, o, p}));
  }

  VectorType<T, 4> swizzle(mask_index x, mask_index y, mask_index z,
                           mask_index w) const {
    return VectorType<T, 4>::wrap(__builtin_shuffle(
        data, (typename VectorType<T, 4>::mask_type){x, y, z, w}));
  }
  VectorType<T, 8> swizzle(mask_index x, mask_index y, mask_index z,
                           mask_index w, mask_index s, mask_index t,
                           mask_index u, mask_index v) const {
    return VectorType<T, 8>::wrap(__builtin_shuffle(
        data, (typename VectorType<T, 8>::mask_type){x, y, z, w, s, t, u, v}));
  }

  SI VectorType wrap(half_type low, half_type high) {
    VectorType v;
    v.low_half = low;
    v.high_half = high;
    return v;
  }

  VectorType<T, N * 2> combine(VectorType high) const {
    return VectorType<T, N * 2>::wrap(data, high.data);
  }

#  define xyxy swizzle(0, 1, 0, 1)
#  define zwzw swizzle(2, 3, 2, 3)
#  define zyxw swizzle(2, 1, 0, 3)
#  define xyzz swizzle(0, 1, 2, 2)
#  define xxxxyyyy XXXXYYYY()
  VectorType<T, 8> XXXXYYYY() const {
    return swizzle(0, 0, 0, 0).combine(swizzle(1, 1, 1, 1));
  }
#  define zzzzwwww ZZZZWWWW()
  VectorType<T, 8> ZZZZWWWW() const {
    return swizzle(2, 2, 2, 2).combine(swizzle(3, 3, 3, 3));
  }
#  define xyzwxyzw XYZWXYZW()
  VectorType<T, 8> XYZWXYZW() const { return combine(*this); }
#  define xyxyxyxy XYXYXYXY()
  VectorType<T, 8> XYXYXYXY() const {
    return swizzle(0, 1, 0, 1).combine(swizzle(0, 1, 0, 1));
  }
#  define zwzwzwzw ZWZWZWZW()
  VectorType<T, 8> ZWZWZWZW() const {
    return swizzle(2, 3, 2, 3).combine(swizzle(2, 3, 2, 3));
  }
#  define xxyyzzww XXYYZZWW()
  VectorType<T, 8> XXYYZZWW() const {
    return swizzle(0, 0, 1, 1).combine(swizzle(2, 2, 3, 3));
  }
};

template <typename T>
struct VectorType<T, 2> {
  typedef T data_type __attribute__((vector_size(sizeof(T) * 2)));
  union {
    data_type data;
    struct {
      T x, y;
    };
    T elements[2];
  };
};

#  define CONVERT(vector, type) ((type)(vector))
#  define SHUFFLE(a, b, ...) a.shuffle(b, __VA_ARGS__)

template <typename T, int N>
SI VectorType<T, N * 2> combine(VectorType<T, N> a, VectorType<T, N> b) {
  return VectorType<T, N * 2>::wrap(a.data, b.data);
}

template <typename T, int N>
SI VectorType<T, N / 2> lowHalf(VectorType<T, N> a) {
  return VectorType<T, N / 2>::wrap(a.low_half);
}

template <typename T, int N>
SI VectorType<T, N / 2> highHalf(VectorType<T, N> a) {
  return VectorType<T, N / 2>::wrap(a.high_half);
}

template <typename T, int N>
SI VectorType<T, N * 2> expand(VectorType<T, N> a) {
  return combine(a, a);
}
#endif

template <typename T>
SI VectorType<T, 4> zipLow(VectorType<T, 4> a, VectorType<T, 4> b) {
  return SHUFFLE(a, b, 0, 4, 1, 5);
}

template <typename T>
SI VectorType<T, 4> zipHigh(VectorType<T, 4> a, VectorType<T, 4> b) {
  return SHUFFLE(a, b, 2, 6, 3, 7);
}

template <typename T>
SI VectorType<T, 8> zipLow(VectorType<T, 8> a, VectorType<T, 8> b) {
  return SHUFFLE(a, b, 0, 8, 1, 9, 2, 10, 3, 11);
}

template <typename T>
SI VectorType<T, 8> zipHigh(VectorType<T, 8> a, VectorType<T, 8> b) {
  return SHUFFLE(a, b, 4, 12, 5, 13, 6, 14, 7, 15);
}

template <typename T>
SI VectorType<T, 16> zipLow(VectorType<T, 16> a, VectorType<T, 16> b) {
  return SHUFFLE(a, b, 0, 1, 2, 3, 4, 5, 6, 7, 16, 17, 18, 19, 20, 21, 22, 23);
}

template <typename T>
SI VectorType<T, 16> zipHigh(VectorType<T, 16> a, VectorType<T, 16> b) {
  return SHUFFLE(a, b, 8, 9, 10, 11, 12, 13, 14, 15, 24, 25, 26, 27, 28, 29, 30,
                 31);
}

template <typename T>
SI VectorType<T, 8> zip2Low(VectorType<T, 8> a, VectorType<T, 8> b) {
  return SHUFFLE(a, b, 0, 1, 8, 9, 2, 3, 10, 11);
}

template <typename T>
SI VectorType<T, 8> zip2High(VectorType<T, 8> a, VectorType<T, 8> b) {
  return SHUFFLE(a, b, 4, 5, 12, 13, 6, 7, 14, 15);
}

template <typename T>
struct Unaligned {
  template <typename P>
  SI T load(const P* p) {
    T v;
    memcpy(&v, p, sizeof(v));
    return v;
  }

  template <typename P>
  SI void store(P* p, T v) {
    memcpy(p, &v, sizeof(v));
  }
};

#ifndef __clang__
template <typename T, int N>
struct Unaligned<VectorType<T, N>> {
  template <typename P>
  SI VectorType<T, N> load(const P* p) {
    VectorType<T, N> v;
    memcpy(v.elements, p, sizeof(v));
    return v;
  }

  template <typename P>
  SI void store(P* p, VectorType<T, N> v) {
    memcpy(p, v.elements, sizeof(v));
  }
};
#endif

template <typename T, typename P>
SI T unaligned_load(const P* p) {
  return Unaligned<T>::load(p);
}

template <typename T, typename P>
SI void unaligned_store(P* p, T v) {
  Unaligned<T>::store(p, v);
}

template <typename D, typename S>
SI D bit_cast(const S& src) {
  static_assert(sizeof(D) == sizeof(S), "");
  return unaligned_load<D>(&src);
}

template <typename T>
using V2 = VectorType<T, 2>;
template <typename T>
using V4 = VectorType<T, 4>;
using Float = V4<float>;
using I32 = V4<int32_t>;
using I16 = V4<int16_t>;
using U64 = V4<uint64_t>;
using U32 = V4<uint32_t>;
using U16 = V4<uint16_t>;
using U8 = V4<uint8_t>;
using Bool = V4<int>;
template <typename T>
using V8 = VectorType<T, 8>;
template <typename T>
using V16 = VectorType<T, 16>;

}  // namespace glsl
