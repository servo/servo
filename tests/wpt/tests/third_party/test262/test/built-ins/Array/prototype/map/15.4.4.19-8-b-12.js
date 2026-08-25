// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - deleting own property with prototype
    property causes prototype index property to be visited on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1 && val === 3) {
    return false;
  } else {
    return true;
  }
}
var obj = {
  0: 0,
  1: 1,
  2: 2,
  length: 10
};

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

Object.prototype[1] = 3;
var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[1], false, 'testResult[1]');
