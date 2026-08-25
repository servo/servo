// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-91
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, both properties.[[Get]] and P.[[Get]] are two objects which
    refer to different objects (8.12.9 step 11.a.ii)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

function get_func1() {
  return 10;
}

Object.defineProperty(obj, "foo", {
  get: get_func1,
  set: set_func,
  enumerable: false,
  configurable: false
});

function get_func2() {
  return 20;
}

try {
  Object.defineProperties(obj, {
    foo: {
      get: get_func2
    }
  });
  throw new Test262Error("Expected an exception");
} catch (e) {
  verifyEqualTo(obj, "foo", get_func1());

  verifyWritable(obj, "foo", "setVerifyHelpProp");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
