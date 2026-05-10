// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-183
description: >
    Object.defineProperties - TypeError is not thrown if 'O' is an
    Array, 'P' is an array index named property, [[Writable]]
    attribute of the length property in 'O' is false, value of 'P' is
    less than value of the length property in'O'  (15.4.5.1 step 4.b)
---*/

var arr = [1, 2, 3];

Object.defineProperty(arr, "length", {
  writable: false
});

Object.defineProperties(arr, {
  "1": {
    value: "abc"
  }
});

assert.sameValue(arr[0], 1, 'arr[0]');
assert.sameValue(arr[1], "abc", 'arr[1]');
assert.sameValue(arr[2], 3, 'arr[2]');
