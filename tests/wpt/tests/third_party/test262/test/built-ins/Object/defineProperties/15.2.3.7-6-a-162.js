// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-162
description: >
    Object.defineProperties - 'O' is an Array, 'P' is the length
    property of 'O', the [[Value]] field of 'desc' is less than value
    of  the length property,  test the [[Writable]] attribute of the
    length property is set to true at last after deleting properties
    with large index named if the [[Writable]] field of 'desc' is true
    (15.4.5.1 step 3.h)
---*/

var arr = [0, 1];

Object.defineProperties(arr, {
  length: {
    value: 1,
    writable: true
  }
});

arr.length = 10; //try to overwrite length value of arr

assert.sameValue(arr.hasOwnProperty("1"), false, 'arr.hasOwnProperty("1")');
assert.sameValue(arr.length, 10, 'arr.length');
assert.sameValue(arr[0], 0, 'arr[0]');
