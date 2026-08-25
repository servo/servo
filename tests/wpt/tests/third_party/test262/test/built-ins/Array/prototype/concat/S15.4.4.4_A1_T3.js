// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
info: |
    When the concat method is called with zero or more arguments item1, item2,
    etc., it returns an array containing the array elements of the object followed by
    the array elements of each argument in order
es5id: 15.4.4.4_A1_T3
description: Checking this algorithm with no items
---*/

var x = [0, 1];
var arr = x.concat();

arr.getClass = Object.prototype.toString;
assert.sameValue(arr.getClass(), "[object Array]", 'arr.getClass() must return "[object Array]"');
assert.sameValue(arr[0], 0, 'The value of arr[0] is expected to be 0');
assert.sameValue(arr[1], 1, 'The value of arr[1] is expected to be 1');
assert.sameValue(arr.length, 2, 'The value of arr.length is expected to be 2');
assert.notSameValue(arr, x, 'The value of arr is expected to not equal the value of `x`');
