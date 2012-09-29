/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_DOMJSProxyHandler_h
#define mozilla_dom_DOMJSProxyHandler_h

#include "jsapi.h"
#include "jsfriendapi.h"
#include "jsproxy.h"
#include "xpcpublic.h"
#include "nsString.h"
#include "mozilla/Likely.h"

#define DOM_PROXY_OBJECT_SLOT js::JSSLOT_PROXY_PRIVATE

namespace mozilla {
namespace dom {

enum {
  JSPROXYSLOT_EXPANDO = 0
};

template<typename T> struct Prefable;

class DOMProxyHandler : public DOMBaseProxyHandler
{
public:
  DOMProxyHandler(const DOMClass& aClass)
    : DOMBaseProxyHandler(true),
      mClass(aClass)
  {
  }

  bool getPropertyDescriptor(JSContext* cx, JSObject* proxy, jsid id, bool set,
                             JSPropertyDescriptor* desc);
  bool defineProperty(JSContext* cx, JSObject* proxy, jsid id,
                      JSPropertyDescriptor* desc);
  bool delete_(JSContext* cx, JSObject* proxy, jsid id, bool* bp);
  bool enumerate(JSContext* cx, JSObject* proxy, JS::AutoIdVector& props);
  bool fix(JSContext* cx, JSObject* proxy, JS::Value* vp);
  bool has(JSContext* cx, JSObject* proxy, jsid id, bool* bp);
  using js::BaseProxyHandler::obj_toString;

  static JSObject* GetExpandoObject(JSObject* obj)
  {
    MOZ_ASSERT(IsDOMProxy(obj), "expected a DOM proxy object");
    JS::Value v = js::GetProxyExtra(obj, JSPROXYSLOT_EXPANDO);
    return v.isUndefined() ? NULL : v.toObjectOrNull();
  }
  static JSObject* EnsureExpandoObject(JSContext* cx, JSObject* obj);

  const DOMClass& mClass;

protected:
  static JSString* obj_toString(JSContext* cx, const char* className);
};

extern jsid s_length_id;

int32_t IdToInt32(JSContext* cx, jsid id);

inline int32_t
GetArrayIndexFromId(JSContext* cx, jsid id)
{
  if (MOZ_LIKELY(JSID_IS_INT(id))) {
    return JSID_TO_INT(id);
  }
  if (MOZ_LIKELY(id == s_length_id)) {
    return -1;
  }
  if (MOZ_LIKELY(JSID_IS_ATOM(id))) {
    JSAtom* atom = JSID_TO_ATOM(id);
    jschar s = *js::GetAtomChars(atom);
    if (MOZ_LIKELY((unsigned)s >= 'a' && (unsigned)s <= 'z'))
      return -1;

    uint32_t i;
    JSLinearString* str = js::AtomToLinearString(JSID_TO_ATOM(id));
    return js::StringIsArrayIndex(str, &i) ? i : -1;
  }
  return IdToInt32(cx, id);
}

inline void
FillPropertyDescriptor(JSPropertyDescriptor* desc, JSObject* obj, bool readonly)
{
  desc->obj = obj;
  desc->attrs = (readonly ? JSPROP_READONLY : 0) | JSPROP_ENUMERATE;
  desc->getter = NULL;
  desc->setter = NULL;
  desc->shortid = 0;
}

inline void
FillPropertyDescriptor(JSPropertyDescriptor* desc, JSObject* obj, jsval v, bool readonly)
{
  desc->value = v;
  FillPropertyDescriptor(desc, obj, readonly);
}

JSObject* 
EnsureExpandoObject(JSContext* cx, JSObject* obj);

} // namespace dom
} // namespace mozilla

#endif /* mozilla_dom_DOMProxyHandler_h */
