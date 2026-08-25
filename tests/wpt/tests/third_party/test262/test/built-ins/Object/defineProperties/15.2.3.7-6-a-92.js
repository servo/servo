// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-92
description: >
    Object.defineProperties throws TypeError when P.configurable is
    false, P.[[Get]] is undefined, properties.[[Get]] refers to an
    objcet (8.12.9 step 11.a.ii)
includes: [propertyHelper.js]
---*/


var obj = {};

function set_func(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  get: undefined,
  set: set_func,
  enumerable: false,
  configurable: false
});

function get_func() {
  return 0;
}

try {
  Object.defineProperties(obj, {
    foo: {
      get: get_func
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyWritable(obj, "foo", "setVerifyHelpProp");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "foo", {
  enumerable: false,
  configurable: false,
});
