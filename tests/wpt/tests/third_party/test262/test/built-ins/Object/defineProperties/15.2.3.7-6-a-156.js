// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-156
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', test the [[Value]] field of 'desc' which equals
    to value of the length property is defined into 'O' without
    deleting any property with large index named (15.4.5.1 step 3.f)
---*/

var arr = [0, , 2];

Object.defineProperties(arr, {
  length: {
    value: 3
  }
});

assert.sameValue(arr.length, 3, 'arr.length');
assert.sameValue(arr[0], 0, 'arr[0]');
assert.sameValue(arr.hasOwnProperty("1"), false, 'arr.hasOwnProperty("1")');
assert.sameValue(arr[2], 2, 'arr[2]');
