// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - non-indexed properties are not called
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val !== 8;
}

var obj = {
  0: 11,
  10: 12,
  non_index_property: 8,
  length: 20
};

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert.sameValue(called, 2, 'called');
