// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-241
description: >
    Object.defineProperties - TypeError is thrown if 'O' is an Array,
    'P' is an array index named property that already exists on 'O' is
    accessor property with  [[Configurable]] false, 'desc' is accessor
    descriptor, the [[Set]] field of 'desc' is present, and the
    [[Set]] field of 'desc' and the [[Set]] attribute value of 'P' are
    two objects which refer to the different objects  (15.4.5.1 step
    4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

function set_fun(value) {
  arr.setVerifyHelpProp = value;
}
Object.defineProperty(arr, "1", {
  set: set_fun
});

try {
  Object.defineProperties(arr, {
    "1": {
      set: function() {}
    }
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyWritable(arr, "1", "setVerifyHelpProp");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arr, "1", {
  enumerable: false,
  configurable: false,
});
