// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-281
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is own
    data property of 'O' which is also defined in [[ParameterMap]] of
    'O', and 'desc' is data descriptor, test updating multiple
    attribute values of 'P' (10.6 [[DefineOwnProperty]] step 3)
includes: [propertyHelper.js]
---*/


var arg;

(function fun(a, b, c) {
  arg = arguments;
}(0, 1, 2));

Object.defineProperties(arg, {
  "0": {
    value: 20,
    writable: false,
    enumerable: false,
    configurable: false
  }
});

verifyProperty(arg, "0", {
  value: 20,
  writable: false,
  enumerable: false,
  configurable: false,
});
