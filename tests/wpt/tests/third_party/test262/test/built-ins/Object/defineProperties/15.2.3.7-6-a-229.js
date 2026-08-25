// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-229
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property, TypeError is thrown if 'P' is accessor property, and
    'desc' is data descriptor, and the [[Configurable]] attribute
    value of 'P' is false  (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arr = [];

function set_fun(value) {
  arr.setVerifyHelpProp = value;
}

Object.defineProperty(arr, "1", {
  set: set_fun,
  configurable: false

});

try {
  Object.defineProperties(arr, {
    "1": {
      value: 13
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
