// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is a string containing
    +/-Infinity
---*/

var objOne = {
  0: true,
  1: true,
  length: "Infinity"
};
var objTwo = {
  0: true,
  1: true,
  length: "+Infinity"
};
var objThree = {
  0: true,
  1: true,
  length: "-Infinity"
};

assert.sameValue(Array.prototype.indexOf.call(objOne, true), 0, 'Array.prototype.indexOf.call(objOne, true)');
assert.sameValue(Array.prototype.indexOf.call(objTwo, true), 0, 'Array.prototype.indexOf.call(objTwo, true)');
assert.sameValue(Array.prototype.indexOf.call(objThree, true), -1, 'Array.prototype.indexOf.call(objThree, true)');
