/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* vim: set ts=2 sw=2 et tw=79: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_BindingUtils_h__
#define mozilla_dom_BindingUtils_h__

#include "mozilla/dom/DOMJSClass.h"
#include "mozilla/dom/DOMJSProxyHandler.h"
#include "mozilla/dom/workers/Workers.h"
#include "mozilla/ErrorResult.h"

#include "jsapi.h"
#include "jsfriendapi.h"
#include "jswrapper.h"

#include "nsIXPConnect.h"
#include "qsObjectHelper.h"
#include "xpcpublic.h"
#include "nsTraceRefcnt.h"
#include "nsWrapperCacheInlines.h"
#include "mozilla/Likely.h"

// nsGlobalWindow implements nsWrapperCache, but doesn't always use it. Don't
// try to use it without fixing that first.
class nsGlobalWindow;

namespace mozilla {
namespace dom {

enum ErrNum {
#define MSG_DEF(_name, _argc, _str) \
  _name,
#include "mozilla/dom/Errors.msg"
#undef MSG_DEF
  Err_Limit
};

bool
ThrowErrorMessage(JSContext* aCx, const ErrNum aErrorNumber, ...);

template<bool mainThread>
inline bool
Throw(JSContext* cx, nsresult rv)
{
  using mozilla::dom::workers::exceptions::ThrowDOMExceptionForNSResult;

  // XXX Introduce exception machinery.
  if (mainThread) {
    xpc::Throw(cx, rv);
  } else {
    if (!JS_IsExceptionPending(cx)) {
      ThrowDOMExceptionForNSResult(cx, rv);
    }
  }
  return false;
}

template<bool mainThread>
inline bool
ThrowMethodFailedWithDetails(JSContext* cx, const ErrorResult& rv,
                             const char* /* ifaceName */,
                             const char* /* memberName */)
{
  return Throw<mainThread>(cx, rv.ErrorCode());
}

inline bool
IsDOMClass(const JSClass* clasp)
{
  return clasp->flags & JSCLASS_IS_DOMJSCLASS;
}

inline bool
IsDOMClass(const js::Class* clasp)
{
  return IsDOMClass(Jsvalify(clasp));
}

// It's ok for eRegularDOMObject and eProxyDOMObject to be the same, but
// eNonDOMObject should always be different from the other two. This enum
// shouldn't be used to differentiate between non-proxy and proxy bindings.
enum DOMObjectSlot {
  eNonDOMObject = -1,
  eRegularDOMObject = DOM_OBJECT_SLOT,
  eProxyDOMObject = DOM_PROXY_OBJECT_SLOT
};

template <class T>
inline T*
UnwrapDOMObject(JSObject* obj, DOMObjectSlot slot)
{
  MOZ_ASSERT(slot != eNonDOMObject,
             "Don't pass non-DOM objects to this function");

#ifdef DEBUG
  if (IsDOMClass(js::GetObjectClass(obj))) {
    MOZ_ASSERT(slot == eRegularDOMObject);
  } else {
    MOZ_ASSERT(js::IsObjectProxyClass(js::GetObjectClass(obj)) ||
               js::IsFunctionProxyClass(js::GetObjectClass(obj)));
    MOZ_ASSERT(js::GetProxyHandler(obj)->family() == ProxyFamily());
    MOZ_ASSERT(IsNewProxyBinding(js::GetProxyHandler(obj)));
    MOZ_ASSERT(slot == eProxyDOMObject);
  }
#endif

  JS::Value val = js::GetReservedSlot(obj, slot);
  // XXXbz/khuey worker code tries to unwrap interface objects (which have
  // nothing here).  That needs to stop.
  // XXX We don't null-check UnwrapObject's result; aren't we going to crash
  // anyway?
  if (val.isUndefined()) {
    return NULL;
  }
  
  return static_cast<T*>(val.toPrivate());
}

// Only use this with a new DOM binding object (either proxy or regular).
inline const DOMClass*
GetDOMClass(JSObject* obj)
{
  js::Class* clasp = js::GetObjectClass(obj);
  if (IsDOMClass(clasp)) {
    return &DOMJSClass::FromJSClass(clasp)->mClass;
  }

  js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
  MOZ_ASSERT(handler->family() == ProxyFamily());
  MOZ_ASSERT(IsNewProxyBinding(handler));
  return &static_cast<DOMProxyHandler*>(handler)->mClass;
}

inline DOMObjectSlot
GetDOMClass(JSObject* obj, const DOMClass*& result)
{
  js::Class* clasp = js::GetObjectClass(obj);
  if (IsDOMClass(clasp)) {
    result = &DOMJSClass::FromJSClass(clasp)->mClass;
    return eRegularDOMObject;
  }

  if (js::IsObjectProxyClass(clasp) || js::IsFunctionProxyClass(clasp)) {
    js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
    if (handler->family() == ProxyFamily() && IsNewProxyBinding(handler)) {
      result = &static_cast<DOMProxyHandler*>(handler)->mClass;
      return eProxyDOMObject;
    }
  }

  return eNonDOMObject;
}

inline bool
UnwrapDOMObjectToISupports(JSObject* obj, nsISupports*& result)
{
  const DOMClass* clasp;
  DOMObjectSlot slot = GetDOMClass(obj, clasp);
  if (slot == eNonDOMObject || !clasp->mDOMObjectIsISupports) {
    return false;
  }
 
  result = UnwrapDOMObject<nsISupports>(obj, slot);
  return true;
}

inline bool
IsDOMObject(JSObject* obj)
{
  js::Class* clasp = js::GetObjectClass(obj);
  return IsDOMClass(clasp) ||
         ((js::IsObjectProxyClass(clasp) || js::IsFunctionProxyClass(clasp)) &&
          (js::GetProxyHandler(obj)->family() == ProxyFamily() &&
           IsNewProxyBinding(js::GetProxyHandler(obj))));
}

// Some callers don't want to set an exception when unwrapping fails
// (for example, overload resolution uses unwrapping to tell what sort
// of thing it's looking at).
// U must be something that a T* can be assigned to (e.g. T* or an nsRefPtr<T>).
template <prototypes::ID PrototypeID, class T, typename U>
inline nsresult
UnwrapObject(JSContext* cx, JSObject* obj, U& value)
{
  /* First check to see whether we have a DOM object */
  const DOMClass* domClass;
  DOMObjectSlot slot = GetDOMClass(obj, domClass);
  if (slot == eNonDOMObject) {
    /* Maybe we have a security wrapper or outer window? */
    if (!js::IsWrapper(obj)) {
      /* Not a DOM object, not a wrapper, just bail */
      return NS_ERROR_XPC_BAD_CONVERT_JS;
    }

    obj = xpc::Unwrap(cx, obj, false);
    if (!obj) {
      return NS_ERROR_XPC_SECURITY_MANAGER_VETO;
    }
    MOZ_ASSERT(!js::IsWrapper(obj));
    slot = GetDOMClass(obj, domClass);
    if (slot == eNonDOMObject) {
      /* We don't have a DOM object */
      return NS_ERROR_XPC_BAD_CONVERT_JS;
    }
  }

  /* This object is a DOM object.  Double-check that it is safely
     castable to T by checking whether it claims to inherit from the
     class identified by protoID. */
  if (domClass->mInterfaceChain[PrototypeTraits<PrototypeID>::Depth] ==
      PrototypeID) {
    value = UnwrapDOMObject<T>(obj, slot);
    return NS_OK;
  }

  /* It's the wrong sort of DOM object */
  return NS_ERROR_XPC_BAD_CONVERT_JS;
}

inline bool
IsArrayLike(JSContext* cx, JSObject* obj)
{
  MOZ_ASSERT(obj);
  // For simplicity, check for security wrappers up front.  In case we
  // have a security wrapper, don't forget to enter the compartment of
  // the underlying object after unwrapping.
  Maybe<JSAutoCompartment> ac;
  if (js::IsWrapper(obj)) {
    obj = xpc::Unwrap(cx, obj, false);
    if (!obj) {
      // Let's say it's not
      return false;
    }

    ac.construct(cx, obj);
  }

  // XXXbz need to detect platform objects (including listbinding
  // ones) with indexGetters here!
  return JS_IsArrayObject(cx, obj) || JS_IsTypedArrayObject(obj, cx);
}

inline bool
IsPlatformObject(JSContext* cx, JSObject* obj)
{
  // XXXbz Should be treating list-binding objects as platform objects
  // too?  The one consumer so far wants non-array-like platform
  // objects, so listbindings that have an indexGetter should test
  // false from here.  Maybe this function should have a different
  // name?
  MOZ_ASSERT(obj);
  // Fast-path the common case
  JSClass* clasp = js::GetObjectJSClass(obj);
  if (IsDOMClass(clasp)) {
    return true;
  }
  // Now for simplicity check for security wrappers before anything else
  if (js::IsWrapper(obj)) {
    obj = xpc::Unwrap(cx, obj, false);
    if (!obj) {
      // Let's say it's not
      return false;
    }
    clasp = js::GetObjectJSClass(obj);
  }
  return IS_WRAPPER_CLASS(js::Valueify(clasp)) || IsDOMClass(clasp) ||
    JS_IsArrayBufferObject(obj, cx);
}

// U must be something that a T* can be assigned to (e.g. T* or an nsRefPtr<T>).
template <class T, typename U>
inline nsresult
UnwrapObject(JSContext* cx, JSObject* obj, U& value)
{
  return UnwrapObject<static_cast<prototypes::ID>(
           PrototypeIDMap<T>::PrototypeID), T>(cx, obj, value);
}

const size_t kProtoOrIfaceCacheCount =
  prototypes::id::_ID_Count + constructors::id::_ID_Count;

inline void
AllocateProtoOrIfaceCache(JSObject* obj)
{
  MOZ_ASSERT(js::GetObjectClass(obj)->flags & JSCLASS_DOM_GLOBAL);
  MOZ_ASSERT(js::GetReservedSlot(obj, DOM_PROTOTYPE_SLOT).isUndefined());

  // Important: The () at the end ensure zero-initialization
  JSObject** protoOrIfaceArray = new JSObject*[kProtoOrIfaceCacheCount]();

  js::SetReservedSlot(obj, DOM_PROTOTYPE_SLOT,
                      JS::PrivateValue(protoOrIfaceArray));
}

inline void
TraceProtoOrIfaceCache(JSTracer* trc, JSObject* obj)
{
  MOZ_ASSERT(js::GetObjectClass(obj)->flags & JSCLASS_DOM_GLOBAL);

  if (!HasProtoOrIfaceArray(obj))
    return;
  JSObject** protoOrIfaceArray = GetProtoOrIfaceArray(obj);
  for (size_t i = 0; i < kProtoOrIfaceCacheCount; ++i) {
    JSObject* proto = protoOrIfaceArray[i];
    if (proto) {
      JS_CALL_OBJECT_TRACER(trc, proto, "protoOrIfaceArray[i]");
    }
  }
}

inline void
DestroyProtoOrIfaceCache(JSObject* obj)
{
  MOZ_ASSERT(js::GetObjectClass(obj)->flags & JSCLASS_DOM_GLOBAL);

  JSObject** protoOrIfaceArray = GetProtoOrIfaceArray(obj);

  delete [] protoOrIfaceArray;
}

struct ConstantSpec
{
  const char* name;
  JS::Value value;
};

/**
 * Add constants to an object.
 */
bool
DefineConstants(JSContext* cx, JSObject* obj, ConstantSpec* cs);

template<typename T>
struct Prefable {
  // A boolean indicating whether this set of specs is enabled
  bool enabled;
  // Array of specs, terminated in whatever way is customary for T.
  // Null to indicate a end-of-array for Prefable, when such an
  // indicator is needed.
  T* specs;
};

/*
 * Create a DOM interface object (if constructorClass is non-null) and/or a
 * DOM interface prototype object (if protoClass is non-null).
 *
 * global is used as the parent of the interface object and the interface
 *        prototype object
 * receiver is the object on which we need to define the interface object as a
 *          property
 * protoProto is the prototype to use for the interface prototype object.
 * protoClass is the JSClass to use for the interface prototype object.
 *            This is null if we should not create an interface prototype
 *            object.
 * constructorClass is the JSClass to use for the interface object.
 *                  This is null if we should not create an interface object or
 *                  if it should be a function object.
 * constructor is the JSNative to use as a constructor.  If this is non-null, it
 *             should be used as a JSNative to back the interface object, which
 *             should be a Function.  If this is null, then we should create an
 *             object of constructorClass, unless that's also null, in which
 *             case we should not create an interface object at all.
 * ctorNargs is the length of the constructor function; 0 if no constructor
 * instanceClass is the JSClass of instance objects for this class.  This can
 *               be null if this is not a concrete proto.
 * methods and properties are to be defined on the interface prototype object;
 *                        these arguments are allowed to be null if there are no
 *                        methods or properties respectively.
 * constants are to be defined on the interface object and on the interface
 *           prototype object; allowed to be null if there are no constants.
 * staticMethods are to be defined on the interface object; allowed to be null
 *               if there are no static methods.
 *
 * At least one of protoClass and constructorClass should be non-null.
 * If constructorClass is non-null, the resulting interface object will be
 * defined on the given global with property name |name|, which must also be
 * non-null.
 *
 * returns the interface prototype object if protoClass is non-null, else it
 * returns the interface object.
 */
JSObject*
CreateInterfaceObjects(JSContext* cx, JSObject* global, JSObject* receiver,
                       JSObject* protoProto, JSClass* protoClass,
                       JSClass* constructorClass, JSNative constructor,
                       unsigned ctorNargs, const DOMClass* domClass,
                       Prefable<JSFunctionSpec>* methods,
                       Prefable<JSPropertySpec>* properties,
                       Prefable<ConstantSpec>* constants,
                       Prefable<JSFunctionSpec>* staticMethods, const char* name);

template <class T>
inline bool
WrapNewBindingObject(JSContext* cx, JSObject* scope, T* value, JS::Value* vp)
{
  JSObject* obj = value->GetWrapper();
  if (obj && js::GetObjectCompartment(obj) == js::GetObjectCompartment(scope)) {
    *vp = JS::ObjectValue(*obj);
    return true;
  }

  if (!obj) {
    bool triedToWrap;
    obj = value->WrapObject(cx, scope, &triedToWrap);
    if (!obj) {
      // At this point, obj is null, so just return false.  We could
      // try to communicate triedToWrap to the caller, but in practice
      // callers seem to be testing JS_IsExceptionPending(cx) to
      // figure out whether WrapObject() threw instead.
      return false;
    }
  }

  // When called via XrayWrapper, we end up here while running in the
  // chrome compartment.  But the obj we have would be created in
  // whatever the content compartment is.  So at this point we need to
  // make sure it's correctly wrapped for the compartment of |scope|.
  // cx should already be in the compartment of |scope| here.
  MOZ_ASSERT(js::IsObjectInContextCompartment(scope, cx));
  *vp = JS::ObjectValue(*obj);
  return JS_WrapValue(cx, vp);
}

// Helper for smart pointers (nsAutoPtr/nsRefPtr/nsCOMPtr).
template <template <typename> class SmartPtr, class T>
inline bool
WrapNewBindingObject(JSContext* cx, JSObject* scope, const SmartPtr<T>& value,
                     JS::Value* vp)
{
  return WrapNewBindingObject(cx, scope, value.get(), vp);
}

template <class T>
inline bool
WrapNewBindingNonWrapperCachedObject(JSContext* cx, JSObject* scope, T* value,
                                     JS::Value* vp)
{
  // We try to wrap in the compartment of the underlying object of "scope"
  JSObject* obj;
  {
    // scope for the JSAutoCompartment so that we restore the compartment
    // before we call JS_WrapValue.
    Maybe<JSAutoCompartment> ac;
    if (js::IsWrapper(scope)) {
      scope = xpc::Unwrap(cx, scope, false);
      if (!scope)
        return false;
      ac.construct(cx, scope);
    }

    obj = value->WrapObject(cx, scope);
  }

  // We can end up here in all sorts of compartments, per above.  Make
  // sure to JS_WrapValue!
  *vp = JS::ObjectValue(*obj);
  return JS_WrapValue(cx, vp);
}

// Helper for smart pointers (nsAutoPtr/nsRefPtr/nsCOMPtr).
template <template <typename> class SmartPtr, typename T>
inline bool
WrapNewBindingNonWrapperCachedObject(JSContext* cx, JSObject* scope,
                                     const SmartPtr<T>& value, JS::Value* vp)
{
  return WrapNewBindingNonWrapperCachedObject(cx, scope, value.get(), vp);
}

/**
 * A method to handle new-binding wrap failure, by possibly falling back to
 * wrapping as a non-new-binding object.
 */
bool
DoHandleNewBindingWrappingFailure(JSContext* cx, JSObject* scope,
                                  nsISupports* value, JS::Value* vp);

/**
 * An easy way to call the above when you have a value which
 * multiply-inherits from nsISupports.
 */
template <class T>
bool
HandleNewBindingWrappingFailure(JSContext* cx, JSObject* scope, T* value,
                                JS::Value* vp)
{
  nsCOMPtr<nsISupports> val;
  CallQueryInterface(value, getter_AddRefs(val));
  return DoHandleNewBindingWrappingFailure(cx, scope, val, vp);
}

// Helper for smart pointers (nsAutoPtr/nsRefPtr/nsCOMPtr).
template <template <typename> class SmartPtr, class T>
MOZ_ALWAYS_INLINE bool
HandleNewBindingWrappingFailure(JSContext* cx, JSObject* scope,
                                const SmartPtr<T>& value, JS::Value* vp)
{
  return HandleNewBindingWrappingFailure(cx, scope, value.get(), vp);
}

struct EnumEntry {
  const char* value;
  size_t length;
};

template<bool Fatal>
inline bool
EnumValueNotFound(JSContext* cx, const jschar* chars, size_t length,
                  const char* type)
{
  return false;
}

template<>
inline bool
EnumValueNotFound<false>(JSContext* cx, const jschar* chars, size_t length,
                         const char* type)
{
  // TODO: Log a warning to the console.
  return true;
}

template<>
inline bool
EnumValueNotFound<true>(JSContext* cx, const jschar* chars, size_t length,
                        const char* type)
{
  NS_LossyConvertUTF16toASCII deflated(static_cast<const PRUnichar*>(chars),
                                       length);
  return ThrowErrorMessage(cx, MSG_INVALID_ENUM_VALUE, deflated.get(), type);
}


template<bool InvalidValueFatal>
inline int
FindEnumStringIndex(JSContext* cx, JS::Value v, const EnumEntry* values,
                    const char* type, bool* ok)
{
  // JS_StringEqualsAscii is slow as molasses, so don't use it here.
  JSString* str = JS_ValueToString(cx, v);
  if (!str) {
    *ok = false;
    return 0;
  }
  JS::Anchor<JSString*> anchor(str);
  size_t length;
  const jschar* chars = JS_GetStringCharsAndLength(cx, str, &length);
  if (!chars) {
    *ok = false;
    return 0;
  }
  int i = 0;
  for (const EnumEntry* value = values; value->value; ++value, ++i) {
    if (length != value->length) {
      continue;
    }

    bool equal = true;
    const char* val = value->value;
    for (size_t j = 0; j != length; ++j) {
      if (unsigned(val[j]) != unsigned(chars[j])) {
        equal = false;
        break;
      }
    }

    if (equal) {
      *ok = true;
      return i;
    }
  }

  *ok = EnumValueNotFound<InvalidValueFatal>(cx, chars, length, type);
  return -1;
}

inline nsWrapperCache*
GetWrapperCache(nsWrapperCache* cache)
{
  return cache;
}

inline nsWrapperCache*
GetWrapperCache(nsGlobalWindow* not_allowed);

inline nsWrapperCache*
GetWrapperCache(void* p)
{
  return NULL;
}

struct ParentObject {
  template<class T>
  ParentObject(T* aObject) :
    mObject(aObject),
    mWrapperCache(GetWrapperCache(aObject))
  {}

  template<class T, template<typename> class SmartPtr>
  ParentObject(const SmartPtr<T>& aObject) :
    mObject(aObject.get()),
    mWrapperCache(GetWrapperCache(aObject.get()))
  {}

  ParentObject(nsISupports* aObject, nsWrapperCache* aCache) :
    mObject(aObject),
    mWrapperCache(aCache)
  {}

  nsISupports* const mObject;
  nsWrapperCache* const mWrapperCache;
};

inline nsWrapperCache*
GetWrapperCache(const ParentObject& aParentObject)
{
  return aParentObject.mWrapperCache;
}

template<class T>
inline nsISupports*
GetParentPointer(T* aObject)
{
  return ToSupports(aObject);
}

inline nsISupports*
GetParentPointer(const ParentObject& aObject)
{
  return ToSupports(aObject.mObject);
}

template<class T>
inline void
ClearWrapper(T* p, nsWrapperCache* cache)
{
  cache->ClearWrapper();
}

template<class T>
inline void
ClearWrapper(T* p, void*)
{
  nsWrapperCache* cache;
  CallQueryInterface(p, &cache);
  ClearWrapper(p, cache);
}

// Can only be called with the immediate prototype of the instance object. Can
// only be called on the prototype of an object known to be a DOM instance.
JSBool
InstanceClassHasProtoAtDepth(JSHandleObject protoObject, uint32_t protoID,
                             uint32_t depth);

// Only set allowNativeWrapper to false if you really know you need it, if in
// doubt use true. Setting it to false disables security wrappers.
bool
XPCOMObjectToJsval(JSContext* cx, JSObject* scope, xpcObjectHelper &helper,
                   const nsIID* iid, bool allowNativeWrapper, JS::Value* rval);

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, T* p, nsWrapperCache* cache,
           const nsIID* iid, JS::Value* vp)
{
  if (xpc_FastGetCachedWrapper(cache, scope, vp))
    return true;
  qsObjectHelper helper(p, cache);
  return XPCOMObjectToJsval(cx, scope, helper, iid, true, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, T* p, const nsIID* iid,
           JS::Value* vp)
{
  return WrapObject(cx, scope, p, GetWrapperCache(p), iid, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, T* p, JS::Value* vp)
{
  return WrapObject(cx, scope, p, NULL, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, nsCOMPtr<T> &p, const nsIID* iid,
           JS::Value* vp)
{
  return WrapObject(cx, scope, p.get(), iid, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, nsCOMPtr<T> &p, JS::Value* vp)
{
  return WrapObject(cx, scope, p, NULL, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, nsRefPtr<T> &p, const nsIID* iid,
           JS::Value* vp)
{
  return WrapObject(cx, scope, p.get(), iid, vp);
}

template<class T>
inline bool
WrapObject(JSContext* cx, JSObject* scope, nsRefPtr<T> &p, JS::Value* vp)
{
  return WrapObject(cx, scope, p, NULL, vp);
}

template<>
inline bool
WrapObject<JSObject>(JSContext* cx, JSObject* scope, JSObject* p, JS::Value* vp)
{
  vp->setObjectOrNull(p);
  return true;
}

template<typename T>
static inline JSObject*
WrapNativeParent(JSContext* cx, JSObject* scope, const T& p)
{
  if (!GetParentPointer(p))
    return scope;

  nsWrapperCache* cache = GetWrapperCache(p);
  JSObject* obj;
  if (cache && (obj = cache->GetWrapper())) {
#ifdef DEBUG
    qsObjectHelper helper(GetParentPointer(p), cache);
    JS::Value debugVal;

    bool ok = XPCOMObjectToJsval(cx, scope, helper, NULL, false, &debugVal);
    NS_ASSERTION(ok && JSVAL_TO_OBJECT(debugVal) == obj,
                 "Unexpected object in nsWrapperCache");
#endif
    return obj;
  }

  qsObjectHelper helper(GetParentPointer(p), cache);
  JS::Value v;
  return XPCOMObjectToJsval(cx, scope, helper, NULL, false, &v) ?
         JSVAL_TO_OBJECT(v) :
         NULL;
}

static inline bool
InternJSString(JSContext* cx, jsid& id, const char* chars)
{
  if (JSString *str = ::JS_InternString(cx, chars)) {
    id = INTERNED_STRING_TO_JSID(cx, str);
    return true;
  }
  return false;
}

// Spec needs a name property
template <typename Spec>
static bool
InitIds(JSContext* cx, Prefable<Spec>* prefableSpecs, jsid* ids)
{
  MOZ_ASSERT(prefableSpecs);
  MOZ_ASSERT(prefableSpecs->specs);
  do {
    // We ignore whether the set of ids is enabled and just intern all the IDs,
    // because this is only done once per application runtime.
    Spec* spec = prefableSpecs->specs;
    do {
      if (!InternJSString(cx, *ids, spec->name)) {
        return false;
      }
    } while (++ids, (++spec)->name);

    // We ran out of ids for that pref.  Put a JSID_VOID in on the id
    // corresponding to the list terminator for the pref.
    *ids = JSID_VOID;
    ++ids;
  } while ((++prefableSpecs)->specs);

  return true;
}

JSBool
QueryInterface(JSContext* cx, unsigned argc, JS::Value* vp);
JSBool
ThrowingConstructor(JSContext* cx, unsigned argc, JS::Value* vp);

bool
GetPropertyOnPrototype(JSContext* cx, JSObject* proxy, jsid id, bool* found,
                       JS::Value* vp);

bool
HasPropertyOnPrototype(JSContext* cx, JSObject* proxy, DOMProxyHandler* handler,
                       jsid id);

template<class T>
class NonNull
{
public:
  NonNull()
#ifdef DEBUG
    : inited(false)
#endif
  {}

  operator T&() {
    MOZ_ASSERT(inited);
    MOZ_ASSERT(ptr, "NonNull<T> was set to null");
    return *ptr;
  }

  operator const T&() const {
    MOZ_ASSERT(inited);
    MOZ_ASSERT(ptr, "NonNull<T> was set to null");
    return *ptr;
  }

  void operator=(T* t) {
    ptr = t;
    MOZ_ASSERT(ptr);
#ifdef DEBUG
    inited = true;
#endif
  }

  template<typename U>
  void operator=(U* t) {
    ptr = t->ToAStringPtr();
    MOZ_ASSERT(ptr);
#ifdef DEBUG
    inited = true;
#endif
  }

  T** Slot() {
#ifdef DEBUG
    inited = true;
#endif
    return &ptr;
  }

protected:
  T* ptr;
#ifdef DEBUG
  bool inited;
#endif
};

template<class T>
class OwningNonNull
{
public:
  OwningNonNull()
#ifdef DEBUG
    : inited(false)
#endif
  {}

  operator T&() {
    MOZ_ASSERT(inited);
    MOZ_ASSERT(ptr, "OwningNonNull<T> was set to null");
    return *ptr;
  }

  void operator=(T* t) {
    init(t);
  }

  void operator=(const already_AddRefed<T>& t) {
    init(t);
  }

protected:
  template<typename U>
  void init(U t) {
    ptr = t;
    MOZ_ASSERT(ptr);
#ifdef DEBUG
    inited = true;
#endif
  }

  nsRefPtr<T> ptr;
#ifdef DEBUG
  bool inited;
#endif
};

// A struct that has the same layout as an nsDependentString but much
// faster constructor and destructor behavior
struct FakeDependentString {
  FakeDependentString() :
    mFlags(nsDependentString::F_TERMINATED)
  {
  }

  void SetData(const nsDependentString::char_type* aData,
               nsDependentString::size_type aLength) {
    MOZ_ASSERT(mFlags == nsDependentString::F_TERMINATED);
    mData = aData;
    mLength = aLength;
  }

  void Truncate() {
    mData = nsDependentString::char_traits::sEmptyBuffer;
    mLength = 0;
  }

  void SetNull() {
    Truncate();
    mFlags |= nsDependentString::F_VOIDED;
  }

  const nsAString* ToAStringPtr() const {
    return reinterpret_cast<const nsDependentString*>(this);
  }

  nsAString* ToAStringPtr() {
    return reinterpret_cast<nsDependentString*>(this);
  }

  operator const nsAString& () const {
    return *reinterpret_cast<const nsDependentString*>(this);
  }

private:
  const nsDependentString::char_type* mData;
  nsDependentString::size_type mLength;
  uint32_t mFlags;

  // A class to use for our static asserts to ensure our object layout
  // matches that of nsDependentString.
  class DependentStringAsserter;
  friend class DependentStringAsserter;

  class DepedentStringAsserter : public nsDependentString {
  public:
    static void StaticAsserts() {
      MOZ_STATIC_ASSERT(sizeof(FakeDependentString) == sizeof(nsDependentString),
                        "Must have right object size");
      MOZ_STATIC_ASSERT(offsetof(FakeDependentString, mData) ==
                          offsetof(DepedentStringAsserter, mData),
                        "Offset of mData should match");
      MOZ_STATIC_ASSERT(offsetof(FakeDependentString, mLength) ==
                          offsetof(DepedentStringAsserter, mLength),
                        "Offset of mLength should match");
      MOZ_STATIC_ASSERT(offsetof(FakeDependentString, mFlags) ==
                          offsetof(DepedentStringAsserter, mFlags),
                        "Offset of mFlags should match");
    }
  };
};

enum StringificationBehavior {
  eStringify,
  eEmpty,
  eNull
};

// pval must not be null and must point to a rooted JS::Value
static inline bool
ConvertJSValueToString(JSContext* cx, const JS::Value& v, JS::Value* pval,
                       StringificationBehavior nullBehavior,
                       StringificationBehavior undefinedBehavior,
                       FakeDependentString& result)
{
  JSString *s;
  if (v.isString()) {
    s = v.toString();
  } else {
    StringificationBehavior behavior;
    if (v.isNull()) {
      behavior = nullBehavior;
    } else if (v.isUndefined()) {
      behavior = undefinedBehavior;
    } else {
      behavior = eStringify;
    }

    if (behavior != eStringify) {
      if (behavior == eEmpty) {
        result.Truncate();
      } else {
        result.SetNull();
      }
      return true;
    }

    s = JS_ValueToString(cx, v);
    if (!s) {
      return false;
    }
    pval->setString(s);  // Root the new string.
  }

  size_t len;
  const jschar *chars = JS_GetStringCharsZAndLength(cx, s, &len);
  if (!chars) {
    return false;
  }

  result.SetData(chars, len);
  return true;
}

// Class for representing optional arguments.
template<typename T>
class Optional {
public:
  Optional() {}

  bool WasPassed() const {
    return !mImpl.empty();
  }

  void Construct() {
    mImpl.construct();
  }

  template <class T1, class T2>
  void Construct(const T1 &t1, const T2 &t2) {
    mImpl.construct(t1, t2);
  }

  const T& Value() const {
    return mImpl.ref();
  }

  T& Value() {
    return mImpl.ref();
  }

private:
  // Forbid copy-construction and assignment
  Optional(const Optional& other) MOZ_DELETE;
  const Optional &operator=(const Optional &other) MOZ_DELETE;
  
  Maybe<T> mImpl;
};

// Specialization for strings.
template<>
class Optional<nsAString> {
public:
  Optional() : mPassed(false) {}

  bool WasPassed() const {
    return mPassed;
  }

  void operator=(const nsAString* str) {
    MOZ_ASSERT(str);
    mStr = str;
    mPassed = true;
  }

  void operator=(const FakeDependentString* str) {
    MOZ_ASSERT(str);
    mStr = str->ToAStringPtr();
    mPassed = true;
  }

  const nsAString& Value() const {
    MOZ_ASSERT(WasPassed());
    return *mStr;
  }

private:
  // Forbid copy-construction and assignment
  Optional(const Optional& other) MOZ_DELETE;
  const Optional &operator=(const Optional &other) MOZ_DELETE;
  
  bool mPassed;
  const nsAString* mStr;
};

// Class for representing sequences in arguments.  We use an auto array that can
// hold 16 elements, to avoid having to allocate in common cases.  This needs to
// be fallible because web content controls the length of the array, and can
// easily try to create very large lengths.
template<typename T>
class Sequence : public AutoFallibleTArray<T, 16>
{
public:
  Sequence() : AutoFallibleTArray<T, 16>() {}
};

// Class for holding the type of members of a union. The union type has an enum
// to keep track of which of its UnionMembers has been constructed.
template<class T>
class UnionMember {
    AlignedStorage2<T> storage;

public:
    T& SetValue() {
      new (storage.addr()) T();
      return *storage.addr();
    }
    const T& Value() const {
      return *storage.addr();
    }
    void Destroy() {
      storage.addr()->~T();
    }
};

// Implementation of the bits that XrayWrapper needs
bool
XrayResolveProperty(JSContext* cx, JSObject* wrapper, jsid id,
                    JSPropertyDescriptor* desc,
                    // And the things we need to determine the descriptor
                    Prefable<JSFunctionSpec>* methods,
                    jsid* methodIds,
                    JSFunctionSpec* methodSpecs,
                    size_t methodCount,
                    Prefable<JSPropertySpec>* attributes,
                    jsid* attributeIds,
                    JSPropertySpec* attributeSpecs,
                    size_t attributeCount,
                    Prefable<ConstantSpec>* constants,
                    jsid* constantIds,
                    ConstantSpec* constantSpecs,
                    size_t constantCount);

bool
XrayEnumerateProperties(JS::AutoIdVector& props,
                        Prefable<JSFunctionSpec>* methods,
                        jsid* methodIds,
                        JSFunctionSpec* methodSpecs,
                        size_t methodCount,
                        Prefable<JSPropertySpec>* attributes,
                        jsid* attributeIds,
                        JSPropertySpec* attributeSpecs,
                        size_t attributeCount,
                        Prefable<ConstantSpec>* constants,
                        jsid* constantIds,
                        ConstantSpec* constantSpecs,
                        size_t constantCount);

} // namespace dom
} // namespace mozilla

#endif /* mozilla_dom_BindingUtils_h__ */
