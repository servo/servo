/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* vim: set ts=2 sw=2 et tw=79: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_TypedArray_h
#define mozilla_dom_TypedArray_h

#include "jsfriendapi.h"

namespace mozilla {
namespace dom {

/*
 * Various typed array classes for argument conversion.  We have a base class
 * that has a way of initializing a TypedArray from an existing typed array, and
 * a subclass of the base class that supports creation of a relevant typed array
 * or array buffer object.
 */
template<typename T,
         JSObject* UnboxArray(JSContext*, JSObject*, uint32_t*, T**)>
struct TypedArray_base {
  TypedArray_base(JSContext* cx, JSObject* obj)
  {
    mObj = UnboxArray(cx, obj, &mLength, &mData);
  }

private:
  T* mData;
  uint32_t mLength;
  JSObject* mObj;

public:
  inline bool inited() const {
    return !!mObj;
  }

  inline T *Data() const {
    MOZ_ASSERT(inited());
    return mData;
  }

  inline uint32_t Length() const {
    MOZ_ASSERT(inited());
    return mLength;
  }

  inline JSObject *Obj() const {
    MOZ_ASSERT(inited());
    return mObj;
  }
};


template<typename T,
         T* GetData(JSObject*, JSContext*),
         JSObject* UnboxArray(JSContext*, JSObject*, uint32_t*, T**),
         JSObject* CreateNew(JSContext*, uint32_t)>
struct TypedArray : public TypedArray_base<T,UnboxArray> {
  TypedArray(JSContext* cx, JSObject* obj) :
    TypedArray_base<T,UnboxArray>(cx, obj)
  {}

  static inline JSObject*
  Create(JSContext* cx, nsWrapperCache* creator, uint32_t length,
         const T* data = NULL) {
    JSObject* creatorWrapper;
    Maybe<JSAutoCompartment> ac;
    if (creator && (creatorWrapper = creator->GetWrapperPreserveColor())) {
      ac.construct(cx, creatorWrapper);
    }
    JSObject* obj = CreateNew(cx, length);
    if (!obj) {
      return NULL;
    }
    if (data) {
      T* buf = static_cast<T*>(GetData(obj, cx));
      memcpy(buf, data, length*sizeof(T));
    }
    return obj;
  }
};

typedef TypedArray<int8_t, JS_GetInt8ArrayData, JS_GetObjectAsInt8Array,
                   JS_NewInt8Array>
        Int8Array;
typedef TypedArray<uint8_t, JS_GetUint8ArrayData,
                   JS_GetObjectAsUint8Array, JS_NewUint8Array>
        Uint8Array;
typedef TypedArray<uint8_t, JS_GetUint8ClampedArrayData,
                   JS_GetObjectAsUint8ClampedArray, JS_NewUint8ClampedArray>
        Uint8ClampedArray;
typedef TypedArray<int16_t, JS_GetInt16ArrayData,
                   JS_GetObjectAsInt16Array, JS_NewInt16Array>
        Int16Array;
typedef TypedArray<uint16_t, JS_GetUint16ArrayData,
                   JS_GetObjectAsUint16Array, JS_NewUint16Array>
        Uint16Array;
typedef TypedArray<int32_t, JS_GetInt32ArrayData,
                   JS_GetObjectAsInt32Array, JS_NewInt32Array>
        Int32Array;
typedef TypedArray<uint32_t, JS_GetUint32ArrayData,
                   JS_GetObjectAsUint32Array, JS_NewUint32Array>
        Uint32Array;
typedef TypedArray<float, JS_GetFloat32ArrayData,
                   JS_GetObjectAsFloat32Array, JS_NewFloat32Array>
        Float32Array;
typedef TypedArray<double, JS_GetFloat64ArrayData,
                   JS_GetObjectAsFloat64Array, JS_NewFloat64Array>
        Float64Array;
typedef TypedArray_base<uint8_t, JS_GetObjectAsArrayBufferView>
        ArrayBufferView;
typedef TypedArray<uint8_t, JS_GetArrayBufferData,
                   JS_GetObjectAsArrayBuffer, JS_NewArrayBuffer>
        ArrayBuffer;

} // namespace dom
} // namespace mozilla

#endif /* mozilla_dom_TypedArray_h */
