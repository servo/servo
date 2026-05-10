// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-182
description: >
    Object.defineProperties - 'O' is an Array, 'P' is an array index
    named property, 'P' is boundary value 2^32 + 1 (15.4.5.1 step 4.a)
---*/

var arr = [];

Object.defineProperties(arr, {
  "4294967297": {
    value: 100
  }
});

assert(arr.hasOwnProperty("4294967297"), 'arr.hasOwnProperty("4294967297") !== true');
assert.sameValue(arr.length, 0, 'arr.length');
assert.sameValue(arr[4294967297], 100, 'arr[4294967297]');
