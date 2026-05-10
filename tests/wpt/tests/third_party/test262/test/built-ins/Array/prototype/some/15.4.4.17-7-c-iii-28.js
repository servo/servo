// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - true prevents further side effects
---*/

var result = false;

function callbackfn(val, idx, obj) {
  if (idx > 1) {
    result = true;
  }
  return val > 10;
}

var obj = {
  length: 20
};

Object.defineProperty(obj, "0", {
  get: function() {
    return 8;
  },
  configurable: true
});

Object.defineProperty(obj, "1", {
  get: function() {
    return 11;
  },
  configurable: true
});

Object.defineProperty(obj, "2", {
  get: function() {
    result = true;
    return 11;
  },
  configurable: true
});

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
assert.sameValue(result, false, 'result');
