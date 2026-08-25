// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - no observable effects occur if len is 0
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var obj = {
  0: 11,
  1: 12,
  length: 0
};

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert.sameValue(accessed, false, 'accessed');
