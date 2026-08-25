// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - value of 'length' is boundary value (2^32)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var obj = {
  0: 12,
  length: 4294967296
};

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
assert(accessed, 'accessed !== true');
