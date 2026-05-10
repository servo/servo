// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - Exception in getter terminate
    iteration on an Array-like object
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx <= 1) {
    accessed = true;
  }
}

var obj = {
  0: 0,
  1: 1,
  length: 3
};
Object.defineProperty(obj, "2", {
  get: function() {
    throw new RangeError("unhandle exception happened in getter");
  },
  configurable: true
});
assert.throws(RangeError, function() {
  Array.prototype.reduceRight.call(obj, callbackfn);
});
assert.sameValue(accessed, false, 'accessed');
