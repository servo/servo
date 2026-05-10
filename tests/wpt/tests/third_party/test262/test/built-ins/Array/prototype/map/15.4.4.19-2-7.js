// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - applied to Array-like object, 'length' is an
    own accessor property
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

obj[0] = 12;
obj[1] = 11;
obj[2] = 9;

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult.length, 2, 'testResult.length');
