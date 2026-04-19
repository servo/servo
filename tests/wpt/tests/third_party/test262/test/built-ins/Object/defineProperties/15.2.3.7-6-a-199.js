// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-199
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' property doesn't exist in 'O', test
    [[Configurable]] of 'P' property in 'Attributes' is set as false
    value if [[Configurable]] is absent in data descriptor 'desc'
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];
var beforeDeleted = false;
var afterDeleted = false;

Object.defineProperties(arr, {
  "0": {
    value: 1001,
    writable: true,
    enumerable: true
  }
});

verifyProperty(arr, "0", {
  configurable: false,
});
