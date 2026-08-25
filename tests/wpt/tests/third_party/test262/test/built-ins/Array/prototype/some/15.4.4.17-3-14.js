// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - 'length' is a string containing +/-Infinity
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var objOne = {
  0: 11,
  length: "Infinity"
};
var objTwo = {
  0: 11,
  length: "+Infinity"
};
var objThree = {
  0: 11,
  length: "-Infinity"
};

assert(Array.prototype.some.call(objOne, callbackfn), 'Array.prototype.some.call(objOne, callbackfn) !== true');
assert(Array.prototype.some.call(objTwo, callbackfn), 'Array.prototype.some.call(objTwo, callbackfn) !== true');
assert.sameValue(Array.prototype.some.call(objThree, callbackfn), false, 'Array.prototype.some.call(objThree, callbackfn)');
assert(accessed, 'accessed !== true');
