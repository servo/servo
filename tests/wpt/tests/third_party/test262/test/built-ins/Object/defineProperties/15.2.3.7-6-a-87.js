// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-87
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, both properties.[[Set]] and P.[[Set]] are two objects which
    refer to different objects (8.12.9 step 11.a.i)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func1(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  set: set_func1,
  configurable: false
});

function set_func2() {}

try {
  Object.defineProperties(obj, {
    foo: {
      set: set_func2
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyWritable(obj, "foo", "setVerifyHelpProp");

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
