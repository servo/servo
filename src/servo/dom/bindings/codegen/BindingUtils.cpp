/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* vim: set ts=2 sw=2 et tw=79: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <stdarg.h>

#include "BindingUtils.h"

#include "WrapperFactory.h"
#include "xpcprivate.h"
#include "XPCQuickStubs.h"

namespace mozilla {
namespace dom {

JSErrorFormatString ErrorFormatString[] = {
#define MSG_DEF(_name, _argc, _str) \
  { _str, _argc, JSEXN_TYPEERR },
#include "mozilla/dom/Errors.msg"
#undef MSG_DEF
};

const JSErrorFormatString*
GetErrorMessage(void* aUserRef, const char* aLocale,
                const unsigned aErrorNumber)
{
  MOZ_ASSERT(aErrorNumber < ArrayLength(ErrorFormatString));
  return &ErrorFormatString[aErrorNumber];
}

bool
ThrowErrorMessage(JSContext* aCx, const ErrNum aErrorNumber, ...)
{
  va_list ap;
  va_start(ap, aErrorNumber);
  JS_ReportErrorNumberVA(aCx, GetErrorMessage, NULL,
                         static_cast<const unsigned>(aErrorNumber), ap);
  va_end(ap);
  return false;
}

bool
DefineConstants(JSContext* cx, JSObject* obj, ConstantSpec* cs)
{
  for (; cs->name; ++cs) {
    JSBool ok =
      JS_DefineProperty(cx, obj, cs->name, cs->value, NULL, NULL,
                        JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT);
    if (!ok) {
      return false;
    }
  }
  return true;
}

static inline bool
Define(JSContext* cx, JSObject* obj, JSFunctionSpec* spec) {
  return JS_DefineFunctions(cx, obj, spec);
}
static inline bool
Define(JSContext* cx, JSObject* obj, JSPropertySpec* spec) {
  return JS_DefineProperties(cx, obj, spec);
}
static inline bool
Define(JSContext* cx, JSObject* obj, ConstantSpec* spec) {
  return DefineConstants(cx, obj, spec);
}

template<typename T>
bool
DefinePrefable(JSContext* cx, JSObject* obj, Prefable<T>* props)
{
  MOZ_ASSERT(props);
  MOZ_ASSERT(props->specs);
  do {
    // Define if enabled
    if (props->enabled) {
      if (!Define(cx, obj, props->specs)) {
        return false;
      }
    }
  } while ((++props)->specs);
  return true;
}

// We should use JSFunction objects for interface objects, but we need a custom
// hasInstance hook because we have new interface objects on prototype chains of
// old (XPConnect-based) bindings. Because Function.prototype.toString throws if
// passed a non-Function object we also need to provide our own toString method
// for interface objects.

enum {
  TOSTRING_CLASS_RESERVED_SLOT = 0,
  TOSTRING_NAME_RESERVED_SLOT = 1
};

JSBool
InterfaceObjectToString(JSContext* cx, unsigned argc, JS::Value *vp)
{
  JSObject* callee = JSVAL_TO_OBJECT(JS_CALLEE(cx, vp));

  JSObject* obj = JS_THIS_OBJECT(cx, vp);
  if (!obj) {
    JS_ReportErrorNumber(cx, js_GetErrorMessage, NULL, JSMSG_CANT_CONVERT_TO,
                         "null", "object");
    return false;
  }

  jsval v = js::GetFunctionNativeReserved(callee, TOSTRING_CLASS_RESERVED_SLOT);
  JSClass* clasp = static_cast<JSClass*>(JSVAL_TO_PRIVATE(v));

  v = js::GetFunctionNativeReserved(callee, TOSTRING_NAME_RESERVED_SLOT);
  JSString* jsname = static_cast<JSString*>(JSVAL_TO_STRING(v));
  size_t length;
  const jschar* name = JS_GetInternedStringCharsAndLength(jsname, &length);

  if (js::GetObjectJSClass(obj) != clasp) {
    JS_ReportErrorNumber(cx, js_GetErrorMessage, NULL, JSMSG_INCOMPATIBLE_PROTO,
                         NS_ConvertUTF16toUTF8(name).get(), "toString",
                         "object");
    return false;
  }

  nsString str;
  str.AppendLiteral("function ");
  str.Append(name, length);
  str.AppendLiteral("() {");
  str.Append('\n');
  str.AppendLiteral("    [native code]");
  str.Append('\n');
  str.AppendLiteral("}");

  return xpc::NonVoidStringToJsval(cx, str, vp);
}

static JSObject*
CreateInterfaceObject(JSContext* cx, JSObject* global, JSObject* receiver,
                      JSClass* constructorClass, JSNative constructorNative,
                      unsigned ctorNargs, JSObject* proto,
                      Prefable<JSFunctionSpec>* staticMethods,
                      Prefable<ConstantSpec>* constants,
                      const char* name)
{
  JSObject* constructor;
  if (constructorClass) {
    JSObject* functionProto = JS_GetFunctionPrototype(cx, global);
    if (!functionProto) {
      return NULL;
    }
    constructor = JS_NewObject(cx, constructorClass, functionProto, global);
  } else {
    MOZ_ASSERT(constructorNative);
    JSFunction* fun = JS_NewFunction(cx, constructorNative, ctorNargs,
                                     JSFUN_CONSTRUCTOR, global, name);
    if (!fun) {
      return NULL;
    }
    constructor = JS_GetFunctionObject(fun);
  }
  if (!constructor) {
    return NULL;
  }

  if (staticMethods && !DefinePrefable(cx, constructor, staticMethods)) {
    return NULL;
  }

  if (constructorClass) {
    JSFunction* toString = js::DefineFunctionWithReserved(cx, constructor,
                                                          "toString",
                                                          InterfaceObjectToString,
                                                          0, 0);
    if (!toString) {
      return NULL;
    }

    JSObject* toStringObj = JS_GetFunctionObject(toString);
    js::SetFunctionNativeReserved(toStringObj, TOSTRING_CLASS_RESERVED_SLOT,
                                  PRIVATE_TO_JSVAL(constructorClass));

    JSString *str = ::JS_InternString(cx, name);
    if (!str) {
      return NULL;
    }
    js::SetFunctionNativeReserved(toStringObj, TOSTRING_NAME_RESERVED_SLOT,
                                  STRING_TO_JSVAL(str));
  }

  if (constants && !DefinePrefable(cx, constructor, constants)) {
    return NULL;
  }

  if (proto && !JS_LinkConstructorAndPrototype(cx, constructor, proto)) {
    return NULL;
  }

  JSBool alreadyDefined;
  if (!JS_AlreadyHasOwnProperty(cx, receiver, name, &alreadyDefined)) {
    return NULL;
  }

  // This is Enumerable: False per spec.
  if (!alreadyDefined &&
      !JS_DefineProperty(cx, receiver, name, OBJECT_TO_JSVAL(constructor), NULL,
                         NULL, 0)) {
    return NULL;
  }

  return constructor;
}

static JSObject*
CreateInterfacePrototypeObject(JSContext* cx, JSObject* global,
                               JSObject* parentProto, JSClass* protoClass,
                               Prefable<JSFunctionSpec>* methods,
                               Prefable<JSPropertySpec>* properties,
                               Prefable<ConstantSpec>* constants)
{
  JSObject* ourProto = JS_NewObjectWithUniqueType(cx, protoClass, parentProto,
                                                  global);
  if (!ourProto) {
    return NULL;
  }

  if (methods && !DefinePrefable(cx, ourProto, methods)) {
    return NULL;
  }

  if (properties && !DefinePrefable(cx, ourProto, properties)) {
    return NULL;
  }

  if (constants && !DefinePrefable(cx, ourProto, constants)) {
    return NULL;
  }

  return ourProto;
}

JSObject*
CreateInterfaceObjects(JSContext* cx, JSObject* global, JSObject *receiver,
                       JSObject* protoProto, JSClass* protoClass,
                       JSClass* constructorClass, JSNative constructor,
                       unsigned ctorNargs, const DOMClass* domClass,
                       Prefable<JSFunctionSpec>* methods,
                       Prefable<JSPropertySpec>* properties,
                       Prefable<ConstantSpec>* constants,
                       Prefable<JSFunctionSpec>* staticMethods, const char* name)
{
  MOZ_ASSERT(protoClass || constructorClass || constructor,
             "Need at least one class or a constructor!");
  MOZ_ASSERT(!(methods || properties) || protoClass,
             "Methods or properties but no protoClass!");
  MOZ_ASSERT(!staticMethods || constructorClass || constructor,
             "Static methods but no constructorClass or constructor!");
  MOZ_ASSERT(bool(name) == bool(constructorClass || constructor),
             "Must have name precisely when we have an interface object");
  MOZ_ASSERT(!constructorClass || !constructor);

  JSObject* proto;
  if (protoClass) {
    proto = CreateInterfacePrototypeObject(cx, global, protoProto, protoClass,
                                           methods, properties, constants);
    if (!proto) {
      return NULL;
    }

    js::SetReservedSlot(proto, DOM_PROTO_INSTANCE_CLASS_SLOT,
                        JS::PrivateValue(const_cast<DOMClass*>(domClass)));
  }
  else {
    proto = NULL;
  }

  JSObject* interface;
  if (constructorClass || constructor) {
    interface = CreateInterfaceObject(cx, global, receiver, constructorClass,
                                      constructor, ctorNargs, proto,
                                      staticMethods, constants, name);
    if (!interface) {
      return NULL;
    }
  }

  return protoClass ? proto : interface;
}

static bool
NativeInterface2JSObjectAndThrowIfFailed(XPCLazyCallContext& aLccx,
                                         JSContext* aCx,
                                         JS::Value* aRetval,
                                         xpcObjectHelper& aHelper,
                                         const nsIID* aIID,
                                         bool aAllowNativeWrapper)
{
  nsresult rv;
  if (!XPCConvert::NativeInterface2JSObject(aLccx, aRetval, NULL, aHelper, aIID,
                                            NULL, aAllowNativeWrapper, &rv)) {
    // I can't tell if NativeInterface2JSObject throws JS exceptions
    // or not.  This is a sloppy stab at the right semantics; the
    // method really ought to be fixed to behave consistently.
    if (!JS_IsExceptionPending(aCx)) {
      Throw<true>(aCx, NS_FAILED(rv) ? rv : NS_ERROR_UNEXPECTED);
    }
    return false;
  }
  return true;
}

bool
DoHandleNewBindingWrappingFailure(JSContext* cx, JSObject* scope,
                                  nsISupports* value, JS::Value* vp)
{
  if (JS_IsExceptionPending(cx)) {
    return false;
  }

  XPCLazyCallContext lccx(JS_CALLER, cx, scope);

  if (value) {
    xpcObjectHelper helper(value);
    return NativeInterface2JSObjectAndThrowIfFailed(lccx, cx, vp, helper, NULL,
                                                    true);
  }

  return Throw<true>(cx, NS_ERROR_XPC_BAD_CONVERT_JS);
}

// Can only be called with the immediate prototype of the instance object. Can
// only be called on the prototype of an object known to be a DOM instance.
JSBool
InstanceClassHasProtoAtDepth(JSHandleObject protoObject, uint32_t protoID,
                             uint32_t depth)
{
  const DOMClass* domClass = static_cast<DOMClass*>(
    js::GetReservedSlot(protoObject, DOM_PROTO_INSTANCE_CLASS_SLOT).toPrivate());
  return (uint32_t)domClass->mInterfaceChain[depth] == protoID;
}

// Only set allowNativeWrapper to false if you really know you need it, if in
// doubt use true. Setting it to false disables security wrappers.
bool
XPCOMObjectToJsval(JSContext* cx, JSObject* scope, xpcObjectHelper &helper,
                   const nsIID* iid, bool allowNativeWrapper, JS::Value* rval)
{
  XPCLazyCallContext lccx(JS_CALLER, cx, scope);

  if (!NativeInterface2JSObjectAndThrowIfFailed(lccx, cx, rval, helper, iid,
                                                allowNativeWrapper)) {
    return false;
  }

#ifdef DEBUG
  JSObject* jsobj = JSVAL_TO_OBJECT(*rval);
  if (jsobj && !js::GetObjectParent(jsobj))
    NS_ASSERTION(js::GetObjectClass(jsobj)->flags & JSCLASS_IS_GLOBAL,
                 "Why did we recreate this wrapper?");
#endif

  return true;
}

JSBool
QueryInterface(JSContext* cx, unsigned argc, JS::Value* vp)
{
  JS::Value thisv = JS_THIS(cx, vp);
  if (thisv == JSVAL_NULL)
    return false;

  // Get the object. It might be a security wrapper, in which case we do a checked
  // unwrap.
  JSObject* origObj = JSVAL_TO_OBJECT(thisv);
  JSObject* obj = js::UnwrapObjectChecked(cx, origObj);
  if (!obj)
      return false;

  nsISupports* native;
  if (!UnwrapDOMObjectToISupports(obj, native)) {
    return Throw<true>(cx, NS_ERROR_FAILURE);
  }

  if (argc < 1) {
    return Throw<true>(cx, NS_ERROR_XPC_NOT_ENOUGH_ARGS);
  }

  JS::Value* argv = JS_ARGV(cx, vp);
  if (!argv[0].isObject()) {
    return Throw<true>(cx, NS_ERROR_XPC_BAD_CONVERT_JS);
  }

  nsIJSIID* iid;
  xpc_qsSelfRef iidRef;
  if (NS_FAILED(xpc_qsUnwrapArg<nsIJSIID>(cx, argv[0], &iid, &iidRef.ptr,
                                          &argv[0]))) {
    return Throw<true>(cx, NS_ERROR_XPC_BAD_CONVERT_JS);
  }
  MOZ_ASSERT(iid);

  if (iid->GetID()->Equals(NS_GET_IID(nsIClassInfo))) {
    nsresult rv;
    nsCOMPtr<nsIClassInfo> ci = do_QueryInterface(native, &rv);
    if (NS_FAILED(rv)) {
      return Throw<true>(cx, rv);
    }

    return WrapObject(cx, origObj, ci, &NS_GET_IID(nsIClassInfo), vp);
  }

  // Lie, otherwise we need to check classinfo or QI
  *vp = thisv;
  return true;
}

JSBool
ThrowingConstructor(JSContext* cx, unsigned argc, JS::Value* vp)
{
  return ThrowErrorMessage(cx, MSG_ILLEGAL_CONSTRUCTOR);
}

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
                    size_t constantCount)
{
  for (size_t prefIdx = 0; prefIdx < methodCount; ++prefIdx) {
    MOZ_ASSERT(methods[prefIdx].specs);
    if (methods[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = methods[prefIdx].specs - methodSpecs;
      for ( ; methodIds[i] != JSID_VOID; ++i) {
        if (id == methodIds[i]) {
          JSFunction *fun = JS_NewFunctionById(cx, methodSpecs[i].call.op,
                                               methodSpecs[i].nargs, 0,
                                               wrapper, id);
          if (!fun) {
            return false;
          }
          SET_JITINFO(fun, methodSpecs[i].call.info);
          JSObject *funobj = JS_GetFunctionObject(fun);
          desc->value.setObject(*funobj);
          desc->attrs = methodSpecs[i].flags;
          desc->obj = wrapper;
          desc->setter = nullptr;
          desc->getter = nullptr;
          return true;
        }
      }
    }
  }

  for (size_t prefIdx = 0; prefIdx < attributeCount; ++prefIdx) {
    MOZ_ASSERT(attributes[prefIdx].specs);
    if (attributes[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = attributes[prefIdx].specs - attributeSpecs;
      for ( ; attributeIds[i] != JSID_VOID; ++i) {
        if (id == attributeIds[i]) {
          // Because of centralization, we need to make sure we fault in the
          // JitInfos as well. At present, until the JSAPI changes, the easiest
          // way to do this is wrap them up as functions ourselves.
          desc->attrs = attributeSpecs[i].flags & ~JSPROP_NATIVE_ACCESSORS;
          // They all have getters, so we can just make it.
          JSObject *global = JS_GetGlobalForObject(cx, wrapper);
          JSFunction *fun = JS_NewFunction(cx, (JSNative)attributeSpecs[i].getter.op,
                                           0, 0, global, NULL);
          if (!fun)
            return false;
          SET_JITINFO(fun, attributeSpecs[i].getter.info);
          JSObject *funobj = JS_GetFunctionObject(fun);
          desc->getter = js::CastAsJSPropertyOp(funobj);
          desc->attrs |= JSPROP_GETTER;
          if (attributeSpecs[i].setter.op) {
            // We have a setter! Make it.
            fun = JS_NewFunction(cx, (JSNative)attributeSpecs[i].setter.op,
                                 1, 0, global, NULL);
            if (!fun)
              return false;
            SET_JITINFO(fun, attributeSpecs[i].setter.info);
            funobj = JS_GetFunctionObject(fun);
            desc->setter = js::CastAsJSStrictPropertyOp(funobj);
            desc->attrs |= JSPROP_SETTER;
          } else {
            desc->setter = NULL;
          }
          desc->obj = wrapper;
          return true;
        }
      }
    }
  }

  for (size_t prefIdx = 0; prefIdx < constantCount; ++prefIdx) {
    MOZ_ASSERT(constants[prefIdx].specs);
    if (constants[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = constants[prefIdx].specs - constantSpecs;
      for ( ; constantIds[i] != JSID_VOID; ++i) {
        if (id == constantIds[i]) {
          desc->attrs = JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT;
          desc->obj = wrapper;
          desc->value = constantSpecs[i].value;
          return true;
        }
      }
    }
  }

  return true;
}

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
                        size_t constantCount)
{
  for (size_t prefIdx = 0; prefIdx < methodCount; ++prefIdx) {
    MOZ_ASSERT(methods[prefIdx].specs);
    if (methods[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = methods[prefIdx].specs - methodSpecs;
      for ( ; methodIds[i] != JSID_VOID; ++i) {
        if ((methodSpecs[i].flags & JSPROP_ENUMERATE) &&
            !props.append(methodIds[i])) {
          return false;
        }
      }
    }
  }

  for (size_t prefIdx = 0; prefIdx < attributeCount; ++prefIdx) {
    MOZ_ASSERT(attributes[prefIdx].specs);
    if (attributes[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = attributes[prefIdx].specs - attributeSpecs;
      for ( ; attributeIds[i] != JSID_VOID; ++i) {
        if ((attributeSpecs[i].flags & JSPROP_ENUMERATE) &&
            !props.append(attributeIds[i])) {
          return false;
        }
      }
    }
  }

  for (size_t prefIdx = 0; prefIdx < constantCount; ++prefIdx) {
    MOZ_ASSERT(constants[prefIdx].specs);
    if (constants[prefIdx].enabled) {
      // Set i to be the index into our full list of ids/specs that we're
      // looking at now.
      size_t i = constants[prefIdx].specs - constantSpecs;
      for ( ; constantIds[i] != JSID_VOID; ++i) {
        if (!props.append(constantIds[i])) {
          return false;
        }
      }
    }
  }

  return true;
}

bool
GetPropertyOnPrototype(JSContext* cx, JSObject* proxy, jsid id, bool* found,
                       JS::Value* vp)
{
  JSObject* proto;
  if (!js::GetObjectProto(cx, proxy, &proto)) {
    return false;
  }
  if (!proto) {
    *found = false;
    return true;
  }

  JSBool hasProp;
  if (!JS_HasPropertyById(cx, proto, id, &hasProp)) {
    return false;
  }

  *found = hasProp;
  if (!hasProp || !vp) {
    return true;
  }

  return JS_ForwardGetPropertyTo(cx, proto, id, proxy, vp);
}

bool
HasPropertyOnPrototype(JSContext* cx, JSObject* proxy, DOMProxyHandler* handler,
                       jsid id)
{
  Maybe<JSAutoCompartment> ac;
  if (xpc::WrapperFactory::IsXrayWrapper(proxy)) {
    proxy = js::UnwrapObject(proxy);
    ac.construct(cx, proxy);
  }
  MOZ_ASSERT(js::IsProxy(proxy) && js::GetProxyHandler(proxy) == handler);

  bool found;
  // We ignore an error from GetPropertyOnPrototype.
  return !GetPropertyOnPrototype(cx, proxy, id, &found, NULL) || found;
}

} // namespace dom
} // namespace mozilla
