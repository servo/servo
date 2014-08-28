/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* vim: set ts=2 sw=2 et tw=79: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
 * Conversions from jsval to primitive values
 */

#ifndef mozilla_dom_PrimitiveConversions_h
#define mozilla_dom_PrimitiveConversions_h

#include <limits>
#include <math.h>
#include "mozilla/Assertions.h"
#include "mozilla/dom/BindingUtils.h"
#include "mozilla/FloatingPoint.h"
#include "xpcpublic.h"

namespace mozilla {
namespace dom {

template<typename T>
struct TypeName {
};

template<>
struct TypeName<int8_t> {
  static const char* value() {
    return "byte";
  }
};
template<>
struct TypeName<uint8_t> {
  static const char* value() {
    return "octet";
  }
};
template<>
struct TypeName<int16_t> {
  static const char* value() {
    return "short";
  }
};
template<>
struct TypeName<uint16_t> {
  static const char* value() {
    return "unsigned short";
  }
};
template<>
struct TypeName<int32_t> {
  static const char* value() {
    return "long";
  }
};
template<>
struct TypeName<uint32_t> {
  static const char* value() {
    return "unsigned long";
  }
};
template<>
struct TypeName<int64_t> {
  static const char* value() {
    return "long long";
  }
};
template<>
struct TypeName<uint64_t> {
  static const char* value() {
    return "unsigned long long";
  }
};


enum ConversionBehavior {
  eDefault,
  eEnforceRange,
  eClamp
};

template<typename T, ConversionBehavior B>
struct PrimitiveConversionTraits {
};

template<typename T>
struct DisallowedConversion {
  typedef int jstype;
  typedef int intermediateType;

private:
  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    MOZ_NOT_REACHED("This should never be instantiated!");
    return false;
  }
};

struct PrimitiveConversionTraits_smallInt {
  // The output of JS::ToInt32 is determined as follows:
  //   1) The value is converted to a double
  //   2) Anything that's not a finite double returns 0
  //   3) The double is rounded towards zero to the nearest integer
  //   4) The resulting integer is reduced mod 2^32.  The output of this
  //      operation is an integer in the range [0, 2^32).
  //   5) If the resulting number is >= 2^31, 2^32 is subtracted from it.
  //
  // The result of all this is a number in the range [-2^31, 2^31)
  //
  // WebIDL conversions for the 8-bit, 16-bit, and 32-bit integer types
  // are defined in the same way, except that step 4 uses reduction mod
  // 2^8 and 2^16 for the 8-bit and 16-bit types respectively, and step 5
  // is only done for the signed types.
  //
  // C/C++ define integer conversion semantics to unsigned types as taking
  // your input integer mod (1 + largest value representable in the
  // unsigned type).  Since 2^32 is zero mod 2^8, 2^16, and 2^32,
  // converting to the unsigned int of the relevant width will correctly
  // perform step 4; in particular, the 2^32 possibly subtracted in step 5
  // will become 0.
  //
  // Once we have step 4 done, we're just going to assume 2s-complement
  // representation and cast directly to the type we really want.
  //
  // So we can cast directly for all unsigned types and for int32_t; for
  // the smaller-width signed types we need to cast through the
  // corresponding unsigned type.
  typedef int32_t jstype;
  typedef int32_t intermediateType;
  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    return JS::ToInt32(cx, v, retval);
  }
};
template<>
struct PrimitiveConversionTraits<int8_t, eDefault> : PrimitiveConversionTraits_smallInt {
  typedef uint8_t intermediateType;
};
template<>
struct PrimitiveConversionTraits<uint8_t, eDefault> : PrimitiveConversionTraits_smallInt {
};
template<>
struct PrimitiveConversionTraits<int16_t, eDefault> : PrimitiveConversionTraits_smallInt {
  typedef uint16_t intermediateType;
};
template<>
struct PrimitiveConversionTraits<uint16_t, eDefault> : PrimitiveConversionTraits_smallInt {
};
template<>
struct PrimitiveConversionTraits<int32_t, eDefault> : PrimitiveConversionTraits_smallInt {
};
template<>
struct PrimitiveConversionTraits<uint32_t, eDefault> : PrimitiveConversionTraits_smallInt {
};

template<>
struct PrimitiveConversionTraits<int64_t, eDefault> {
  typedef int64_t jstype;
  typedef int64_t intermediateType;
  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    return JS::ToInt64(cx, v, retval);
  }
};

template<>
struct PrimitiveConversionTraits<uint64_t, eDefault> {
  typedef uint64_t jstype;
  typedef uint64_t intermediateType;
  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    return JS::ToUint64(cx, v, retval);
  }
};

template<typename T>
struct PrimitiveConversionTraits_Limits {
  static inline T min() {
    return std::numeric_limits<T>::min();
  }
  static inline T max() {
    return std::numeric_limits<T>::max();
  }
};

template<>
struct PrimitiveConversionTraits_Limits<int64_t> {
  static inline int64_t min() {
    return -(1LL << 53) + 1;
  }
  static inline int64_t max() {
    return (1LL << 53) - 1;
  }
};

template<>
struct PrimitiveConversionTraits_Limits<uint64_t> {
  static inline uint64_t min() {
    return 0;
  }
  static inline uint64_t max() {
    return (1LL << 53) - 1;
  }
};

template<typename T, bool (*Enforce)(JSContext* cx, const double& d, T* retval)>
struct PrimitiveConversionTraits_ToCheckedIntHelper {
  typedef T jstype;
  typedef T intermediateType;

  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    double intermediate;
    if (!JS::ToNumber(cx, v, &intermediate)) {
      return false;
    }

    return Enforce(cx, intermediate, retval);
  }
};

template<typename T>
inline bool
PrimitiveConversionTraits_EnforceRange(JSContext* cx, const double& d, T* retval)
{
  MOZ_STATIC_ASSERT(std::numeric_limits<T>::is_integer,
                    "This can only be applied to integers!");

  if (!MOZ_DOUBLE_IS_FINITE(d)) {
    return ThrowErrorMessage(cx, MSG_ENFORCE_RANGE_NON_FINITE, TypeName<T>::value());
  }

  bool neg = (d < 0);
  double rounded = floor(neg ? -d : d);
  rounded = neg ? -rounded : rounded;
  if (rounded < PrimitiveConversionTraits_Limits<T>::min() ||
      rounded > PrimitiveConversionTraits_Limits<T>::max()) {
    return ThrowErrorMessage(cx, MSG_ENFORCE_RANGE_OUT_OF_RANGE, TypeName<T>::value());
  }

  *retval = static_cast<T>(rounded);
  return true;
}

template<typename T>
struct PrimitiveConversionTraits<T, eEnforceRange> :
  public PrimitiveConversionTraits_ToCheckedIntHelper<T, PrimitiveConversionTraits_EnforceRange<T> > {
};

template<typename T>
inline bool
PrimitiveConversionTraits_Clamp(JSContext* cx, const double& d, T* retval)
{
  MOZ_STATIC_ASSERT(std::numeric_limits<T>::is_integer,
                    "This can only be applied to integers!");

  if (MOZ_DOUBLE_IS_NaN(d)) {
    *retval = 0;
    return true;
  }
  if (d >= PrimitiveConversionTraits_Limits<T>::max()) {
    *retval = PrimitiveConversionTraits_Limits<T>::max();
    return true;
  }
  if (d <= PrimitiveConversionTraits_Limits<T>::min()) {
    *retval = PrimitiveConversionTraits_Limits<T>::min();
    return true;
  }

  MOZ_ASSERT(MOZ_DOUBLE_IS_FINITE(d));

  // Banker's rounding (round ties towards even).
  // We move away from 0 by 0.5f and then truncate.  That gets us the right
  // answer for any starting value except plus or minus N.5.  With a starting
  // value of that form, we now have plus or minus N+1.  If N is odd, this is
  // the correct result.  If N is even, plus or minus N is the correct result.
  double toTruncate = (d < 0) ? d - 0.5 : d + 0.5;

  T truncated(toTruncate);

  if (truncated == toTruncate) {
    /*
     * It was a tie (since moving away from 0 by 0.5 gave us the exact integer
     * we want). Since we rounded away from 0, we either already have an even
     * number or we have an odd number but the number we want is one closer to
     * 0. So just unconditionally masking out the ones bit should do the trick
     * to get us the value we want.
     */
    truncated &= ~1;
  }

  *retval = truncated;
  return true;
}

template<typename T>
struct PrimitiveConversionTraits<T, eClamp> :
  public PrimitiveConversionTraits_ToCheckedIntHelper<T, PrimitiveConversionTraits_Clamp<T> > {
};


template<ConversionBehavior B>
struct PrimitiveConversionTraits<bool, B> : public DisallowedConversion<bool> {};

template<>
struct PrimitiveConversionTraits<bool, eDefault> {
  typedef JSBool jstype;
  typedef bool intermediateType;
  static inline bool converter(JSContext* /* unused */, JS::Value v, jstype* retval) {
    *retval = JS::ToBoolean(v);
    return true;
  }
};


template<ConversionBehavior B>
struct PrimitiveConversionTraits<float, B> : public DisallowedConversion<float> {};

template<ConversionBehavior B>
struct PrimitiveConversionTraits<double, B> : public DisallowedConversion<double> {};

struct PrimitiveConversionTraits_float {
  typedef double jstype;
  typedef double intermediateType;
  static inline bool converter(JSContext* cx, JS::Value v, jstype* retval) {
    return JS::ToNumber(cx, v, retval);
  }
};

template<>
struct PrimitiveConversionTraits<float, eDefault> : PrimitiveConversionTraits_float {
};
template<>
struct PrimitiveConversionTraits<double, eDefault> : PrimitiveConversionTraits_float {
};


template<typename T, ConversionBehavior B>
bool ValueToPrimitive(JSContext* cx, JS::Value v, T* retval)
{
  typename PrimitiveConversionTraits<T, B>::jstype t;
  if (!PrimitiveConversionTraits<T, B>::converter(cx, v, &t))
    return false;

  *retval =
    static_cast<typename PrimitiveConversionTraits<T, B>::intermediateType>(t);
  return true;
}

} // namespace dom
} // namespace mozilla

#endif /* mozilla_dom_PrimitiveConversions_h */
