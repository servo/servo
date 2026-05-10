// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - false prevents further side effects
---*/

var result = false;
var obj = {
  length: 20
};

function callbackfn(val, idx, obj) {
  if (idx > 1) {
    result = true;
  }
  return val > 10;
}

Object.defineProperty(obj, "0", {
  get: function() {
    return 11;
  },
  configurable: true
});

Object.defineProperty(obj, "1", {
  get: function() {
    return 8;
  },
  configurable: true
});

Object.defineProperty(obj, "2", {
  get: function() {
    result = true;
    return 8;
  },
  configurable: true
});

assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');
assert.sameValue(result, false, 'result');
