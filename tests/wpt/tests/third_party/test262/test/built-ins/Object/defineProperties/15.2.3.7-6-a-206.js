// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-206
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' makes no change if every field in 'desc' is
    absent (name is data property)  (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

arr[0] = 101; // default value of attributes: writable: true, configurable: true, enumerable: true


Object.defineProperties(arr, {
  "0": {}
});

verifyProperty(arr, "0", {
  value: 101,
  writable: true,
  enumerable: true,
  configurable: true,
});
