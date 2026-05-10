// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - non-indexed properties are not called
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val === 11;
}

var obj = {
  0: 9,
  10: 8,
  non_index_property: 11,
  length: 20
};

assert.sameValue(Array.prototype.some.call(obj, callbackfn), false, 'Array.prototype.some.call(obj, callbackfn)');
assert.sameValue(called, 2, 'called');
