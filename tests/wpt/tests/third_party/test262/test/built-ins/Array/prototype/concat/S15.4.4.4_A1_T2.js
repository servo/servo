// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
info: |
    When the concat method is called with zero or more arguments item1, item2,
    etc., it returns an array containing the array elements of the object followed by
    the array elements of each argument in order
es5id: 15.4.4.4_A1_T2
description: Checking this algorithm, items are objects and primitives
---*/

var x = [0];
var y = new Object();
var z = new Array(1, 2);
var arr = x.concat(y, z, -1, true, "NaN");

arr.getClass = Object.prototype.toString;
assert.sameValue(arr.getClass(), "[object Array]", 'arr.getClass() must return "[object Array]"');
assert.sameValue(arr[0], 0, 'The value of arr[0] is expected to be 0');
assert.sameValue(arr[1], y, 'The value of arr[1] is expected to equal the value of y');
assert.sameValue(arr[2], 1, 'The value of arr[2] is expected to be 1');
assert.sameValue(arr[3], 2, 'The value of arr[3] is expected to be 2');
assert.sameValue(arr[4], -1, 'The value of arr[4] is expected to be -1');
assert.sameValue(arr[5], true, 'The value of arr[5] is expected to be true');
assert.sameValue(arr[6], "NaN", 'The value of arr[6] is expected to be "NaN"');
assert.sameValue(arr.length, 7, 'The value of arr.length is expected to be 7');
