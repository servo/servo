// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - modifications to length don't change
    number of iterations in step 9
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return idx;
}

var obj = {
  3: 12,
  4: 9,
  length: 4
};

Object.defineProperty(obj, "2", {
  get: function() {
    obj.length = 10;
    return 11;
  },
  configurable: true
});

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn), 3, 'Array.prototype.reduce.call(obj, callbackfn)');
