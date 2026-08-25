// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-195
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test 'P' is
    defined as data property when 'desc' is generic descriptor
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

Object.defineProperties(arr, {
  "0": {
    enumerable: true
  }
});

verifyProperty(arr, "0", {
  value: undefined,
  writable: false,
  enumerable: true,
  configurable: false,
});
