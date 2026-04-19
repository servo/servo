// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-89
description: >
    Object.defineProperties will not throw TypeError when
    P.configurable is false, P.[[Set]] and properties.[[Set]] are
    undefined (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_Func() {
  return 0;
}

Object.defineProperty(obj, "foo", {
  get: get_Func,
  set: undefined,
  enumerable: false,
  configurable: false
});

Object.defineProperties(obj, {
  foo: {
    set: undefined
  }
});

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});

var desc = Object.getOwnPropertyDescriptor(obj, "foo");

if (typeof(desc.set) !== "undefined") {
  throw new Test262Error('Expected typeof (desc.set) === "undefined", actually ' + typeof(desc.set));
}
