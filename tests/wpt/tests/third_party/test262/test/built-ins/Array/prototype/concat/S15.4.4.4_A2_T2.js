// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
info: |
    The concat function is intentionally generic.
    It does not require that its this value be an Array object
es5id: 15.4.4.4_A2_T2
description: Checking this for Object object with no items
---*/

var x = {};
x.concat = Array.prototype.concat;
var arr = x.concat();

arr.getClass = Object.prototype.toString;
assert.sameValue(arr.getClass(), "[object Array]", 'arr.getClass() must return "[object Array]"');
assert.sameValue(arr[0], x, 'The value of arr[0] is expected to equal the value of x');
assert.sameValue(arr.length, 1, 'The value of arr.length is expected to be 1');
