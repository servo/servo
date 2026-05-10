// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - 'length' is a string containing +/-Infinity
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var objOne = {
  0: 9,
  length: "Infinity"
};
var objTwo = {
  0: 9,
  length: "+Infinity"
};
var objThree = {
  0: 9,
  length: "-Infinity"
};

assert.sameValue(Array.prototype.every.call(objOne, callbackfn), false, 'Array.prototype.every.call(objOne, callbackfn)');
assert.sameValue(Array.prototype.every.call(objTwo, callbackfn), false, 'Array.prototype.every.call(objTwo, callbackfn)');
assert(Array.prototype.every.call(objThree, callbackfn), 'Array.prototype.every.call(objThree, callbackfn) !== true');
assert(accessed, 'accessed !== true');
