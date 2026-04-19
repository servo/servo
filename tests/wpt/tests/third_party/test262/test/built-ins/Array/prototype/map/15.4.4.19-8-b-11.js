// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - deleting property of prototype causes
    prototype index property not to be visited on an Array
---*/

function callbackfn(val, idx, obj) {
  return idx === 1 && typeof val === "undefined";
}
var arr = [0, , 2];

Object.defineProperty(arr, "0", {
  get: function() {
    delete Array.prototype[1];
    return 0;
  },
  configurable: true
});

Array.prototype[1] = 1;
var testResult = arr.map(callbackfn);

assert.sameValue(testResult.length, 3, 'testResult.length');
assert.sameValue(typeof testResult[1], "undefined", 'typeof testResult[1]');
