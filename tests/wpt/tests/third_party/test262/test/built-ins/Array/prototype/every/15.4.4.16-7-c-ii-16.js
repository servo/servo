// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - 'this' of 'callbackfn' is a Boolean object
    when T is not an object (T is a boolean primitive)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this.valueOf() !== false;
}

var obj = {
  0: 11,
  length: 2
};

assert.sameValue(Array.prototype.every.call(obj, callbackfn, false), false, 'Array.prototype.every.call(obj, callbackfn, false)');
assert(accessed, 'accessed !== true');
