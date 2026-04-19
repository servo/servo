// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-250
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property that already exists on 'O' is data property and
    'desc' is data descriptor, test updating the [[Writable]]
    attribute value of 'P'  (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/


var arr = [100];

Object.defineProperties(arr, {
  "0": {
    writable: false
  }
});

verifyProperty(arr, "0", {
  value: 100,
  writable: false,
  enumerable: true,
  configurable: true,
});
