// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - unhandled exceptions happened in
    callbackfn terminate iteration
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  if (called === 1) {
    throw new Error("Exception occurred in callbackfn");
  }
  return true;
}

var obj = {
  0: 11,
  4: 10,
  10: 8,
  length: 20
};
assert.throws(Error, function() {
  Array.prototype.filter.call(obj, callbackfn);
});
assert.sameValue(called, 1, 'called');
