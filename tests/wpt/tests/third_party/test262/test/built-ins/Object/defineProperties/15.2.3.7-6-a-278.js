// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-278
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is own
    property which is ever defined in both [[ParameterMap]] of 'O' and
    'O', and is deleted afterwards, and 'desc' is data descriptor,
    test 'P' is redefined in 'O' with all correct attribute values
    (10.6 [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/


var arg;

(function fun(a, b, c) {
  arg = arguments;
}(0, 1, 2));

delete arg[0];

Object.defineProperties(arg, {
  "0": {
    value: 10,
    writable: true,
    enumerable: true,
    configurable: true
  }
});

verifyProperty(arg, "0", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: true,
});
