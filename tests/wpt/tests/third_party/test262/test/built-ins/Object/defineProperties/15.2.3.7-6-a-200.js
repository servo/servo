// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-200
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'desc' is data descriptor, test updating all
    attribute values of 'P'  (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [1]; // default value of attributes: writable: true, configurable: true, enumerable: true

Object.defineProperties(arr, {
  "0": {
    value: 1001,
    writable: false,
    enumerable: false,
    configurable: false
  }
});

verifyProperty(arr, "0", {
  value: 1001,
  writable: false,
  enumerable: false,
  configurable: false,
});
