// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
info: "[[Get]] from not an inherited property"
es5id: 15.4.4.4_A3_T1
description: >
    [[Prototype]] of Array instance is Array.prototype, [[Prototype]
    of Array.prototype is Object.prototype
---*/

Array.prototype[1] = 1;
var x = [0];
x.length = 2;
var arr = x.concat();

assert.sameValue(arr[0], 0, 'The value of arr[0] is expected to be 0');
assert.sameValue(arr[1], 1, 'The value of arr[1] is expected to be 1');
assert.sameValue(arr.hasOwnProperty('1'), true, 'arr.hasOwnProperty("1") must return true');

Object.prototype[1] = 1;
Object.prototype.length = 2;
Object.prototype.concat = Array.prototype.concat;
x = {
  0: 0
};
var arr = x.concat();

assert.sameValue(arr[0], x, 'The value of arr[0] is expected to equal the value of x');
assert.sameValue(arr[1], 1, 'The value of arr[1] is expected to be 1');
assert.sameValue(arr.hasOwnProperty('1'), false, 'arr.hasOwnProperty("1") must return false');
