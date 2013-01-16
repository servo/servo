/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

interface ClientRect;

interface ClientRectList {
  readonly attribute unsigned long length;
  getter ClientRect? item(unsigned long index);
};

/* Helpers

unsafe fn unwrap<T>(obj: *JSObject) -> *rust_box<T> {
  let val = JS_GetReservedSlot(obj, 0);
  cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val))
}

trait ToJsval {
  fn to_jsval(cx: *JSContext) -> jsval;
}

impl Option : ToJsval {
  fn to_jsval(cx: *JSContext) -> jsval {
    match self {
      Some(v) => v.to_jsval(),
      None => JSVAL_NULL
    }
  }
}

 */

/*

trait ClientRectList {
  fn getLength() -> u32;
  fn getItem(u32 index) -> Option<@ClientRect>;
}

mod ClientRectList {
mod bindings {

fn getLength(cx: *JSContext, argc: c_uint, argv: *jsval) -> JSBool unsafe {
  let obj = JS_THIS_OBJECT(cx, unsafe::reinterpret_cast(&vp));
  if obj.is_null() {
    return 0;
  }

  let conrete = unwrap<ClientRectList>(obj);
  let rval = (*concrete).getLength();

  JS_SET_RVAL(argv, rval);
  return 1;
}

fn getItem(cx: *JSContext, argc: c_uint, vp: *jsval) -> JSBool unsafe {
  let obj = JS_THIS_OBJECT(cx, unsafe::reinterpret_cast(&vp));
  if obj.is_null() {
    return 0;
  }

  let raw_arg1 = if argc < 1 {
    //XXX convert null
  } else {
    JS_ARGV(vp, 0);
  };

  let arg1 = if !RUST_JSVAL_IS_INT(raw_arg1) {
    //XXX convert to int
  } else {
    RUST_JSVAL_TO_INT(raw_arg1);
  } as u32;

  let conrete = unwrap<ClientRectList>(obj);
  let rval = (*concrete).getItem(arg1);

  JS_SET_RVAL(vp, rval.to_jsval())
  return 1;
}
}

*/
