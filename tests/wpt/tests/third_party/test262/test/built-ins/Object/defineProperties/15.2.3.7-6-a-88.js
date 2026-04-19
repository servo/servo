// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-88
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, P.[[Set]] is undefined, properties.[[Set]] refers to an
    object (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function get_Func() {
  return 0;
}

Object.defineProperty(obj, "foo", {
  set: undefined,
  get: get_Func,
  enumerable: false,
  configurable: false
});

function set_Func() {}

try {
  Object.defineProperties(obj, {
    foo: {
      set: set_Func
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});

var desc = Object.getOwnPropertyDescriptor(obj, "foo");

if (typeof(desc.set) !== "undefined") {
  throw new Test262Error('Expected typeof (desc.set) === "undefined", actually ' + typeof(desc.set));
}
