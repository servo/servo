// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-69
description: >
    Object.defineProperties - 'P' is data property and P.configurable
    is true, desc is accessor property (8.12.9 step 9.b.i)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  configurable: true
});

function get_Func() {
  return 20;
}

Object.defineProperties(obj, {
  foo: {
    get: get_Func
  }
});

assert.sameValue(obj.foo, 20);

var desc = Object.getOwnPropertyDescriptor(obj, "foo");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: true,
});

assert.sameValue(typeof desc.set, "undefined", 'typeof desc.set');
assert.sameValue(desc.get, get_Func, 'desc.get');
