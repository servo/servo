// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-221
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    property, the [[Get]] field of 'desc' and the [[Get]] attribute
    value of 'P' are two objects which refer to the same object
    (15.4.5.1 step 4.c)
includes: [propertyHelper.js]
---*/

var arr = [];

function get_func() {
  return 10;
}

Object.defineProperty(arr, "0", {
  get: get_func
});

Object.defineProperties(arr, {
  "0": {
    get: get_func
  }
});
verifyEqualTo(arr, "0", get_func());

verifyProperty(arr, "0", {
  enumerable: false,
  configurable: false,
});
