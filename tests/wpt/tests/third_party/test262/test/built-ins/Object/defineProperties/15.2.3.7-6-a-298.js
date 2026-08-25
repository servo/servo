// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-298
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is an
    array index named accessor property of 'O' but not defined in
    [[ParameterMap]] of 'O', test TypeError is thrown when updating
    the [[Get]] attribute value of 'P' which is not configurable (10.6
    [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/


var arg;

(function fun() {
  arg = arguments;
}());

function get_func1() {
  return 0;
}

Object.defineProperty(arg, "0", {
  get: get_func1,
  enumerable: false,
  configurable: false
});

function get_func2() {
  return 10;
}
try {
  Object.defineProperties(arg, {
    "0": {
      get: get_func2
    }
  });
  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(arg, "0", get_func1());

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arg, "0", {
  enumerable: false,
  configurable: false,
});
