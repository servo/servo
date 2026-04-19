// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - unnhandled exceptions happened in getter
    terminate iteration on an Array-like object
---*/

var obj = {
  0: 11,
  5: 10,
  10: 8,
  length: 20
};
var accessed = false;

function callbackfn(val, idx, obj) {
  if (idx > 1) {
    accessed = true;
  }
}

Object.defineProperty(obj, "1", {
  get: function() {
    throw new RangeError("unhandle exception happened in getter");
  },
  configurable: true
});

Object.defineProperty(obj, "2", {
  get: function() {
    accessed = true;
    return 100;
  },
  configurable: true
});
assert.throws(RangeError, function() {
  Array.prototype.forEach.call(obj, callbackfn);
});
assert.sameValue(accessed, false, 'accessed');
