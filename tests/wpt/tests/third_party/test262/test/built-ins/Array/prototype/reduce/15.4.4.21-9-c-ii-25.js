// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - Function object can be used as accumulator
---*/

var objFunction = function() {};

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return prevVal === objFunction;
}

var obj = {
  0: 11,
  length: 1
};

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, objFunction), true, 'Array.prototype.reduce.call(obj, callbackfn, objFunction)');
assert(accessed, 'accessed !== true');
