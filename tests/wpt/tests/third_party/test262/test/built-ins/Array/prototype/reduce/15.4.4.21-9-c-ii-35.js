// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - the Arguments object can be used as
    accumulator
---*/

var accessed = false;
var arg;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return prevVal === arg;
}

var obj = {
  0: 11,
  length: 1
};

(function fun() {
  arg = arguments;
}(10, 11, 12, 13));

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, arg), true, 'Array.prototype.reduce.call(obj, callbackfn, arg)');
assert(accessed, 'accessed !== true');
